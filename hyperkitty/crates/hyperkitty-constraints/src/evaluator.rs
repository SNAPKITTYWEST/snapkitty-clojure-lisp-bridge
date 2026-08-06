use crate::ast::ConstraintProgram;
use std::collections::HashMap;

/// Evaluator for constraint programs.
///
/// The Evaluator executes a compiled ConstraintProgram against a set of variable bindings.
/// Each constraint is tested: if all its requirements pass, evaluation succeeds. If any
/// requirement fails, the "otherwise" action determines the constraint's result (reject = false,
/// accept = true).
///
/// NAND Lowering (Future Enhancement):
/// The current evaluator uses boolean algebra directly. Future versions will lower constraints
/// to NAND gates for cryptographic finality. Mapping:
///   - Predicate(x) → lookup x in bindings
///   - Check(name) → built-in gate or binding lookup
///   - AND of requirements → cascaded NAND implementations
///   - Constraint evaluation → NAND commitment to immutable ledger (WORM)
pub struct Evaluator {
    pub program: ConstraintProgram,
}

impl Evaluator {
    pub fn new(program: ConstraintProgram) -> Self {
        Self { program }
    }

    /// Evaluate the constraint program against provided bindings.
    ///
    /// Returns Ok(true) if all constraints pass, Ok(false) if any fails.
    /// Returns Err only on internal evaluation errors.
    pub fn evaluate(&self, bindings: &HashMap<String, bool>) -> hyperkitty_core::Result<bool> {
        Ok(self.program.evaluate(bindings))
    }

    /// Evaluate a single constraint by name.
    ///
    /// Useful for debugging or selective constraint testing.
    pub fn evaluate_constraint(
        &self,
        name: &str,
        bindings: &HashMap<String, bool>,
    ) -> hyperkitty_core::Result<bool> {
        for constraint in &self.program.constraints {
            if constraint.name == name {
                return Ok(constraint.evaluate(bindings));
            }
        }
        Err(hyperkitty_core::Error::Custom(
            format!("Constraint not found: {}", name)
        ))
    }

    /// Count how many constraints pass.
    pub fn count_passing(&self, bindings: &HashMap<String, bool>) -> usize {
        self.program
            .constraints
            .iter()
            .filter(|c| c.evaluate(bindings))
            .count()
    }

    /// Get detailed evaluation report for debugging.
    pub fn report(&self, bindings: &HashMap<String, bool>) -> String {
        let mut report = String::new();
        report.push_str(&format!(
            "Constraint Evaluation Report ({} constraints)\n",
            self.program.constraints.len()
        ));
        report.push_str(&"=".repeat(60));
        report.push('\n');

        for constraint in &self.program.constraints {
            let result = constraint.evaluate(bindings);
            let status = if result { "PASS" } else { "FAIL" };
            report.push_str(&format!(
                "  [{}] {}: {} (otherwise: {:?})\n",
                status, constraint.name, constraint.param, constraint.otherwise
            ));
            for req in &constraint.requires {
                report.push_str(&format!("      - {:?}\n", req));
            }
        }

        report.push_str(&"=".repeat(60));
        report.push('\n');
        report.push_str(&format!(
            "Overall: {}/{} constraints passed\n",
            self.count_passing(bindings),
            self.program.constraints.len()
        ));

        report
    }
}

impl Default for Evaluator {
    fn default() -> Self {
        Self {
            program: ConstraintProgram::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Constraint, OtherwiseAction, Requirement};

    #[test]
    fn test_evaluator_all_pass() {
        let mut prog = ConstraintProgram::new();
        let mut c = Constraint::new("test".to_string(), "msg".to_string(), OtherwiseAction::Reject);
        c.add_requirement(Requirement::Check("always_true".to_string()));
        prog.add_constraint(c);

        let evaluator = Evaluator::new(prog);
        let bindings = HashMap::new();
        assert!(evaluator.evaluate(&bindings).unwrap());
    }

    #[test]
    fn test_evaluator_one_fails() {
        let mut prog = ConstraintProgram::new();

        let mut c1 = Constraint::new("c1".to_string(), "msg".to_string(), OtherwiseAction::Reject);
        c1.add_requirement(Requirement::Check("always_true".to_string()));
        prog.add_constraint(c1);

        let mut c2 = Constraint::new("c2".to_string(), "msg".to_string(), OtherwiseAction::Reject);
        c2.add_requirement(Requirement::Check("always_false".to_string()));
        prog.add_constraint(c2);

        let evaluator = Evaluator::new(prog);
        let bindings = HashMap::new();
        assert!(!evaluator.evaluate(&bindings).unwrap());
    }

    #[test]
    fn test_evaluate_single_constraint() {
        let mut prog = ConstraintProgram::new();

        let mut c1 = Constraint::new("first".to_string(), "msg".to_string(), OtherwiseAction::Reject);
        c1.add_requirement(Requirement::Check("always_true".to_string()));
        prog.add_constraint(c1);

        let mut c2 = Constraint::new("second".to_string(), "msg".to_string(), OtherwiseAction::Reject);
        c2.add_requirement(Requirement::Check("always_false".to_string()));
        prog.add_constraint(c2);

        let evaluator = Evaluator::new(prog);
        let bindings = HashMap::new();

        assert!(evaluator.evaluate_constraint("first", &bindings).unwrap());
        assert!(!evaluator.evaluate_constraint("second", &bindings).unwrap());
    }

    #[test]
    fn test_count_passing() {
        let mut prog = ConstraintProgram::new();

        for i in 0..3 {
            let mut c = Constraint::new(
                format!("c{}", i),
                "msg".to_string(),
                OtherwiseAction::Reject,
            );
            c.add_requirement(Requirement::Check("always_true".to_string()));
            prog.add_constraint(c);
        }

        let evaluator = Evaluator::new(prog);
        let bindings = HashMap::new();
        assert_eq!(evaluator.count_passing(&bindings), 3);
    }

    #[test]
    fn test_evaluator_report() {
        let mut prog = ConstraintProgram::new();
        let mut c = Constraint::new("test".to_string(), "msg".to_string(), OtherwiseAction::Reject);
        c.add_requirement(Requirement::Check("always_true".to_string()));
        prog.add_constraint(c);

        let evaluator = Evaluator::new(prog);
        let bindings = HashMap::new();
        let report = evaluator.report(&bindings);

        assert!(report.contains("Constraint Evaluation Report"));
        assert!(report.contains("test"));
        assert!(report.contains("PASS"));
    }
}
