//! Primitive Idempotents for Jordan algebra

use hyperkitty_core::float_eq;

/// Primitive idempotents in a Jordan algebra
#[derive(Debug, Clone)]
pub struct PrimitiveIdempotents {
    pub elements: Vec<Vec<f64>>,
    pub orthogonal: bool,
}

impl PrimitiveIdempotents {
    pub fn new() -> Self {
        Self {
            elements: vec![],
            orthogonal: true,
        }
    }

    pub fn with_elements(elements: Vec<Vec<f64>>) -> Self {
        Self {
            elements,
            orthogonal: true,
        }
    }

    pub fn verify_idempotency(&self) -> bool {
        for e in &self.elements {
            let squared: Vec<f64> = e.iter().map(|x| x * x).collect();
            if !self.vector_equal(e, &squared) {
                return false;
            }
        }
        true
    }

    pub fn verify_orthogonality(&self) -> bool {
        for (i, e1) in self.elements.iter().enumerate() {
            for (j, e2) in self.elements.iter().enumerate() {
                if i != j && !self.vector_is_zero(&self.jordan_product(e1, e2)) {
                    return false;
                }
            }
        }
        true
    }

    fn jordan_product(&self, a: &[f64], b: &[f64]) -> Vec<f64> {
        let dot: f64 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
        a.iter().zip(b.iter()).map(|(&ai, &bi)| dot * ai + dot * bi).collect()
    }

    fn dot_product(&self, a: &[f64], b: &[f64]) -> f64 {
        a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
    }

    fn vector_square(&self, v: &[f64]) -> Vec<f64> {
        v.iter().map(|x| x * x).collect()
    }

    fn vector_equal(&self, a: &[f64], b: &[f64]) -> bool {
        if a.len() != b.len() {
            return false;
        }
        for (x, y) in a.iter().zip(b.iter()) {
            if !float_eq(*x, *y, 1e-10) {
                return false;
            }
        }
        true
    }

    fn vector_is_zero(&self, v: &[f64]) -> bool {
        v.iter().all(|&x| float_eq(x, 0.0, 1e-10))
    }

    pub fn sum_to_identity(&self) -> bool {
        if self.elements.is_empty() {
            return false;
        }
        let n = self.elements[0].len();
        let sum: Vec<f64> = self.elements.iter().fold(vec![0.0; n], |acc, e| {
            acc.iter().zip(e.iter()).map(|(&a, &b)| a + b).collect()
        });
        self.vector_equal(&sum, &vec![1.0; n])
    }

    pub fn peirce_decompose(&self, v: &[f64]) -> Vec<Vec<f64>> {
        let mut result = Vec::new();
        for e in &self.elements {
            let dot: f64 = v.iter().zip(e.iter()).map(|(&vi, &ej)| vi * ej).sum();
            let proj: Vec<f64> = e.iter().map(|&ei| dot * ei).collect();
            result.push(proj);
        }
        result
    }
}
