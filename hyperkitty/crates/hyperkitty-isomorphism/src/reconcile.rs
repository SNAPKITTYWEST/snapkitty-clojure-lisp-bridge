//! Round-trip isomorphism reconciliation (stub for Phase 02)

use crate::Result;

/// Verify round-trip: A → B → C → A equals A
pub fn verify_round_trip(p: &[i32; 3]) -> Result<bool> {
    // p (QLG) → ledger (SLA)
    let ledger = super::qlg_to_sla::convert(p)?;

    // ledger (SLA) → glyph (QRA)
    let glyph = super::sla_to_qra::convert(&ledger)?;

    // glyph (QRA) → p' (QLG)
    let p_prime = super::qra_to_qlg::convert(glyph);

    // Verify invariant preserved
    Ok(p == &p_prime)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_pi() {
        let p = [1, 0, 0];
        assert!(verify_round_trip(&p).unwrap());
    }
}
