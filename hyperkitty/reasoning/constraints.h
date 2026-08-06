/*
 * constraints.h — Constraint Evaluation Engine interface
 *
 * The validity predicate gates all routing decisions.
 * No output propagates unless all constraints are satisfied.
 *
 * V(message) = 1  ⟺  accounting_balance ∧ invariant_preserved ∧ entropy ≤ 0.20 ∧ proof_exists
 *
 * If V(message) = 0, the message is blocked (fail-closed).
 */

#pragma once
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/* Accounting ledger: δ_A + δ_E = δ_L + δ_R */
typedef struct {
    double asset;
    double equity;
    double liability;
    double revenue;
} AccountingLedger;

/* Invariant tracking */
typedef struct {
    double *current;    /* I(X_t) */
    double *next;       /* I(X_{t+1}) */
    int dimension;
} InvariantPair;

/* Proof certificate with cryptographic seal */
typedef struct {
    uint8_t hash[32];
    uint8_t signature[64];
    uint64_t timestamp;
    int valid;
} ProofCertificate;

/* Full constraint context */
typedef struct {
    AccountingLedger ledger;    /* Balance equation */
    InvariantPair invariant;    /* I(X_t) = I(X_{t+1}) */
    double entropy;             /* H(X) in nats */
    ProofCertificate proof;     /* Proof certificate */
} hk_constraint_context;

/**
 * hk_constraint_check_all — Evaluate all constraints
 *
 * Returns 1 if all constraints pass (message propagates):
 *   1. Accounting balance: δ_A + δ_E = δ_L + δ_R
 *   2. Invariant preservation: I(X_t) = I(X_{t+1})
 *   3. Entropy bound: H(X) ≤ 0.20 nats
 *   4. Proof certificate: exists and valid
 *
 * Returns 0 if any constraint fails (message blocked).
 *
 * @param ctx  Constraint context
 * @return 1 if all constraints satisfied, 0 otherwise
 */
int hk_constraint_check_all(const hk_constraint_context *ctx);

/**
 * hk_constraint_report — Print detailed constraint validation report
 *
 * Outputs human-readable summary of each constraint check.
 *
 * @param ctx  Constraint context
 */
void hk_constraint_report(const hk_constraint_context *ctx);

/**
 * hk_constraint_context_alloc — Create constraint context
 *
 * @param invariant_dim  Dimension of invariant vectors
 * @return Allocated context, or NULL on error
 */
hk_constraint_context *hk_constraint_context_alloc(int invariant_dim);

/**
 * hk_constraint_context_free — Free constraint context
 */
void hk_constraint_context_free(hk_constraint_context *ctx);

#ifdef __cplusplus
}
#endif
