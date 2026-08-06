//! Partial composition for SLA ledgers

use super::ledger::Ledger;
use crate::Result;
use hyperkitty_core::Error;

/// Partial composition: λ_A ⊕ λ_B defined iff ω_A = ω_B
pub fn partial_compose(a: &Ledger, b: &Ledger) -> Result<Ledger> {
    // Check conserved values match
    if a.omega != b.omega {
        return Err(Error::SLABalance(format!(
            "Cannot compose: omega mismatch {} != {}",
            a.omega, b.omega
        )));
    }

    // Compose by adding deltas (which maintains invariant)
    Ledger::new(a.s + b.s, a.delta + b.delta, a.omega)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_composition() {
        let a = Ledger::new(100, 50, 42).unwrap();
        let b = Ledger::new(50, 25, 42).unwrap();
        let result = partial_compose(&a, &b).unwrap();
        assert_eq!(result.s, 150);
        assert_eq!(result.delta, 75);
        assert_eq!(result.omega, 42);
    }

    #[test]
    fn omega_mismatch() {
        let a = Ledger::new(100, 50, 42).unwrap();
        let b = Ledger::new(50, 25, 99).unwrap();
        assert!(partial_compose(&a, &b).is_err());
    }

    #[test]
    fn composition_preserves_invariant() {
        let a = Ledger::new(100, 50, 42).unwrap();
        let b = Ledger::new(50, -30, 42).unwrap();
        let result = partial_compose(&a, &b).unwrap();
        assert!(result.verify_invariant());
    }
}
