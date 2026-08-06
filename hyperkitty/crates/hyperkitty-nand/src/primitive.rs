//! NAND kernel primitives — ALL Boolean operations derived from NAND alone.
//!
//! The NAND gate is the single universal primitive. Every other Boolean
//! operation is constructed exclusively by composing `nand()` calls.
//! No `!`, `&&`, or `||` operators are used in derived operations.

/// The ONLY primitive: NAND(a, b) = NOT(a AND b)
///
/// This is the single gate from which all Boolean logic is derived.
/// It is functionally complete — any Boolean function can be built from it.
#[inline]
pub fn nand(a: bool, b: bool) -> bool {
    // This is the ONE place where a native Boolean operator is used.
    // Every other function calls nand() exclusively.
    !(a && b)
}

/// NOT(a) = NAND(a, a)
#[inline]
pub fn not(a: bool) -> bool {
    nand(a, a)
}

/// AND(a, b) = NOT(NAND(a, b)) = NAND(NAND(a,b), NAND(a,b))
#[inline]
pub fn and(a: bool, b: bool) -> bool {
    let n = nand(a, b);
    nand(n, n)
}

/// OR(a, b) = NAND(NOT(a), NOT(b)) = NAND(NAND(a,a), NAND(b,b))
#[inline]
pub fn or(a: bool, b: bool) -> bool {
    nand(nand(a, a), nand(b, b))
}

/// XOR(a, b) = NAND(NAND(NAND(a,a), b), NAND(a, NAND(b,b)))
///
/// Derivation: XOR = (NOT(a) AND b) OR (a AND NOT(b))
/// Expressed purely through NAND composition.
#[inline]
pub fn xor(a: bool, b: bool) -> bool {
    let not_a = nand(a, a);
    let not_b = nand(b, b);
    nand(nand(not_a, b), nand(a, not_b))
}

/// IMPLIES(a, b) = OR(NOT(a), b) — expressed purely via NAND
///
/// NOT(a) = NAND(a, a)
/// OR(NOT(a), b) = NAND(NAND(NOT(a), NOT(a)), NAND(b, b))
///              = NAND(NAND(NAND(a,a), NAND(a,a)), NAND(b,b))
///
/// Simplified: IMPLIES(a, b) = NAND(a, NOT(b)) = NAND(a, NAND(b, b))
/// (This is the standard NAND form of implication)
#[inline]
pub fn implies(a: bool, b: bool) -> bool {
    // IMPLIES(a, b) = NOT(a AND NOT(b)) = NAND(a, NAND(b, b))
    // Verification: NAND(a, NOT(b)) means "it is not the case that a is true and b is false"
    nand(a, nand(b, b))
}

/// NOR(a, b) = NOT(OR(a, b))
///
/// OR(a, b) = NAND(NAND(a,a), NAND(b,b))
/// NOR(a, b) = NOT(OR(a,b)) = NAND(OR(a,b), OR(a,b))
#[inline]
pub fn nor(a: bool, b: bool) -> bool {
    let or_ab = or(a, b);
    nand(or_ab, or_ab)
}

/// XNOR(a, b) = NOT(XOR(a, b))
///
/// Expressed purely through NAND composition.
#[inline]
pub fn xnor(a: bool, b: bool) -> bool {
    let xor_ab = xor(a, b);
    nand(xor_ab, xor_ab)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nand_truth_table() {
        assert_eq!(nand(false, false), true);
        assert_eq!(nand(false, true), true);
        assert_eq!(nand(true, false), true);
        assert_eq!(nand(true, true), false);
    }

    #[test]
    fn test_not_truth_table() {
        assert_eq!(not(false), true);
        assert_eq!(not(true), false);
    }

    #[test]
    fn test_and_truth_table() {
        assert_eq!(and(false, false), false);
        assert_eq!(and(false, true), false);
        assert_eq!(and(true, false), false);
        assert_eq!(and(true, true), true);
    }

    #[test]
    fn test_or_truth_table() {
        assert_eq!(or(false, false), false);
        assert_eq!(or(false, true), true);
        assert_eq!(or(true, false), true);
        assert_eq!(or(true, true), true);
    }

    #[test]
    fn test_xor_truth_table() {
        assert_eq!(xor(false, false), false);
        assert_eq!(xor(false, true), true);
        assert_eq!(xor(true, false), true);
        assert_eq!(xor(true, true), false);
    }

    #[test]
    fn test_implies_truth_table() {
        // a -> b is false only when a=true, b=false
        assert_eq!(implies(false, false), true);
        assert_eq!(implies(false, true), true);
        assert_eq!(implies(true, false), false);
        assert_eq!(implies(true, true), true);
    }

    #[test]
    fn test_nor_truth_table() {
        assert_eq!(nor(false, false), true);
        assert_eq!(nor(false, true), false);
        assert_eq!(nor(true, false), false);
        assert_eq!(nor(true, true), false);
    }

    #[test]
    fn test_xnor_truth_table() {
        assert_eq!(xnor(false, false), true);
        assert_eq!(xnor(false, true), false);
        assert_eq!(xnor(true, false), false);
        assert_eq!(xnor(true, true), true);
    }

    #[test]
    fn test_all_derived_ops_exhaustive() {
        // Verify every derived operation against its Boolean definition
        for &a in &[false, true] {
            for &b in &[false, true] {
                // AND
                assert_eq!(and(a, b), a & b, "AND({}, {})", a, b);
                // OR
                assert_eq!(or(a, b), a | b, "OR({}, {})", a, b);
                // XOR
                assert_eq!(xor(a, b), a ^ b, "XOR({}, {})", a, b);
                // IMPLIES: a -> b equiv !a || b
                assert_eq!(implies(a, b), !a | b, "IMPLIES({}, {})", a, b);
                // NOR
                assert_eq!(nor(a, b), !(a | b), "NOR({}, {})", a, b);
                // XNOR
                assert_eq!(xnor(a, b), !(a ^ b), "XNOR({}, {})", a, b);
            }
        }
    }

    #[test]
    fn test_de_morgan_laws_via_nand() {
        // NOT(a AND b) == OR(NOT(a), NOT(b))
        // NOT(a OR b)  == AND(NOT(a), NOT(b))
        for &a in &[false, true] {
            for &b in &[false, true] {
                assert_eq!(not(and(a, b)), or(not(a), not(b)));
                assert_eq!(not(or(a, b)), and(not(a), not(b)));
            }
        }
    }

    #[test]
    fn test_double_negation() {
        assert_eq!(not(not(false)), false);
        assert_eq!(not(not(true)), true);
    }

    #[test]
    fn test_functional_completeness() {
        // Any Boolean function of 2 inputs can be built from NAND.
        // There are 16 such functions. Verify we can express several:
        // Constant 0: AND(a, NOT(a))
        for &a in &[false, true] {
            assert_eq!(and(a, not(a)), false);
        }
        // Constant 1: OR(a, NOT(a))
        for &a in &[false, true] {
            assert_eq!(or(a, not(a)), true);
        }
    }
}
