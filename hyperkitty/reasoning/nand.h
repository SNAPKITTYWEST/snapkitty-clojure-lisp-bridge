/*
 * nand.h — NAND Filter interface
 * Stage 8: Functionally complete Boolean logic via NAND primitive
 *
 * NAND is the universal Boolean gate: all other operators can be built from it.
 * By using NAND for all expert conflict resolution, we minimize the trusted base.
 *
 * NAND truth table:
 *   NAND(0, 0) = 1
 *   NAND(0, 1) = 1
 *   NAND(1, 0) = 1
 *   NAND(1, 1) = 0
 *
 * Derivations:
 *   NOT(a) = NAND(a, a)
 *   AND(a, b) = NAND(NAND(a, b), NAND(a, b))
 *   OR(a, b) = NAND(NOT(a), NOT(b))
 *   XOR(a, b) = NAND(AND(a, NOT(b)), AND(NOT(a), b))
 */

#pragma once
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

/**
 * hk_nand_filter — Resolve conflicts between active experts
 *
 * For each pair of simultaneously-active experts:
 *   - If they conflict, suppress the lower-weight one
 *   - Use NAND logic to determine suppression
 *
 * @param active_experts  Bitmask of active expert indices (bit i = expert i)
 * @return Filtered bitmask (some experts may be suppressed)
 */
uint32_t hk_nand_filter(uint32_t active_experts);

/**
 * hk_nand_filter_cascading — Iteratively apply NAND filter until stable
 *
 * Applies hk_nand_filter repeatedly until the result converges
 * (i.e., no more experts are suppressed).
 *
 * @param active_experts  Initial active expert bitmask
 * @return Final (stable) expert bitmask
 */
uint32_t hk_nand_filter_cascading(uint32_t active_experts);

/**
 * hk_nand_authority_check — Authority predicate using NAND
 *
 * Authority = capability ∧ trusted ∧ approval
 * Implemented entirely using NAND.
 *
 * @param capability   1 if agent has capability, 0 otherwise
 * @param trusted      1 if agent is trusted, 0 otherwise
 * @param approval     1 if action has been approved, 0 otherwise
 * @return Authority: 1 if all three conditions met, 0 otherwise
 */
int hk_nand_authority_check(int capability, int trusted, int approval);

/**
 * hk_nand_admissible — Admissibility predicate using NAND
 *
 * Admissible = proof ∧ entropy_ok ∧ ¬dangerous
 * Implemented entirely using NAND.
 *
 * @param has_proof    1 if proof certificate exists, 0 otherwise
 * @param entropy_ok   1 if entropy ≤ 0.20 nats, 0 otherwise
 * @param not_dangerous 1 if not a dangerous pattern, 0 otherwise
 * @return Admissibility: 1 if action can propagate, 0 if blocked
 */
int hk_nand_admissible(int has_proof, int entropy_ok, int not_dangerous);

/**
 * hk_nand_print_circuit — Debug output
 *
 * Prints truth tables for NOT, AND, OR, IMPLIES constructed from NAND.
 */
void hk_nand_print_circuit(void);

#ifdef __cplusplus
}
#endif
