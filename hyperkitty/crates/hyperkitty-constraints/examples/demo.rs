use hyperkitty_constraints::{compile, Evaluator};
use std::collections::HashMap;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== HyperKitty Constraint Language (HKCL) Compiler Demo ===\n");

    // Example 1: Simple signature validation
    println!("Example 1: Signature Validation");
    println!("{}", "-".repeat(50));
    let source1 = r#"
        validity(SigValid) "signature must be valid" {
            require signature_check();
            require not_expired();
            otherwise reject;
        }
    "#;

    let program1 = compile(source1)?;
    let evaluator1 = Evaluator::new(program1);

    let mut bindings1 = HashMap::new();
    // These built-in checks would normally come from external verification
    bindings1.insert("signature_check".to_string(), true);
    bindings1.insert("not_expired".to_string(), true);

    let result1 = evaluator1.evaluate(&bindings1)?;
    println!("Source:\n{}\n", source1);
    println!("Result: {}\n", if result1 { "PASS" } else { "FAIL" });
    println!("{}\n", evaluator1.report(&bindings1));

    // Example 2: Multiple constraints with mixed results
    println!("\nExample 2: Multi-Constraint Validation");
    println!("{}", "-".repeat(50));
    let source2 = r#"
        validity(V1) "balance check" {
            require check_balance();
            otherwise reject;
        }

        validity(V2) "authorization check" {
            require is_authorized();
            otherwise accept;
        }

        validity(V3) "audit check" {
            require audit_passed();
            otherwise reject;
        }
    "#;

    let program2 = compile(source2)?;
    let evaluator2 = Evaluator::new(program2);

    let mut bindings2 = HashMap::new();
    bindings2.insert("check_balance".to_string(), true);
    bindings2.insert("is_authorized".to_string(), false); // Will fail but "otherwise accept"
    bindings2.insert("audit_passed".to_string(), true);

    let result2 = evaluator2.evaluate(&bindings2)?;
    println!("Constraints defined: {}", evaluator2.program.constraints.len());
    println!("Passing constraints: {}\n", evaluator2.count_passing(&bindings2));
    println!("Overall result: {}\n", if result2 { "PASS" } else { "FAIL" });
    println!("{}", evaluator2.report(&bindings2));

    // Example 3: Demonstrating NAND lowering comment
    println!("\n\nExample 3: Constraint Lowering (FUTURE)");
    println!("{}", "-".repeat(50));
    println!("Current evaluator uses boolean algebra.");
    println!("Future NAND lowering will:");
    println!("  1. Convert all requirements to NAND-tree representations");
    println!("  2. Generate Blake3/Ed25519 cryptographic commitments");
    println!("  3. Persist evaluation traces to immutable WORM ledger");
    println!("  4. Enable formal verification and constraint replay");
    println!("\nExample constraint tree after lowering:");
    println!("  V1: AND(signature_check, not_expired) → NAND(NAND(sig, exp), 0)");
    println!("  V2: AND(is_auth, audit) → NAND(NAND(auth, audit), 0)");
    println!("\nCommitment: blake3(WORM[ledger_id][tick])");

    println!("\n=== Demo Complete ===");
    Ok(())
}
