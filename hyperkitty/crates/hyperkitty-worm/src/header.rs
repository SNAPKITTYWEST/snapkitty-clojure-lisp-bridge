/// Binary WORM record header — 152 bytes fixed-size.
///
/// Layout (big-endian):
/// ```text
/// Field               Bytes   Offset
/// magic               4       0       "WORM" = [0x57, 0x4F, 0x52, 0x4D]
/// version             1       4
/// flags               1       5
/// event_length        2       6       (u16 big-endian)
/// data_length         4       8       (u32 big-endian)
/// metadata_length     4       12      (u32 big-endian)
/// timestamp_ns        8       16      (u64 big-endian)
/// previous_hash       32      24      Blake2b-256 of previous record
/// content_hash        32      56      Blake2b-256 of payload
/// signature           64      88      Ed25519 signature
/// ```
///
/// Total: 4+1+1+2+4+4+8+32+32+64 = 152 bytes

/// Magic bytes: ASCII "WORM"
pub const MAGIC: [u8; 4] = [0x57, 0x4F, 0x52, 0x4D];

/// Current format version
pub const VERSION: u8 = 1;

/// Fixed header size in bytes
pub const HEADER_SIZE: usize = 152;

/// Hash size (Blake2b-256)
pub const HASH_SIZE: usize = 32;

/// Signature size (Ed25519)
pub const SIGNATURE_SIZE: usize = 64;

/// The genesis record uses all-zero previous_hash
pub const GENESIS_HASH: [u8; HASH_SIZE] = [0u8; HASH_SIZE];

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WormHeader {
    pub magic: [u8; 4],
    pub version: u8,
    pub flags: u8,
    pub event_length: u16,
    pub data_length: u32,
    pub metadata_length: u32,
    pub timestamp_ns: u64,
    pub previous_hash: [u8; HASH_SIZE],
    pub content_hash: [u8; HASH_SIZE],
    pub signature: [u8; SIGNATURE_SIZE],
}

impl WormHeader {
    /// Create a new header with default magic and version.
    pub fn new() -> Self {
        Self {
            magic: MAGIC,
            version: VERSION,
            flags: 0,
            event_length: 0,
            data_length: 0,
            metadata_length: 0,
            timestamp_ns: 0,
            previous_hash: GENESIS_HASH,
            content_hash: GENESIS_HASH,
            signature: [0u8; SIGNATURE_SIZE],
        }
    }

    /// Total payload size following the header.
    pub fn payload_size(&self) -> usize {
        self.event_length as usize + self.data_length as usize + self.metadata_length as usize
    }

    /// Total record size (header + payload).
    pub fn record_size(&self) -> usize {
        HEADER_SIZE + self.payload_size()
    }

    /// Serialize header to bytes (big-endian)
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::with_capacity(HEADER_SIZE);
        bytes.extend_from_slice(&self.magic);
        bytes.push(self.version);
        bytes.push(self.flags);
        bytes.extend_from_slice(&self.event_length.to_be_bytes());
        bytes.extend_from_slice(&self.data_length.to_be_bytes());
        bytes.extend_from_slice(&self.metadata_length.to_be_bytes());
        bytes.extend_from_slice(&self.timestamp_ns.to_be_bytes());
        bytes.extend_from_slice(&self.previous_hash);
        bytes.extend_from_slice(&self.content_hash);
        bytes.extend_from_slice(&self.signature);
        bytes
    }

    /// Deserialize header from bytes (big-endian)
    pub fn from_bytes(bytes: &[u8]) -> hyperkitty_core::Result<Self> {
        if bytes.len() < HEADER_SIZE {
            return Err(hyperkitty_core::Error::StorageError(
                format!("header too short: {} < {}", bytes.len(), HEADER_SIZE),
            ));
        }

        let mut offset = 0;

        let mut magic = [0u8; 4];
        magic.copy_from_slice(&bytes[offset..offset + 4]);
        offset += 4;

        if magic != MAGIC {
            return Err(hyperkitty_core::Error::StorageError(
                "invalid magic bytes".to_string(),
            ));
        }

        let version = bytes[offset];
        offset += 1;

        if version != VERSION {
            return Err(hyperkitty_core::Error::StorageError(
                format!("unsupported version: {}", version),
            ));
        }

        let flags = bytes[offset];
        offset += 1;

        let event_length = u16::from_be_bytes([bytes[offset], bytes[offset + 1]]);
        offset += 2;

        let data_length = u32::from_be_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;

        let metadata_length = u32::from_be_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
        ]);
        offset += 4;

        let timestamp_ns = u64::from_be_bytes([
            bytes[offset],
            bytes[offset + 1],
            bytes[offset + 2],
            bytes[offset + 3],
            bytes[offset + 4],
            bytes[offset + 5],
            bytes[offset + 6],
            bytes[offset + 7],
        ]);
        offset += 8;

        let mut previous_hash = [0u8; HASH_SIZE];
        previous_hash.copy_from_slice(&bytes[offset..offset + HASH_SIZE]);
        offset += HASH_SIZE;

        let mut content_hash = [0u8; HASH_SIZE];
        content_hash.copy_from_slice(&bytes[offset..offset + HASH_SIZE]);
        offset += HASH_SIZE;

        let mut signature = [0u8; SIGNATURE_SIZE];
        signature.copy_from_slice(&bytes[offset..offset + SIGNATURE_SIZE]);

        Ok(Self {
            magic,
            version,
            flags,
            event_length,
            data_length,
            metadata_length,
            timestamp_ns,
            previous_hash,
            content_hash,
            signature,
        })
    }
}

impl Default for WormHeader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_round_trip() {
        let h = WormHeader::new();
        let bytes = h.to_bytes();
        assert_eq!(bytes.len(), HEADER_SIZE);
        let h2 = WormHeader::from_bytes(&bytes).unwrap();
        assert_eq!(h.magic, h2.magic);
        assert_eq!(h.version, h2.version);
        assert_eq!(h.flags, h2.flags);
    }

    #[test]
    fn header_magic_validation() {
        let mut bad_header = vec![0u8; HEADER_SIZE];
        bad_header[0] = 0xFF;
        let result = WormHeader::from_bytes(&bad_header);
        assert!(result.is_err());
    }

    #[test]
    fn header_with_data() {
        let mut h = WormHeader::new();
        h.event_length = 100;
        h.data_length = 200;
        h.metadata_length = 50;
        h.timestamp_ns = 12345678;
        let bytes = h.to_bytes();
        let h2 = WormHeader::from_bytes(&bytes).unwrap();
        assert_eq!(h2.event_length, 100);
        assert_eq!(h2.data_length, 200);
        assert_eq!(h2.metadata_length, 50);
        assert_eq!(h2.timestamp_ns, 12345678);
    }
}
