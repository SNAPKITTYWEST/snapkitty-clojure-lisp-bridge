//! `bifrost-policy` — runtime decision procedures extracted from the Lean 4 formal spec.
//!
//! # Architecture
//! ```text
//! bifrost-policy/lean/      ← Lean 4 source (types + propositions + proofs)
//!   Bifrost/Event.lean      ← inductive Event, Cid, PubKey, Rights
//!   Bifrost/State.lean      ← State (worm set, caps map, epoch roots)
//!   Bifrost/Policy.lean     ← valid_event, decide, decide_sound theorem
//!   Bifrost/Attestation.lean← valid_attestation
//!   Bifrost/Main.lean       ← package entry + export stubs
//!   BifrostPolicy.lean      ← re-exports all modules
//!
//! This crate (Rust) re-implements `decide` exactly matching the Lean 4
//! definition.  The Lean proof `decide_sound` guarantees that returning `true`
//! implies `valid_event`.  When `lean4-rust` extraction matures this stub
//! will be replaced by auto-generated code from `lake build`.
//! ```
//!
//! # Running the proofs
//! ```sh
//! cd snap-os/bifrost-policy
//! lake build   # requires lean4 + lake in PATH
//! ```

use bifrost::event::EventPayload;
use bifrost::worm::WormFs;

// ── PolicyState ───────────────────────────────────────────────────────────────

/// Runtime state consumed by the policy decision procedure.
///
/// Mirrors `Bifrost.State` in the Lean 4 spec.
pub struct PolicyState<'a> {
    pub worm: &'a WormFs,
}

// ── policy_decide ─────────────────────────────────────────────────────────────

/// Runtime decision procedure — mirrors `Bifrost.Policy.decide` in the Lean 4 spec.
///
/// Returns `true` iff the event is permitted by the policy given the current state.
///
/// | Event variant  | Policy rule |
/// |----------------|-------------|
/// | `CapTransfer`  | `policy_cid` is sealed in WORM_FS.  Zero CID = genesis-class (always allowed). |
/// | `JitCompile`   | `soulir_cid` is sealed in WORM_FS (source is immutable). |
///
/// The Lean 4 theorem `decide_sound` guarantees: `policy_decide(e, s) = true → valid_event e s`.
/// The `sorry`-free proof is tracked in `Bifrost/Policy.lean`.
pub fn policy_decide(payload: &EventPayload, state: &PolicyState<'_>) -> bool {
    match payload {
        EventPayload::CapTransfer { policy_cid, .. } => {
            // Zero CID = genesis-class event (runtime mint) — always allowed.
            policy_cid.0 == [0u8; 32] || state.worm.is_sealed(policy_cid)
        }
        EventPayload::JitCompile { soulir_cid, .. } => {
            state.worm.is_sealed(soulir_cid)
        }
        // Context must be sealed before the handoff is recorded.
        // summary_cid is optional but also must be sealed if provided.
        EventPayload::ContextHandoff { context_cid, summary_cid, .. } => {
            state.worm.is_sealed(context_cid)
                && summary_cid
                    .as_ref()
                    .map_or(true, |s| state.worm.is_sealed(s))
        }
    }
}

/// Enforce policy: returns `Err` with a description if the event is not permitted.
pub fn enforce(payload: &EventPayload, state: &PolicyState<'_>) -> Result<(), PolicyViolation> {
    if policy_decide(payload, state) {
        Ok(())
    } else {
        Err(PolicyViolation { payload: format!("{payload:?}") })
    }
}

// ── PolicyViolation ───────────────────────────────────────────────────────────

#[derive(Debug)]
pub struct PolicyViolation {
    pub payload: String,
}

impl std::fmt::Display for PolicyViolation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "policy violation: {}", self.payload)
    }
}

impl std::error::Error for PolicyViolation {}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use bifrost::event::Cid;
    use bifrost::worm::WormFs;
    use tempfile::TempDir;

    fn worm() -> (WormFs, TempDir) {
        let dir = tempfile::tempdir().unwrap();
        let w = WormFs::open(&dir.path().join("worm")).unwrap();
        (w, dir)
    }

    #[test]
    fn cap_transfer_allowed_for_zero_policy_cid() {
        let (w, _dir) = worm();
        let state = PolicyState { worm: &w };
        let payload = EventPayload::CapTransfer {
            from: Cid([0u8; 32]),
            to:   Cid([0u8; 32]),
            cap_hash:   Cid::of(b"cap"),
            policy_cid: Cid([0u8; 32]), // zero = genesis-class
        };
        assert!(policy_decide(&payload, &state));
    }

    #[test]
    fn cap_transfer_blocked_when_policy_not_in_worm() {
        let (w, _dir) = worm();
        let state = PolicyState { worm: &w };
        let payload = EventPayload::CapTransfer {
            from: Cid::of(b"a"),
            to:   Cid::of(b"b"),
            cap_hash:   Cid::of(b"cap"),
            policy_cid: Cid::of(b"missing_policy"),
        };
        assert!(!policy_decide(&payload, &state));
    }

    #[test]
    fn cap_transfer_allowed_when_policy_sealed() {
        let (mut w, _dir) = worm();
        let policy_cid = w.seal_blob(b"allow_all", None).unwrap();
        let state = PolicyState { worm: &w };
        let payload = EventPayload::CapTransfer {
            from: Cid::of(b"a"),
            to:   Cid::of(b"b"),
            cap_hash:   Cid::of(b"cap"),
            policy_cid,
        };
        assert!(policy_decide(&payload, &state));
    }

    #[test]
    fn jit_compile_allowed_when_soulir_sealed() {
        let (mut w, _dir) = worm();
        let soulir_cid = w.seal_blob(b"SoulIR bytecode", None).unwrap();
        let state = PolicyState { worm: &w };
        let payload = EventPayload::JitCompile {
            soulir_cid,
            wasm_cid:     Cid::of(b"wasm"),
            opt_level:    1,
            gas_estimate: 100,
        };
        assert!(policy_decide(&payload, &state));
    }

    #[test]
    fn jit_compile_blocked_when_soulir_missing() {
        let (w, _dir) = worm();
        let state = PolicyState { worm: &w };
        let payload = EventPayload::JitCompile {
            soulir_cid:   Cid::of(b"not_sealed"),
            wasm_cid:     Cid::of(b"wasm"),
            opt_level:    1,
            gas_estimate: 100,
        };
        assert!(!policy_decide(&payload, &state));
    }

    // ── ContextHandoff policy tests ────────────────────────────────────────────

    #[test]
    fn context_handoff_blocked_when_context_missing() {
        let (w, _dir) = worm();
        let state = PolicyState { worm: &w };
        let payload = EventPayload::ContextHandoff {
            from_agent:     Cid::of(b"agent_a"),
            to_agent:       Cid::of(b"agent_b"),
            context_cid:    Cid::of(b"not_sealed"),
            epoch:          0,
            schema_version: 1,
            summary_cid:    None,
        };
        assert!(!policy_decide(&payload, &state));
    }

    #[test]
    fn context_handoff_allowed_when_context_sealed_no_summary() {
        let (mut w, _dir) = worm();
        let ctx = w.seal_blob(b"session state", None).unwrap();
        let state = PolicyState { worm: &w };
        let payload = EventPayload::ContextHandoff {
            from_agent:     Cid::of(b"agent_a"),
            to_agent:       Cid::of(b"agent_b"),
            context_cid:    ctx,
            epoch:          1,
            schema_version: 1,
            summary_cid:    None,
        };
        assert!(policy_decide(&payload, &state));
    }

    #[test]
    fn context_handoff_blocked_when_summary_missing() {
        let (mut w, _dir) = worm();
        let ctx = w.seal_blob(b"session state", None).unwrap();
        let state = PolicyState { worm: &w };
        let payload = EventPayload::ContextHandoff {
            from_agent:     Cid::of(b"agent_a"),
            to_agent:       Cid::of(b"agent_b"),
            context_cid:    ctx,
            epoch:          1,
            schema_version: 1,
            summary_cid:    Some(Cid::of(b"summary_not_sealed")),
        };
        assert!(!policy_decide(&payload, &state));
    }

    #[test]
    fn context_handoff_allowed_when_both_sealed() {
        let (mut w, _dir) = worm();
        let ctx = w.seal_blob(b"full lossless state", None).unwrap();
        let sum = w.seal_blob(b"human-readable summary", None).unwrap();
        let state = PolicyState { worm: &w };
        let payload = EventPayload::ContextHandoff {
            from_agent:     Cid::of(b"agent_a"),
            to_agent:       Cid::of(b"agent_b"),
            context_cid:    ctx,
            epoch:          3,
            schema_version: 1,
            summary_cid:    Some(sum),
        };
        assert!(policy_decide(&payload, &state));
    }
}
