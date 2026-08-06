//! Spin Factor Algebra
//!
//! A spin factor V_n is the Jordan algebra R + R^n with product:
//!   (alpha, v) . (beta, w) = (alpha*beta + <v,w>, alpha*w + beta*v)
//!
//! This product is commutative but NOT associative.

use hyperkitty_core::float_eq;

/// Spin factor element: x = (alpha, v) where alpha in R, v in R^n
#[derive(Debug, Clone, PartialEq)]
pub struct SpinFactor {
    pub alpha: f64,
    pub v: Vec<f64>,
}

impl SpinFactor {
    /// Create a new spin factor element.
    pub fn new(alpha: f64, v: Vec<f64>) -> Self {
        Self { alpha, v }
    }

    /// Dimension of the vector part.
    pub fn dim(&self) -> usize {
        self.v.len()
    }

    /// The identity element: (1, 0)
    pub fn identity(n: usize) -> Self {
        Self {
            alpha: 1.0,
            v: vec![0.0; n],
        }
    }

    /// The zero element: (0, 0)
    pub fn zero(n: usize) -> Self {
        Self {
            alpha: 0.0,
            v: vec![0.0; n],
        }
    }

    /// Norm of the vector part: |v|
    pub fn v_norm(&self) -> f64 {
        self.v.iter().map(|x| x * x).sum::<f64>().sqrt()
    }

    /// Unit vector in the direction of v (or zero if v = 0)
    pub fn v_hat(&self) -> Vec<f64> {
        let norm = self.v_norm();
        if norm < 1e-15 {
            vec![0.0; self.v.len()]
        } else {
            self.v.iter().map(|x| x / norm).collect()
        }
    }

    /// Jordan product: (alpha, v) . (beta, w) = (alpha*beta + <v,w>, alpha*w + beta*v)
    ///
    /// This product is:
    /// - Commutative: x . y = y . x
    /// - Power-associative: x^m . x^n = x^(m+n)
    /// - NOT associative in general: (x . y) . z != x . (y . z)
    pub fn product(&self, other: &Self) -> Self {
        assert_eq!(self.v.len(), other.v.len(), "Dimension mismatch in Jordan product");

        // Scalar part: alpha*beta + <v, w>
        let dot: f64 = self.v.iter().zip(&other.v).map(|(a, b)| a * b).sum();
        let alpha = self.alpha * other.alpha + dot;

        // Vector part: alpha*w + beta*v
        let v: Vec<f64> = self
            .v
            .iter()
            .zip(&other.v)
            .map(|(&vi, &wi)| self.alpha * wi + other.alpha * vi)
            .collect();

        Self { alpha, v }
    }

    /// Scalar multiplication: c * (alpha, v) = (c*alpha, c*v)
    pub fn scale(&self, c: f64) -> Self {
        Self {
            alpha: c * self.alpha,
            v: self.v.iter().map(|x| c * x).collect(),
        }
    }

    /// Addition: (alpha, v) + (beta, w) = (alpha+beta, v+w)
    pub fn add(&self, other: &Self) -> Self {
        assert_eq!(self.v.len(), other.v.len(), "Dimension mismatch in addition");
        Self {
            alpha: self.alpha + other.alpha,
            v: self.v.iter().zip(&other.v).map(|(a, b)| a + b).collect(),
        }
    }

    /// Subtraction
    pub fn sub(&self, other: &Self) -> Self {
        assert_eq!(self.v.len(), other.v.len(), "Dimension mismatch in subtraction");
        Self {
            alpha: self.alpha - other.alpha,
            v: self.v.iter().zip(&other.v).map(|(a, b)| a - b).collect(),
        }
    }

    /// Check commutativity: x . y == y . x
    pub fn is_commutative_with(&self, other: &Self) -> bool {
        let xy = self.product(other);
        let yx = other.product(self);
        float_eq(xy.alpha, yx.alpha, 1e-10)
            && xy
                .v
                .iter()
                .zip(&yx.v)
                .all(|(a, b)| float_eq(*a, *b, 1e-10))
    }

    /// Check whether (x . y) . z == x . (y . z) — associativity test
    pub fn is_associative_triple(&self, y: &Self, z: &Self) -> bool {
        let xy = self.product(y);
        let left = xy.product(z); // (x . y) . z

        let yz = y.product(z);
        let right = self.product(&yz); // x . (y . z)

        float_eq(left.alpha, right.alpha, 1e-10)
            && left
                .v
                .iter()
                .zip(&right.v)
                .all(|(a, b)| float_eq(*a, *b, 1e-10))
    }

    /// Check if this element is idempotent: e . e = e
    pub fn is_idempotent(&self) -> bool {
        let ee = self.product(self);
        float_eq(ee.alpha, self.alpha, 1e-10)
            && ee
                .v
                .iter()
                .zip(&self.v)
                .all(|(a, b)| float_eq(*a, *b, 1e-10))
    }

    /// Approximate equality check
    pub fn approx_eq(&self, other: &Self, epsilon: f64) -> bool {
        float_eq(self.alpha, other.alpha, epsilon)
            && self.v.len() == other.v.len()
            && self
                .v
                .iter()
                .zip(&other.v)
                .all(|(a, b)| float_eq(*a, *b, epsilon))
    }
}

/// Demonstrate non-associativity of spin factors.
///
/// Returns (left, right) where left = (x . y) . z, right = x . (y . z)
/// and left != right, proving the Jordan product is non-associative.
pub fn demonstrate_non_associativity() -> (SpinFactor, SpinFactor) {
    // Choose elements that will NOT associate:
    // x = (1, [1, 0]), y = (0, [0, 1]), z = (1, [1, 1])
    let x = SpinFactor::new(1.0, vec![1.0, 0.0]);
    let y = SpinFactor::new(0.0, vec![0.0, 1.0]);
    let z = SpinFactor::new(1.0, vec![1.0, 1.0]);

    // (x . y) . z
    // x . y = (1*0 + 1*0+0*1, 1*[0,1] + 0*[1,0]) = (0, [0, 1])
    // (x . y) . z = (0*1 + 0*1+1*1, 0*[1,1] + 1*[0,1]) = (1, [0, 1])
    let xy = x.product(&y);
    let left = xy.product(&z);

    // x . (y . z)
    // y . z = (0*1 + 0*1+1*1, 0*[1,1] + 1*[0,1]) = (1, [0, 1])
    // x . (y . z) = (1*1 + 1*0+0*1, 1*[0,1] + 1*[1,0]) = (1, [1, 1])
    let yz = y.product(&z);
    let right = x.product(&yz);

    (left, right)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jordan_product_basic() {
        let x = SpinFactor::new(2.0, vec![1.0, 0.0, 0.0]);
        let y = SpinFactor::new(3.0, vec![0.0, 1.0, 0.0]);

        let xy = x.product(&y);
        // alpha = 2*3 + (1*0 + 0*1 + 0*0) = 6
        assert!(float_eq(xy.alpha, 6.0, 1e-10));
        // v = 2*[0,1,0] + 3*[1,0,0] = [3, 2, 0]
        assert!(float_eq(xy.v[0], 3.0, 1e-10));
        assert!(float_eq(xy.v[1], 2.0, 1e-10));
        assert!(float_eq(xy.v[2], 0.0, 1e-10));
    }

    #[test]
    fn test_commutativity() {
        let x = SpinFactor::new(1.5, vec![2.0, -1.0, 3.0]);
        let y = SpinFactor::new(-0.5, vec![1.0, 4.0, -2.0]);
        assert!(x.is_commutative_with(&y));
    }

    #[test]
    fn test_commutativity_many() {
        // Test with various random-ish elements
        let elements = vec![
            SpinFactor::new(1.0, vec![1.0, 0.0]),
            SpinFactor::new(0.0, vec![0.0, 1.0]),
            SpinFactor::new(-2.5, vec![3.14, -1.7]),
            SpinFactor::new(0.618, vec![-0.5, 0.5]),
        ];

        for i in 0..elements.len() {
            for j in (i + 1)..elements.len() {
                assert!(
                    elements[i].is_commutative_with(&elements[j]),
                    "Commutativity failed for elements {} and {}",
                    i,
                    j
                );
            }
        }
    }

    #[test]
    fn test_non_associativity() {
        let (left, right) = demonstrate_non_associativity();
        // They must differ
        assert!(
            !left.approx_eq(&right, 1e-10),
            "Expected non-associativity but got equal results"
        );
    }

    #[test]
    fn test_non_associativity_detailed() {
        let x = SpinFactor::new(1.0, vec![1.0, 0.0]);
        let y = SpinFactor::new(0.0, vec![0.0, 1.0]);
        let z = SpinFactor::new(1.0, vec![1.0, 1.0]);
        assert!(!x.is_associative_triple(&y, &z));
    }

    #[test]
    fn test_identity_is_identity() {
        let e = SpinFactor::identity(3);
        let x = SpinFactor::new(2.5, vec![1.0, -1.0, 0.5]);
        let ex = e.product(&x);
        assert!(ex.approx_eq(&x, 1e-10));
    }

    #[test]
    fn test_identity_is_idempotent() {
        let e = SpinFactor::identity(4);
        assert!(e.is_idempotent());
    }

    #[test]
    fn test_zero_product() {
        let z = SpinFactor::zero(3);
        let x = SpinFactor::new(5.0, vec![1.0, 2.0, 3.0]);
        let zx = z.product(&x);
        assert!(zx.approx_eq(&SpinFactor::zero(3), 1e-10));
    }

    #[test]
    fn test_scale() {
        let x = SpinFactor::new(2.0, vec![1.0, -1.0]);
        let scaled = x.scale(3.0);
        assert!(float_eq(scaled.alpha, 6.0, 1e-10));
        assert!(float_eq(scaled.v[0], 3.0, 1e-10));
        assert!(float_eq(scaled.v[1], -3.0, 1e-10));
    }

    #[test]
    fn test_add_sub() {
        let x = SpinFactor::new(1.0, vec![2.0, 3.0]);
        let y = SpinFactor::new(4.0, vec![5.0, 6.0]);
        let sum = x.add(&y);
        assert!(float_eq(sum.alpha, 5.0, 1e-10));
        assert!(float_eq(sum.v[0], 7.0, 1e-10));
        assert!(float_eq(sum.v[1], 9.0, 1e-10));

        let diff = x.sub(&y);
        assert!(float_eq(diff.alpha, -3.0, 1e-10));
        assert!(float_eq(diff.v[0], -3.0, 1e-10));
        assert!(float_eq(diff.v[1], -3.0, 1e-10));
    }

    #[test]
    fn test_v_norm() {
        let x = SpinFactor::new(0.0, vec![3.0, 4.0]);
        assert!(float_eq(x.v_norm(), 5.0, 1e-10));
    }

    #[test]
    fn test_v_hat() {
        let x = SpinFactor::new(0.0, vec![3.0, 4.0]);
        let hat = x.v_hat();
        assert!(float_eq(hat[0], 0.6, 1e-10));
        assert!(float_eq(hat[1], 0.8, 1e-10));
    }
}
