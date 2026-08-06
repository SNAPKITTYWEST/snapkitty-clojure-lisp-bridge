//! SLA Ledger structure and construction

use crate::Result;
use hyperkitty_core::Error;

/// Symbolic Ledger: λ = (s, δ, ι, ω)
/// Hard invariant: ι = -δ
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Ledger {
    pub s: i32,      // scalar/source component
    pub delta: i32,   // delta (differential)
    pub iota: i32,    // iota (must equal -delta)
    pub omega: i32,   // omega (conserved value)
}

impl Ledger {
    /// Create a new balanced ledger
    pub fn new(s: i32, delta: i32, omega: i32) -> Result<Self> {
        let iota = -delta;
        let ledger = Ledger {
            s,
            delta,
            iota,
            omega,
        };

        // Verify invariant holds
        if ledger.iota != -ledger.delta {
            return Err(Error::SLABalance(
                "Invariant violated: iota != -delta".to_string(),
            ));
        }

        Ok(ledger)
    }

    /// Verify the hard invariant
    pub fn verify_invariant(&self) -> bool {
        self.iota == -self.delta
    }

    /// Compute balance: δ + ι (must be 0)
    pub fn balance(&self) -> i32 {
        self.delta + self.iota
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_ledger() {
        let ledger = Ledger::new(100, 50, 42).unwrap();
        assert!(ledger.verify_invariant());
        assert_eq!(ledger.balance(), 0);
    }

    #[test]
    fn invariant_enforcement() {
        // Manually construct invalid ledger
        let bad = Ledger {
            s: 100,
            delta: 50,
            iota: 50, // Wrong! Should be -50
            omega: 42,
        };
        assert!(!bad.verify_invariant());
    }

    #[test]
    fn round_trip() {
        let ledger = Ledger::new(200, -75, 99).unwrap();
        assert_eq!(ledger.balance(), 0);
        assert_eq!(ledger.iota, 75); // iota = -(-75) = 75
    }
}
