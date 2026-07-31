//! Core event model for the bifrost Merkle-DAG audit chain.
//!
//! # Invariants (enforced by construction)
//! 1. `event.cid == blake3(canonical_bytes(event))` — verified in `Event::verify_cid`.
//! 2. `event.sig == ed25519_sign(signing_key, event.cid.as_bytes())`.
//! 3. `event.prev == None` iff `event.height == 0` (genesis).
//! 4. Payload CIDs referenced by `CapTransfer::policy_cid` and `JitCompile::soulir_cid`
//!    MUST be sealed in WORM_FS before the event is accepted — checked in `seal.rs`.
//!
//! # Canonical-bytes format
//! CIDs are computed over a deterministic JSON envelope:
//! ```json
//! {
//!   "height": <u64>,
//!   "timestamp": <u64>,
//!   "prev": "<hex_cid>" | null,
//!   "pubkey": "<hex_32_bytes>",
//!   "payload": { ... }
//! }
//! ```
//! Fields are serialised in this fixed order (serde struct field order).
//! Production upgrade path: replace with DAG-CBOR + multihash for IPLD compatibility.

use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::{BifrostError, BifrostResult};

// ── Cid ─────────────────────────────────────────────────────────────────────

/// Content Identifier — blake3 hash (256-bit) of a serialised entity.
///
/// `Cid` is the universal key in bifrost: events, blobs, and policies all
/// have CIDs.  Two byte strings with the same CID are guaranteed identical by
/// the collision-resistance of blake3.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Cid(#[serde(with = "hex_array_32")] pub [u8; 32]);

impl Cid {
    /// Compute the CID of an arbitrary byte slice.
    pub fn of(bytes: &[u8]) -> Self {
        Self(*blake3::hash(bytes).as_bytes())
    }

    pub fn as_bytes(&self) -> &[u8; 32] { &self.0 }

    pub fn to_hex(&self) -> String { hex::encode(self.0) }

    pub fn from_hex(s: &str) -> BifrostResult<Self> {
        let b = hex::decode(s).map_err(|e| BifrostError::Serialize(e.to_string()))?;
        if b.len() != 32 {
            return Err(BifrostError::Serialize(format!("CID hex must be 64 chars, got {}", s.len())));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&b);
        Ok(Self(arr))
    }
}

impl fmt::Debug for Cid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Cid({})", &self.to_hex()[..12])
    }
}
impl fmt::Display for Cid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_hex())
    }
}

// ── Sig ─────────────────────────────────────────────────────────────────────

/// Ed25519 signature (64 bytes).
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sig(#[serde(with = "hex_array_64")] pub [u8; 64]);

impl Sig {
    pub fn as_bytes(&self) -> &[u8] { &self.0 }
    pub fn to_hex(&self) -> String { hex::encode(self.0) }
}

impl fmt::Debug for Sig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Sig({}…)", &self.to_hex()[..16])
    }
}

// ── PubKey ───────────────────────────────────────────────────────────────────

/// Ed25519 verifying key (32 bytes).
#[derive(Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PubKey(#[serde(with = "hex_array_32")] pub [u8; 32]);

impl PubKey {
    pub fn to_hex(&self) -> String { hex::encode(self.0) }
}

impl fmt::Debug for PubKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PubKey({})", &self.to_hex()[..12])
    }
}

// ── EventPayload ──────────────────────────────────────────────────────────────

/// The typed payload that distinguishes event classes in the DAG.
///
/// New payload variants MUST be introduced with a schema version bump;
/// old verifiers will reject unknown variants.
///
/// Schema version: 2 — adds `ContextHandoff`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum EventPayload {
    /// A silverback capability was transferred from one soul to another.
    ///
    /// # Fields
    /// - `from` / `to`: CIDs of the sender/receiver soul topology snapshots.
    /// - `cap_hash`: blake3 of the serialised `Cap` struct at transfer time.
    /// - `policy_cid`: CID of the Prolog policy document that approved the transfer.
    ///   MUST be sealed in WORM_FS before this event is accepted.
    CapTransfer {
        from:       Cid,
        to:         Cid,
        cap_hash:   Cid,
        policy_cid: Cid,
    },

    /// A SoulVM function was JIT-compiled by the Cranelift engine.
    ///
    /// # Fields
    /// - `soulir_cid`: CID of the source SoulIR bytecode blob in WORM_FS.
    /// - `wasm_cid`:   CID of the resulting native code blob (or WASM if cross-compiled).
    /// - `opt_level`:  Cranelift optimisation level (0=none, 1=speed, 2=speed_and_size).
    /// - `gas_estimate`: Fuel units estimated for this function's execution budget.
    JitCompile {
        soulir_cid:   Cid,
        wasm_cid:     Cid,
        opt_level:    u8,
        gas_estimate: u64,
    },

    /// Context handoff between two agents — the lossless session transfer primitive.
    ///
    /// Solves the compression/handoff problem: instead of a lossy summary, the
    /// full session state is WORM-sealed and passed as a verifiable CID.
    /// The receiving agent loads `context_cid` from WORM_FS and has the exact
    /// state the sender had — no data loss, no summarisation artefacts.
    ///
    /// # Fields
    /// - `from_agent`: CID of the sending agent's soul snapshot.
    /// - `to_agent`:   CID of the receiving agent's soul snapshot.
    /// - `context_cid`: CID of the WORM-sealed context blob (full session state).
    ///   MUST be sealed in WORM_FS before this event is accepted.
    /// - `epoch`: The epoch counter at handoff time — links to the
    ///   `AttestationEnvelope` produced by `bifrost-attest` for this epoch.
    /// - `schema_version`: Caller-defined version tag for the context blob format.
    ///   Receivers MUST reject versions they don't understand.
    /// - `summary_cid`: Optional CID of a human-readable summary blob in WORM_FS.
    ///   Separate from the full context — lossy summaries are allowed here,
    ///   but `context_cid` is always the authoritative lossless record.
    ContextHandoff {
        from_agent:     Cid,
        to_agent:       Cid,
        context_cid:    Cid,
        epoch:          u64,
        schema_version: u32,
        summary_cid:    Option<Cid>,
    },
}

// ── CanonicalEnvelope (internal: used for CID computation only) ───────────────

/// Deterministic JSON structure hashed by `blake3` to produce the event CID.
///
/// Field order is fixed by the struct definition (serde serialises in declaration order).
/// Any change to this struct is a **breaking protocol change** and requires a
/// chain-version migration.
#[derive(Serialize)]
pub(crate) struct CanonicalEnvelope<'a> {
    pub height:    u64,
    pub timestamp: u64,
    pub prev:      Option<&'a Cid>,
    pub pubkey:    &'a PubKey,
    pub payload:   &'a EventPayload,
}

// ── Event ─────────────────────────────────────────────────────────────────────

/// A single sealed entry in the bifrost Merkle-DAG audit chain.
///
/// Invariants:
/// - `self.cid` must equal `blake3(canonical_json)` — checked by `verify_cid()`.
/// - `self.sig` must verify under `self.pubkey` over `self.cid.as_bytes()`.
/// - `self.prev == None` iff `self.height == 0`.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    /// Content identifier — canonical hash of this event's wire representation.
    pub cid:       Cid,
    /// Previous event CID; `None` for the genesis event.
    pub prev:      Option<Cid>,
    /// Event payload (CapTransfer or JitCompile).
    pub payload:   EventPayload,
    /// Ed25519 signature over `cid.as_bytes()` by `pubkey`.
    pub sig:       Sig,
    /// Verifying key of the soul that sealed this event.
    pub pubkey:    PubKey,
    /// Monotonically increasing chain height (genesis = 0).
    pub height:    u64,
    /// Unix epoch seconds at sealing time.
    pub timestamp: u64,
}

impl Event {
    /// Compute the canonical bytes that produce this event's CID.
    ///
    /// The CID field itself is excluded to avoid circular dependency.
    pub fn canonical_bytes(&self) -> BifrostResult<Vec<u8>> {
        let env = CanonicalEnvelope {
            height:    self.height,
            timestamp: self.timestamp,
            prev:      self.prev.as_ref(),
            pubkey:    &self.pubkey,
            payload:   &self.payload,
        };
        serde_json::to_vec(&env).map_err(BifrostError::Json)
    }

    /// Verify that `self.cid` matches the blake3 hash of `canonical_bytes`.
    ///
    /// Returns `Err(BifrostError::CidMismatch)` if the check fails.
    pub fn verify_cid(&self) -> BifrostResult<()> {
        let bytes = self.canonical_bytes()?;
        let computed = Cid::of(&bytes);
        if computed != self.cid {
            return Err(BifrostError::CidMismatch {
                expected: self.cid.to_hex(),
                computed: computed.to_hex(),
            });
        }
        Ok(())
    }

    /// Verify the Ed25519 signature `self.sig` over `self.cid`.
    ///
    /// Fails with `BifrostError::BadSignature` if invalid.
    pub fn verify_sig(&self) -> BifrostResult<()> {
        use ed25519_dalek::{Signature, VerifyingKey, Verifier};
        let vk = VerifyingKey::from_bytes(&self.pubkey.0)
            .map_err(|_| BifrostError::BadSignature { cid: self.cid.to_hex() })?;
        let sig = Signature::from_bytes(&self.sig.0);
        vk.verify(self.cid.as_bytes(), &sig)
            .map_err(|_| BifrostError::BadSignature { cid: self.cid.to_hex() })
    }

    /// Run all local integrity checks (CID + signature).
    pub fn verify(&self) -> BifrostResult<()> {
        self.verify_cid()?;
        self.verify_sig()
    }
}

// ── Serde helpers for fixed-size byte arrays ──────────────────────────────────

mod hex_array_32 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(arr: &[u8; 32], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(arr))
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 32], D::Error> {
        let s = String::deserialize(d)?;
        let b = hex::decode(&s).map_err(serde::de::Error::custom)?;
        if b.len() != 32 {
            return Err(serde::de::Error::custom("expected 32 bytes"));
        }
        let mut arr = [0u8; 32];
        arr.copy_from_slice(&b);
        Ok(arr)
    }
}

mod hex_array_64 {
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S: Serializer>(arr: &[u8; 64], s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(&hex::encode(arr))
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<[u8; 64], D::Error> {
        let s = String::deserialize(d)?;
        let b = hex::decode(&s).map_err(serde::de::Error::custom)?;
        if b.len() != 64 {
            return Err(serde::de::Error::custom("expected 64 bytes"));
        }
        let mut arr = [0u8; 64];
        arr.copy_from_slice(&b);
        Ok(arr)
    }
}

// ── Unit tests ────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_cid(seed: u8) -> Cid { Cid::of(&[seed; 32]) }

    fn make_cap_transfer_event() -> Event {
        use ed25519_dalek::{SigningKey, Signer};
        use rand::rngs::OsRng;

        let sk = SigningKey::generate(&mut OsRng);
        let pubkey = PubKey(sk.verifying_key().to_bytes());
        let payload = EventPayload::CapTransfer {
            from:       dummy_cid(1),
            to:         dummy_cid(2),
            cap_hash:   dummy_cid(3),
            policy_cid: dummy_cid(4),
        };
        let env = CanonicalEnvelope {
            height: 1, timestamp: 1_000_000,
            prev: Some(&dummy_cid(0)), pubkey: &pubkey, payload: &payload,
        };
        let canonical = serde_json::to_vec(&env).unwrap();
        let cid = Cid::of(&canonical);
        let sig_bytes: [u8; 64] = sk.sign(cid.as_bytes()).to_bytes();
        Event {
            cid, prev: Some(dummy_cid(0)), payload,
            sig: Sig(sig_bytes), pubkey, height: 1, timestamp: 1_000_000,
        }
    }

    #[test]
    fn cid_deterministic() {
        let a = Cid::of(b"hello bifrost");
        let b = Cid::of(b"hello bifrost");
        assert_eq!(a, b);
    }

    #[test]
    fn cid_hex_roundtrip() {
        let c = Cid::of(b"snap-os");
        let hex = c.to_hex();
        let decoded = Cid::from_hex(&hex).unwrap();
        assert_eq!(c, decoded);
    }

    #[test]
    fn event_verify_cid_passes() {
        let ev = make_cap_transfer_event();
        ev.verify_cid().expect("CID must verify");
    }

    #[test]
    fn event_verify_sig_passes() {
        let ev = make_cap_transfer_event();
        ev.verify_sig().expect("sig must verify");
    }

    #[test]
    fn event_verify_cid_fails_on_tamper() {
        let mut ev = make_cap_transfer_event();
        ev.height = 999; // tamper with height — canonical bytes change, CID won't match
        let err = ev.verify_cid().unwrap_err();
        assert!(matches!(err, BifrostError::CidMismatch { .. }));
    }

    #[test]
    fn event_serialise_roundtrip() {
        let ev = make_cap_transfer_event();
        let json = serde_json::to_string(&ev).unwrap();
        let back: Event = serde_json::from_str(&json).unwrap();
        assert_eq!(ev, back);
    }
}
