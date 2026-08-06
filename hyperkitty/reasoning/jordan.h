/*
 * jordan.h — Jordan Spectral Transformer
 * Stage 4: Eigendecomposition and stable attractor identification
 *
 * Mathematical framework:
 *
 * Density-matrix evolution: ρ' = φ⁻¹ U ρ U† + φ⁻² ρ
 *   φ = (1+√5)/2 (golden ratio)
 *   Fixed points: [U, ρ*] = 0 (commutation)
 *   Stable attractors: e ∘ e = e (idempotents)
 *   Routing decisiveness: λ₊ - λ₋ (spectral gap)
 */

#pragma once
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    HK_OK = 0,
    HK_ERR_NULL = 1,
    HK_ERR_ALLOC = 2,
} hk_status;

typedef struct {
    double spectral_radius;         /* max |λ| */
    double stability_score;         /* [0,1]: how quickly system decays */
    int jordan_blocks;              /* Number of Jordan blocks */
    double gershgorin_radius;       /* Upper bound on eigenvalue magnitudes */
    double spectral_gap;            /* λ₊ - λ₋: routing decisiveness */
    double convergence_tolerance;   /* Tolerance for fixed-point detection */
    hk_status status;
} hk_jordan_features;

/**
 * hk_jordan_transform — Compute spectral features of routing graph
 *
 * Eigendecomposes the adjacency matrix and extracts:
 *   - Spectral radius (largest |λ|)
 *   - Stability score (measure of decay)
 *   - Jordan block structure
 *   - Gershgorin circle radius
 *   - Spectral gap (λ₊ - λ₋)
 *
 * @param adj_matrix  Adjacency matrix [dim x dim]
 * @param dim         Matrix dimension
 * @return Features with status code
 */
hk_jordan_features hk_jordan_transform(double **adj_matrix, int dim);

/**
 * hk_jordan_detect_fixed_point — Check if matrix is a fixed point
 *
 * A fixed point ρ* satisfies [U, ρ*] = 0 (commutes with evolution).
 *
 * @param matrix      Input matrix [dim x dim]
 * @param dim         Matrix dimension
 * @param tolerance   Tolerance for commutation check
 * @return 1 if fixed point detected, 0 otherwise
 */
int hk_jordan_detect_fixed_point(double **matrix, int dim,
                                  double tolerance);

/**
 * hk_jordan_converge_idempotent — Iterate to idempotent
 *
 * Repeatedly squares the matrix A until A² = A (idempotent).
 * Stable routing attractors satisfy e ∘ e = e.
 *
 * @param matrix      Input matrix [dim x dim]
 * @param dim         Matrix dimension
 * @param max_iter    Maximum iteration count
 * @param tolerance   Convergence tolerance
 * @return Number of iterations to convergence, -1 if failed
 */
int hk_jordan_converge_idempotent(double **matrix, int dim,
                                   int max_iterations,
                                   double tolerance);

/**
 * hk_jordan_print_features — Debug output
 */
void hk_jordan_print_features(hk_jordan_features feat);

#ifdef __cplusplus
}
#endif
