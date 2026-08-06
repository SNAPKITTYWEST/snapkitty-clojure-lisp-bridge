/*
 * worm.h — Write-Once-Read-Many (WORM) Storage
 * Immutable append-only cryptographic ledger
 *
 * Binary format (152-byte fixed header + variable body):
 *
 *   magic       4 bytes   "WORM"
 *   version     1 byte    0x01
 *   flags       1 byte    0x00
 *   event_len   2 bytes   (LE) string length
 *   data_len    4 bytes   (LE) payload size
 *   meta_len    4 bytes   (LE) metadata size
 *   timestamp   8 bytes   (LE) Unix nanoseconds
 *   prev_hash  32 bytes   SHA-256 of previous entry chain hash
 *   content_hash 32 bytes   SHA-256 of payload
 *   signature  64 bytes   Ed25519(chain_hash)
 *
 * Each entry is cryptographically linked to its predecessor.
 * Replay by recomputing all chain hashes verifies integrity.
 */

#pragma once
#include <stdint.h>
#include <stddef.h>

#ifdef __cplusplus
extern "C" {
#endif

#define HK_WORM_HASH_SIZE 32
#define HK_WORM_SIG_SIZE 64

typedef struct {
    uint64_t sequence;                      /* Entry index in ledger */
    uint64_t timestamp_ns;                  /* Unix nanoseconds */
    char *event_type;                       /* Event classification string */
    uint8_t *payload;                       /* Entry data */
    size_t payload_len;
    uint8_t *metadata;                      /* Optional metadata */
    uint8_t prev_hash[HK_WORM_HASH_SIZE];   /* Hash of previous chain_hash */
    uint8_t content_hash[HK_WORM_HASH_SIZE]; /* SHA-256(payload) */
    uint8_t chain_hash[HK_WORM_HASH_SIZE];  /* SHA-256(prev_hash || content_hash) */
    uint8_t signature[HK_WORM_SIG_SIZE];    /* Ed25519(chain_hash) */
} hk_worm_entry;

/**
 * hk_worm_entry_alloc — Allocate WORM entry
 */
hk_worm_entry *hk_worm_entry_alloc(void);

/**
 * hk_worm_entry_free — Free WORM entry and all data
 */
void hk_worm_entry_free(hk_worm_entry *e);

/**
 * hk_worm_entry_serialize — Convert entry to binary format
 *
 * Produces 152-byte header + variable-length body.
 *
 * @param e        Entry to serialize
 * @param out_len  Receives total serialized length
 * @return Allocated buffer, or NULL on error
 */
uint8_t *hk_worm_entry_serialize(hk_worm_entry *e, size_t *out_len);

/**
 * hk_worm_entry_deserialize — Parse binary entry
 *
 * @param buf      Binary buffer (at least 152 bytes)
 * @param buf_len  Total buffer size
 * @return Parsed entry, or NULL if invalid
 */
hk_worm_entry *hk_worm_entry_deserialize(const uint8_t *buf, size_t buf_len);

/**
 * hk_worm_compute_chain_hash — Compute chain hash
 *
 * chain_hash = SHA-256(prev_hash || content_hash)
 *
 * @param prev_hash     Previous entry chain hash (32 bytes)
 * @param content_hash  Payload hash (32 bytes)
 * @param out           Output: 32-byte chain hash
 */
void hk_worm_compute_chain_hash(const uint8_t *prev_hash,
                                 const uint8_t *content_hash,
                                 uint8_t out[32]);

/**
 * hk_worm_verify_chain — Verify ledger integrity
 *
 * Replays all entries and verifies chain hashes match.
 * Returns 0 if any link is broken, 1 if valid.
 *
 * @param entries  Entry array
 * @param count    Number of entries
 * @return 1 if chain valid, 0 if broken
 */
int hk_worm_verify_chain(const hk_worm_entry *entries, size_t count);

/**
 * hk_worm_write_file — Serialize ledger to file
 *
 * @param path     File path
 * @param entries  Entry array
 * @param count    Number of entries
 * @return 1 on success, 0 on failure
 */
int hk_worm_write_file(const char *path, const hk_worm_entry *entries, size_t count);

/**
 * hk_worm_read_file — Deserialize ledger from file
 *
 * @param path      File path
 * @param out_count Receives number of entries read
 * @return Allocated entry array, or NULL on error
 */
hk_worm_entry *hk_worm_read_file(const char *path, size_t *out_count);

#ifdef __cplusplus
}
#endif
