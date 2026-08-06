/*
 * master_pipeline.h — Master Routing Pipeline Orchestrator
 * Orchestrates all 10 stages: Parser → AST → Graph → Jordan → Jacobian → Constraints → Activation → NAND → Dispatch → Merge
 */

#pragma once
#include <stdint.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    HK_PIPELINE_OK = 0,
    HK_PIPELINE_REJECTED = 1,
    HK_PIPELINE_ERROR = 2
} hk_pipeline_status;

typedef struct {
    hk_pipeline_status status;
    const char *error;              /* Error reason if rejected */
    char *output;                   /* Routed output */
    uint32_t expert_mask;           /* Bitmask of active experts */
    double entropy;                 /* H(next | current, prev) in nats */
    uint8_t worm_hash[32];          /* SHA-256 of execution */
} hk_pipeline_receipt;

/**
 * hk_routing_pipeline_execute — Execute complete 10-stage routing pipeline
 *
 * Orchestrates:
 *   1. RegexParser - danger detection
 *   2. ASTBuilder - inverted AST construction
 *   3. SymbolicGraph - adjacency matrix
 *   4. JordanTransformer - spectral decomposition
 *   5. JacobianLens - condition number estimation
 *   6. ConstraintEval - routing constraints
 *   7. SparseActivation - expert gating
 *   8. NANDFilter - Boolean conflict resolution
 *   9. AgentDispatch - concurrent expert execution
 *  10. MergeOutput - output recombination
 *
 * Zero entropy: H(next | current, previous) = 0 nats
 *
 * @param input  Input text to route
 * @return Pipeline receipt with output and metadata
 */
hk_pipeline_receipt hk_routing_pipeline_execute(const char *input);

/**
 * hk_pipeline_receipt_print — Print human-readable receipt
 */
void hk_pipeline_receipt_print(hk_pipeline_receipt receipt);

#ifdef __cplusplus
}
#endif
