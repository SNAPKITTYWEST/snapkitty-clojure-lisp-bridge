//! Constraint language definition

#[derive(Debug, Clone)]
pub enum Constraint {
    Balance(String),
    InvariantPreserved(String),
    Entropy(f64),
    ProofCertificate(String),
    RuntimeSource(String),
    IdentitySignature(String),
    PhaseComplete(String),
}

#[derive(Debug, Clone)]
pub struct ValidityProgram {
    pub constraints: Vec<Constraint>,
    pub otherwise_reject: bool,
}

impl ValidityProgram {
    pub fn new() -> Self {
        ValidityProgram {
            constraints: Vec::new(),
            otherwise_reject: false,
        }
    }

    pub fn add_constraint(&mut self, constraint: Constraint) {
        self.constraints.push(constraint);
    }

    pub fn set_reject_on_fail(&mut self, reject: bool) {
        self.otherwise_reject = reject;
    }
}

impl Default for ValidityProgram {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn program_creation() {
        let mut prog = ValidityProgram::new();
        prog.add_constraint(Constraint::Balance("test".to_string()));
        assert_eq!(prog.constraints.len(), 1);
    }
}
