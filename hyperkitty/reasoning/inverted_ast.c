/*
 * inverted_ast.c — Stage 2: Inverted Abstract Syntax Tree
 *
 * Core security principle: Payload nodes CANNOT control routing.
 *
 * Node weights:
 *   - Structural (control flow, declarations, types): weight = 1.0
 *   - Payload (literals, data, identifiers, comments):  weight = 0.0
 *
 * Edges:
 *   - payload → structural: weight = 0 (blocked)
 *   - structural → structural: weight = 1
 *   - structural → payload: weight = 0.1 (muted, carries data only)
 *   - payload ↔ payload: weight = 0 (no sibling authority)
 *
 * This separation guarantees that routing authority derives ONLY from
 * structural nodes (language keywords, syntactic markers, type annotations).
 * Adversarial payloads cannot propagate routing weight upward.
 */

#include "hyperkitty/ast.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>

/* ================================================================
 * Node types and weights
 * ================================================================ */

typedef enum {
    AST_UNKNOWN,

    /* Structural nodes (weight = 1.0) */
    AST_PROGRAM,        /* Root */
    AST_FUNCTION,       /* Function declaration */
    AST_CLASS,          /* Class declaration */
    AST_INTERFACE,      /* Interface/trait */
    AST_MODULE,         /* Module/namespace */
    AST_IF,             /* if statement */
    AST_LOOP,           /* for/while/loop */
    AST_BLOCK,          /* { ... } block */
    AST_TYPE,           /* Type annotation */
    AST_CONSTRAINT,     /* Constraint declaration */
    AST_ROUTE,          /* Routing decision point */

    /* Payload nodes (weight = 0.0) */
    AST_IDENTIFIER,     /* Variable/function name */
    AST_LITERAL_INT,    /* Integer literal */
    AST_LITERAL_STR,    /* String literal */
    AST_LITERAL_BOOL,   /* Boolean literal */
    AST_LITERAL_NULL,   /* Null literal */
    AST_COMMENT,        /* Comment text */
    AST_PARAMETER,      /* Function parameter (name only) */

    AST_NUM_TYPES
} ASTNodeType;

typedef struct ASTNode {
    ASTNodeType type;
    double weight;          /* 1.0 for structural, 0.0 for payload */
    char *label;
    char *value;            /* For payload nodes: literal text */
    struct ASTNode **children;
    int child_count;
    int parent_idx;         /* Back-reference to parent */
} ASTNode;

/* ================================================================
 * Node allocation and type classification
 * ================================================================ */

static double get_node_weight(ASTNodeType type) {
    switch (type) {
        /* Structural nodes */
        case AST_PROGRAM:
        case AST_FUNCTION:
        case AST_CLASS:
        case AST_INTERFACE:
        case AST_MODULE:
        case AST_IF:
        case AST_LOOP:
        case AST_BLOCK:
        case AST_TYPE:
        case AST_CONSTRAINT:
        case AST_ROUTE:
            return 1.0;

        /* Payload nodes */
        case AST_IDENTIFIER:
        case AST_LITERAL_INT:
        case AST_LITERAL_STR:
        case AST_LITERAL_BOOL:
        case AST_LITERAL_NULL:
        case AST_COMMENT:
        case AST_PARAMETER:
        default:
            return 0.0;
    }
}

static const char *node_type_name(ASTNodeType type) {
    const char *names[] = {
        [AST_UNKNOWN] = "UNKNOWN",
        [AST_PROGRAM] = "PROGRAM",
        [AST_FUNCTION] = "FUNCTION",
        [AST_CLASS] = "CLASS",
        [AST_INTERFACE] = "INTERFACE",
        [AST_MODULE] = "MODULE",
        [AST_IF] = "IF",
        [AST_LOOP] = "LOOP",
        [AST_BLOCK] = "BLOCK",
        [AST_TYPE] = "TYPE",
        [AST_CONSTRAINT] = "CONSTRAINT",
        [AST_ROUTE] = "ROUTE",
        [AST_IDENTIFIER] = "IDENTIFIER",
        [AST_LITERAL_INT] = "LITERAL_INT",
        [AST_LITERAL_STR] = "LITERAL_STR",
        [AST_LITERAL_BOOL] = "LITERAL_BOOL",
        [AST_LITERAL_NULL] = "LITERAL_NULL",
        [AST_COMMENT] = "COMMENT",
        [AST_PARAMETER] = "PARAMETER",
    };
    return names[type];
}

ASTNode *hk_ast_node_alloc(ASTNodeType type, const char *label) {
    ASTNode *node = calloc(1, sizeof(ASTNode));
    if (!node) return NULL;

    node->type = type;
    node->weight = get_node_weight(type);
    node->children = calloc(16, sizeof(ASTNode *));
    if (!node->children) {
        free(node);
        return NULL;
    }
    node->child_count = 0;
    node->parent_idx = -1;

    if (label) {
        node->label = malloc(strlen(label) + 1);
        strcpy(node->label, label);
    }

    return node;
}

void hk_ast_node_free(ASTNode *node) {
    if (!node) return;
    for (int i = 0; i < node->child_count; i++) {
        hk_ast_node_free(node->children[i]);
    }
    free(node->children);
    free(node->label);
    free(node->value);
    free(node);
}

/* ================================================================
 * Edge insertion with weight enforcement
 * ================================================================ */

hk_status hk_ast_add_child(ASTNode *parent, ASTNode *child) {
    if (!parent || !child) return HK_ERR_NULL;

    /* Security: payload → structural is blocked (weight = 0) */
    if (parent->weight == 0.0 && child->weight == 1.0) {
        return HK_ERR_INVALID_STATE; /* Payload cannot claim structural authority */
    }

    if (parent->child_count >= 16) return HK_ERR_ALLOC;

    parent->children[parent->child_count] = child;
    child->parent_idx = parent->child_count;
    parent->child_count++;

    return HK_OK;
}

/* ================================================================
 * Edge weight calculation
 * ================================================================ */

static double get_edge_weight(ASTNode *from, ASTNode *to) {
    if (!from || !to) return 0.0;

    /* structural → structural: weight = 1.0 */
    if (from->weight == 1.0 && to->weight == 1.0) {
        return 1.0;
    }

    /* structural → payload: weight = 0.1 (muted, carries data only) */
    if (from->weight == 1.0 && to->weight == 0.0) {
        return 0.1;
    }

    /* payload → *: weight = 0 (blocked) */
    return 0.0;
}

/* ================================================================
 * Tree traversal and weight propagation
 * ================================================================ */

double hk_ast_compute_weight(ASTNode *node) {
    if (!node) return 0.0;

    double total = node->weight;

    for (int i = 0; i < node->child_count; i++) {
        double edge_weight = get_edge_weight(node, node->children[i]);
        double child_weight = hk_ast_compute_weight(node->children[i]);
        total += edge_weight * child_weight;
    }

    return total;
}

/* ================================================================
 * Serialization to adjacency matrix for next stage
 * ================================================================ */

typedef struct {
    double **adj;        /* Adjacency matrix */
    int dim;
    char **node_labels;
} ASTGraph;

static int count_nodes(ASTNode *root) {
    if (!root) return 0;
    int count = 1;
    for (int i = 0; i < root->child_count; i++) {
        count += count_nodes(root->children[i]);
    }
    return count;
}

static void flatten_tree(ASTNode *root, ASTNode **flat, int *idx) {
    if (!root) return;
    flat[*idx] = root;
    (*idx)++;
    for (int i = 0; i < root->child_count; i++) {
        flatten_tree(root->children[i], flat, idx);
    }
}

ASTGraph *hk_ast_to_graph(ASTNode *root) {
    if (!root) return NULL;

    int dim = count_nodes(root);
    ASTGraph *graph = calloc(1, sizeof(ASTGraph));
    if (!graph) return NULL;

    graph->dim = dim;
    graph->adj = calloc(dim, sizeof(double *));
    graph->node_labels = calloc(dim, sizeof(char *));

    for (int i = 0; i < dim; i++) {
        graph->adj[i] = calloc(dim, sizeof(double));
    }

    /* Flatten tree */
    ASTNode **flat = calloc(dim, sizeof(ASTNode *));
    int idx = 0;
    flatten_tree(root, flat, &idx);

    /* Build adjacency matrix */
    for (int i = 0; i < dim; i++) {
        graph->node_labels[i] = malloc(256);
        snprintf(graph->node_labels[i], 256, "%s_%d",
                node_type_name(flat[i]->type), i);

        for (int j = 0; j < flat[i]->child_count; j++) {
            ASTNode *child = flat[i]->children[j];
            /* Find child index in flat array */
            for (int k = 0; k < dim; k++) {
                if (flat[k] == child) {
                    graph->adj[i][k] = get_edge_weight(flat[i], child);
                    break;
                }
            }
        }
    }

    free(flat);
    return graph;
}

void hk_ast_graph_free(ASTGraph *graph) {
    if (!graph) return;
    for (int i = 0; i < graph->dim; i++) {
        free(graph->adj[i]);
        free(graph->node_labels[i]);
    }
    free(graph->adj);
    free(graph->node_labels);
    free(graph);
}

/* ================================================================
 * Pretty-print for debugging
 * ================================================================ */

void hk_ast_print(ASTNode *node, int depth) {
    if (!node) return;

    for (int i = 0; i < depth; i++) printf("  ");
    printf("[%s] (w=%.1f)\n", node_type_name(node->type), node->weight);

    for (int i = 0; i < node->child_count; i++) {
        hk_ast_print(node->children[i], depth + 1);
    }
}
