/// Encode module — delegates to WormRecord::to_bytes()
use crate::record::WormRecord;

/// Serialize a WormRecord to its binary representation.
pub fn encode_record(record: &WormRecord) -> Vec<u8> {
    record.to_bytes()
}
