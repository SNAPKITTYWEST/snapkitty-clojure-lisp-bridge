//! SLA Ledger → QRA Routing class conversion (stub for Phase 02)

use crate::Result;

/// Map SLA ledger to QRA glyph via ω classification
pub fn convert(ledger: &(i32, i32, i32, i32)) -> Result<u8> {
    // Returns glyph index (0-5)
    let (_, _, _, omega) = ledger;
    let idx = match omega {
        10 => 0, // Pi
        20 => 1, // Gamma
        30 => 2, // Delta
        40 => 3, // Omega
        50 => 4, // Lambda
        60 => 5, // Psi
        _ => 3,  // Default to absorber
    };
    Ok(idx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_ledger() {
        let ledger = (0, 5, -5, 10);
        let glyph = convert(&ledger).unwrap();
        assert_eq!(glyph, 0); // Pi
    }
}
