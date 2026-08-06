//! Conserved value enforcement for ω

use super::ledger::Ledger;

/// Check if omega (conserved value) is consistent
pub fn is_conserved(ledger: &Ledger) -> bool {
    // ω must not be zero (reserved value)
    ledger.omega != 0
}

/// Enforce conservation across composition
pub fn conserves_in_composition(a: &Ledger, b: &Ledger) -> bool {
    a.omega == b.omega
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonzero_conserved() {
        let ledger = Ledger {
            s: 100,
            delta: 50,
            iota: -50,
            omega: 42,
        };
        assert!(is_conserved(&ledger));
    }

    #[test]
    fn zero_not_conserved() {
        let ledger = Ledger {
            s: 100,
            delta: 50,
            iota: -50,
            omega: 0,
        };
        assert!(!is_conserved(&ledger));
    }

    #[test]
    fn composition_conservation() {
        let a = Ledger {
            s: 100,
            delta: 50,
            iota: -50,
            omega: 42,
        };
        let b = Ledger {
            s: 50,
            delta: 25,
            iota: -25,
            omega: 42,
        };
        assert!(conserves_in_composition(&a, &b));
    }
}
