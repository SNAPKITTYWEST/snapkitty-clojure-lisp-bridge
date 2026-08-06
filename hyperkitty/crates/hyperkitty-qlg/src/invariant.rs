//! QLG Invariant: Membership validation and proof certificates

use crate::Point;

/// Invariant certificate: proof that point is on sphere
#[derive(Debug, Clone, PartialEq)]
pub struct Certificate {
    pub point: Point,
    pub verified: bool,
}

impl Certificate {
    /// Create and verify certificate
    pub fn new(p: &Point) -> Result<Self, String> {
        let sum = p.0 * p.0 + p.1 * p.1 + p.2 * p.2;
        if sum == 1 {
            Ok(Certificate {
                point: *p,
                verified: true,
            })
        } else {
            Err(format!("Point {:?} not on sphere (sum={})", p, sum))
        }
    }

    /// Verify certificate is still valid
    pub fn validate(&self) -> bool {
        self.verified
            && self.point.0 * self.point.0
                + self.point.1 * self.point.1
                + self.point.2 * self.point.2
                == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_certificate() {
        let p = Point(1, 0, 0);
        let cert = Certificate::new(&p).unwrap();
        assert!(cert.validate());
    }

    #[test]
    fn invalid_certificate() {
        let p = Point(1, 1, 0);
        let result = Certificate::new(&p);
        assert!(result.is_err());
    }

    #[test]
    fn certificate_persistence() {
        let p = Point(0, 1, 0);
        let cert = Certificate::new(&p).unwrap();
        assert!(cert.validate());
        assert!(cert.validate()); // Call twice
    }
}
