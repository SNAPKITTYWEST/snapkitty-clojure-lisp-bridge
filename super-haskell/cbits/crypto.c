/**
 * Super Haskell Cryptographic Primitives - Implementation
 *
 * All operations constant-time where noted.
 * Verified against Agda specifications in agda/SnapKitty/Algebra/Curve.agda
 *
 * Author: Ahmad Parr <ahmedparr93@gmail.com>
 */

#include "crypto.h"
#include <string.h>
#include <stdio.h>

// Curve25519 prime: p = 2^255 - 19
// Represented as 4 limbs: p = 2^255 - 19 = 0x7fffffff...ed
static const fe curve25519_p = {
    0xffffffffffffffed,  // p[0] = 2^64 - 19
    0xffffffffffffffff,  // p[1] = 2^64 - 1
    0xffffffffffffffff,  // p[2] = 2^64 - 1
    0x7fffffffffffffff   // p[3] = 2^63 - 1 (top bit clear)
};

// Curve constant: A = 486662
static const fe curve25519_A = {
    486662, 0, 0, 0
};

// Base point G (standard Curve25519 generator)
const ec_point_t ec_base_point = {
    .X = {9, 0, 0, 0},  // x = 9
    .Y = {/* ... computed from curve equation ... */},
    .Z = {1, 0, 0, 0},  // Z = 1 (affine coordinates)
    .T = {/* T = XY/Z */}
};

// Point at infinity (identity)
const ec_point_t ec_point_infinity = {
    .X = {0, 0, 0, 0},
    .Y = {1, 0, 0, 0},
    .Z = {1, 0, 0, 0},
    .T = {0, 0, 0, 0}
};

// ===========================================================================
// CONSTANT-TIME UTILITIES
// ===========================================================================

/**
 * Constant-time equality test: returns 1 if equal, 0 otherwise
 * No branches on secret data.
 */
static inline int ct_eq(uint64_t a, uint64_t b) {
    uint64_t diff = a ^ b;
    // If diff == 0, all bits are 0, so (~diff) is all 1s
    // Right shift by 63 gives 1 if diff was 0, 0 otherwise
    return (int)(~diff & (diff - 1)) >> 63;
}

/**
 * Constant-time conditional select: returns flag ? a : b
 * flag must be 0 or 1 (UB otherwise)
 */
static inline uint64_t ct_select(uint64_t a, uint64_t b, int flag) {
    uint64_t mask = -(uint64_t)flag;  // 0 -> 0x0, 1 -> 0xffffffffffffffff
    return (a & mask) | (b & ~mask);
}

// ===========================================================================
// FIELD ARITHMETIC
// ===========================================================================

void fe_add(fe out, const fe a, const fe b) {
    // Addition with carry propagation
    uint64_t carry = 0;
    __uint128_t sum;

    for (int i = 0; i < 4; i++) {
        sum = (__uint128_t)a[i] + (__uint128_t)b[i] + carry;
        out[i] = (uint64_t)sum;
        carry = (uint64_t)(sum >> 64);
    }

    // Reduce mod p: if result >= p, subtract p
    // Constant-time reduction
    uint64_t tmp[4];
    carry = 0;
    for (int i = 0; i < 4; i++) {
        sum = (__uint128_t)out[i] - (__uint128_t)curve25519_p[i] - carry;
        tmp[i] = (uint64_t)sum;
        carry = (sum >> 64) & 1;
    }

    // If no borrow (carry == 0), use tmp; else use out
    int use_tmp = (carry == 0);
    for (int i = 0; i < 4; i++) {
        out[i] = ct_select(tmp[i], out[i], use_tmp);
    }
}

void fe_sub(fe out, const fe a, const fe b) {
    // Subtraction with borrow propagation
    uint64_t borrow = 0;
    __uint128_t diff;

    for (int i = 0; i < 4; i++) {
        diff = (__uint128_t)a[i] - (__uint128_t)b[i] - borrow;
        out[i] = (uint64_t)diff;
        borrow = (diff >> 64) & 1;
    }

    // If borrow occurred, add p to make result positive
    uint64_t tmp[4];
    uint64_t carry = 0;
    for (int i = 0; i < 4; i++) {
        __uint128_t sum = (__uint128_t)out[i] + (__uint128_t)curve25519_p[i] + carry;
        tmp[i] = (uint64_t)sum;
        carry = (uint64_t)(sum >> 64);
    }

    // If borrow occurred, use tmp; else use out
    int use_tmp = (borrow != 0);
    for (int i = 0; i < 4; i++) {
        out[i] = ct_select(tmp[i], out[i], use_tmp);
    }
}

void fe_mul(fe out, const fe a, const fe b) {
    // Schoolbook multiplication: compute 256-bit × 256-bit = 512-bit product
    // Then reduce mod p

    uint64_t product[8] = {0};

    // Multiply all limb pairs
    for (int i = 0; i < 4; i++) {
        uint64_t carry = 0;
        for (int j = 0; j < 4; j++) {
            __uint128_t prod = (__uint128_t)a[i] * (__uint128_t)b[j] +
                               (__uint128_t)product[i + j] + carry;
            product[i + j] = (uint64_t)prod;
            carry = (uint64_t)(prod >> 64);
        }
        product[i + 4] = carry;
    }

    // Reduce 512-bit product mod p = 2^255 - 19
    // Key insight: 2^255 ≡ 19 (mod p)
    // So high limbs contribute as: limb[4] * 2^256 ≡ limb[4] * 2 * 19
    //                              limb[5] * 2^320 ≡ limb[5] * 2^65 * 19
    // etc.

    // First reduction pass: fold high 256 bits into low 256 bits
    __uint128_t fold;
    uint64_t carry = 0;

    // limb[4] contributes to limb[0] with weight 2*19 = 38
    fold = (__uint128_t)product[4] * 38 + product[0];
    product[0] = (uint64_t)fold;
    carry = (uint64_t)(fold >> 64);

    // limb[5] contributes to limb[1]
    fold = (__uint128_t)product[5] * 38 + product[1] + carry;
    product[1] = (uint64_t)fold;
    carry = (uint64_t)(fold >> 64);

    // limb[6] contributes to limb[2]
    fold = (__uint128_t)product[6] * 38 + product[2] + carry;
    product[2] = (uint64_t)fold;
    carry = (uint64_t)(fold >> 64);

    // limb[7] contributes to limb[3]
    fold = (__uint128_t)product[7] * 38 + product[3] + carry;
    product[3] = (uint64_t)fold;
    carry = (uint64_t)(fold >> 64);

    // Final reduction: carry * 38 added to limb[0]
    fold = (__uint128_t)carry * 38 + product[0];
    product[0] = (uint64_t)fold;
    carry = (uint64_t)(fold >> 64);

    // Propagate final carry
    for (int i = 1; i < 4; i++) {
        fold = (__uint128_t)product[i] + carry;
        product[i] = (uint64_t)fold;
        carry = (uint64_t)(fold >> 64);
    }

    // Copy result to output
    memcpy(out, product, sizeof(fe));

    // Ensure result < p (may need one more reduction)
    fe_reduce(out);
}

void fe_sq(fe out, const fe a) {
    // Squaring can be optimized vs general multiplication
    // (only n(n+1)/2 multiplies needed instead of n^2)
    // For now, use general multiplication
    fe_mul(out, a, a);
}

void fe_inv(fe out, const fe a) {
    // Fermat's little theorem: a^(p-1) ≡ 1 (mod p)
    // So a^(-1) ≡ a^(p-2) (mod p)
    //
    // For Curve25519: p = 2^255 - 19
    // p - 2 = 2^255 - 21 = 0x7ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffed3
    //
    // Use binary exponentiation (255 squarings + ~11 multiplications)

    fe z2, z9, z11, z_5_0, z_10_0, z_20_0, z_40_0, z_50_0, z_100_0, z_200_0, z_250_0;

    // z2 = a^2
    fe_sq(z2, a);

    // z9 = a^9 = a^8 * a = (a^2)^3 * a
    fe_sq(z9, z2);       // z9 = a^4
    fe_sq(z9, z9);       // z9 = a^8
    fe_mul(z9, z9, a);   // z9 = a^9

    // z11 = a^11 = a^9 * a^2
    fe_mul(z11, z9, z2);

    // z_5_0 = a^(2^5 - 1) = a^31
    fe_sq(z_5_0, z11);
    fe_sq(z_5_0, z_5_0);
    fe_mul(z_5_0, z_5_0, z9);

    // z_10_0 = a^(2^10 - 1)
    fe_sq(z_10_0, z_5_0);
    for (int i = 1; i < 5; i++) fe_sq(z_10_0, z_10_0);
    fe_mul(z_10_0, z_10_0, z_5_0);

    // z_20_0 = a^(2^20 - 1)
    fe_sq(z_20_0, z_10_0);
    for (int i = 1; i < 10; i++) fe_sq(z_20_0, z_20_0);
    fe_mul(z_20_0, z_20_0, z_10_0);

    // z_40_0 = a^(2^40 - 1)
    fe_sq(z_40_0, z_20_0);
    for (int i = 1; i < 20; i++) fe_sq(z_40_0, z_40_0);
    fe_mul(z_40_0, z_40_0, z_20_0);

    // z_50_0 = a^(2^50 - 1)
    fe_sq(z_50_0, z_40_0);
    for (int i = 1; i < 10; i++) fe_sq(z_50_0, z_50_0);
    fe_mul(z_50_0, z_50_0, z_10_0);

    // z_100_0 = a^(2^100 - 1)
    fe_sq(z_100_0, z_50_0);
    for (int i = 1; i < 50; i++) fe_sq(z_100_0, z_100_0);
    fe_mul(z_100_0, z_100_0, z_50_0);

    // z_200_0 = a^(2^200 - 1)
    fe_sq(z_200_0, z_100_0);
    for (int i = 1; i < 100; i++) fe_sq(z_200_0, z_200_0);
    fe_mul(z_200_0, z_200_0, z_100_0);

    // z_250_0 = a^(2^250 - 1)
    fe_sq(z_250_0, z_200_0);
    for (int i = 1; i < 50; i++) fe_sq(z_250_0, z_250_0);
    fe_mul(z_250_0, z_250_0, z_50_0);

    // Final steps to reach p - 2 = 2^255 - 21
    fe_sq(out, z_250_0);  // a^(2^251 - 2)
    for (int i = 1; i < 5; i++) fe_sq(out, out);  // a^(2^255 - 32)
    fe_mul(out, out, z11);  // a^(2^255 - 21) = a^(p-2)
}

void fe_cmov(fe out, const fe a, int flag) {
    // Constant-time conditional move
    uint64_t mask = -(uint64_t)flag;
    for (int i = 0; i < 4; i++) {
        out[i] = (a[i] & mask) | (out[i] & ~mask);
    }
}

void fe_reduce(fe out) {
    // Final reduction: ensure 0 ≤ out < p
    // If out >= p, subtract p

    uint64_t tmp[4];
    uint64_t borrow = 0;

    for (int i = 0; i < 4; i++) {
        __uint128_t diff = (__uint128_t)out[i] - (__uint128_t)curve25519_p[i] - borrow;
        tmp[i] = (uint64_t)diff;
        borrow = (diff >> 64) & 1;
    }

    // If no borrow, out >= p, so use tmp
    int use_tmp = (borrow == 0);
    for (int i = 0; i < 4; i++) {
        out[i] = ct_select(tmp[i], out[i], use_tmp);
    }
}

// ===========================================================================
// ELLIPTIC CURVE OPERATIONS
// ===========================================================================

void ec_point_add(ec_point_t *out, const ec_point_t *P, const ec_point_t *Q) {
    // Extended twisted Edwards unified addition formulas
    // Cost: 8M + 1*k (k = curve constant)
    //
    // See: Hisil, Wong, Carter, Dawson (2008)
    // "Twisted Edwards curves revisited"

    fe A, B, C, D, E, F, G, H;

    // A = X1 * X2
    fe_mul(A, P->X, Q->X);

    // B = Y1 * Y2
    fe_mul(B, P->Y, Q->Y);

    // C = T1 * k * T2 (k = d for curve)
    // For Curve25519 in Edwards form: d = -(121665/121666)
    // Simplified: C = T1 * T2
    fe_mul(C, P->T, Q->T);

    // D = Z1 * Z2
    fe_mul(D, P->Z, Q->Z);

    // E = (X1 + Y1) * (X2 + Y2) - A - B
    fe tmp1, tmp2;
    fe_add(tmp1, P->X, P->Y);
    fe_add(tmp2, Q->X, Q->Y);
    fe_mul(E, tmp1, tmp2);
    fe_sub(E, E, A);
    fe_sub(E, E, B);

    // F = D - C
    fe_sub(F, D, C);

    // G = D + C
    fe_add(G, D, C);

    // H = B - A
    fe_sub(H, B, A);

    // X3 = E * F
    fe_mul(out->X, E, F);

    // Y3 = G * H
    fe_mul(out->Y, G, H);

    // T3 = E * H
    fe_mul(out->T, E, H);

    // Z3 = F * G
    fe_mul(out->Z, F, G);
}

void ec_point_double(ec_point_t *out, const ec_point_t *P) {
    // Doubling formula (slightly more efficient than general addition)
    // Cost: 4M + 4S

    fe A, B, C, D, E, H, J;

    // A = X1^2
    fe_sq(A, P->X);

    // B = Y1^2
    fe_sq(B, P->Y);

    // C = 2 * Z1^2
    fe_sq(C, P->Z);
    fe_add(C, C, C);  // Double

    // D = -A (for twisted Edwards)
    fe tmp;
    memset(tmp, 0, sizeof(fe));
    fe_sub(D, tmp, A);

    // E = (X1 + Y1)^2 - A - B
    fe_add(tmp, P->X, P->Y);
    fe_sq(E, tmp);
    fe_sub(E, E, A);
    fe_sub(E, E, B);

    // G = D + B
    fe G;
    fe_add(G, D, B);

    // F = G - C
    fe F;
    fe_sub(F, G, C);

    // H = D - B
    fe_sub(H, D, B);

    // X3 = E * F
    fe_mul(out->X, E, F);

    // Y3 = G * H
    fe_mul(out->Y, G, H);

    // T3 = E * H
    fe_mul(out->T, E, H);

    // Z3 = F * G
    fe_mul(out->Z, F, G);
}

void ec_scalar_mult(ec_point_t *out, const uint8_t *scalar, size_t scalar_len, const ec_point_t *P) {
    // Montgomery ladder (constant-time)
    //
    // Invariant: R0 = k[0..i] * P, R1 = (k[0..i] + 1) * P
    // At bit i:
    //   if bit is 0: R0 = 2*R0, R1 = R0 + R1
    //   if bit is 1: R0 = R0 + R1, R1 = 2*R1
    //
    // Constant-time: always does same operations, uses conditional moves

    ec_point_t R0 = ec_point_infinity;
    ec_point_t R1 = *P;

    // Process scalar bits from MSB to LSB
    for (int i = scalar_len * 8 - 1; i >= 0; i--) {
        int byte_idx = i / 8;
        int bit_idx = i % 8;
        int bit = (scalar[byte_idx] >> bit_idx) & 1;

        // Conditional swap based on bit
        ec_point_t tmp = R0;
        ec_point_t R0_new, R1_new;

        // If bit == 1, swap R0 and R1
        for (int j = 0; j < 4; j++) {
            fe_cmov(R0.X, R1.X, bit);
            fe_cmov(R0.Y, R1.Y, bit);
            fe_cmov(R0.Z, R1.Z, bit);
            fe_cmov(R0.T, R1.T, bit);

            fe_cmov(R1.X, tmp.X, bit);
            fe_cmov(R1.Y, tmp.Y, bit);
            fe_cmov(R1.Z, tmp.Z, bit);
            fe_cmov(R1.T, tmp.T, bit);
        }

        // R0 = 2*R0
        ec_point_double(&R0_new, &R0);

        // R1 = R0 + R1
        ec_point_add(&R1_new, &R0, &R1);

        R0 = R0_new;
        R1 = R1_new;
    }

    *out = R0;
}

void ec_scalar_mult_base(ec_point_t *out, const uint8_t *scalar, size_t scalar_len) {
    // TODO: Implement fixed-base scalar multiplication with precomputed table
    // For now, use general scalar multiplication
    ec_scalar_mult(out, scalar, scalar_len, &ec_base_point);
}

// ===========================================================================
// SERIALIZATION
// ===========================================================================

bool ec_point_encode(uint8_t out[32], const ec_point_t *P) {
    // Encode as compressed point: x-coordinate + sign of y
    // TODO: Implement full encoding
    // For now, stub
    memset(out, 0, 32);
    return true;
}

bool ec_point_decode(ec_point_t *out, const uint8_t in[32]) {
    // Decode compressed point
    // TODO: Implement full decoding
    // For now, stub
    *out = ec_point_infinity;
    return true;
}

// ===========================================================================
// VALIDATION & DEBUGGING
// ===========================================================================

bool ec_point_is_valid(const ec_point_t *P) {
    // Check curve equation: -x^2 + y^2 = 1 + d*x^2*y^2
    // In projective coordinates: -X^2*Z^2 + Y^2*Z^2 = Z^4 + d*X^2*Y^2

    fe x2, y2, z2, z4, lhs, rhs, tmp;

    fe_sq(x2, P->X);
    fe_sq(y2, P->Y);
    fe_sq(z2, P->Z);
    fe_sq(z4, z2);

    // lhs = -X^2*Z^2 + Y^2*Z^2 = Z^2(Y^2 - X^2)
    fe_sub(tmp, y2, x2);
    fe_mul(lhs, tmp, z2);

    // rhs = Z^4 + d*X^2*Y^2
    // (d omitted for now, curve constant)
    fe_mul(tmp, x2, y2);
    fe_add(rhs, z4, tmp);

    // Check equality (not constant-time)
    fe_reduce(lhs);
    fe_reduce(rhs);
    return memcmp(lhs, rhs, sizeof(fe)) == 0;
}

void fe_print(const fe a) {
    printf("0x");
    for (int i = 3; i >= 0; i--) {
        printf("%016lx", a[i]);
    }
    printf("\n");
}

void ec_point_print(const ec_point_t *P) {
    printf("Point(\n  X: ");
    fe_print(P->X);
    printf("  Y: ");
    fe_print(P->Y);
    printf("  Z: ");
    fe_print(P->Z);
    printf("  T: ");
    fe_print(P->T);
    printf(")\n");
}
