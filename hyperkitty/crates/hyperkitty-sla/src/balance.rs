//! Balance validation for SLA ledgers

use super::ledger::Ledger;

/// Check if ledger satisfies balance axiom: R(λ) = δ + ι = 0
pub fn is_balanced(ledger: &Ledger) -> bool {
    ledger.delta + ledger.iota == 0
}

/// Zero-crossing detection: detect if balance equation transitions through zero
pub fn crosses_zero(before: &Ledger, after: &Ledger) -> bool {
    let b_before = is_balanced(before);
    let b_after = is_balanced(after);
    b_before != b_after
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_balance() {
        let ledger = Ledger {
            s: 100,
            delta: 50,
            iota: -50,
            omega: 42,
        };
        assert!(is_balanced(&ledger));
    }

    #[test]
    fn invalid_balance() {
        let ledger = Ledger {
            s: 100,
            delta: 50,
            iota: 30,
            omega: 42,
        };
        assert!(!is_balanced(&ledger));
    }

    #[test]
    fn zero_crossing() {
        let before = Ledger {
            s: 100,
            delta: 50,
            iota: -50,
            omega: 42,
        };
        let after = Ledger {
            s: 100,
            delta: 60,
            iota: -50,
            omega: 42,
        };
        assert!(crosses_zero(&before, &after));
    }
}
