//! QLG Point → SLA Ledger conversion (stub for Phase 02)

use crate::Result;

/// Map QLG point to SLA ledger via K (conserved value)
pub fn convert(_p: &[i32; 3]) -> Result<(i32, i32, i32, i32)> {
    // Stub: returns (s, delta, iota, omega)
    Ok((0, 5, -5, 10))
}

fn point_to_k(p: &[i32; 3]) -> i32 {
    // Map canonical points to distinct K values
    match *p {
        [1, 0, 0] => 10,
        [-1, 0, 0] => 20,
        [0, 1, 0] => 30,
        [0, -1, 0] => 40,
        [0, 0, 1] => 50,
        [0, 0, -1] => 60,
        _ => 99, // Invalid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_canonical() {
        let p = [1, 0, 0];
        let (_, delta, iota, _) = convert(&p).unwrap();
        assert_eq!(iota, -delta);
    }
}
