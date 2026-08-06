//! HyperKitty Constraint Language (HKCL) Compiler
//!
//! A full compiler pipeline for constraint definitions:
//!   1. Lexer: Tokenize source code with keyword recognition and literal parsing
//!   2. Parser: Build typed AST from token stream
//!   3. AST: Immutable constraint program with named constraints and requirements
//!   4. Evaluator: Execute constraints against variable bindings
//!
//! Grammar:
//! ```text
//! program := constraint*
//! constraint := 'validity' '(' IDENT ')' param '{' requirements 'otherwise' action '}'
//! param := IDENT | STRING
//! requirements := ('require' requirement ';')*
//! requirement := IDENT '(' ')'  (* check *)
//!              | IDENT           (* predicate *)
//! action := 'reject' | 'accept'
//! ```
//!
//! Example:
//! ```ignore
//! validity(V1) "signature valid" {
//!     require sig_check();
//!     require not_revoked();
//!     otherwise reject;
//! }
//! ```
//!
//! NAND Lowering (Future Enhancement):
//! Constraints currently evaluate using boolean algebra. Future versions will:
//!   - Lower all requirements to NAND-tree representations
//!   - Generate cryptographic commitments (Blake3/Ed25519)
//!   - Persist evaluation traces to WORM ledger (immutable log)
//!   - Enable formal verification and constraint replay

pub mod lexer;
pub mod parser;
pub mod ast;
pub mod evaluator;
pub mod xslt_processor;

pub use ast::{ConstraintProgram, Constraint, Requirement, OtherwiseAction};
pub use evaluator::Evaluator;
pub use lexer::Lexer;
pub use parser::Parser;
pub use xslt_processor::{
    FormalizationMachine, InvariantRegistry, Invariant, ConstraintKind, Polarity,
    ProverArtifact, ProverStatus, CorrespondenceObligation, CorrespondenceStatus,
    AgdaIterationObligation, ExecutionSchedule, FormalizationSummary,
};

/// Compile HKCL source code into an executable constraint program.
///
/// # Steps:
/// 1. Tokenize: Source → Token stream
/// 2. Parse: Token stream → AST (ConstraintProgram)
/// 3. Validate: Check AST structure (no duplicates, valid names, etc.)
///
/// # Example:
/// ```ignore
/// let source = r#"validity(V) msg { require check(); otherwise reject; }"#;
/// let program = hyperkitty_constraints::compile(source)?;
/// ```
pub fn compile(source: &str) -> hyperkitty_core::Result<ConstraintProgram> {
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_compile_simple_program() -> hyperkitty_core::Result<()> {
        let source = r#"validity(V1) msg { require check(); otherwise reject; }"#;
        let program = compile(source)?;
        assert_eq!(program.constraints.len(), 1);
        Ok(())
    }

    #[test]
    fn test_full_pipeline() -> hyperkitty_core::Result<()> {
        let source = r#"
            validity(V1) "check1" {
                require always_true();
                otherwise reject;
            }
            validity(V2) "check2" {
                require always_true();
                otherwise accept;
            }
        "#;

        let program = compile(source)?;
        let evaluator = Evaluator::new(program);
        let bindings = HashMap::new();

        assert!(evaluator.evaluate(&bindings)?);
        Ok(())
    }

    #[test]
    fn test_multiple_requirements() -> hyperkitty_core::Result<()> {
        let source = r#"
            validity(V) msg {
                require always_true();
                require always_true();
                otherwise reject;
            }
        "#;

        let program = compile(source)?;
        assert_eq!(program.constraints[0].requires.len(), 2);
        Ok(())
    }

    #[test]
    fn test_otherwise_accept() -> hyperkitty_core::Result<()> {
        let source = r#"validity(V) msg { require always_false(); otherwise accept; }"#;
        let program = compile(source)?;
        let evaluator = Evaluator::new(program);
        let bindings = HashMap::new();

        // Requirement fails but "otherwise accept" means the constraint passes
        assert!(evaluator.evaluate(&bindings)?);
        Ok(())
    }

    #[test]
    fn test_otherwise_reject() -> hyperkitty_core::Result<()> {
        let source = r#"validity(V) msg { require always_false(); otherwise reject; }"#;
        let program = compile(source)?;
        let evaluator = Evaluator::new(program);
        let bindings = HashMap::new();

        // Requirement fails and "otherwise reject" means the constraint fails
        assert!(!evaluator.evaluate(&bindings)?);
        Ok(())
    }
}
