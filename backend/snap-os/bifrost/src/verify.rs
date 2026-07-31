//! Chain verification — replay the bifrost DAG and enforce all invariants.
//!
//! # Verification layers (applied per event, in order)
//!
//! | Layer | Check | Error |
//! |---|---|---|
//! | **CID integrity** | `blake3(canonical_json) == event.cid` | `CidMismatch` |
//! | **Signature** | Ed25519 sig over `cid` by `event.pubkey` | `BadSignature` |
//! | **Chain continuity** | `event.prev` is in DAG (or genesis) | `BrokenChain` |
//! | **Height monotone** | `event.height == prev.height + 1` | included in report |
//! | **WORM containment** | payload CIDs exist in WORM_FS | `NotFound` |
//!
//! # Output
//! `VerifyReport` is the authoritative audit document — suitable for logging
//! to SIEM or serving via `bifrost verify` CLI.

use crate::dag::DagStore;
use crate::error::BifrostResult;
use crate::event::{Cid, Event, EventPayload};
use crate::worm::WormFs;

// ── VerifyReport ──────────────────────────────────────────────────────────────

/// The result of a full chain verification run.
#[derive(Debug)]
pub struct VerifyReport {
    /// CID of the chain head that was verified.
    pub head:         Cid,
    /// Total events walked (including genesis).
    pub event_count:  usize,
    /// Chain height at head.
    pub height:       u64,
    /// All CIDs in the chain, newest first.
    pub chain:        Vec<Cid>,
    /// Accumulated errors. Empty = chain is valid.
    pub errors:       Vec<String>,
    /// True iff `errors` is empty.
    pub ok:           bool,
}

// ── verify_chain ──────────────────────────────────────────────────────────────

/// Walk the entire chain from `head` to genesis and verify every invariant.
///
/// This is an O(n) operation over the full chain.  For very long chains use
/// snapshot-based verification (verify from snapshot root only).
///
/// # Arguments
/// - `head`: the CID to start from (usually `dag.head()?`).
/// - `dag`: the DAG store to read events from.
/// - `worm`: the WORM_FS to check payload CID containment.
///
/// # Returns
/// Always returns `Ok(VerifyReport)` — errors are collected in `report.errors`.
/// Returns `Err` only on I/O or DB failures.
pub fn verify_chain(head: &Cid, dag: &DagStore, worm: &WormFs) -> BifrostResult<VerifyReport> {
    let mut chain   = Vec::new();
    let mut errors  = Vec::new();
    let mut cur_cid = Some(*head);
    let mut expected_height: Option<u64> = None;

    while let Some(cid) = cur_cid {
        let event = match dag.get(&cid)? {
            Some(ev) => ev,
            None => {
                errors.push(format!("DAG missing event {cid}"));
                break;
            }
        };

        chain.push(cid);

        // ── Layer 1: CID integrity ────────────────────────────────────────────
        if let Err(e) = event.verify_cid() {
            errors.push(format!("{e}"));
        }

        // ── Layer 2: Signature ────────────────────────────────────────────────
        if let Err(e) = event.verify_sig() {
            errors.push(format!("{e}"));
        }

        // ── Layer 3: Chain continuity ─────────────────────────────────────────
        match (&event.prev, event.height) {
            (None, 0) => {} // genesis: OK
            (None, h) => errors.push(format!(
                "event {cid} has no prev but height={h} (expected 0)"
            )),
            (Some(prev_cid), h) => {
                if !dag.contains(prev_cid)? {
                    errors.push(format!(
                        "broken chain at {cid}: prev {prev_cid} not in DAG"
                    ));
                }
                if let Some(expected) = expected_height {
                    if h != expected {
                        errors.push(format!(
                            "non-monotone height: event {cid} height={h} expected={expected}"
                        ));
                    }
                }
            }
        }

        // ── Layer 4: WORM containment (payload CIDs) ──────────────────────────
        check_worm_containment(&event, worm, &mut errors);

        // Prepare for next iteration
        expected_height = event.height.checked_sub(1);
        cur_cid = event.prev;
    }

    let height = chain.first()
        .and_then(|cid| dag.get(cid).ok().flatten())
        .map(|ev| ev.height)
        .unwrap_or(0);

    let ok = errors.is_empty();
    Ok(VerifyReport {
        head: *head,
        event_count: chain.len(),
        height,
        chain,
        errors,
        ok,
    })
}

/// Verify a single event in isolation (does not walk the chain).
///
/// Use this for online validation of incoming events before inserting.
pub fn verify_event(event: &Event, worm: &WormFs) -> BifrostResult<Vec<String>> {
    let mut errors = Vec::new();
    if let Err(e) = event.verify_cid() { errors.push(format!("{e}")); }
    if let Err(e) = event.verify_sig() { errors.push(format!("{e}")); }
    check_worm_containment(event, worm, &mut errors);
    Ok(errors)
}

fn check_worm_containment(event: &Event, worm: &WormFs, errors: &mut Vec<String>) {
    match &event.payload {
        EventPayload::CapTransfer { policy_cid, .. } => {
            // Genesis events use zero CIDs — skip the check.
            // cap_hash is a hash reference to a cap struct, not a stored blob.
            // Only policy_cid is required to be sealed in WORM_FS.
            if policy_cid.0 != [0u8; 32] && !worm.is_sealed(policy_cid) {
                errors.push(format!(
                    "CapTransfer {}: policy_cid {} not in WORM_FS",
                    event.cid, policy_cid
                ));
            }
        }
        EventPayload::JitCompile { soulir_cid, .. } => {
            if !worm.is_sealed(soulir_cid) {
                errors.push(format!(
                    "JitCompile {}: soulir_cid {} not in WORM_FS",
                    event.cid, soulir_cid
                ));
            }
        }
        EventPayload::ContextHandoff { context_cid, summary_cid, .. } => {
            if !worm.is_sealed(context_cid) {
                errors.push(format!(
                    "ContextHandoff {}: context_cid {} not in WORM_FS",
                    event.cid, context_cid
                ));
            }
            if let Some(scid) = summary_cid {
                if !worm.is_sealed(scid) {
                    errors.push(format!(
                        "ContextHandoff {}: summary_cid {} not in WORM_FS",
                        event.cid, scid
                    ));
                }
            }
        }
    }
}

// ── Unit tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::seal::Bifrost;
    use tempfile::tempdir;

    fn build_chain(n: usize) -> (Bifrost, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let mut b = Bifrost::open(dir.path()).unwrap();
        b.seal_genesis().unwrap();

        if n > 0 {
            let policy = b.worm.seal_blob(b"prolog_policy", None).unwrap();
            for i in 0..n {
                b.seal_cap_transfer(
                    Cid::of(&[i as u8]), Cid::of(&[i as u8 + 1]),
                    Cid::of(&[i as u8 + 2]), policy,
                ).unwrap();
            }
        }
        (b, dir)
    }

    #[test]
    fn verify_clean_chain_passes() {
        let (b, _dir) = build_chain(3);
        let head = b.dag.head().unwrap().unwrap();
        let report = verify_chain(&head, &b.dag, &b.worm).unwrap();
        assert!(report.ok, "errors: {:?}", report.errors);
        assert_eq!(report.event_count, 4); // genesis + 3
        assert_eq!(report.height, 3);
    }

    #[test]
    fn verify_genesis_only() {
        let (b, _dir) = build_chain(0);
        let head = b.dag.head().unwrap().unwrap();
        let report = verify_chain(&head, &b.dag, &b.worm).unwrap();
        assert!(report.ok, "errors: {:?}", report.errors);
        assert_eq!(report.event_count, 1);
        assert_eq!(report.height, 0);
    }

    #[test]
    fn chain_order_is_newest_first() {
        let (b, _dir) = build_chain(2);
        let head = b.dag.head().unwrap().unwrap();
        let report = verify_chain(&head, &b.dag, &b.worm).unwrap();
        // First in chain vec is the head (most recent)
        assert_eq!(report.chain[0], head);
        assert_eq!(report.chain.len(), 3);
    }

    #[test]
    fn verify_single_event_clean() {
        let (b, _dir) = build_chain(1);
        let head = b.dag.head().unwrap().unwrap();
        let ev = b.dag.get(&head).unwrap().unwrap();
        let errors = verify_event(&ev, &b.worm).unwrap();
        assert!(errors.is_empty(), "single event errors: {errors:?}");
    }
}
