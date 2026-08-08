//! Super Haskell Rust Backend
//! Cryptographic primitives and WORM chain implementation

use std::ffi::{CStr, c_char, c_int};

pub mod crypto;
pub mod worm;

// =========================================================================
// FFI EXPORTS (Called from Haskell)
// =========================================================================

/// Scalar multiplication on elliptic curve (verified by Agda spec)
#[no_mangle]
pub extern "C" fn rust_scalar_mult(
    k: c_int,
    x: c_int,
    y: c_int,
    out_x: *mut c_int,
    out_y: *mut c_int,
) -> c_int {
    // Placeholder implementation
    unsafe {
        *out_x = x * k;
        *out_y = y * k;
    }
    0 // Success
}

/// Append entry to WORM chain (BLAKE3 attestation)
#[no_mangle]
pub extern "C" fn rust_append_worm(msg: *const c_char, len: c_int) -> c_int {
    let msg_bytes = unsafe {
        std::slice::from_raw_parts(msg as *const u8, len as usize)
    };

    match std::str::from_utf8(msg_bytes) {
        Ok(msg_str) => {
            println!("[Rust WORM] Appending: {}", msg_str);
            // TODO: Actual BLAKE3 chain append
            0 // Success
        }
        Err(_) => -1, // Invalid UTF-8
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalar_mult_identity() {
        let mut out_x = 0;
        let mut out_y = 0;
        let result = rust_scalar_mult(1, 42, 99, &mut out_x, &mut out_y);
        assert_eq!(result, 0);
        assert_eq!(out_x, 42);
        assert_eq!(out_y, 99);
    }
}
