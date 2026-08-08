/**
 * Super Haskell Cryptographic Primitives
 *
 * Elliptic curve operations verified by Agda specifications.
 * All operations are constant-time to prevent side-channel attacks.
 *
 * Curve: Curve25519 (y² = x³ + 486662x² + x mod 2^255 - 19)
 *
 * Author: Ahmad Parr <ahmedparr93@gmail.com>
 * License: BSL 1.1 + AGPL 3.0 + MPL 2.0
 */

#ifndef SUPERHASKELL_CRYPTO_H
#define SUPERHASKELL_CRYPTO_H

#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>

#ifdef __cplusplus
extern "C" {
#endif

// ===========================================================================
// FIELD ARITHMETIC (Curve25519: p = 2^255 - 19)
// ===========================================================================

/**
 * Field element representation: 256-bit integer as 4 × 64-bit limbs
 * Limbs are in little-endian order: fe[0] is least significant
 */
typedef uint64_t fe[4];

/**
 * Field addition: out = (a + b) mod p
 * Constant-time, no branches dependent on secret data.
 *
 * Agda proof: fe_add_correct : ∀ a b → toInt (fe_add a b) ≡ (toInt a + toInt b) mod p
 */
void fe_add(fe out, const fe a, const fe b);

/**
 * Field subtraction: out = (a - b) mod p
 * Constant-time.
 *
 * Agda proof: fe_sub_correct : ∀ a b → toInt (fe_sub a b) ≡ (toInt a - toInt b) mod p
 */
void fe_sub(fe out, const fe a, const fe b);

/**
 * Field multiplication: out = (a * b) mod p
 * Uses Karatsuba algorithm for efficiency.
 * Constant-time.
 *
 * Agda proof: fe_mul_correct : ∀ a b → toInt (fe_mul a b) ≡ (toInt a * toInt b) mod p
 */
void fe_mul(fe out, const fe a, const fe b);

/**
 * Field squaring: out = a² mod p
 * Optimized version of fe_mul(out, a, a).
 * Constant-time.
 */
void fe_sq(fe out, const fe a);

/**
 * Field inversion: out = a^(-1) mod p
 * Uses Fermat's little theorem: a^(-1) ≡ a^(p-2) mod p
 * Constant-time (uses constant-time exponentiation).
 *
 * Agda proof: fe_inv_correct : ∀ a → a ≠ 0 → fe_mul a (fe_inv a) ≡ 1
 */
void fe_inv(fe out, const fe a);

/**
 * Constant-time conditional move: out = flag ? a : out
 * flag must be 0 or 1.
 *
 * This is the fundamental building block for all constant-time operations.
 * Implemented using bitwise operations, no branches.
 */
void fe_cmov(fe out, const fe a, int flag);

/**
 * Reduce field element to canonical form: 0 ≤ out < p
 * Required before serialization or comparison.
 */
void fe_reduce(fe out);

// ===========================================================================
// ELLIPTIC CURVE POINT OPERATIONS
// ===========================================================================

/**
 * Point on Curve25519 in extended twisted Edwards coordinates
 * Representation: (X:Y:Z:T) where x = X/Z, y = Y/Z, T = XY/Z
 *
 * This representation allows unified addition formulas (no special cases).
 */
typedef struct {
    fe X;
    fe Y;
    fe Z;
    fe T;
} ec_point_t;

/**
 * Point at infinity (identity element)
 * Represented as (0:1:1:0) in extended coordinates
 */
extern const ec_point_t ec_point_infinity;

/**
 * Base point G for Curve25519
 * This is the standard generator with order 2^252 + 27742317777372353535851937790883648493
 */
extern const ec_point_t ec_base_point;

/**
 * Point addition: out = P + Q
 * Constant-time unified formula (works for P = Q and P ≠ Q).
 *
 * Agda proofs:
 *   - ec_add_correct : ∀ P Q → toAffine (ec_add P Q) ≡ affineAdd (toAffine P) (toAffine Q)
 *   - ec_add_assoc : ∀ P Q R → ec_add (ec_add P Q) R ≡ ec_add P (ec_add Q R)
 *   - ec_add_comm : ∀ P Q → ec_add P Q ≡ ec_add Q P
 */
void ec_point_add(ec_point_t *out, const ec_point_t *P, const ec_point_t *Q);

/**
 * Point doubling: out = 2·P
 * Optimized version of ec_point_add(out, P, P).
 * Constant-time.
 *
 * Agda proof: ec_double_correct : ∀ P → ec_double P ≡ ec_add P P
 */
void ec_point_double(ec_point_t *out, const ec_point_t *P);

/**
 * Scalar multiplication: out = k·P
 * Uses Montgomery ladder for constant-time execution.
 *
 * scalar: big-endian byte array representing integer k
 * scalar_len: length of scalar in bytes (typically 32 for 256-bit scalars)
 *
 * Constant-time: execution does NOT depend on scalar value (only scalar_len).
 *
 * Agda proofs:
 *   - ec_scalar_mult_correct : ∀ k P → ec_scalar_mult k P ≡ iterate k (ec_add P) (identity)
 *   - ec_scalar_mult_distrib : ∀ k P Q → ec_scalar_mult k (ec_add P Q)
 *                             ≡ ec_add (ec_scalar_mult k P) (ec_scalar_mult k Q)
 *   - ec_scalar_mult_assoc : ∀ j k P → ec_scalar_mult j (ec_scalar_mult k P)
 *                           ≡ ec_scalar_mult (j*k) P
 */
void ec_scalar_mult(ec_point_t *out, const uint8_t *scalar, size_t scalar_len, const ec_point_t *P);

/**
 * Scalar multiplication with base point: out = k·G
 * Optimized using precomputed lookup table.
 *
 * Up to 2× faster than general ec_scalar_mult when multiplying by base point.
 */
void ec_scalar_mult_base(ec_point_t *out, const uint8_t *scalar, size_t scalar_len);

/**
 * Point encoding: serialize point to 32-byte compressed form
 * Encodes x-coordinate and sign of y-coordinate.
 *
 * Returns: true on success, false if point is not on curve
 */
bool ec_point_encode(uint8_t out[32], const ec_point_t *P);

/**
 * Point decoding: deserialize point from 32-byte compressed form
 * Recovers y-coordinate from x and sign bit.
 *
 * Returns: true on success, false if encoding is invalid or point is not on curve
 */
bool ec_point_decode(ec_point_t *out, const uint8_t in[32]);

// ===========================================================================
// ASSEMBLY OPTIMIZATIONS
// ===========================================================================

/**
 * Field multiplication (x86-64 assembly, AVX2/BMI2)
 * Optimized using mulx instruction for efficient multiply.
 *
 * Only available when compiled with -march=haswell or later.
 * Falls back to fe_mul() if CPU doesn't support BMI2.
 */
#if defined(__x86_64__) && defined(__BMI2__)
void fe_mul_asm(fe out, const fe a, const fe b);
#endif

/**
 * Scalar multiplication (x86-64 assembly, AVX-512)
 * Vectorized point operations using 512-bit registers.
 *
 * Only available when compiled with -march=skylake-avx512 or later.
 */
#if defined(__x86_64__) && defined(__AVX512F__)
void ec_scalar_mult_asm(ec_point_t *out, const uint8_t *scalar, size_t scalar_len, const ec_point_t *P);
#endif

/**
 * Field multiplication (ARM NEON assembly)
 * Optimized using NEON SIMD instructions.
 *
 * Only available on ARM64 with NEON support.
 */
#if defined(__aarch64__) && defined(__ARM_NEON)
void fe_mul_neon(fe out, const fe a, const fe b);
#endif

// ===========================================================================
// TESTING & DEBUGGING
// ===========================================================================

/**
 * Check if point is on curve: y² = x³ + 486662x² + x
 * Used for validation after deserialization.
 *
 * NOT constant-time (result is public information).
 */
bool ec_point_is_valid(const ec_point_t *P);

/**
 * Print field element in hexadecimal (for debugging)
 * NOT constant-time.
 */
void fe_print(const fe a);

/**
 * Print point in affine coordinates (for debugging)
 * NOT constant-time.
 */
void ec_point_print(const ec_point_t *P);

#ifdef __cplusplus
}
#endif

#endif // SUPERHASKELL_CRYPTO_H
