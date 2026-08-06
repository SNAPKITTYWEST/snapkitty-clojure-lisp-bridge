/*
 * nand_filter.c — Stage 8: NAND Filter
 *
 * Resolves conflicts between active experts using Boolean logic.
 *
 * Core principle: NAND is functionally complete. Any Boolean predicate
 * can be built from NAND alone. By using NAND for all conflict resolution,
 * we minimize the trusted verification base.
 *
 * Truth table:
 *   NAND(0, 0) = 1
 *   NAND(0, 1) = 1
 *   NAND(1, 0) = 1
 *   NAND(1, 1) = 0
 *
 * NOT(a) = NAND(a, a)
 * AND(a, b) = NAND(NAND(a, b), NAND(a, b))
 * OR(a, b) = NAND(NOT(a), NOT(b)) = NAND(NAND(a, a), NAND(b, b))
 * a → b = OR(NOT(a), b)
 */

#include "hyperkitty/nand.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

/* ================================================================
 * NAND primitive
 * ================================================================ */

static int nand(int a, int b) {
    return !(a && b);
}

/* ================================================================
 * Derived Boolean operators (built from NAND)
 * ================================================================ */

static int not_op(int a) {
    return nand(a, a);
}

static int and_op(int a, int b) {
    return nand(nand(a, b), nand(a, b));
}

static int or_op(int a, int b) {
    return nand(not_op(a), not_op(b));
}

static int implies(int a, int b) {
    return or_op(not_op(a), b);
}

static int xor_op(int a, int b) {
    return and_op(nand(a, b), or_op(a, b));
}

/* ================================================================
 * Expert conflict matrix
 * ================================================================ */

typedef struct {
    int **conflicts;    /* conflicts[i][j] = 1 if experts i and j conflict */
    int expert_count;
} ConflictMatrix;

static ConflictMatrix *conflict_matrix_alloc(int expert_count) {
    ConflictMatrix *cm = malloc(sizeof(ConflictMatrix));
    if (!cm) return NULL;

    cm->conflicts = calloc(expert_count, sizeof(int *));
    if (!cm->conflicts) {
        free(cm);
        return NULL;
    }

    for (int i = 0; i < expert_count; i++) {
        cm->conflicts[i] = calloc(expert_count, sizeof(int));
        if (!cm->conflicts[i]) {
            for (int j = 0; j < i; j++) free(cm->conflicts[j]);
            free(cm->conflicts);
            free(cm);
            return NULL;
        }
    }

    cm->expert_count = expert_count;
    return cm;
}

static void conflict_matrix_free(ConflictMatrix *cm) {
    if (!cm) return;
    for (int i = 0; i < cm->expert_count; i++) {
        free(cm->conflicts[i]);
    }
    free(cm->conflicts);
    free(cm);
}

/* ================================================================
 * Example conflict definitions (compile-time)
 * ================================================================ */

static void populate_conflict_matrix(ConflictMatrix *cm) {
    if (!cm) return;

    /* Example: some experts have conflicting responsibilities */
    /* These would be domain-specific and calibrated per use case */

    /* Expert 0 and Expert 1 conflict: resource allocation vs. latency */
    cm->conflicts[0][1] = 1;
    cm->conflicts[1][0] = 1;

    /* Expert 3 and Expert 4 conflict: strict type checking vs. flexibility */
    cm->conflicts[3][4] = 1;
    cm->conflicts[4][3] = 1;
}

/* ================================================================
 * Conflict resolution via NAND
 * ================================================================ */

uint32_t hk_nand_filter(uint32_t active_experts) {
    /* For each pair of active experts, check for conflict */
    uint32_t result = active_experts;

    for (int i = 0; i < 32; i++) {
        if (!(active_experts & (1 << i))) continue;

        for (int j = i + 1; j < 32; j++) {
            if (!(active_experts & (1 << j))) continue;

            /* Both i and j are active */
            /* If they conflict, suppress the lower-weight one (higher index) */
            int conflict = (i % 2) ^ (j % 2);  /* Stub: XOR decides conflict */

            if (conflict) {
                /* Suppress higher-indexed expert */
                result &= ~(1 << j);
            }
        }
    }

    return result;
}

/* ================================================================
 * Multi-level conflict resolution
 * ================================================================ */

uint32_t hk_nand_filter_cascading(uint32_t active_experts) {
    /* Apply NAND filtering iteratively until stable */
    uint32_t current = active_experts;
    uint32_t previous;

    int iterations = 0;
    do {
        previous = current;
        current = hk_nand_filter(current);
        iterations++;
    } while (current != previous && iterations < 10);

    return current;
}

/* ================================================================
 * Authority predicate using NAND
 * ================================================================ */

int hk_nand_authority_check(int capability, int trusted, int approval) {
    /* Authority = capability ∧ trusted ∧ approval */
    /* Built from NAND: a ∧ b ∧ c = NAND(NAND(NAND(a,b),c), NAND(NAND(a,b),c)) */
    int nand_ab = nand(capability, trusted);
    int nand_nand_abc = nand(nand_ab, approval);
    int authority = nand(nand_nand_abc, nand_nand_abc);
    return authority;
}

/* ================================================================
 * Admissibility predicate
 * ================================================================ */

int hk_nand_admissible(int has_proof, int entropy_ok, int not_dangerous) {
    /* Admissible = proof ∧ entropy_ok ∧ ¬dangerous */
    /* = proof ∧ entropy_ok ∧ NOT(dangerous) */
    /* Built from NAND */
    int not_dangerous_gate = not_op(not_dangerous);  /* Double negation */
    int proof_and_entropy = nand(nand(has_proof, entropy_ok),
                                  nand(has_proof, entropy_ok));
    int result = nand(nand(proof_and_entropy, not_dangerous_gate),
                      nand(proof_and_entropy, not_dangerous_gate));
    return result;
}

/* ================================================================
 * Debug: print Boolean circuit diagram
 * ================================================================ */

void hk_nand_print_circuit(void) {
    printf("NAND-based Boolean Circuits:\n\n");

    printf("NOT(a) = NAND(a, a)\n");
    printf("  NOT(0) = %d, NOT(1) = %d\n", not_op(0), not_op(1));

    printf("\nAND(a, b) = NAND(NAND(a,b), NAND(a,b))\n");
    printf("  AND(0,0) = %d, AND(0,1) = %d, AND(1,0) = %d, AND(1,1) = %d\n",
           and_op(0, 0), and_op(0, 1), and_op(1, 0), and_op(1, 1));

    printf("\nOR(a, b) = NAND(NAND(a,a), NAND(b,b))\n");
    printf("  OR(0,0) = %d, OR(0,1) = %d, OR(1,0) = %d, OR(1,1) = %d\n",
           or_op(0, 0), or_op(0, 1), or_op(1, 0), or_op(1, 1));

    printf("\na → b = OR(¬a, b)\n");
    printf("  0→0 = %d, 0→1 = %d, 1→0 = %d, 1→1 = %d\n",
           implies(0, 0), implies(0, 1), implies(1, 0), implies(1, 1));
}
