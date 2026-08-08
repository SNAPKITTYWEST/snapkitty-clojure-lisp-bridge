//! Cryptographic primitives (FFI bridge for Agda-verified operations)

use blake3::Hasher;

/// BLAKE3 hash (for WORM chain attestation)
pub fn blake3_hash(data: &[u8]) -> [u8; 32] {
    let mut hasher = Hasher::new();
    hasher.update(data);
    hasher.finalize().into()
}

/// Scalar multiplication stub (real implementation would use curve25519-dalek)
pub fn scalar_mult(k: i64, point: (i64, i64)) -> (i64, i64) {
    // Placeholder: real curve ops go here
    (point.0 * k, point.1 * k)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_deterministic() {
        let h1 = blake3_hash(b"test");
        let h2 = blake3_hash(b"test");
        assert_eq!(h1, h2);
    }

    #[test]
    fn test_scalar_mult_zero() {
        let result = scalar_mult(0, (42, 99));
        assert_eq!(result, (0, 0));
    }
}
