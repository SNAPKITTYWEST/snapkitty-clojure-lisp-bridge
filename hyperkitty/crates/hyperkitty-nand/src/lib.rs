//! # HyperKitty NAND Kernel
//!
//! Minimal Boolean trust foundation. ALL validity-predicate Boolean logic
//! is built from NAND — the single functionally complete gate.
//!
//! ## Architecture
//!
//! - **primitive** — NAND gate and all derived operations (NOT, AND, OR, XOR,
//!   IMPLIES, NOR, XNOR), each built exclusively from `nand()` calls.
//! - **boolean** — BoolExpr AST with evaluation, truth table generation,
//!   and NAND lowering.
//! - **conflicts** — Expert conflict registry for pairwise conflict declaration.
//! - **filter** — NANDFilter: deterministic conflict resolution engine that
//!   suppresses incompatible experts based on routing weights.

pub mod primitive;
pub mod boolean;
pub mod conflicts;
pub mod filter;

// Re-export primary types for ergonomic use
pub use primitive::{nand, not, and, or, xor, implies, nor, xnor};
pub use boolean::{BoolExpr, eval, truth_table, lower_to_nand, functionally_equivalent};
pub use conflicts::{ExpertId, ConflictRegistry};
pub use filter::{ExpertActivation, NANDFilter};

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_primitive_matches_ast_evaluation() {
        // Verify that primitive functions produce the same results
        // as AST evaluation for all 2-input combinations
        for &a in &[false, true] {
            for &b in &[false, true] {
                let bindings = &[a, b];

                // NAND
                let nand_expr = BoolExpr::nand(BoolExpr::Var(0), BoolExpr::Var(1));
                assert_eq!(nand(a, b), eval(&nand_expr, bindings));

                // AND
                let and_expr = BoolExpr::from_and(BoolExpr::Var(0), BoolExpr::Var(1));
                assert_eq!(and(a, b), eval(&and_expr, bindings));

                // OR
                let or_expr = BoolExpr::from_or(BoolExpr::Var(0), BoolExpr::Var(1));
                assert_eq!(or(a, b), eval(&or_expr, bindings));

                // XOR
                let xor_expr = BoolExpr::from_xor(BoolExpr::Var(0), BoolExpr::Var(1));
                assert_eq!(xor(a, b), eval(&xor_expr, bindings));

                // IMPLIES
                let imp_expr = BoolExpr::from_implies(BoolExpr::Var(0), BoolExpr::Var(1));
                assert_eq!(implies(a, b), eval(&imp_expr, bindings));
            }
        }
    }

    #[test]
    fn test_not_primitive_matches_ast() {
        for &a in &[false, true] {
            let not_expr = BoolExpr::from_not(BoolExpr::Var(0));
            assert_eq!(not(a), eval(&not_expr, &[a]));
        }
    }

    #[test]
    fn test_full_pipeline_conflict_resolution() {
        // Build a scenario: 4 experts, two conflict pairs
        let mut reg = ConflictRegistry::new();
        reg.register_conflict(ExpertId(10), ExpertId(20));
        reg.register_conflict(ExpertId(30), ExpertId(40));

        let filter = NANDFilter::new(reg);

        let activations = vec![
            ExpertActivation::new(ExpertId(10), 0.9),
            ExpertActivation::new(ExpertId(20), 0.4),
            ExpertActivation::new(ExpertId(30), 0.6),
            ExpertActivation::new(ExpertId(40), 0.6), // equal weight with 30
        ];

        let result = filter.filter(&activations);

        // (10, 20): suppress 20 (lower weight 0.4)
        // (30, 40): equal weights, suppress lower ID (30)
        let ids: Vec<ExpertId> = result.iter().map(|a| a.id).collect();
        assert!(ids.contains(&ExpertId(10)));
        assert!(!ids.contains(&ExpertId(20)));
        assert!(!ids.contains(&ExpertId(30)));
        assert!(ids.contains(&ExpertId(40)));
    }

    #[test]
    fn test_validity_predicate_as_nand_tree() {
        // Construct a validity predicate: (a AND b) IMPLIES (NOT c OR d)
        // This represents: "if experts A and B are both active, then
        // either C is inactive or D is active"
        let a_and_b = BoolExpr::from_and(BoolExpr::Var(0), BoolExpr::Var(1));
        let not_c = BoolExpr::from_not(BoolExpr::Var(2));
        let not_c_or_d = BoolExpr::from_or(not_c, BoolExpr::Var(3));
        let predicate = BoolExpr::from_implies(a_and_b, not_c_or_d);

        // Verify it's pure NAND
        assert!(predicate.is_pure_nand());

        // Verify truth table properties
        let table = truth_table(&predicate, 4);
        assert_eq!(table.len(), 16); // 2^4 rows

        // Key case: a=true, b=true, c=true, d=false => false
        // (both active, c active, d inactive violates the predicate)
        assert_eq!(eval(&predicate, &[true, true, true, false]), false);

        // Key case: a=true, b=true, c=false, d=false => true
        // (both active, c inactive satisfies regardless of d)
        assert_eq!(eval(&predicate, &[true, true, false, false]), true);

        // Key case: a=false, b=true, c=true, d=false => true
        // (antecedent false, so implication holds)
        assert_eq!(eval(&predicate, &[false, true, true, false]), true);
    }

    #[test]
    fn test_nand_lowering_complex_expression() {
        // Build a complex expression and verify lowering preserves semantics
        let expr = BoolExpr::from_or(
            BoolExpr::from_and(
                BoolExpr::from_not(BoolExpr::Var(0)),
                BoolExpr::Var(1),
            ),
            BoolExpr::from_xor(BoolExpr::Var(2), BoolExpr::Var(3)),
        );

        let lowered = lower_to_nand(&expr);
        assert!(lowered.is_pure_nand());
        assert!(functionally_equivalent(&expr, &lowered, 4));
    }
}
