/// Decode module — delegates to WormRecord::from_bytes()
use crate::record::WormRecord;

/// Deserialize a WormRecord from a byte slice.
pub fn decode_record(bytes: &[u8]) -> hyperkitty_core::Result<WormRecord> {
    WormRecord::from_bytes(bytes)
}
