//! Spectral decomposition (numerical eigenvalue approximation)

use super::spin_factor::SpinFactor;

/// Simple 2-eigenvalue approximation for 2D spin factors
pub fn eigenvalues(sf: &SpinFactor) -> Option<(f64, f64)> {
    if sf.vector.len() != 2 {
        return None;
    }

    let a = sf.alpha;
    let vx = sf.vector[0];
    let vy = sf.vector[1];
    let v_norm_sq = vx * vx + vy * vy;

    let discriminant = a * a + v_norm_sq;
    let sqrt_d = discriminant.sqrt();

    let lambda_plus = a + sqrt_d;
    let lambda_minus = a - sqrt_d;

    Some((lambda_plus, lambda_minus))
}

/// Spectral gap: λ₊ - λ₋
pub fn spectral_gap(sf: &SpinFactor) -> Option<f64> {
    eigenvalues(sf).map(|(l_plus, l_minus)| l_plus - l_minus)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn eigenvalue_computation() {
        let sf = SpinFactor::new(1.0, vec![1.0, 0.0]);
        let eigs = eigenvalues(&sf);
        assert!(eigs.is_some());
    }

    #[test]
    fn gap_positive() {
        let sf = SpinFactor::new(1.0, vec![1.0, 1.0]);
        if let Some(gap) = spectral_gap(&sf) {
            assert!(gap > 0.0);
        }
    }
}
