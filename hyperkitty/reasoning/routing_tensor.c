/*
 * routing_tensor.c — QRA Routing Tensor
 *
 * Discrete routing alphabet:
 *   Π = 0x01 (pi, identity/continuation)
 *   Γ = 0x03 (gamma, intermediate state)
 *   Δ = 0x04 (delta, transition)
 *   Ω = 0x0A (omega, absorbing/termination)
 *   Λ = 0xFF (lambda, left identity)
 *   Ψ = 0x0B (psi, control)
 *
 * The routing tensor Q ∈ {0,...,5}^{6×6} maps (current_glyph, prev_glyph) → next_glyph.
 *
 * Key property: H(next | current, previous) = 0 nats
 * The next state is deterministic given current state and one predecessor.
 * Zero entropy: no sampling, no randomness.
 *
 * Distinguished elements:
 *   - Λ (left identity): Q[Λ][j] = j ∀j  (continuation)
 *   - Ω (absorber): Q[Ω][j] = Ω ∀j       (termination sink)
 */

#include "hyperkitty/qra.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

/* ================================================================
 * Routing glyphs and encoding
 * ================================================================ */

#define GLYPH_PI   0    /* Π = 0x01 → index 0 */
#define GLYPH_GAM  1    /* Γ = 0x03 → index 1 */
#define GLYPH_DEL  2    /* Δ = 0x04 → index 2 */
#define GLYPH_OME  3    /* Ω = 0x0A → index 3 (absorber) */
#define GLYPH_LAM  4    /* Λ = 0xFF → index 4 (identity) */
#define GLYPH_PSI  5    /* Ψ = 0x0B → index 5 */

#define NUM_GLYPHS 6

static const char *glyph_name(int g) {
    const char *names[] = { "PI", "GAMMA", "DELTA", "OMEGA", "LAMBDA", "PSI" };
    return (g >= 0 && g < NUM_GLYPHS) ? names[g] : "UNKNOWN";
}

static int glyph_from_byte(uint8_t b) {
    switch (b) {
        case 0x01: return GLYPH_PI;
        case 0x03: return GLYPH_GAM;
        case 0x04: return GLYPH_DEL;
        case 0x0A: return GLYPH_OME;
        case 0xFF: return GLYPH_LAM;
        case 0x0B: return GLYPH_PSI;
        default: return -1;
    }
}

static uint8_t byte_from_glyph(int g) {
    const uint8_t bytes[] = { 0x01, 0x03, 0x04, 0x0A, 0xFF, 0x0B };
    return (g >= 0 && g < NUM_GLYPHS) ? bytes[g] : 0;
}

/* ================================================================
 * Routing tensor Q[6][6]
 * ================================================================ */

typedef struct {
    int Q[NUM_GLYPHS][NUM_GLYPHS];  /* Next glyph given (current, prev) */
} RoutingTensor;

static RoutingTensor *routing_tensor_alloc(void) {
    RoutingTensor *rt = malloc(sizeof(RoutingTensor));
    if (!rt) return NULL;

    /* Initialize: identity behavior by default */
    for (int i = 0; i < NUM_GLYPHS; i++) {
        for (int j = 0; j < NUM_GLYPHS; j++) {
            rt->Q[i][j] = i;  /* Stay in current glyph */
        }
    }

    /* Lambda (left identity): Q[LAM][j] = j (continue to j) */
    for (int j = 0; j < NUM_GLYPHS; j++) {
        rt->Q[GLYPH_LAM][j] = j;
    }

    /* Omega (absorber): Q[OME][j] = OME (sink to absorber) */
    for (int j = 0; j < NUM_GLYPHS; j++) {
        rt->Q[GLYPH_OME][j] = GLYPH_OME;
    }

    /* Example transitions */
    rt->Q[GLYPH_PI][GLYPH_GAM] = GLYPH_DEL;    /* PI ← GAM → DEL */
    rt->Q[GLYPH_GAM][GLYPH_PI] = GLYPH_DEL;    /* GAM ← PI → DEL */
    rt->Q[GLYPH_DEL][GLYPH_DEL] = GLYPH_OME;   /* DEL ← DEL → OME (absorb) */

    return rt;
}

void routing_tensor_free(RoutingTensor *rt) {
    free(rt);
}

/* ================================================================
 * Witness evolution (for token state machines)
 * ================================================================ */

int hk_qra_step(int current, int previous, RoutingTensor *rt) {
    if (!rt || current < 0 || current >= NUM_GLYPHS ||
        previous < 0 || previous >= NUM_GLYPHS) {
        return GLYPH_OME;  /* Default to absorber on error */
    }

    return rt->Q[current][previous];
}

/* ================================================================
 * Witness progression (used in token state)
 * ================================================================ */

typedef struct {
    int glyphs[3];  /* Current witness state: [w0, w1, w2] */
} Witness;

static Witness witness_init(int g0, int g1, int g2) {
    Witness w;
    w.glyphs[0] = g0;
    w.glyphs[1] = g1;
    w.glyphs[2] = g2;
    return w;
}

Witness hk_qra_evolve_witness(Witness w, RoutingTensor *rt) {
    /* Evolve: w' = [Q(w0, w1), Q(w1, w2), Q(w2, w0)] */
    Witness next;
    next.glyphs[0] = hk_qra_step(w.glyphs[0], w.glyphs[1], rt);
    next.glyphs[1] = hk_qra_step(w.glyphs[1], w.glyphs[2], rt);
    next.glyphs[2] = hk_qra_step(w.glyphs[2], w.glyphs[0], rt);
    return next;
}

/* ================================================================
 * Absorption detection
 * ================================================================ */

int hk_qra_is_absorbed(Witness w) {
    /* Absorbed if all glyphs are omega */
    return (w.glyphs[0] == GLYPH_OME &&
            w.glyphs[1] == GLYPH_OME &&
            w.glyphs[2] == GLYPH_OME);
}

int hk_qra_is_fixed(Witness w) {
    /* Fixed if all glyphs are lambda (identity) */
    return (w.glyphs[0] == GLYPH_LAM &&
            w.glyphs[1] == GLYPH_LAM &&
            w.glyphs[2] == GLYPH_LAM);
}

/* ================================================================
 * Token lifetime via algebraic exhaustion
 * ================================================================ */

typedef struct {
    uint64_t sequence;
    Witness witness;
    int steps_to_absorption;  /* How many evolution steps until absorption */
} TokenState;

int hk_qra_token_lifetime(Witness initial, RoutingTensor *rt, int max_steps) {
    if (!rt) return -1;

    Witness w = initial;

    /* Fixed points (lambda loop) are invalid—tokens with all-lambda witness
     * never exhaust and would bypass replay resistance. */
    if (hk_qra_is_fixed(w)) {
        return -1;
    }

    for (int step = 0; step < max_steps; step++) {
        w = hk_qra_evolve_witness(w, rt);
        if (hk_qra_is_absorbed(w)) {
            return step;
        }
    }

    return -1;  /* Did not absorb within max_steps */
}

/* ================================================================
 * Debug output
 * ================================================================ */

void hk_qra_print_tensor(RoutingTensor *rt) {
    if (!rt) return;

    printf("Routing Tensor Q[current][previous]:\n\n");
    printf("     ");
    for (int j = 0; j < NUM_GLYPHS; j++) {
        printf("%8s ", glyph_name(j));
    }
    printf("\n");

    for (int i = 0; i < NUM_GLYPHS; i++) {
        printf("%5s ", glyph_name(i));
        for (int j = 0; j < NUM_GLYPHS; j++) {
            printf("%8s ", glyph_name(rt->Q[i][j]));
        }
        printf("\n");
    }
}

void hk_qra_print_witness(Witness w) {
    printf("Witness: [%s, %s, %s]\n",
           glyph_name(w.glyphs[0]),
           glyph_name(w.glyphs[1]),
           glyph_name(w.glyphs[2]));
}
