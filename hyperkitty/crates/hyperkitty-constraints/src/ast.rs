use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct ConstraintProgram {
    pub constraints: Vec<Constraint>,
}

#[derive(Debug, Clone)]
pub struct Constraint {
    pub name: String,
    pub param: String,
    pub requires: Vec<Requirement>,
    pub otherwise: OtherwiseAction,
}

#[derive(Debug, Clone)]
pub enum Requirement {
    Predicate(String),
    Check(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OtherwiseAction {
    Reject,
    Accept,
}

impl ConstraintProgram {
    pub fn new() -> Self {
        Self {
            constraints: vec![],
        }
    }

    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    pub fn evaluate(&self, bindings: &HashMap<String, bool>) -> bool {
        self.constraints.iter().all(|c| c.evaluate(bindings))
    }
}

impl Default for ConstraintProgram {
    fn default() -> Self {
        Self::new()
    }
}

impl Constraint {
    pub fn new(name: String, param: String, otherwise: OtherwiseAction) -> Self {
        Self {
            name,
            param,
            requires: vec![],
            otherwise,
        }
    }

    pub fn add_requirement(&mut self, req: Requirement) {
        self.requires.push(req);
    }

    pub fn evaluate(&self, bindings: &HashMap<String, bool>) -> bool {
        for req in &self.requires {
            if !req.evaluate(bindings) {
                // Requirement failed: apply otherwise action
                return self.otherwise == OtherwiseAction::Accept;
            }
        }
        // All requirements passed
        true
    }
}

impl Requirement {
    pub fn evaluate(&self, bindings: &HashMap<String, bool>) -> bool {
        match self {
            Requirement::Predicate(name) => bindings.get(name).copied().unwrap_or(false),
            Requirement::Check(name) => {
                // Built-in checks: always_true, always_false, etc.
                match name.as_str() {
                    "always_true" => true,
                    "always_false" => false,
                    _ => bindings.get(name).copied().unwrap_or(false),
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_constraint_program_new() {
        let prog = ConstraintProgram::new();
        assert_eq!(prog.constraints.len(), 0);
    }

    #[test]
    fn test_add_constraint() {
        let mut prog = ConstraintProgram::new();
        let c = Constraint::new("test".to_string(), "param".to_string(), OtherwiseAction::Reject);
        prog.add_constraint(c);
        assert_eq!(prog.constraints.len(), 1);
    }

    #[test]
    fn test_requirement_predicate() {
        let req = Requirement::Predicate("x".to_string());
        let mut bindings = HashMap::new();
        bindings.insert("x".to_string(), true);
        assert!(req.evaluate(&bindings));

        bindings.insert("x".to_string(), false);
        assert!(!req.evaluate(&bindings));
    }

    #[test]
    fn test_requirement_check() {
        let req = Requirement::Check("always_true".to_string());
        let bindings = HashMap::new();
        assert!(req.evaluate(&bindings));

        let req_false = Requirement::Check("always_false".to_string());
        assert!(!req_false.evaluate(&bindings));
    }

    #[test]
    fn test_constraint_evaluate_all_pass() {
        let mut c = Constraint::new("test".to_string(), "x".to_string(), OtherwiseAction::Reject);
        c.add_requirement(Requirement::Predicate("x".to_string()));
        c.add_requirement(Requirement::Check("always_true".to_string()));

        let mut bindings = HashMap::new();
        bindings.insert("x".to_string(), true);
        assert!(c.evaluate(&bindings));
    }

    #[test]
    fn test_constraint_evaluate_fail_reject() {
        let mut c = Constraint::new("test".to_string(), "x".to_string(), OtherwiseAction::Reject);
        c.add_requirement(Requirement::Check("always_false".to_string()));

        let bindings = HashMap::new();
        assert!(!c.evaluate(&bindings));
    }

    #[test]
    fn test_constraint_evaluate_fail_accept() {
        let mut c = Constraint::new("test".to_string(), "x".to_string(), OtherwiseAction::Accept);
        c.add_requirement(Requirement::Check("always_false".to_string()));

        let bindings = HashMap::new();
        assert!(c.evaluate(&bindings));
    }

    #[test]
    fn test_program_evaluate() {
        let mut prog = ConstraintProgram::new();

        let mut c1 = Constraint::new("c1".to_string(), "x".to_string(), OtherwiseAction::Reject);
        c1.add_requirement(Requirement::Check("always_true".to_string()));
        prog.add_constraint(c1);

        let mut c2 = Constraint::new("c2".to_string(), "y".to_string(), OtherwiseAction::Reject);
        c2.add_requirement(Requirement::Check("always_true".to_string()));
        prog.add_constraint(c2);

        let bindings = HashMap::new();
        assert!(prog.evaluate(&bindings));
    }
}
