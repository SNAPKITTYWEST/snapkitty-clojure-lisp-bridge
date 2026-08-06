use crate::header::WormHeader;

/// A complete WORM record: fixed-size header followed by variable-length payload.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WormRecord {
    pub header: WormHeader,
    pub event_type: Vec<u8>,
    pub data: Vec<u8>,
    pub metadata: Vec<u8>,
}

impl WormRecord {
    /// Create a new record from parts.
    pub fn new(
        header: WormHeader,
        event_type: Vec<u8>,
        data: Vec<u8>,
        metadata: Vec<u8>,
    ) -> Self {
        Self {
            header,
            event_type,
            data,
            metadata,
        }
    }

    /// Total size of this record in bytes when serialized.
    pub fn total_size(&self) -> usize {
        self.header.record_size()
    }

    /// Serialize this record to bytes (header + payload).
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = self.header.to_bytes();
        bytes.extend_from_slice(&self.event_type);
        bytes.extend_from_slice(&self.data);
        bytes.extend_from_slice(&self.metadata);
        bytes
    }

    /// Deserialize a record from bytes.
    pub fn from_bytes(bytes: &[u8]) -> hyperkitty_core::Result<Self> {
        let header_size = crate::header::HEADER_SIZE;
        if bytes.len() < header_size {
            return Err(hyperkitty_core::Error::StorageError(
                format!("record too short: {} < {}", bytes.len(), header_size),
            ));
        }

        // Parse header
        let header = WormHeader::from_bytes(&bytes[..header_size])?;

        // Extract payload
        let offset = header_size;
        let event_length = header.event_length as usize;
        let data_length = header.data_length as usize;
        let metadata_length = header.metadata_length as usize;

        let expected_len = offset + event_length + data_length + metadata_length;
        if bytes.len() < expected_len {
            return Err(hyperkitty_core::Error::StorageError(
                format!(
                    "record payload incomplete: {} < {}",
                    bytes.len(),
                    expected_len
                ),
            ));
        }

        let event_type = bytes[offset..offset + event_length].to_vec();
        let data = bytes[offset + event_length..offset + event_length + data_length].to_vec();
        let metadata = bytes[offset + event_length + data_length
            ..offset + event_length + data_length + metadata_length]
            .to_vec();

        Ok(Self {
            header,
            event_type,
            data,
            metadata,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn record_basic() {
        let header = WormHeader::new();
        let record = WormRecord::new(header, vec![1, 2, 3], vec![4, 5, 6], vec![7, 8, 9]);
        let bytes = record.to_bytes();
        assert!(!bytes.is_empty());
        assert!(bytes.len() >= crate::header::HEADER_SIZE);
    }

    #[test]
    fn record_round_trip() {
        let mut header = WormHeader::new();
        header.event_length = 3;
        header.data_length = 3;
        header.metadata_length = 3;

        let event_type = vec![1, 2, 3];
        let data = vec![4, 5, 6];
        let metadata = vec![7, 8, 9];

        let record = WormRecord::new(header, event_type.clone(), data.clone(), metadata.clone());
        let bytes = record.to_bytes();

        let record2 = WormRecord::from_bytes(&bytes).unwrap();
        assert_eq!(record2.event_type, event_type);
        assert_eq!(record2.data, data);
        assert_eq!(record2.metadata, metadata);
    }

    #[test]
    fn record_empty_payload() {
        let header = WormHeader::new();
        let record = WormRecord::new(header, vec![], vec![], vec![]);
        let bytes = record.to_bytes();
        let record2 = WormRecord::from_bytes(&bytes).unwrap();
        assert_eq!(record2.event_type.len(), 0);
        assert_eq!(record2.data.len(), 0);
        assert_eq!(record2.metadata.len(), 0);
    }

    #[test]
    fn record_total_size() {
        let mut header = WormHeader::new();
        header.event_length = 10;
        header.data_length = 20;
        header.metadata_length = 30;

        let record =
            WormRecord::new(header, vec![0; 10], vec![0; 20], vec![0; 30]);
        assert_eq!(
            record.total_size(),
            crate::header::HEADER_SIZE + 10 + 20 + 30
        );
    }
}
