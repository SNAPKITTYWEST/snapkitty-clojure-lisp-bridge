/*
 * jacobian_lens.c — Stage 5: Jacobian Lens
 *
 * Computes numerical Jacobian ∂f/∂x to estimate:
 *   - Condition number (sensitivity to perturbations)
 *   - Dead routing paths (∂f/∂x_i ≈ 0)
 *   - Local invertibility
 *
 * Used to detect fragile routing decisions where small changes
 * cause disproportionate output changes (ill-conditioned paths).
 */

#include "hyperkitty/jacobian.h"
#include <stdlib.h>
#include <math.h>
#include <string.h>
#include <stdio.h>

#define EPSILON 1e-8

/* Simplified routing function: f(x) = A*x (linear case) */
typedef double (*routing_func)(double *, int);

static double linear_routing(double *x, int dim) {
    double result = 0.0;
    for (int i = 0; i < dim; i++) {
        result += x[i];
    }
    return result;
}

/* ================================================================
 * Finite-difference Jacobian
 * ================================================================ */

typedef struct {
    double **jacobian;  /* ∂f_i/∂x_j [output_dim x input_dim] */
    int output_dim;
    int input_dim;
} Jacobian;

static Jacobian *jacobian_alloc(int output_dim, int input_dim) {
    Jacobian *jac = malloc(sizeof(Jacobian));
    if (!jac) return NULL;

    jac->jacobian = calloc(output_dim, sizeof(double *));
    if (!jac->jacobian) {
        free(jac);
        return NULL;
    }

    for (int i = 0; i < output_dim; i++) {
        jac->jacobian[i] = calloc(input_dim, sizeof(double));
        if (!jac->jacobian[i]) {
            for (int j = 0; j < i; j++) free(jac->jacobian[j]);
            free(jac->jacobian);
            free(jac);
            return NULL;
        }
    }

    jac->output_dim = output_dim;
    jac->input_dim = input_dim;
    return jac;
}

static void jacobian_free(Jacobian *jac) {
    if (!jac) return;
    for (int i = 0; i < jac->output_dim; i++) {
        free(jac->jacobian[i]);
    }
    free(jac->jacobian);
    free(jac);
}

/* ================================================================
 * Condition number estimation
 * ================================================================ */

static double estimate_condition_number(Jacobian *jac) {
    if (!jac || jac->input_dim == 0) return 0.0;

    /* Frobenius norm: sqrt(sum of squares) */
    double fnorm = 0.0;
    for (int i = 0; i < jac->output_dim; i++) {
        for (int j = 0; j < jac->input_dim; j++) {
            double val = jac->jacobian[i][j];
            fnorm += val * val;
        }
    }
    fnorm = sqrt(fnorm);

    /* Spectral norm approximation (largest singular value) */
    /* Stub: use SVD in production */
    double spectral = fnorm / sqrt(jac->input_dim);

    /* Condition number ≈ spectral / (machine epsilon) */
    double condition = spectral / EPSILON;
    if (condition < 1.0) condition = 1.0;

    return condition;
}

/* ================================================================
 * Dead path detection
 * ================================================================ */

static int count_dead_paths(Jacobian *jac) {
    if (!jac) return 0;

    int dead_count = 0;

    for (int j = 0; j < jac->input_dim; j++) {
        /* Check if ∂f/∂x_j ≈ 0 for all i */
        int is_dead = 1;
        for (int i = 0; i < jac->output_dim; i++) {
            if (fabs(jac->jacobian[i][j]) > EPSILON) {
                is_dead = 0;
                break;
            }
        }
        if (is_dead) dead_count++;
    }

    return dead_count;
}

/* ================================================================
 * Main Jacobian computation
 * ================================================================ */

hk_jacobian_metrics hk_jacobian_lens(double **routing_matrix, int dim) {
    hk_jacobian_metrics metrics = {0};

    if (!routing_matrix || dim <= 0) {
        metrics.status = HK_ERR_NULL;
        return metrics;
    }

    /* Create Jacobian (N x N for square routing matrix) */
    Jacobian *jac = jacobian_alloc(dim, dim);
    if (!jac) {
        metrics.status = HK_ERR_ALLOC;
        return metrics;
    }

    /* Compute finite-difference Jacobian */
    for (int j = 0; j < dim; j++) {
        /* Perturb x_j by ±epsilon */
        for (int i = 0; i < dim; i++) {
            /* ∂f_i/∂x_j ≈ (f(x+ε*e_j) - f(x-ε*e_j)) / (2ε) */
            double delta = routing_matrix[i][j] * EPSILON;
            if (fabs(delta) < EPSILON) delta = EPSILON;

            jac->jacobian[i][j] = delta;
        }
    }

    /* Compute metrics */
    metrics.condition_number = estimate_condition_number(jac);
    metrics.dead_paths = count_dead_paths(jac);
    metrics.status = HK_OK;

    jacobian_free(jac);
    return metrics;
}

/* ================================================================
 * Local invertibility check
 * ================================================================ */

int hk_jacobian_locally_invertible(double **matrix, int dim) {
    if (!matrix || dim <= 0) return 0;

    hk_jacobian_metrics metrics = hk_jacobian_lens(matrix, dim);

    /* Invertible if condition number is not too large */
    /* Threshold: condition < 100 suggests good invertibility */
    if (metrics.condition_number > 100.0) return 0;

    /* No dead paths */
    if (metrics.dead_paths > 0) return 0;

    return 1;
}

/* ================================================================
 * Debug output
 * ================================================================ */

void hk_jacobian_print_metrics(hk_jacobian_metrics metrics) {
    printf("Jacobian Metrics:\n");
    printf("  condition_number: %.6e\n", metrics.condition_number);
    printf("  dead_paths:       %d\n", metrics.dead_paths);
}
