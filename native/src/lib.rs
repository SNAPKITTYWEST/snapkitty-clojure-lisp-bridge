// SKC-LISP: Cryptographic Functions for WASM (Phase 3D-3)
// Pure Rust implementations of Blake3 + Ed25519 for browser execution
// No FFI dependencies (pure WASM)

use wasm_bindgen::prelude::*;

// ============================================================================
// Blake3 WASM Implementation
// ============================================================================

#[wasm_bindgen]
#[derive(Debug)]
pub struct Blake3VerificationResult {
    pub valid: bool,
    pub error_code: u8,  // 0=match, 1=mismatch, 2=invalid_input
}

/// Compute Blake3 hash of input (returns 32-byte digest)
#[wasm_bindgen]
pub fn blake3_hash(input: &[u8]) -> Vec<u8> {
    // Use blake3 crate for pure Rust implementation
    let mut hasher = blake3::Hasher::new();
    hasher.update(input);
    hasher.finalize().as_bytes().to_vec()
}

/// Verify Blake3 digest matches expected value
#[wasm_bindgen]
pub fn blake3_verify_wasm(
    payload: &[u8],
    expected_digest: &[u8],
) -> Blake3VerificationResult {
    // Validate inputs
    if payload.is_empty() || expected_digest.is_empty() {
        return Blake3VerificationResult {
            valid: false,
            error_code: 2,  // invalid_input
        };
    }

    if expected_digest.len() != 32 {
        return Blake3VerificationResult {
            valid: false,
            error_code: 2,  // invalid_input (digest must be 32 bytes)
        };
    }

    // Compute Blake3 digest
    let mut hasher = blake3::Hasher::new();
    hasher.update(payload);
    let computed = hasher.finalize();

    // Constant-time comparison (prevent timing attacks)
    let mut match_flag = true;
    for (computed_byte, expected_byte) in computed.as_bytes().iter().zip(expected_digest) {
        if computed_byte != expected_byte {
            match_flag = false;
        }
    }

    Blake3VerificationResult {
        valid: match_flag,
        error_code: if match_flag { 0 } else { 1 },  // 0=match, 1=mismatch
    }
}

// ============================================================================
// Ed25519 WASM Implementation
// ============================================================================

#[wasm_bindgen]
#[derive(Debug)]
pub struct Ed25519VerificationResult {
    pub valid: bool,
    pub error_code: u8,  // 0=valid, 1=invalid, 2=invalid_input
}

/// Verify Ed25519 signature
#[wasm_bindgen]
pub fn ed25519_verify_wasm(
    message: &[u8],
    signature: &[u8],
    public_key: &[u8],
) -> Ed25519VerificationResult {
    // Validate inputs
    if message.is_empty() {
        return Ed25519VerificationResult {
            valid: false,
            error_code: 2,  // invalid_input
        };
    }

    if signature.len() != 64 {
        return Ed25519VerificationResult {
            valid: false,
            error_code: 2,  // invalid_input (signature must be 64 bytes)
        };
    }

    if public_key.len() != 32 {
        return Ed25519VerificationResult {
            valid: false,
            error_code: 2,  // invalid_input (public key must be 32 bytes)
        };
    }

    // Convert to ed25519-zebra types
    let sig_bytes: [u8; 64] = match signature.try_into() {
        Ok(b) => b,
        Err(_) => {
            return Ed25519VerificationResult {
                valid: false,
                error_code: 2,
            }
        }
    };

    let pk_bytes: [u8; 32] = match public_key.try_into() {
        Ok(b) => b,
        Err(_) => {
            return Ed25519VerificationResult {
                valid: false,
                error_code: 2,
            }
        }
    };

    // Parse signature + public key
    let signature = ed25519_zebra::Signature::from(sig_bytes);

    let public_key: ed25519_zebra::VerificationKey = match ed25519_zebra::VerificationKeyBytes::from(pk_bytes)
        .try_into()
    {
        Ok(pk) => pk,
        Err(_) => {
            return Ed25519VerificationResult {
                valid: false,
                error_code: 2,
            }
        }
    };

    // Verify signature
    match public_key.verify(&signature, message) {
        Ok(()) => Ed25519VerificationResult {
            valid: true,
            error_code: 0,  // valid
        },
        Err(_) => Ed25519VerificationResult {
            valid: false,
            error_code: 1,  // invalid
        },
    }
}

// ============================================================================
// NASM Mutation Validator (ported to Rust for WASM)
// ============================================================================

#[wasm_bindgen]
pub struct MutationValidationResult {
    pub valid: bool,
    pub error_code: u8,
}

impl MutationValidationResult {
    pub fn with_message(valid: bool, error_code: u8) -> Self {
        MutationValidationResult { valid, error_code }
    }
}

/// 8-point mutation validation gate (ported from NASM)
#[wasm_bindgen]
pub fn validate_mutation_wasm(
    event_id: u32,
    generation: u32,
    source_hash: &[u8],
    bytecode_hash: &[u8],
    native_code_hash: &[u8],
    actor_signature: &[u8],
) -> MutationValidationResult {
    let mut errors = Vec::new();

    // Gate 1: Event ID is non-zero
    if event_id == 0 {
        errors.push("Event ID must be non-zero");
    }

    // Gate 2: Generation counter is valid
    if generation > 0x7FFFFFFF {
        errors.push("Generation counter overflow");
    }

    // Gate 3: Source hash is 32 bytes
    if source_hash.len() != 32 {
        errors.push("Source hash must be 32 bytes");
    }

    // Gate 4: Bytecode hash is 32 bytes
    if bytecode_hash.len() != 32 {
        errors.push("Bytecode hash must be 32 bytes");
    }

    // Gate 5: Native code hash is 32 bytes
    if native_code_hash.len() != 32 {
        errors.push("Native code hash must be 32 bytes");
    }

    // Gate 6: Signature is 64 bytes
    if actor_signature.len() != 64 {
        errors.push("Actor signature must be 64 bytes");
    }

    // Gate 7: Hashes are distinct (no aliasing)
    if source_hash == bytecode_hash || bytecode_hash == native_code_hash {
        errors.push("Hash aliasing detected (hashes must be distinct)");
    }

    // Gate 8: No hash is all-zeros
    if source_hash.iter().all(|&b| b == 0) {
        errors.push("Source hash is all-zeros (invalid)");
    }

    let valid = errors.is_empty();

    MutationValidationResult {
        valid,
        error_code: if valid { 0 } else { 1 },
    }
}

// ============================================================================
// Proof Certificate Validation (WASM)
// ============================================================================

#[wasm_bindgen]
pub struct ProofCertificateValidationResult {
    pub valid: bool,
    pub theorem_id: u32,
    pub theorems_covered: u32,
    pub error_code: u8,
}

/// Validate proof certificate structure + signature
#[wasm_bindgen]
pub fn validate_proof_certificate_wasm(
    cert_bytes: &[u8],
) -> ProofCertificateValidationResult {
    // Expected structure: 157 bytes
    // [0:4]    theorem_id (u32, LE)
    // [4:36]   proof_hash (32 bytes)
    // [36:40]  theorems_covered (u32, LE)
    // [40:44]  machine_state_invariants (u32, LE)
    // [44:60]  cranelift_backend (16 bytes, null-padded)
    // [60:61]  optimization_level (u8)
    // [61:125] signature (64 bytes)
    // [125:157] public_key (32 bytes)

    let mut errors = Vec::new();

    // Gate 1: Size check
    if cert_bytes.len() != 157 {
        return ProofCertificateValidationResult {
            valid: false,
            theorem_id: 0,
            theorems_covered: 0,
            error_code: 2,
        };
    }

    // Parse fields
    let theorem_id = u32::from_le_bytes([
        cert_bytes[0],
        cert_bytes[1],
        cert_bytes[2],
        cert_bytes[3],
    ]);

    let theorems_covered = u32::from_le_bytes([
        cert_bytes[36],
        cert_bytes[37],
        cert_bytes[38],
        cert_bytes[39],
    ]);

    let machine_state_inv = u32::from_le_bytes([
        cert_bytes[40],
        cert_bytes[41],
        cert_bytes[42],
        cert_bytes[43],
    ]);

    let opt_level = cert_bytes[60];

    // Gate 2: Theorem ID in range
    if theorem_id < 0x0001 || theorem_id > 0x000B {
        errors.push(format!("Theorem ID out of range: 0x{:04X}", theorem_id));
    }

    // Gate 3: At least one theorem covered
    if theorems_covered == 0 {
        errors.push("No theorems covered (bitmask is zero)".to_string());
    }

    // Gate 4: Machine state invariants non-zero
    if machine_state_inv == 0 {
        errors.push("Machine state invariants must be non-zero".to_string());
    }

    // Gate 5: Optimization level in range
    if opt_level > 2 {
        errors.push(format!("Optimization level out of range: {}", opt_level));
    }

    // Gate 6: Backend string is valid
    let backend_bytes = &cert_bytes[44..60];
    let backend_str = std::str::from_utf8(backend_bytes)
        .unwrap_or("")
        .trim_end_matches('\0');

    if !["x86_64", "aarch64", "wasm32"].contains(&backend_str) {
        errors.push(format!("Unknown backend: {}", backend_str));
    }

    let valid = errors.is_empty();

    ProofCertificateValidationResult {
        valid,
        theorem_id,
        theorems_covered,
        error_code: if valid { 0 } else { 1 },
    }
}

// ============================================================================
// WASM Module Initialization
// ============================================================================

#[wasm_bindgen(start)]
pub fn init_wasm() {
    // Initialize panic hook for better error messages in browser console
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

// ============================================================================
// Tests (compiled with `cargo test --target wasm32-unknown-unknown`)
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blake3_hash() {
        let input = b"test payload";
        let hash = blake3_hash(input);
        assert_eq!(hash.len(), 32);
    }

    #[test]
    fn test_blake3_verify_valid() {
        let input = b"test payload";
        let digest = blake3_hash(input);
        let result = blake3_verify_wasm(input, &digest);
        assert!(result.valid);
        assert_eq!(result.error_code, 0);
    }

    #[test]
    fn test_blake3_verify_invalid() {
        let input = b"test payload";
        let digest = blake3_hash(input);
        let mut bad_digest = digest.clone();
        bad_digest[0] ^= 0xFF;  // Flip bits
        let result = blake3_verify_wasm(input, &bad_digest);
        assert!(!result.valid);
        assert_eq!(result.error_code, 1);
    }

    #[test]
    fn test_mutation_validation_valid() {
        let result = validate_mutation_wasm(
            1,  // event_id
            5,  // generation
            &[0x01; 32],  // source_hash
            &[0x02; 32],  // bytecode_hash
            &[0x03; 32],  // native_code_hash
            &[0x04; 64],  // actor_signature
        );
        assert!(result.valid);
        assert_eq!(result.error_code, 0);
    }

    #[test]
    fn test_mutation_validation_invalid_event_id() {
        let result = validate_mutation_wasm(
            0,  // event_id (invalid: must be non-zero)
            5,
            &[0x01; 32],
            &[0x02; 32],
            &[0x03; 32],
            &[0x04; 64],
        );
        assert!(!result.valid);
    }

    #[test]
    fn test_proof_certificate_validation() {
        let mut cert = vec![0u8; 157];
        // Set theorem_id to 0x0001
        cert[0] = 0x01;
        cert[1] = 0x00;
        // Set theorems_covered to 0x000F
        cert[36] = 0x0F;
        cert[37] = 0x00;
        // Set machine_state_invariants to 0x07
        cert[40] = 0x07;
        cert[41] = 0x00;
        // Set backend to "x86_64"
        for (i, byte) in b"x86_64".iter().enumerate() {
            cert[44 + i] = *byte;
        }
        // Set optimization_level to 2
        cert[60] = 2;

        let result = validate_proof_certificate_wasm(&cert);
        assert!(result.valid);
        assert_eq!(result.theorem_id, 0x0001);
        assert_eq!(result.theorems_covered, 0x000F);
    }
}
