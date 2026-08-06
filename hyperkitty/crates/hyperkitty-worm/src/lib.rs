pub mod header;
pub mod record;
pub mod append;
pub mod verify;
pub mod encode;
pub mod decode;

pub use record::WormRecord;
pub use header::WormHeader;

/// Main WORM Chain struct
pub struct Chain {
    pub records: Vec<WormRecord>,
}

impl Chain {
    /// Create a new empty chain
    pub fn new() -> Self {
        Self {
            records: vec![],
        }
    }

    /// Append a new record to the chain
    pub fn append(
        &mut self,
        event_type: Vec<u8>,
        data: Vec<u8>,
        metadata: Vec<u8>,
        timestamp_ns: u64,
    ) -> hyperkitty_core::Result<()> {
        let mut header = WormHeader::new();
        header.event_length = event_type.len() as u16;
        header.data_length = data.len() as u32;
        header.metadata_length = metadata.len() as u32;
        header.timestamp_ns = timestamp_ns;

        // Set previous_hash to hash of last record, or zero if genesis
        if !self.records.is_empty() {
            let last_bytes = self.records.last().unwrap().to_bytes();
            header.previous_hash = Self::hash_sha256(&last_bytes);
        }

        // Compute content hash from concatenated payload
        let content = [
            event_type.as_slice(),
            data.as_slice(),
            metadata.as_slice(),
        ]
        .concat();
        header.content_hash = Self::hash_sha256(&content);

        let record = WormRecord::new(header, event_type, data, metadata);
        self.records.push(record);
        Ok(())
    }

    /// Verify the integrity of the entire chain
    pub fn verify(&self) -> hyperkitty_core::Result<bool> {
        let mut prev_hash = [0u8; 32];

        for record in &self.records {
            // Check that previous_hash links correctly
            if record.header.previous_hash != prev_hash {
                return Err(hyperkitty_core::Error::TamperDetected);
            }

            // Check that content_hash matches payload
            let content = [
                record.event_type.as_slice(),
                record.data.as_slice(),
                record.metadata.as_slice(),
            ]
            .concat();
            let computed_hash = Self::hash_sha256(&content);
            if computed_hash != record.header.content_hash {
                return Err(hyperkitty_core::Error::TamperDetected);
            }

            // Update prev_hash for next iteration
            prev_hash = Self::hash_sha256(&record.to_bytes());
        }

        Ok(true)
    }

    /// SHA256 hash helper
    fn hash_sha256(data: &[u8]) -> [u8; 32] {
        use sha2::{Digest, Sha256};
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&result);
        hash
    }

    /// Get the number of records in the chain
    pub fn len(&self) -> usize {
        self.records.len()
    }

    /// Check if the chain is empty
    pub fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

impl Default for Chain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_append_verify() {
        let mut chain = Chain::new();
        chain.append(vec![1], vec![2], vec![3], 100).unwrap();
        assert!(chain.verify().unwrap());
        assert_eq!(chain.len(), 1);
    }

    #[test]
    fn chain_multiple_records() {
        let mut chain = Chain::new();
        chain
            .append(
                b"type1".to_vec(),
                b"data1".to_vec(),
                b"meta1".to_vec(),
                100,
            )
            .unwrap();
        chain
            .append(
                b"type2".to_vec(),
                b"data2".to_vec(),
                b"meta2".to_vec(),
                200,
            )
            .unwrap();
        assert!(chain.verify().unwrap());
        assert_eq!(chain.len(), 2);
    }

    #[test]
    fn chain_empty_verify() {
        let chain = Chain::new();
        assert!(chain.verify().unwrap());
    }
}
