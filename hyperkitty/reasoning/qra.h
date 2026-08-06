/*
 * qra.h — Discrete Agent Routing Automata (QRA)
 *
 * Routing alphabet (6 glyphs):
 *   Π (PI, 0x01)     - identity/continuation
 *   Γ (GAMMA, 0x03)  - intermediate state
 *   Δ (DELTA, 0x04)  - transition
 *   Ω (OMEGA, 0x0A)  - absorbing element (termination)
 *   Λ (LAMBDA, 0xFF) - left identity
 *   Ψ (PSI, 0x0B)    - control element
 *
 * Routing tensor Q ∈ {0,...,5}^{6×6}:
 *   Q[current_glyph][previous_glyph] = next_glyph
 *
 * Key properties:
 *   - H(next | current, previous) = 0 nats (deterministic)
 *   - Λ is left identity: Q[Λ][j] = j
 *   - Ω is absorber: Q[Ω][j] = Ω
 *   - Token lifetime via algebraic exhaustion (witness → absorption)
 */

#pragma once
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef struct RoutingTensor RoutingTensor;

typedef struct {
    int glyphs[3];  /* [w0, w1, w2]: current witness state */
} Witness;

/**
 * hk_qra_step — Deterministic routing step
 *
 * @param current    Current glyph
 * @param previous   Previous glyph
 * @param rt         Routing tensor
 * @return Next glyph
 */
int hk_qra_step(int current, int previous, RoutingTensor *rt);

/**
 * hk_qra_evolve_witness — Evolve witness vector
 *
 * w' = [Q(w0, w1), Q(w1, w2), Q(w2, w0)]
 *
 * @param w   Current witness
 * @param rt  Routing tensor
 * @return Next witness
 */
Witness hk_qra_evolve_witness(Witness w, RoutingTensor *rt);

/**
 * hk_qra_is_absorbed — Check if witness has reached absorber
 *
 * Returns 1 if all three glyphs are Ω (omega).
 *
 * @param w  Witness
 * @return 1 if absorbed, 0 otherwise
 */
int hk_qra_is_absorbed(Witness w);

/**
 * hk_qra_is_fixed — Check if witness is in fixed-point loop
 *
 * Returns 1 if all three glyphs are Λ (lambda, left identity).
 * Fixed points are INVALID for tokens (no replay resistance).
 *
 * @param w  Witness
 * @return 1 if fixed point, 0 otherwise
 */
int hk_qra_is_fixed(Witness w);

/**
 * hk_qra_token_lifetime — Compute steps to absorption
 *
 * Iteratively evolves witness until absorption (or max_steps reached).
 * Returns number of evolution steps. Rejects fixed points (returns -1).
 *
 * Token lifetime defines stateful replay resistance:
 *   - Token issued with witness w_0
 *   - Each use evolves witness: w_i → w_{i+1}
 *   - After T steps, w_T = [Ω, Ω, Ω] (absorbed, token invalid)
 *
 * @param initial   Initial witness
 * @param rt        Routing tensor
 * @param max_steps Maximum evolution iterations
 * @return Steps to absorption, or -1 if invalid/no absorption
 */
int hk_qra_token_lifetime(Witness initial, RoutingTensor *rt, int max_steps);

/**
 * Debug output
 */
void hk_qra_print_tensor(RoutingTensor *rt);
void hk_qra_print_witness(Witness w);

#ifdef __cplusplus
}
#endif
