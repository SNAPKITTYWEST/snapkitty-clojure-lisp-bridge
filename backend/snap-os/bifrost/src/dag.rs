//! Merkle-DAG store — content-addressed event graph backed by `sled`.
//!
//! # Topology
//! The DAG is a *linear chain* (each event has exactly one `prev`) with
//! *fan-out references* inside payloads (CapTransfer, JitCompile reference
//! external blob CIDs sealed in WORM_FS).  This gives us:
//!
//! ```text
//! genesis ← event_1 ← event_2 ← … ← head
//!               │
//!               └──→ worm:policy_cid   (payload fan-out)
//!               └──→ worm:cap_hash
//! ```
//!
//! # Persistence
//! Events are stored in a `sled::Tree` keyed by `cid_hex → json(Event)`.
//! Two special keys in the same tree track chain state:
//! - `b"_head"` → hex CID of the current chain head
//! - `b"_height"` → big-endian u64 height

use sled::Db;

use crate::error::{BifrostError, BifrostResult};
use crate::event::{Cid, Event};

const KEY_HEAD:   &[u8] = b"__head__";
const KEY_HEIGHT: &[u8] = b"__height__";

/// Content-addressed event store.
///
/// Thread-safety: `sled::Db` is `Send + Sync`; concurrent reads are safe.
/// Writes MUST go through `insert()` which updates head + height atomically
/// via a `sled` batch.
pub struct DagStore {
    #[allow(dead_code)]
    db:     Db,
    events: sled::Tree,
}

impl DagStore {
    /// Open (or create) the DAG store at `path`.
    ///
    /// A fresh directory is initialised as an empty chain (no genesis yet).
    pub fn open(path: &std::path::Path) -> BifrostResult<Self> {
        let db = sled::open(path)?;
        let events = db.open_tree(b"events")?;
        Ok(Self { db, events })
    }

    /// Insert a pre-verified event into the DAG and update `head`.
    ///
    /// # Preconditions
    /// - Caller MUST have run `event.verify()` before inserting.
    /// - `event.prev` must already be in the DAG unless this is the genesis.
    ///
    /// # Atomicity
    /// The event payload, head pointer, and height counter are written in a
    /// single `sled` batch — either all succeed or none do.
    pub fn insert(&self, event: &Event) -> BifrostResult<()> {
        let key = event.cid.to_hex();
        if self.events.contains_key(&key)? {
            // Idempotent: same event already present — no-op.
            return Ok(());
        }

        let value = serde_json::to_vec(event)?;
        let height_bytes = event.height.to_be_bytes();

        let mut batch = sled::Batch::default();
        batch.insert(key.as_bytes(), value.as_slice());
        batch.insert(KEY_HEAD, event.cid.to_hex().as_bytes());
        batch.insert(KEY_HEIGHT, &height_bytes);

        self.events.apply_batch(batch)?;
        self.events.flush()?;
        Ok(())
    }

    /// Retrieve an event by CID.
    ///
    /// Returns `None` if the CID is not in the DAG (not an error — callers
    /// use this to detect chain breaks during verification).
    pub fn get(&self, cid: &Cid) -> BifrostResult<Option<Event>> {
        match self.events.get(cid.to_hex())? {
            None => Ok(None),
            Some(bytes) => {
                let ev: Event = serde_json::from_slice(&bytes)?;
                Ok(Some(ev))
            }
        }
    }

    /// Return the current chain head CID, or `None` if the chain is empty.
    pub fn head(&self) -> BifrostResult<Option<Cid>> {
        match self.events.get(KEY_HEAD)? {
            None => Ok(None),
            Some(bytes) => {
                let hex = std::str::from_utf8(&bytes)
                    .map_err(|e| BifrostError::Serialize(e.to_string()))?;
                Ok(Some(Cid::from_hex(hex)?))
            }
        }
    }

    /// Return the current chain height (number of events − 1; genesis = 0).
    pub fn height(&self) -> BifrostResult<u64> {
        match self.events.get(KEY_HEIGHT)? {
            None => Ok(0),
            Some(bytes) => {
                if bytes.len() != 8 {
                    return Err(BifrostError::Serialize("invalid height bytes".into()));
                }
                Ok(u64::from_be_bytes(bytes.as_ref().try_into().unwrap()))
            }
        }
    }

    /// Return `true` if the DAG contains this CID.
    pub fn contains(&self, cid: &Cid) -> BifrostResult<bool> {
        Ok(self.events.contains_key(cid.to_hex())?)
    }

    /// Walk the chain from `head` backward to genesis, yielding CIDs in
    /// *reverse* order (most-recent first).  Stops at the first missing prev.
    pub fn walk_from(&self, head: &Cid) -> BifrostResult<Vec<Cid>> {
        let mut chain = Vec::new();
        let mut cur = Some(*head);
        while let Some(cid) = cur {
            chain.push(cid);
            match self.get(&cid)? {
                None => break,
                Some(ev) => cur = ev.prev,
            }
        }
        Ok(chain)
    }

    /// Total number of events stored (O(n) scan — use for tests/tooling only).
    #[cfg(test)]
    pub fn event_count(&self) -> usize {
        self.events.iter()
            .filter(|r| {
                if let Ok((k, _)) = r {
                    k.as_ref() != KEY_HEAD && k.as_ref() != KEY_HEIGHT
                } else { false }
            })
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::{EventPayload, PubKey, Sig, CanonicalEnvelope};
    use ed25519_dalek::{SigningKey, Signer};
    use rand::rngs::OsRng;
    use tempfile::tempdir;

    fn make_genesis(sk: &SigningKey) -> Event {
        let pubkey = PubKey(sk.verifying_key().to_bytes());
        let dummy_cid = Cid::of(b"dummy_policy");
        let payload = EventPayload::CapTransfer {
            from: Cid::of(b"root"), to: Cid::of(b"child"),
            cap_hash: Cid::of(b"cap"), policy_cid: dummy_cid,
        };
        let env = CanonicalEnvelope {
            height: 0, timestamp: 1, prev: None, pubkey: &pubkey, payload: &payload,
        };
        let cid = Cid::of(&serde_json::to_vec(&env).unwrap());
        let sig = Sig(sk.sign(cid.as_bytes()).to_bytes());
        Event { cid, prev: None, payload, sig, pubkey, height: 0, timestamp: 1 }
    }

    fn make_next(sk: &SigningKey, prev: &Event, height: u64) -> Event {
        let pubkey = PubKey(sk.verifying_key().to_bytes());
        let payload = EventPayload::JitCompile {
            soulir_cid: Cid::of(b"soulir"), wasm_cid: Cid::of(b"wasm"),
            opt_level: 1, gas_estimate: 1000,
        };
        let env = CanonicalEnvelope {
            height, timestamp: height * 1000,
            prev: Some(&prev.cid), pubkey: &pubkey, payload: &payload,
        };
        let cid = Cid::of(&serde_json::to_vec(&env).unwrap());
        let sig = Sig(sk.sign(cid.as_bytes()).to_bytes());
        Event {
            cid, prev: Some(prev.cid), payload, sig, pubkey,
            height, timestamp: height * 1000,
        }
    }

    #[test]
    fn insert_and_get() {
        let dir = tempdir().unwrap();
        let store = DagStore::open(dir.path()).unwrap();
        let sk = SigningKey::generate(&mut OsRng);
        let genesis = make_genesis(&sk);

        store.insert(&genesis).unwrap();
        let retrieved = store.get(&genesis.cid).unwrap().unwrap();
        assert_eq!(retrieved.cid, genesis.cid);
    }

    #[test]
    fn head_tracks_latest() {
        let dir = tempdir().unwrap();
        let store = DagStore::open(dir.path()).unwrap();
        let sk = SigningKey::generate(&mut OsRng);
        let g = make_genesis(&sk);
        let e1 = make_next(&sk, &g, 1);
        let e2 = make_next(&sk, &e1, 2);

        store.insert(&g).unwrap();
        store.insert(&e1).unwrap();
        store.insert(&e2).unwrap();

        assert_eq!(store.head().unwrap().unwrap(), e2.cid);
        assert_eq!(store.height().unwrap(), 2);
    }

    #[test]
    fn walk_reconstructs_chain() {
        let dir = tempdir().unwrap();
        let store = DagStore::open(dir.path()).unwrap();
        let sk = SigningKey::generate(&mut OsRng);
        let g = make_genesis(&sk);
        let e1 = make_next(&sk, &g, 1);
        let e2 = make_next(&sk, &e1, 2);

        for ev in [&g, &e1, &e2] { store.insert(ev).unwrap(); }

        let chain = store.walk_from(&e2.cid).unwrap();
        assert_eq!(chain.len(), 3);
        assert_eq!(chain[0], e2.cid);
        assert_eq!(chain[2], g.cid);
    }

    #[test]
    fn idempotent_insert() {
        let dir = tempdir().unwrap();
        let store = DagStore::open(dir.path()).unwrap();
        let sk = SigningKey::generate(&mut OsRng);
        let g = make_genesis(&sk);
        store.insert(&g).unwrap();
        store.insert(&g).unwrap(); // second insert must not panic or corrupt
        assert_eq!(store.event_count(), 1);
    }
}
