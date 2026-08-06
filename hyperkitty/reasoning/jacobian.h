/*
 * jacobian.h — Jacobian Lens interface
 * Stage 5: Condition number + dead path detection
 */

#pragma once

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    HK_OK = 0,
    HK_ERR_NULL = 1,
    HK_ERR_ALLOC = 2,
} hk_status;

typedef struct {
    double condition_number;    /* κ(J) = ||J|| * ||J⁻¹||: sensitivity to perturbations */
    int dead_paths;             /* Number of dimensions with ∂f/∂x_i ≈ 0 */
    hk_status status;
} hk_jacobian_metrics;

/**
 * hk_jacobian_lens — Compute Jacobian-based routing metrics
 *
 * Estimates condition number via finite differences: ∂f_i/∂x_j ≈ (f(x+ε*e_j) - f(x-ε*e_j)) / 2ε
 * Dead paths are dimensions where all partial derivatives are near zero.
 *
 * High condition number indicates ill-conditioned routing (fragile to perturbations).
 * Dead paths indicate dimensions that don't influence the routing output.
 *
 * @param routing_matrix  Adjacency matrix [dim x dim]
 * @param dim             Matrix dimension
 * @return Metrics with condition number and dead path count
 */
hk_jacobian_metrics hk_jacobian_lens(double **routing_matrix, int dim);

/**
 * hk_jacobian_locally_invertible — Check if routing map is invertible
 *
 * Returns 1 if the Jacobian suggests local invertibility:
 *   - condition number < 100
 *   - zero dead paths
 *
 * @param matrix  Routing matrix [dim x dim]
 * @param dim     Matrix dimension
 * @return 1 if locally invertible, 0 otherwise
 */
int hk_jacobian_locally_invertible(double **matrix, int dim);

/**
 * hk_jacobian_print_metrics — Debug output
 */
void hk_jacobian_print_metrics(hk_jacobian_metrics metrics);

#ifdef __cplusplus
}
#endif
