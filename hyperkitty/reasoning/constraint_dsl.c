/*
 * constraint_dsl.c — Constraint Evaluation Engine
 *
 * The validity predicate that gates all routing:
 *
 * V(message) = 1  ⟺  accounting_balance(msg) ∧
 *                      invariant_preserved(msg) ∧
 *                      entropy(msg) ≤ 0.20 nats ∧
 *                      proof_certificate_exists(msg)
 *
 * If V(message) ≠ 1, the message does NOT propagate.
 *
 * Constraints are external to probabilistic inference.
 * The model may generate a high-probability output that fails a constraint.
 * The constraint gate rejects it anyway (fail-closed).
 */

#include "hyperkitty/constraints.h"
#include <stdlib.h>
#include <string.h>
#include <math.h>
#include <stdio.h>

/* ================================================================
 * Accounting balance: δ_A + δ_E = δ_L + δ_R
 * ================================================================ */

typedef struct {
    double asset;       /* δ_A */
    double equity;      /* δ_E */
    double liability;   /* δ_L */
    double revenue;     /* δ_R */
} AccountingLedger;

static int check_accounting_balance(const AccountingLedger *ledger) {
    if (!ledger) return 0;

    double lhs = ledger->asset + ledger->equity;
    double rhs = ledger->liability + ledger->revenue;

    /* Tolerance for floating-point comparison */
    return fabs(lhs - rhs) < 1e-10;
}

/* ================================================================
 * Invariant preservation: I(X_t) = I(X_{t+1})
 * ================================================================ */

typedef struct {
    double *current;    /* I(X_t) */
    double *next;       /* I(X_{t+1}) */
    int dimension;
} InvariantPair;

static int check_invariant_preserved(const InvariantPair *inv) {
    if (!inv || !inv->current || !inv->next) return 0;

    double tolerance = 1e-8;

    for (int i = 0; i < inv->dimension; i++) {
        if (fabs(inv->current[i] - inv->next[i]) > tolerance) {
            return 0;
        }
    }

    return 1;
}

/* ================================================================
 * Entropy bound: H(X) ≤ 0.20 nats
 * ================================================================ */

static int check_entropy_bound(double entropy) {
    /* H ≤ 0.20 nats is the admissibility threshold */
    const double ENTROPY_THRESHOLD = 0.20;
    return entropy <= ENTROPY_THRESHOLD;
}

/* ================================================================
 * Proof certificate: exists and valid
 * ================================================================ */

typedef struct {
    uint8_t hash[32];           /* SHA-256 hash of proof */
    uint8_t signature[64];      /* Ed25519 signature */
    uint64_t timestamp;
    int valid;                  /* 1 if verified, 0 otherwise */
} ProofCertificate;

static int check_proof_exists(const ProofCertificate *cert) {
    if (!cert) return 0;
    return cert->valid;
}

/* ================================================================
 * Main validity predicate
 * ================================================================ */

int hk_constraint_check_all(const hk_constraint_context *ctx) {
    if (!ctx) return 0;

    /* All four constraints must pass */
    int balance_ok = check_accounting_balance(&ctx->ledger);
    int invariant_ok = check_invariant_preserved(&ctx->invariant);
    int entropy_ok = check_entropy_bound(ctx->entropy);
    int proof_ok = check_proof_exists(&ctx->proof);

    return balance_ok && invariant_ok && entropy_ok && proof_ok;
}

/* ================================================================
 * Constraint report (for debugging/audit)
 * ================================================================ */

void hk_constraint_report(const hk_constraint_context *ctx) {
    if (!ctx) return;

    printf("=== Constraint Validation Report ===\n\n");

    printf("Accounting Balance:\n");
    double lhs = ctx->ledger.asset + ctx->ledger.equity;
    double rhs = ctx->ledger.liability + ctx->ledger.revenue;
    printf("  LHS (A + E): %.6f\n", lhs);
    printf("  RHS (L + R): %.6f\n", rhs);
    printf("  Balanced:    %s\n", check_accounting_balance(&ctx->ledger) ? "YES" : "NO");

    printf("\nInvariant Preservation:\n");
    printf("  Dimension:   %d\n", ctx->invariant.dimension);
    printf("  Preserved:   %s\n", check_invariant_preserved(&ctx->invariant) ? "YES" : "NO");
    if (ctx->invariant.dimension > 0) {
        printf("  Sample (dim 0): I(X_t)=%.6f, I(X_{t+1})=%.6f\n",
               ctx->invariant.current[0], ctx->invariant.next[0]);
    }

    printf("\nEntropy Bound:\n");
    printf("  H(X):        %.6f nats\n", ctx->entropy);
    printf("  Threshold:   0.20 nats\n");
    printf("  Within bound: %s\n", check_entropy_bound(ctx->entropy) ? "YES" : "NO");

    printf("\nProof Certificate:\n");
    printf("  Valid:       %s\n", check_proof_exists(&ctx->proof) ? "YES" : "NO");

    printf("\n=== Overall: ");
    if (hk_constraint_check_all(ctx)) {
        printf("PASS (message propagates) ===\n");
    } else {
        printf("FAIL (message blocked) ===\n");
    }
}

/* ================================================================
 * Constraint context factory (for testing)
 * ================================================================ */

hk_constraint_context *hk_constraint_context_alloc(int invariant_dim) {
    hk_constraint_context *ctx = malloc(sizeof(hk_constraint_context));
    if (!ctx) return NULL;

    ctx->invariant.current = calloc(invariant_dim, sizeof(double));
    ctx->invariant.next = calloc(invariant_dim, sizeof(double));
    ctx->invariant.dimension = invariant_dim;

    if (!ctx->invariant.current || !ctx->invariant.next) {
        free(ctx->invariant.current);
        free(ctx->invariant.next);
        free(ctx);
        return NULL;
    }

    /* Initialize ledger to balanced */
    ctx->ledger.asset = 1.0;
    ctx->ledger.equity = 1.0;
    ctx->ledger.liability = 2.0;
    ctx->ledger.revenue = 0.0;

    ctx->entropy = 0.0;  /* Deterministic routing: H = 0 */
    ctx->proof.valid = 0;

    return ctx;
}

void hk_constraint_context_free(hk_constraint_context *ctx) {
    if (!ctx) return;
    free(ctx->invariant.current);
    free(ctx->invariant.next);
    free(ctx);
}
