//! QRA Glyph → QLG Point conversion (stub for Phase 02)

/// Map QRA glyph back to QLG point
pub fn convert(glyph: u8) -> [i32; 3] {
    match glyph {
        0 => [1, 0, 0],   // Pi
        1 => [-1, 0, 0],  // Gamma
        2 => [0, 1, 0],   // Delta
        3 => [0, -1, 0],  // Omega
        4 => [0, 0, 1],   // Lambda
        5 => [0, 0, -1],  // Psi
        _ => [0, 0, 0],   // Invalid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_glyph() {
        let p = convert(0);
        assert_eq!(p, [1, 0, 0]);
    }
}
