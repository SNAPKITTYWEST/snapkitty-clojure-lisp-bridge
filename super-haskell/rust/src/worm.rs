//! WORM (Write Once Read Many) Chain Implementation
//! BLAKE3-based content-addressed event log

use crate::crypto::blake3_hash;
use std::collections::VecDeque;

/// WORM Chain Entry (immutable)
#[derive(Clone, Debug)]
pub struct WormEntry {
    pub payload: Vec<u8>,
    pub hash: [u8; 32],
    pub prev_hash: [u8; 32],
    pub timestamp: u64,
}

/// WORM Chain (append-only log)
pub struct WormChain {
    entries: VecDeque<WormEntry>,
    genesis_hash: [u8; 32],
}

impl WormChain {
    /// Create new WORM chain with genesis block
    pub fn new() -> Self {
        let genesis_payload = b"SnapKitty WORM Chain Genesis";
        let genesis_hash = blake3_hash(genesis_payload);

        WormChain {
            entries: VecDeque::new(),
            genesis_hash,
        }
    }

    /// Append entry (write-once, immutable)
    pub fn append(&mut self, payload: Vec<u8>) -> [u8; 32] {
        let prev_hash = self.entries.back()
            .map(|e| e.hash)
            .unwrap_or(self.genesis_hash);

        let hash = {
            let mut data = Vec::with_capacity(payload.len() + 32);
            data.extend_from_slice(&prev_hash);
            data.extend_from_slice(&payload);
            blake3_hash(&data)
        };

        let entry = WormEntry {
            payload,
            hash,
            prev_hash,
            timestamp: 0, // TODO: monotonic timestamp
        };

        self.entries.push_back(entry);
        hash
    }

    /// Verify chain integrity (all hashes valid)
    pub fn verify(&self) -> bool {
        let mut prev_hash = self.genesis_hash;

        for entry in &self.entries {
            if entry.prev_hash != prev_hash {
                return false;
            }

            let mut data = Vec::with_capacity(entry.payload.len() + 32);
            data.extend_from_slice(&entry.prev_hash);
            data.extend_from_slice(&entry.payload);
            let computed_hash = blake3_hash(&data);

            if computed_hash != entry.hash {
                return false;
            }

            prev_hash = entry.hash;
        }

        true
    }

    /// Get entry by index (read-many)
    pub fn get(&self, index: usize) -> Option<&WormEntry> {
        self.entries.get(index)
    }

    /// Number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_worm_append_verify() {
        let mut chain = WormChain::new();
        chain.append(b"entry 1".to_vec());
        chain.append(b"entry 2".to_vec());
        assert!(chain.verify());
        assert_eq!(chain.len(), 2);
    }

    #[test]
    fn test_worm_tamper_detection() {
        let mut chain = WormChain::new();
        chain.append(b"entry 1".to_vec());
        chain.append(b"entry 2".to_vec());

        // Tamper with entry
        if let Some(entry) = chain.entries.get_mut(0) {
            entry.payload = b"tampered".to_vec();
        }

        assert!(!chain.verify());
    }
}
