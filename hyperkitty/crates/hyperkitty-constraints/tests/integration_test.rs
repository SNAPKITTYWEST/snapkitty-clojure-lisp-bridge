use hyperkitty_constraints::{compile, Evaluator, Lexer, Parser};
use std::collections::HashMap;

/// Integration test demonstrating the full HKCL compiler pipeline
#[test]
fn test_full_compilation_pipeline() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
        validity(CheckSum) "all sums valid" {
            require a_valid();
            require b_valid();
            require c_valid();
            otherwise reject;
        }
        validity(CheckAuth) "authorized" {
            require is_admin();
            otherwise accept;
        }
    "#;

    // Step 1: Tokenize
    let mut lexer = Lexer::new(source);
    let tokens = lexer.tokenize()?;
    assert!(tokens.len() > 0);

    // Step 2: Parse
    let mut parser = Parser::new(tokens);
    let program = parser.parse()?;
    assert_eq!(program.constraints.len(), 2);

    // Step 3: Verify AST structure
    let checksum = &program.constraints[0];
    assert_eq!(checksum.name, "CheckSum");
    assert_eq!(checksum.param, "all sums valid");
    assert_eq!(checksum.requires.len(), 3);

    let checkauth = &program.constraints[1];
    assert_eq!(checkauth.name, "CheckAuth");
    assert_eq!(checkauth.param, "authorized");

    // Step 4: Evaluate constraints
    let evaluator = Evaluator::new(program);

    let mut bindings = HashMap::new();
    bindings.insert("a_valid".to_string(), true);
    bindings.insert("b_valid".to_string(), true);
    bindings.insert("c_valid".to_string(), true);
    bindings.insert("is_admin".to_string(), false);

    let result = evaluator.evaluate(&bindings)?;
    assert!(result); // Both constraints should pass (CheckAuth has "otherwise accept")

    // Step 5: Verify individual constraints
    assert!(evaluator.evaluate_constraint("CheckSum", &bindings)?);
    assert!(evaluator.evaluate_constraint("CheckAuth", &bindings)?);

    // Step 6: Get detailed report
    let report = evaluator.report(&bindings);
    assert!(report.contains("CheckSum"));
    assert!(report.contains("CheckAuth"));
    assert!(report.contains("2/2 constraints passed"));

    Ok(())
}

/// Test using the top-level compile() API
#[test]
fn test_compile_api() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
        validity(V1) "test" {
            require always_true();
            otherwise reject;
        }
    "#;

    let program = compile(source)?;
    assert_eq!(program.constraints.len(), 1);
    assert_eq!(program.constraints[0].name, "V1");

    let evaluator = Evaluator::new(program);
    let bindings = HashMap::new();
    assert!(evaluator.evaluate(&bindings)?);

    Ok(())
}

/// Test error handling in lexer
#[test]
fn test_lexer_error_handling() {
    let bad_source = "validity(V) \"unterminated";
    let mut lexer = Lexer::new(bad_source);
    let result = lexer.tokenize();
    assert!(result.is_err());
}

/// Test error handling in parser
#[test]
fn test_parser_error_handling() -> Result<(), Box<dyn std::error::Error>> {
    let bad_source = "validity V) msg { }"; // Missing opening paren after validity
    let mut lexer = Lexer::new(bad_source);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    let result = parser.parse();
    assert!(result.is_err());
    Ok(())
}

/// Test mixed requirement types (predicates and checks)
#[test]
fn test_mixed_requirements() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
        validity(Mixed) "mixed requirements" {
            require check1();
            require predicate_x();
            require check2();
            otherwise reject;
        }
    "#;

    let program = compile(source)?;
    let mixed = &program.constraints[0];
    assert_eq!(mixed.requires.len(), 3);

    let evaluator = Evaluator::new(program);
    let mut bindings = HashMap::new();
    bindings.insert("check1".to_string(), true);
    bindings.insert("predicate_x".to_string(), true);
    bindings.insert("check2".to_string(), true);

    assert!(evaluator.evaluate(&bindings)?);

    Ok(())
}

/// Test counting passing constraints
#[test]
fn test_constraint_counting() -> Result<(), Box<dyn std::error::Error>> {
    let source = r#"
        validity(V1) "msg" { require always_true(); otherwise reject; }
        validity(V2) "msg" { require always_true(); otherwise reject; }
        validity(V3) "msg" { require always_false(); otherwise reject; }
    "#;

    let program = compile(source)?;
    let evaluator = Evaluator::new(program);
    let bindings = HashMap::new();

    assert_eq!(evaluator.count_passing(&bindings), 2);

    Ok(())
}
