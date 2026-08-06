//! Condition number: measure of numerical stability

pub fn compute_condition_number(matrix: &[Vec<f64>]) -> f64 {
    if matrix.is_empty() {
        return 1.0;
    }

    // Simplified: Frobenius norm ratio
    let norm_f = frobenius_norm(matrix);
    if norm_f < 1e-10 {
        return 1.0;
    }

    norm_f / smallest_singular_value(matrix).max(1e-10)
}

fn frobenius_norm(matrix: &[Vec<f64>]) -> f64 {
    matrix
        .iter()
        .map(|row| row.iter().map(|&x| x * x).sum::<f64>())
        .sum::<f64>()
        .sqrt()
}

fn smallest_singular_value(matrix: &[Vec<f64>]) -> f64 {
    // Stub: return minimum non-zero element
    matrix
        .iter()
        .flat_map(|row| row.iter())
        .filter(|&&x| x.abs() > 1e-10)
        .map(|&x| x.abs())
        .min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or(1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_matrix_condition() {
        let matrix = vec![vec![1.0, 0.0], vec![0.0, 1.0]];
        let cond = compute_condition_number(&matrix);
        assert!(cond.is_finite());
    }
}
