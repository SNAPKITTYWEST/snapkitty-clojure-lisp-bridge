//! Jordan product composition

use super::spin_factor::SpinFactor;

/// Compose two spin-factor elements
pub fn compose(x: &SpinFactor, y: &SpinFactor) -> SpinFactor {
    x.jordan_product(y)
}

/// Verify associativity is broken (non-associative algebra)
pub fn verify_non_associativity(a: &SpinFactor, b: &SpinFactor, c: &SpinFactor) -> bool {
    let ab_c = a.jordan_product(b).jordan_product(c);
    let a_bc = a.jordan_product(&b.jordan_product(c));

    // Non-associative: should typically differ
    (ab_c.alpha - a_bc.alpha).abs() > 1e-8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn composition() {
        let a = SpinFactor::new(1.0, vec![0.0]);
        let b = SpinFactor::new(1.0, vec![1.0]);
        let result = compose(&a, &b);
        assert!(result.norm() > 0.0);
    }
}
