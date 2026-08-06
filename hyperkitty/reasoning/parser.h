/*
 * parser.h — Regex parser interface
 * Stage 1 of routing pipeline: tokenization + danger detection
 */

#pragma once
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

typedef enum {
    HK_PARSE_OK,        /* Input accepted, tokens extracted */
    HK_PARSE_REJECTED,  /* Dangerous pattern detected */
    HK_PARSE_ERROR      /* Parsing failed */
} hk_parse_status;

typedef struct {
    void *tokens;           /* Token array (opaque) */
    size_t token_count;
    hk_parse_status status;
    const char *error;      /* Reason code if rejected */
} hk_parse_result;

/**
 * hk_parse_input — Tokenize and validate input
 *
 * Dangerous patterns are detected using regex:
 *   - XXE triggers: <!DOCTYPE, <!ENTITY, SYSTEM, file://
 *   - Code execution: eval, exec, fork, __import__
 *   - Infinite loops: while true
 *   - Credentials: password=, api_key, secret, aws_secret
 *   - Telemetry: beacon, tracker, analytics
 *
 * If any pattern matches, returns HK_PARSE_REJECTED with reason.
 * Otherwise tokenizes and returns HK_PARSE_OK.
 *
 * @param input   Input text
 * @return Parse result with status and tokens/error
 */
hk_parse_result hk_parse_input(const char *input);

/**
 * hk_parse_result_free — Free parsed tokens
 */
void hk_parse_result_free(hk_parse_result *result);

#ifdef __cplusplus
}
#endif
