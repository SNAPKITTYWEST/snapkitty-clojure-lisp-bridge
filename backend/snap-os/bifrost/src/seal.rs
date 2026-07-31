//! `Bifrost` — the sealing engine that writes events to the DAG + WORM_FS.
//!
//! # Usage
//! ```no_run
//! use bifrost::{seal::Bifrost, event::Cid};
//!
//! let mut b = Bifrost::open("./chain").unwrap();
//! let _genesis  = b.seal_genesis().unwrap();
//! let policy    = b.worm.seal_blob(b"%cap_transfer :- true.", None).unwrap();
//! let _cid = b.seal_cap_transfer(
//!     Cid::of(b"soul_a"), Cid::of(b"soul_b"), Cid::of(b"cap"), policy,
//! ).unwrap();
//! ```
//!
//! # Sealing protocol (per event)
//! 1. Resolve `prev` (current head) and `height` from `DagStore`.
//! 2. Construct `CanonicalEnvelope` → JSON → `cid = blake3(json)`.
//! 3. Sign `cid` with the local `SigningKey` → `sig`.
//! 4. Assemble `Event { cid, prev, payload, sig, pubkey, height, timestamp }`.
//! 5. Verify `event.verify()` (self-check before writing).
//! 6. Write event JSON as a blob to `WormFs` (seals the serialised event).
//! 7. Insert into `DagStore` (updates `head`).
//!
//! # Key management
//! `Bifrost` holds an in-memory `ed25519_dalek::SigningKey`.  In production this
//! should be backed by a TPM or HSM — the sealing key is the root of trust for
//! the entire chain.

use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use ed25519_dalek::{SigningKey, Signer};
use rand::rngs::OsRng;

use crate::dag::DagStore;
use crate::error::{BifrostError, BifrostResult};
use crate::event::{CanonicalEnvelope, Cid, Event, EventPayload, PubKey, Sig};
use crate::worm::WormFs;

/// The bifrost sealing engine.
pub struct Bifrost {
    pub dag:  DagStore,
    pub worm: WormFs,
    sk:       SigningKey,
    pubkey:   PubKey,
}

impl Bifrost {
    /// Open the chain at `root`, generating a fresh key if none exists.
    ///
    /// Layout created under `root`:
    /// ```text
    /// root/
    ///   dag/      ← DagStore (sled)
    ///   worm/     ← WormFs (blobs + sled meta)
    /// ```
    ///
    /// The node's Ed25519 verifying key is sealed into WORM_FS immediately so
    /// that any event's `pubkey` field is resolvable by CID from the chain.
    ///
    /// TODO (production): load key from `root/keystore/` (TPM-backed).
    pub fn open(root: impl AsRef<Path>) -> BifrostResult<Self> {
        let root = root.as_ref();
        std::fs::create_dir_all(root)?;

        let dag  = DagStore::open(&root.join("dag"))?;
        let worm = WormFs::open(&root.join("worm"))?;
        let sk   = SigningKey::generate(&mut OsRng);
        let pubkey = PubKey(sk.verifying_key().to_bytes());
        // Seal the verifying key so signing_key_cid is a live WORM reference.
        seal_pubkey_into_worm(&worm, &pubkey);
        Ok(Self { dag, worm, sk, pubkey })
    }

    /// Open with a specific key (for deterministic testing and key import).
    pub fn open_with_key(root: impl AsRef<Path>, sk: SigningKey) -> BifrostResult<Self> {
        let root = root.as_ref();
        std::fs::create_dir_all(root)?;
        let dag    = DagStore::open(&root.join("dag"))?;
        let worm   = WormFs::open(&root.join("worm"))?;
        let pubkey = PubKey(sk.verifying_key().to_bytes());
        seal_pubkey_into_worm(&worm, &pubkey);
        Ok(Self { dag, worm, sk, pubkey })
    }

    /// Return the local node's public key.
    pub fn pubkey(&self) -> PubKey { self.pubkey }

    /// Return the WORM CID of this node's verifying key.
    ///
    /// This is `blake3(verifying_key.as_bytes())` — always resolvable via
    /// `self.worm.read_blob(&self.pubkey_cid())`.
    pub fn pubkey_cid(&self) -> Cid { Cid::of(&self.pubkey.0) }

    /// Seal the genesis event (height 0, prev = None).
    ///
    /// Uses a `CapTransfer` payload with zeroed CIDs to mark the chain origin.
    /// Returns `BifrostError::GenesisExists` if the chain already has a head.
    pub fn seal_genesis(&mut self) -> BifrostResult<Cid> {
        if self.dag.head()?.is_some() {
            return Err(BifrostError::GenesisExists);
        }
        let zero = Cid([0u8; 32]);
        let payload = EventPayload::CapTransfer {
            from:       zero,
            to:         zero,
            cap_hash:   zero,
            policy_cid: zero,
        };
        self.commit(payload, None, 0)
    }

    /// Seal a `CapTransfer` event.
    ///
    /// # Preconditions
    /// - `policy_cid` MUST already be sealed in `self.worm` — enforced here.
    /// - Chain must have a genesis event.
    pub fn seal_cap_transfer(
        &mut self,
        from:       Cid,
        to:         Cid,
        cap_hash:   Cid,
        policy_cid: Cid,
    ) -> BifrostResult<Cid> {
        // Policy CID must exist in WORM_FS.
        if !self.worm.is_sealed(&policy_cid) {
            return Err(BifrostError::NotFound { cid: policy_cid.to_hex() });
        }
        let prev   = self.require_head()?;
        let height = self.dag.height()? + 1;
        let payload = EventPayload::CapTransfer { from, to, cap_hash, policy_cid };
        self.commit(payload, Some(prev), height)
    }

    /// Seal a `JitCompile` event.
    ///
    /// # Preconditions
    /// - `soulir_cid` MUST be sealed in WORM_FS (the input bytecode must be immutable).
    /// - `wasm_cid` must NOT yet exist — it is the NEW output artifact.
    pub fn seal_jit_compile(
        &mut self,
        soulir_cid:   Cid,
        wasm_cid:     Cid,
        opt_level:    u8,
        gas_estimate: u64,
    ) -> BifrostResult<Cid> {
        if !self.worm.is_sealed(&soulir_cid) {
            return Err(BifrostError::NotFound { cid: soulir_cid.to_hex() });
        }
        let prev   = self.require_head()?;
        let height = self.dag.height()? + 1;
        let payload = EventPayload::JitCompile { soulir_cid, wasm_cid, opt_level, gas_estimate };
        self.commit(payload, Some(prev), height)
    }

    /// Seal a `ContextHandoff` event — lossless agent-to-agent session transfer.
    ///
    /// # Preconditions
    /// - `context_cid` MUST be sealed in WORM_FS before calling.  This guarantees
    ///   the receiving agent can always retrieve the full context by CID — no data loss.
    /// - `summary_cid`, if provided, MUST also be sealed in WORM_FS.
    /// - Chain must have a genesis event.
    ///
    /// # Why this exists
    /// Replaces lossy session summarisation.  The sender seals its full state as a
    /// blob (any format — JSON, CBOR, JSONL), passes the CID here, and the chain
    /// records the handoff with a priority timestamp.  The receiver loads the blob
    /// from WORM by CID and has the exact sender state.
    pub fn seal_context_handoff(
        &mut self,
        from_agent:     Cid,
        to_agent:       Cid,
        context_cid:    Cid,
        epoch:          u64,
        schema_version: u32,
        summary_cid:    Option<Cid>,
    ) -> BifrostResult<Cid> {
        if !self.worm.is_sealed(&context_cid) {
            return Err(BifrostError::NotFound { cid: context_cid.to_hex() });
        }
        if let Some(ref scid) = summary_cid {
            if !self.worm.is_sealed(scid) {
                return Err(BifrostError::NotFound { cid: scid.to_hex() });
            }
        }
        let prev   = self.require_head()?;
        let height = self.dag.height()? + 1;
        let payload = EventPayload::ContextHandoff {
            from_agent, to_agent, context_cid, epoch, schema_version, summary_cid,
        };
        self.commit(payload, Some(prev), height)
    }

    /// Seal an arbitrary pre-constructed payload (used by the concurrent appender).
    pub fn seal_payload(&mut self, payload: EventPayload) -> BifrostResult<Cid> {
        let prev   = self.dag.head()?;
        let height = if prev.is_some() { self.dag.height()? + 1 } else { 0 };
        self.commit(payload, prev, height)
    }

    // ── Internal: build, sign, verify, write ──────────────────────────────────

    fn commit(&mut self, payload: EventPayload, prev: Option<Cid>, height: u64) -> BifrostResult<Cid> {
        let timestamp = unix_now();

        // ── Step 1: canonical bytes → CID ────────────────────────────────────
        let env = CanonicalEnvelope {
            height, timestamp, prev: prev.as_ref(),
            pubkey: &self.pubkey, payload: &payload,
        };
        let canonical = serde_json::to_vec(&env)?;
        let cid = Cid::of(&canonical);

        // ── Step 2: sign CID ──────────────────────────────────────────────────
        let sig_bytes: [u8; 64] = self.sk.sign(cid.as_bytes()).to_bytes();
        let sig = Sig(sig_bytes);

        // ── Step 3: assemble Event ────────────────────────────────────────────
        let event = Event { cid, prev, payload, sig, pubkey: self.pubkey, height, timestamp };

        // ── Step 4: self-check (should never fail, but cheaper than chain repair) ──
        event.verify()?;

        // ── Step 5: seal event JSON as a WORM blob ───────────────────────────
        let event_bytes = serde_json::to_vec(&event)?;
        // Event blob already in WORM is fine (idempotent re-seal).
        match self.worm.seal_blob(&event_bytes, Some(&self.pubkey)) {
            Ok(_) | Err(BifrostError::WormViolation { .. }) => {}
            Err(e) => return Err(e),
        }

        // ── Step 6: insert into DAG ───────────────────────────────────────────
        self.dag.insert(&event)?;

        Ok(cid)
    }

    fn require_head(&self) -> BifrostResult<Cid> {
        self.dag.head()?.ok_or(BifrostError::EmptyChain)
    }
}

/// Seal a node's verifying key bytes into WORM_FS.
///
/// Idempotent: `WormViolation` (already sealed) is silently ignored — the
/// same key always produces the same CID, so the blob is already there.
fn seal_pubkey_into_worm(worm: &WormFs, pubkey: &PubKey) {
    match worm.seal_blob(&pubkey.0, None) {
        Ok(_) | Err(BifrostError::WormViolation { .. }) => {}
        Err(e) => eprintln!("bifrost/seal: seal_pubkey_into_worm failed: {e}"),
    }
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

// ── Unit tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    fn make_bifrost() -> (Bifrost, tempfile::TempDir) {
        let dir = tempdir().unwrap();
        let b = Bifrost::open(dir.path()).unwrap();
        (b, dir)
    }

    #[test]
    fn seal_genesis_succeeds() {
        let (mut b, _dir) = make_bifrost();
        let cid = b.seal_genesis().unwrap();
        assert_eq!(b.dag.head().unwrap().unwrap(), cid);
        assert_eq!(b.dag.height().unwrap(), 0);
    }

    #[test]
    fn double_genesis_fails() {
        let (mut b, _dir) = make_bifrost();
        b.seal_genesis().unwrap();
        let err = b.seal_genesis().unwrap_err();
        assert!(matches!(err, BifrostError::GenesisExists));
    }

    #[test]
    fn cap_transfer_requires_policy_in_worm() {
        let (mut b, _dir) = make_bifrost();
        b.seal_genesis().unwrap();

        let from = Cid::of(b"soul_a");
        let to   = Cid::of(b"soul_b");
        let cap  = Cid::of(b"cap_x");
        let policy = Cid::of(b"policy_not_in_worm");

        let err = b.seal_cap_transfer(from, to, cap, policy).unwrap_err();
        assert!(matches!(err, BifrostError::NotFound { .. }));
    }

    #[test]
    fn cap_transfer_succeeds_with_sealed_policy() {
        let (mut b, _dir) = make_bifrost();
        b.seal_genesis().unwrap();

        let policy_bytes = b"%cap_transfer_policy.";
        let policy_cid = b.worm.seal_blob(policy_bytes, None).unwrap();

        let cid = b.seal_cap_transfer(
            Cid::of(b"soul_a"), Cid::of(b"soul_b"),
            Cid::of(b"cap_x"), policy_cid,
        ).unwrap();

        assert_eq!(b.dag.height().unwrap(), 1);
        let ev = b.dag.get(&cid).unwrap().unwrap();
        assert_eq!(ev.height, 1);
    }

    #[test]
    fn jit_compile_seals_chain() {
        let (mut b, _dir) = make_bifrost();
        b.seal_genesis().unwrap();

        let soulir = b"SoulIR bytecode payload";
        let wasm   = b"compiled native code";
        let soulir_cid = b.worm.seal_blob(soulir, None).unwrap();
        let wasm_cid   = Cid::of(wasm);

        let cid = b.seal_jit_compile(soulir_cid, wasm_cid, 1, 42_000).unwrap();
        let ev  = b.dag.get(&cid).unwrap().unwrap();
        assert_eq!(ev.height, 1);

        if let EventPayload::JitCompile { opt_level, gas_estimate, .. } = ev.payload {
            assert_eq!(opt_level, 1);
            assert_eq!(gas_estimate, 42_000);
        } else {
            panic!("wrong payload type");
        }
    }

    #[test]
    fn chain_grows_monotonically() {
        let (mut b, _dir) = make_bifrost();
        b.seal_genesis().unwrap();

        let policy = b.worm.seal_blob(b"pol", None).unwrap();
        for i in 1..=5u64 {
            b.seal_cap_transfer(
                Cid::of(&[i as u8]), Cid::of(&[i as u8 + 1]),
                Cid::of(&[i as u8 + 2]), policy,
            ).unwrap();
            assert_eq!(b.dag.height().unwrap(), i);
        }
    }

    #[test]
    fn event_is_worm_sealed_after_commit() {
        let (mut b, _dir) = make_bifrost();
        let head = b.seal_genesis().unwrap();
        let ev = b.dag.get(&head).unwrap().unwrap();
        let ev_bytes = serde_json::to_vec(&ev).unwrap();
        let ev_cid = Cid::of(&ev_bytes);
        // The event JSON blob must be sealed in WORM_FS
        assert!(b.worm.is_sealed(&ev_cid));
    }

    // ── ContextHandoff tests ───────────────────────────────────────────────────

    #[test]
    fn context_handoff_requires_context_in_worm() {
        let (mut b, _dir) = make_bifrost();
        b.seal_genesis().unwrap();

        let err = b.seal_context_handoff(
            Cid::of(b"agent_a"), Cid::of(b"agent_b"),
            Cid::of(b"context_not_in_worm"), 0, 1, None,
        ).unwrap_err();
        assert!(matches!(err, BifrostError::NotFound { .. }));
    }

    #[test]
    fn context_handoff_requires_summary_in_worm_if_provided() {
        let (mut b, _dir) = make_bifrost();
        b.seal_genesis().unwrap();

        let context = b.worm.seal_blob(b"full session state", None).unwrap();

        let err = b.seal_context_handoff(
            Cid::of(b"agent_a"), Cid::of(b"agent_b"),
            context, 0, 1, Some(Cid::of(b"summary_not_in_worm")),
        ).unwrap_err();
        assert!(matches!(err, BifrostError::NotFound { .. }));
    }

    #[test]
    fn context_handoff_succeeds_without_summary() {
        let (mut b, _dir) = make_bifrost();
        b.seal_genesis().unwrap();

        let context_bytes = b"{\"messages\":[],\"tools_used\":[\"bash\",\"read\"]}";
        let context_cid = b.worm.seal_blob(context_bytes, None).unwrap();

        let cid = b.seal_context_handoff(
            Cid::of(b"claude_a"), Cid::of(b"claude_b"),
            context_cid, 1, 1, None,
        ).unwrap();

        assert_eq!(b.dag.height().unwrap(), 1);
        let ev = b.dag.get(&cid).unwrap().unwrap();
        if let EventPayload::ContextHandoff { epoch, schema_version, summary_cid, .. } = ev.payload {
            assert_eq!(epoch, 1);
            assert_eq!(schema_version, 1);
            assert!(summary_cid.is_none());
        } else {
            panic!("wrong payload type");
        }
    }

    #[test]
    fn context_handoff_succeeds_with_summary() {
        let (mut b, _dir) = make_bifrost();
        b.seal_genesis().unwrap();

        let context_cid = b.worm.seal_blob(b"full lossless state", None).unwrap();
        let summary_cid = b.worm.seal_blob(b"lossy human-readable summary", None).unwrap();

        let cid = b.seal_context_handoff(
            Cid::of(b"agent_a"), Cid::of(b"agent_b"),
            context_cid, 2, 1, Some(summary_cid),
        ).unwrap();

        let ev = b.dag.get(&cid).unwrap().unwrap();
        if let EventPayload::ContextHandoff { summary_cid: sc, epoch, .. } = ev.payload {
            assert_eq!(epoch, 2);
            assert_eq!(sc, Some(summary_cid));
        } else {
            panic!("wrong payload type");
        }
    }

    #[test]
    fn context_handoff_chains_after_cap_transfer() {
        let (mut b, _dir) = make_bifrost();
        b.seal_genesis().unwrap();

        let policy = b.worm.seal_blob(b"policy", None).unwrap();
        b.seal_cap_transfer(
            Cid::of(b"a"), Cid::of(b"b"), Cid::of(b"cap"), policy,
        ).unwrap();
        assert_eq!(b.dag.height().unwrap(), 1);

        let ctx = b.worm.seal_blob(b"post-cap-transfer context", None).unwrap();
        b.seal_context_handoff(
            Cid::of(b"agent_a"), Cid::of(b"agent_b"), ctx, 1, 1, None,
        ).unwrap();
        assert_eq!(b.dag.height().unwrap(), 2);
    }
}
