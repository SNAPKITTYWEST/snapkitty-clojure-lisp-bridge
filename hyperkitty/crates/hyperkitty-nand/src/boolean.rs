//! Boolean Expression AST — compound expressions with NAND lowering.
//!
//! Provides a tree-structured representation of Boolean expressions,
//! evaluation against variable bindings, truth table generation,
//! and lowering of arbitrary expressions to pure NAND form.

use crate::primitive;

/// A Boolean expression built from variables, constants, and NAND gates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoolExpr {
    /// A variable identified by index into the binding array
    Var(usize),
    /// A Boolean constant
    Const(bool),
    /// NAND of two sub-expressions — the universal primitive
    Nand(Box<BoolExpr>, Box<BoolExpr>),
}

impl BoolExpr {
    /// Create a variable reference
    pub fn var(idx: usize) -> Self {
        BoolExpr::Var(idx)
    }

    /// Create a constant
    pub fn constant(val: bool) -> Self {
        BoolExpr::Const(val)
    }

    /// Create a NAND expression
    pub fn nand(a: BoolExpr, b: BoolExpr) -> Self {
        BoolExpr::Nand(Box::new(a), Box::new(b))
    }

    /// Construct NOT(a) as NAND(a, a) in the AST
    pub fn from_not(a: BoolExpr) -> Self {
        BoolExpr::Nand(Box::new(a.clone()), Box::new(a))
    }

    /// Construct AND(a, b) as NAND(NAND(a,b), NAND(a,b)) in the AST
    pub fn from_and(a: BoolExpr, b: BoolExpr) -> Self {
        let nand_ab = BoolExpr::Nand(Box::new(a.clone()), Box::new(b.clone()));
        BoolExpr::Nand(Box::new(nand_ab.clone()), Box::new(nand_ab))
    }

    /// Construct OR(a, b) as NAND(NAND(a,a), NAND(b,b)) in the AST
    pub fn from_or(a: BoolExpr, b: BoolExpr) -> Self {
        let not_a = BoolExpr::Nand(Box::new(a.clone()), Box::new(a));
        let not_b = BoolExpr::Nand(Box::new(b.clone()), Box::new(b));
        BoolExpr::Nand(Box::new(not_a), Box::new(not_b))
    }

    /// Construct XOR(a, b) purely from NAND in the AST
    pub fn from_xor(a: BoolExpr, b: BoolExpr) -> Self {
        let not_a = BoolExpr::Nand(Box::new(a.clone()), Box::new(a.clone()));
        let not_b = BoolExpr::Nand(Box::new(b.clone()), Box::new(b.clone()));
        let left = BoolExpr::Nand(Box::new(not_a), Box::new(b));
        let right = BoolExpr::Nand(Box::new(a), Box::new(not_b));
        BoolExpr::Nand(Box::new(left), Box::new(right))
    }

    /// Construct IMPLIES(a, b) = NAND(a, NAND(b, b)) in the AST
    pub fn from_implies(a: BoolExpr, b: BoolExpr) -> Self {
        let not_b = BoolExpr::Nand(Box::new(b.clone()), Box::new(b));
        BoolExpr::Nand(Box::new(a), Box::new(not_b))
    }

    /// Returns true if the expression is already in pure NAND form
    /// (only contains Var, Const, and Nand nodes — which is always true
    /// for our representation, but this validates no external extensions).
    pub fn is_pure_nand(&self) -> bool {
        match self {
            BoolExpr::Var(_) | BoolExpr::Const(_) => true,
            BoolExpr::Nand(a, b) => a.is_pure_nand() && b.is_pure_nand(),
        }
    }
}

/// Evaluate a Boolean expression given variable bindings.
///
/// # Panics
/// Panics if a `Var(idx)` references an index outside `bindings`.
pub fn eval(expr: &BoolExpr, bindings: &[bool]) -> bool {
    match expr {
        BoolExpr::Var(idx) => bindings[*idx],
        BoolExpr::Const(val) => *val,
        BoolExpr::Nand(a, b) => {
            let va = eval(a, bindings);
            let vb = eval(b, bindings);
            primitive::nand(va, vb)
        }
    }
}

/// Generate an exhaustive truth table for an expression over `n_vars` variables.
///
/// Returns a vector of (input_assignment, output) pairs.
/// Input assignments enumerate all 2^n_vars combinations in binary order.
pub fn truth_table(expr: &BoolExpr, n_vars: usize) -> Vec<(Vec<bool>, bool)> {
    let n_rows = 1usize << n_vars;
    let mut table = Vec::with_capacity(n_rows);

    for row in 0..n_rows {
        let bindings: Vec<bool> = (0..n_vars)
            .map(|var_idx| (row >> (n_vars - 1 - var_idx)) & 1 == 1)
            .collect();
        let result = eval(expr, &bindings);
        table.push((bindings, result));
    }

    table
}

/// Lower any BoolExpr to pure NAND form.
///
/// Since our AST only has Var, Const, and Nand nodes, the expression is
/// already in NAND form. This function is provided for interface completeness
/// and to serve as the identity transformation, verifying structural purity.
///
/// For compound expressions built via `from_and`, `from_or`, etc., those
/// constructors already produce NAND trees, so this is always a no-op
/// structurally — but we traverse to confirm.
pub fn lower_to_nand(expr: &BoolExpr) -> BoolExpr {
    match expr {
        BoolExpr::Var(idx) => BoolExpr::Var(*idx),
        BoolExpr::Const(val) => BoolExpr::Const(*val),
        BoolExpr::Nand(a, b) => {
            let la = lower_to_nand(a);
            let lb = lower_to_nand(b);
            BoolExpr::Nand(Box::new(la), Box::new(lb))
        }
    }
}

/// Verify that two expressions are functionally equivalent over `n_vars` variables.
pub fn functionally_equivalent(a: &BoolExpr, b: &BoolExpr, n_vars: usize) -> bool {
    let ta = truth_table(a, n_vars);
    let tb = truth_table(b, n_vars);
    ta.iter()
        .zip(tb.iter())
        .all(|((_, ra), (_, rb))| ra == rb)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_eval_const() {
        assert_eq!(eval(&BoolExpr::Const(true), &[]), true);
        assert_eq!(eval(&BoolExpr::Const(false), &[]), false);
    }

    #[test]
    fn test_eval_var() {
        assert_eq!(eval(&BoolExpr::Var(0), &[true, false]), true);
        assert_eq!(eval(&BoolExpr::Var(1), &[true, false]), false);
    }

    #[test]
    fn test_eval_nand() {
        let expr = BoolExpr::nand(BoolExpr::Var(0), BoolExpr::Var(1));
        assert_eq!(eval(&expr, &[true, true]), false);
        assert_eq!(eval(&expr, &[true, false]), true);
        assert_eq!(eval(&expr, &[false, true]), true);
        assert_eq!(eval(&expr, &[false, false]), true);
    }

    #[test]
    fn test_from_not() {
        let expr = BoolExpr::from_not(BoolExpr::Var(0));
        assert_eq!(eval(&expr, &[true]), false);
        assert_eq!(eval(&expr, &[false]), true);
    }

    #[test]
    fn test_from_and() {
        let expr = BoolExpr::from_and(BoolExpr::Var(0), BoolExpr::Var(1));
        let table = truth_table(&expr, 2);
        let expected = vec![
            (vec![false, false], false),
            (vec![false, true], false),
            (vec![true, false], false),
            (vec![true, true], true),
        ];
        assert_eq!(table, expected);
    }

    #[test]
    fn test_from_or() {
        let expr = BoolExpr::from_or(BoolExpr::Var(0), BoolExpr::Var(1));
        let table = truth_table(&expr, 2);
        let expected = vec![
            (vec![false, false], false),
            (vec![false, true], true),
            (vec![true, false], true),
            (vec![true, true], true),
        ];
        assert_eq!(table, expected);
    }

    #[test]
    fn test_from_xor() {
        let expr = BoolExpr::from_xor(BoolExpr::Var(0), BoolExpr::Var(1));
        let table = truth_table(&expr, 2);
        let expected = vec![
            (vec![false, false], false),
            (vec![false, true], true),
            (vec![true, false], true),
            (vec![true, true], false),
        ];
        assert_eq!(table, expected);
    }

    #[test]
    fn test_from_implies() {
        let expr = BoolExpr::from_implies(BoolExpr::Var(0), BoolExpr::Var(1));
        let table = truth_table(&expr, 2);
        let expected = vec![
            (vec![false, false], true),
            (vec![false, true], true),
            (vec![true, false], false),
            (vec![true, true], true),
        ];
        assert_eq!(table, expected);
    }

    #[test]
    fn test_truth_table_single_var() {
        let expr = BoolExpr::Var(0);
        let table = truth_table(&expr, 1);
        assert_eq!(table, vec![(vec![false], false), (vec![true], true)]);
    }

    #[test]
    fn test_truth_table_three_vars() {
        // (a AND b) OR c
        let a_and_b = BoolExpr::from_and(BoolExpr::Var(0), BoolExpr::Var(1));
        let expr = BoolExpr::from_or(a_and_b, BoolExpr::Var(2));
        let table = truth_table(&expr, 3);
        assert_eq!(table.len(), 8);
        // Verify specific rows
        // (false, false, false) -> false OR false = false
        assert_eq!(table[0], (vec![false, false, false], false));
        // (false, false, true) -> false OR true = true
        assert_eq!(table[1], (vec![false, false, true], true));
        // (true, true, false) -> true OR false = true
        assert_eq!(table[6], (vec![true, true, false], true));
        // (true, true, true) -> true OR true = true
        assert_eq!(table[7], (vec![true, true, true], true));
    }

    #[test]
    fn test_lower_to_nand_preserves_semantics() {
        // Build a complex expression and verify lowering preserves behavior
        let expr = BoolExpr::from_or(
            BoolExpr::from_and(BoolExpr::Var(0), BoolExpr::Var(1)),
            BoolExpr::from_not(BoolExpr::Var(2)),
        );
        let lowered = lower_to_nand(&expr);
        assert!(functionally_equivalent(&expr, &lowered, 3));
    }

    #[test]
    fn test_lower_to_nand_round_trip_all_ops() {
        // NOT
        let not_expr = BoolExpr::from_not(BoolExpr::Var(0));
        assert!(functionally_equivalent(&not_expr, &lower_to_nand(&not_expr), 1));

        // AND
        let and_expr = BoolExpr::from_and(BoolExpr::Var(0), BoolExpr::Var(1));
        assert!(functionally_equivalent(&and_expr, &lower_to_nand(&and_expr), 2));

        // OR
        let or_expr = BoolExpr::from_or(BoolExpr::Var(0), BoolExpr::Var(1));
        assert!(functionally_equivalent(&or_expr, &lower_to_nand(&or_expr), 2));

        // XOR
        let xor_expr = BoolExpr::from_xor(BoolExpr::Var(0), BoolExpr::Var(1));
        assert!(functionally_equivalent(&xor_expr, &lower_to_nand(&xor_expr), 2));

        // IMPLIES
        let imp_expr = BoolExpr::from_implies(BoolExpr::Var(0), BoolExpr::Var(1));
        assert!(functionally_equivalent(&imp_expr, &lower_to_nand(&imp_expr), 2));
    }

    #[test]
    fn test_is_pure_nand() {
        let expr = BoolExpr::from_and(BoolExpr::Var(0), BoolExpr::Var(1));
        assert!(expr.is_pure_nand());

        let complex = BoolExpr::from_or(
            BoolExpr::from_xor(BoolExpr::Var(0), BoolExpr::Var(1)),
            BoolExpr::from_implies(BoolExpr::Var(2), BoolExpr::Const(true)),
        );
        assert!(complex.is_pure_nand());
    }

    #[test]
    fn test_nested_expression_evaluation() {
        // (a IMPLIES b) AND (NOT c)
        let imp = BoolExpr::from_implies(BoolExpr::Var(0), BoolExpr::Var(1));
        let not_c = BoolExpr::from_not(BoolExpr::Var(2));
        let expr = BoolExpr::from_and(imp, not_c);

        // a=true, b=true, c=false => (true->true) AND (NOT false) = true AND true = true
        assert_eq!(eval(&expr, &[true, true, false]), true);
        // a=true, b=false, c=false => (true->false) AND (NOT false) = false AND true = false
        assert_eq!(eval(&expr, &[true, false, false]), false);
        // a=true, b=true, c=true => (true->true) AND (NOT true) = true AND false = false
        assert_eq!(eval(&expr, &[true, true, true]), false);
    }

    #[test]
    fn test_functional_equivalence() {
        // De Morgan: NOT(a AND b) == OR(NOT(a), NOT(b))
        let lhs = BoolExpr::from_not(BoolExpr::from_and(BoolExpr::Var(0), BoolExpr::Var(1)));
        let rhs = BoolExpr::from_or(
            BoolExpr::from_not(BoolExpr::Var(0)),
            BoolExpr::from_not(BoolExpr::Var(1)),
        );
        assert!(functionally_equivalent(&lhs, &rhs, 2));
    }
}
