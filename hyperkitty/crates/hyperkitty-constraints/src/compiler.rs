//! DSL to constraint compilation

use super::language::ValidityProgram;
use crate::Result;
use hyperkitty_core::Error;

pub fn compile(source: &str) -> Result<ValidityProgram> {
    // Stub: simplified compilation (parser integration deferred)
    let mut prog = ValidityProgram::new();

    if source.contains("balance") {
        prog.add_constraint(super::language::Constraint::Balance("main".to_string()));
    }
    if source.contains("reject") {
        prog.set_reject_on_fail(true);
    }

    Ok(prog)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_simple() {
        let prog = compile("require balance").unwrap();
        assert!(!prog.constraints.is_empty());
    }

    #[test]
    fn compile_with_reject() {
        let prog = compile("require balance otherwise reject").unwrap();
        assert!(prog.otherwise_reject);
    }
}
