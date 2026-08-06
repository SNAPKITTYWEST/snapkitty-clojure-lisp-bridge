//! Density Matrix for Jordan algebra evolution
use hyperkitty_core::{GOLDEN_RATIO_INV, GOLDEN_RATIO_INV_SQ};

#[derive(Debug, Clone)]
pub struct DensityMatrix {
    pub data: Vec<Vec<f64>>,
}

impl DensityMatrix {
    pub fn new(n: usize) -> Self {
        Self { data: vec![vec![0.0; n]; n] }
    }

    pub fn identity(n: usize) -> Self {
        let mut d = Self::new(n);
        for i in 0..n {
            d.data[i][i] = 1.0;
        }
        d
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn multiply(&self, other: &Self) -> Self {
        let n = self.size();
        let mut result = Self::new(n);
        for i in 0..n {
            for j in 0..n {
                let mut sum = 0.0;
                for k in 0..n {
                    sum += self.data[i][k] * other.data[k][j];
                }
                result.data[i][j] = sum;
            }
        }
        result
    }

    pub fn scale(&self, s: f64) -> Self {
        let mut result = self.clone();
        for row in &mut result.data {
            for val in row {
                *val *= s;
            }
        }
        result
    }

    pub fn add(&self, other: &Self) -> Self {
        let n = self.size();
        let mut result = Self::new(n);
        for i in 0..n {
            for j in 0..n {
                result.data[i][j] = self.data[i][j] + other.data[i][j];
            }
        }
        result
    }

    pub fn transpose(&self) -> Self {
        let n = self.size();
        let mut result = Self::new(n);
        for i in 0..n {
            for j in 0..n {
                result.data[i][j] = self.data[j][i];
            }
        }
        result
    }

    pub fn evolve(&self, u: &Self) -> Self {
        let ut = u.transpose();
        let urho = u.multiply(self);
        let urhou = urho.multiply(&ut);
        let term1 = urhou.scale(GOLDEN_RATIO_INV);
        let term2 = self.scale(GOLDEN_RATIO_INV_SQ);
        term1.add(&term2)
    }

    pub fn frobenius_norm(&self) -> f64 {
        self.data.iter().flat_map(|row| row).map(|x| x * x).sum::<f64>().sqrt()
    }

    pub fn distance_to(&self, other: &Self) -> f64 {
        self.add(&other.scale(-1.0)).frobenius_norm()
    }

    pub fn trace(&self) -> f64 {
        let n = self.size();
        (0..n).map(|i| self.data[i][i]).sum()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_trace() {
        let id = DensityMatrix::identity(3);
        assert!((id.trace() - 3.0).abs() < 1e-10);
    }
}
