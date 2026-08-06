/*
 * ast.h — Inverted Abstract Syntax Tree interface
 * Stage 2 of routing pipeline: AST construction with payload/structural separation
 *
 * Payload nodes have weight = 0.0 and CANNOT control routing.
 * Structural nodes have weight = 1.0 and form the routing authority.
 */

#pragma once
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    HK_OK = 0,
    HK_ERR_NULL = 1,
    HK_ERR_ALLOC = 2,
    HK_ERR_INVALID_STATE = 3,
    HK_ERR_CAPABILITY_DENIED = 4,
    HK_ERR_UNTRUSTED_AGENT = 5,
    HK_ERR_PROOF_MISSING = 6,
    HK_ERR_APPROVAL_THRESHOLD = 7,
    HK_ERR_VERIFICATION = 8,
    HK_ERR_PARSE_REJECTED = 9
} hk_status;

/* Opaque types */
typedef struct ASTNode ASTNode;
typedef struct ASTGraph ASTGraph;

/**
 * Node types:
 *   Structural (weight=1.0): PROGRAM, FUNCTION, CLASS, IF, LOOP, BLOCK, TYPE, CONSTRAINT, ROUTE
 *   Payload (weight=0.0): IDENTIFIER, LITERAL_*, COMMENT, PARAMETER
 */

typedef enum {
    AST_UNKNOWN,
    AST_PROGRAM, AST_FUNCTION, AST_CLASS, AST_INTERFACE, AST_MODULE,
    AST_IF, AST_LOOP, AST_BLOCK, AST_TYPE, AST_CONSTRAINT, AST_ROUTE,
    AST_IDENTIFIER, AST_LITERAL_INT, AST_LITERAL_STR, AST_LITERAL_BOOL,
    AST_LITERAL_NULL, AST_COMMENT, AST_PARAMETER,
    AST_NUM_TYPES
} ASTNodeType;

/**
 * hk_ast_node_alloc — Create AST node
 *
 * @param type   Node type (determines weight)
 * @param label  Optional label/name
 * @return Allocated node, or NULL on error
 */
ASTNode *hk_ast_node_alloc(ASTNodeType type, const char *label);

/**
 * hk_ast_node_free — Free node and all descendants
 */
void hk_ast_node_free(ASTNode *node);

/**
 * hk_ast_add_child — Add child node to parent
 *
 * Enforces security:
 *   - Payload nodes cannot have structural children (returns HK_ERR_INVALID_STATE)
 *   - Structural nodes can have both structural and payload children
 *   - Edge weights are assigned automatically
 *
 * @param parent   Parent node
 * @param child    Child node
 * @return HK_OK if added, HK_ERR_* if rejected
 */
hk_status hk_ast_add_child(ASTNode *parent, ASTNode *child);

/**
 * hk_ast_compute_weight — Compute cumulative weight of subtree
 *
 * Weight propagates from leaves up through edges:
 *   - structural → structural: weight = 1.0 * child_weight
 *   - structural → payload: weight = 0.1 * child_weight (muted)
 *   - payload → *: weight = 0 (blocked)
 *
 * @param node  Root of subtree
 * @return Cumulative weight
 */
double hk_ast_compute_weight(ASTNode *node);

/**
 * hk_ast_to_graph — Serialize AST to adjacency matrix
 *
 * Returns weighted adjacency matrix for next stage (SymbolicGraph).
 * Matrix dimensions = number of nodes in tree.
 * Matrix[i][j] = edge weight from node i to node j.
 *
 * @param root  Root of AST
 * @return Allocated graph, or NULL on error
 */
ASTGraph *hk_ast_to_graph(ASTNode *root);

/**
 * hk_ast_graph_free — Free adjacency matrix representation
 */
void hk_ast_graph_free(ASTGraph *graph);

/**
 * hk_ast_print — Pretty-print AST for debugging
 *
 * @param node   Root node
 * @param depth  Initial indentation depth (usually 0)
 */
void hk_ast_print(ASTNode *node, int depth);

#ifdef __cplusplus
}
#endif
