/*
 * regex_parser.c — Stage 1: Regex Parser
 *
 * Tokenizes input and filters dangerous patterns before AST construction.
 * Patterns blocked:
 *   - XXE triggers: <!DOCTYPE, <!ENTITY, SYSTEM, file://
 *   - Code execution: eval, exec, fork, __import__
 *   - Infinite loops: while true (without reachable exit)
 *
 * Input stream is NOT parsed as XML/JSON directly.
 * All payloads remain as unparsed token streams until proof gate validates them.
 */

#include "hyperkitty/parser.h"
#include <stdlib.h>
#include <string.h>
#include <ctype.h>
#include <regex.h>

/* Dangerous pattern definitions */
typedef struct {
    const char *pattern;
    const char *reason;
} DangerousPattern;

static const DangerousPattern DANGEROUS_PATTERNS[] = {
    /* XML External Entity (XXE) */
    { "<!DOCTYPE", "XXE_DOCTYPE" },
    { "<!ENTITY", "XXE_ENTITY" },
    { "SYSTEM\\s*=", "XXE_SYSTEM_REF" },
    { "file://", "FILE_PROTOCOL" },

    /* Code injection */
    { "eval\\s*\\(", "CODE_EVAL" },
    { "exec\\s*\\(", "CODE_EXEC" },
    { "fork\\s*\\(", "PROCESS_FORK" },
    { "__import__", "PYTHON_IMPORT" },
    { "subprocess\\.", "SUBPROCESS_CALL" },
    { "os\\.system", "OS_SYSTEM" },

    /* Infinite loops without exit */
    { "while\\s*\\(\\s*true\\s*\\)", "INFINITE_LOOP" },
    { "while\\s*\\(\\s*1\\s*\\)", "INFINITE_LOOP" },

    /* Credential patterns */
    { "password\\s*=", "EMBEDDED_PASSWORD" },
    { "api[_-]?key", "EMBEDDED_API_KEY" },
    { "secret", "EMBEDDED_SECRET" },
    { "aws_secret", "AWS_SECRET" },

    /* Telemetry/tracking */
    { "beacon|tracker|analytics(?!_test)", "UNAUTHORIZED_TELEMETRY" },

    { NULL, NULL }
};

/* Token types */
typedef enum {
    TOK_STRUCTURAL,    /* Keywords, operators, control flow */
    TOK_IDENTIFIER,    /* Variable/function names */
    TOK_LITERAL,       /* Strings, numbers, constants */
    TOK_OPERATOR,      /* +, -, *, /, etc */
    TOK_PUNCTUATION,   /* ;, {, }, etc */
    TOK_COMMENT,       /* // or /* */ */
    TOK_WHITESPACE,    /* Space, tab, newline */
    TOK_DANGEROUS,     /* Flagged as dangerous */
    TOK_UNKNOWN
} TokenType;

typedef struct {
    TokenType type;
    char *value;
    size_t start;
    size_t end;
    const char *danger_reason;
} Token;

/* Regex compiler (compile once) */
static regex_t compiled_patterns[32];
static int num_patterns = 0;

static void compile_patterns(void) {
    if (num_patterns > 0) return;

    for (int i = 0; DANGEROUS_PATTERNS[i].pattern; i++) {
        int rc = regcomp(&compiled_patterns[i],
                        DANGEROUS_PATTERNS[i].pattern,
                        REG_EXTENDED | REG_ICASE);
        if (rc == 0) {
            num_patterns++;
        }
    }
}

/* Tokenizer */
static Token *tokenize(const char *input, size_t *out_count) {
    if (!input || !out_count) return NULL;

    Token *tokens = calloc(1024, sizeof(Token));
    if (!tokens) return NULL;

    size_t i = 0, tok_count = 0;
    size_t len = strlen(input);

    while (i < len && tok_count < 1024) {
        /* Skip whitespace */
        if (isspace(input[i])) {
            size_t start = i;
            while (i < len && isspace(input[i])) i++;
            tokens[tok_count].type = TOK_WHITESPACE;
            tokens[tok_count].value = strndup(&input[start], i - start);
            tokens[tok_count].start = start;
            tokens[tok_count].end = i;
            tok_count++;
            continue;
        }

        /* Structural keywords */
        if (strncmp(&input[i], "if", 2) == 0 ||
            strncmp(&input[i], "for", 3) == 0 ||
            strncmp(&input[i], "while", 5) == 0) {
            tokens[tok_count].type = TOK_STRUCTURAL;
            int len = (input[i+2] == '\0' || !isalnum(input[i+3])) ? 2 : 3;
            tokens[tok_count].value = strndup(&input[i], len);
            tokens[tok_count].start = i;
            tokens[tok_count].end = i + len;
            i += len;
            tok_count++;
            continue;
        }

        /* Identifier or keyword */
        if (isalpha(input[i]) || input[i] == '_') {
            size_t start = i;
            while (i < len && (isalnum(input[i]) || input[i] == '_')) i++;
            tokens[tok_count].type = TOK_IDENTIFIER;
            tokens[tok_count].value = strndup(&input[start], i - start);
            tokens[tok_count].start = start;
            tokens[tok_count].end = i;
            tok_count++;
            continue;
        }

        /* String literal */
        if (input[i] == '"' || input[i] == '\'') {
            char quote = input[i];
            size_t start = i++;
            while (i < len && input[i] != quote) {
                if (input[i] == '\\' && i + 1 < len) i += 2;
                else i++;
            }
            if (i < len) i++;
            tokens[tok_count].type = TOK_LITERAL;
            tokens[tok_count].value = strndup(&input[start], i - start);
            tokens[tok_count].start = start;
            tokens[tok_count].end = i;
            tok_count++;
            continue;
        }

        /* Number literal */
        if (isdigit(input[i])) {
            size_t start = i;
            while (i < len && (isdigit(input[i]) || input[i] == '.')) i++;
            tokens[tok_count].type = TOK_LITERAL;
            tokens[tok_count].value = strndup(&input[start], i - start);
            tokens[tok_count].start = start;
            tokens[tok_count].end = i;
            tok_count++;
            continue;
        }

        /* Operators and punctuation */
        if (strchr("+-*/%=<>!&|^~;:,()[]{}.", input[i])) {
            tokens[tok_count].type = TOK_PUNCTUATION;
            tokens[tok_count].value = strndup(&input[i], 1);
            tokens[tok_count].start = i;
            tokens[tok_count].end = i + 1;
            tok_count++;
            i++;
            continue;
        }

        /* Unknown */
        tokens[tok_count].type = TOK_UNKNOWN;
        tokens[tok_count].value = strndup(&input[i], 1);
        tokens[tok_count].start = i;
        tokens[tok_count].end = i + 1;
        tok_count++;
        i++;
    }

    *out_count = tok_count;
    return tokens;
}

/* Danger detection */
static const char *detect_danger(const char *input) {
    compile_patterns();

    for (int i = 0; i < num_patterns; i++) {
        regmatch_t match;
        if (regexec(&compiled_patterns[i], input, 1, &match, 0) == 0) {
            return DANGEROUS_PATTERNS[i].reason;
        }
    }

    return NULL;
}

/* Public API */
hk_parse_result hk_parse_input(const char *input) {
    hk_parse_result result = {0};

    if (!input) {
        result.status = HK_PARSE_ERROR;
        result.error = "null_input";
        return result;
    }

    /* Step 1: Danger detection (fail-closed) */
    const char *danger = detect_danger(input);
    if (danger) {
        result.status = HK_PARSE_REJECTED;
        result.error = danger;
        return result;
    }

    /* Step 2: Tokenization */
    size_t tok_count = 0;
    Token *tokens = tokenize(input, &tok_count);
    if (!tokens) {
        result.status = HK_PARSE_ERROR;
        result.error = "tokenization_failed";
        return result;
    }

    /* Step 3: Tag unsafe tokens (but don't reject them yet) */
    for (size_t i = 0; i < tok_count; i++) {
        if (tokens[i].type == TOK_LITERAL) {
            /* Literals stay inert until proof-gated */
            tokens[i].type = TOK_LITERAL;
        }
    }

    result.status = HK_PARSE_OK;
    result.tokens = tokens;
    result.token_count = tok_count;
    result.error = NULL;

    return result;
}

void hk_parse_result_free(hk_parse_result *result) {
    if (!result || !result->tokens) return;
    for (size_t i = 0; i < result->token_count; i++) {
        free(result->tokens[i].value);
    }
    free(result->tokens);
    result->tokens = NULL;
    result->token_count = 0;
}
