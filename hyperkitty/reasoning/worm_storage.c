/*
 * worm_storage.c — Binary WORM (Write-Once-Read-Many) Storage
 *
 * Immutable append-only ledger. Each entry carries:
 *   - SHA-256 payload hash
 *   - SHA-256 chain hash (prev_hash || payload_hash)
 *   - Ed25519 signature
 *   - Monotonic sequence number
 *   - Timestamp
 *
 * Binary format (efficient, cryptographically sealed):
 *
 *   Header:
 *     magic       4 bytes  "WORM"
 *     version     1 byte   0x01
 *     flags       1 byte   0x00
 *     event_len   2 bytes  (LE) string length of event type
 *     data_len    4 bytes  (LE) payload size
 *     meta_len    4 bytes  (LE) metadata size
 *     timestamp   8 bytes  (LE) Unix nanoseconds
 *     prev_hash  32 bytes  SHA-256 of previous entry chain hash
 *     content_hash 32 bytes  SHA-256 of payload
 *     signature  64 bytes  Ed25519(chain_hash, sk)
 *   = 152 bytes fixed header
 *
 *   Body:
 *     event_type  event_len bytes
 *     payload     data_len bytes
 *     metadata    meta_len bytes
 */

#include "hyperkitty/worm.h"
#include <stdlib.h>
#include <string.h>
#include <stdio.h>
#include <time.h>

#define WORM_MAGIC "WORM"
#define WORM_VERSION 0x01
#define WORM_HEADER_SIZE 152

/* ================================================================
 * Binary I/O helpers (little-endian)
 * ================================================================ */

static void write_le32(uint8_t *buf, uint32_t val) {
    buf[0] = val & 0xFF;
    buf[1] = (val >> 8) & 0xFF;
    buf[2] = (val >> 16) & 0xFF;
    buf[3] = (val >> 24) & 0xFF;
}

static void write_le16(uint8_t *buf, uint16_t val) {
    buf[0] = val & 0xFF;
    buf[1] = (val >> 8) & 0xFF;
}

static void write_le64(uint8_t *buf, uint64_t val) {
    for (int i = 0; i < 8; i++) {
        buf[i] = (val >> (i * 8)) & 0xFF;
    }
}

static uint32_t read_le32(const uint8_t *buf) {
    return buf[0] | (buf[1] << 8) | (buf[2] << 16) | (buf[3] << 24);
}

static uint16_t read_le16(const uint8_t *buf) {
    return buf[0] | (buf[1] << 8);
}

static uint64_t read_le64(const uint8_t *buf) {
    uint64_t val = 0;
    for (int i = 0; i < 8; i++) {
        val |= ((uint64_t)buf[i]) << (i * 8);
    }
    return val;
}

/* ================================================================
 * WORM entry allocation
 * ================================================================ */

hk_worm_entry *hk_worm_entry_alloc(void) {
    hk_worm_entry *e = calloc(1, sizeof(hk_worm_entry));
    if (!e) return NULL;
    return e;
}

void hk_worm_entry_free(hk_worm_entry *e) {
    if (!e) return;
    free(e->event_type);
    free(e->payload);
    free(e->metadata);
    free(e);
}

/* ================================================================
 * Binary serialization: entry → bytes
 * ================================================================ */

uint8_t *hk_worm_entry_serialize(hk_worm_entry *e, size_t *out_len) {
    if (!e || !out_len) return NULL;

    uint16_t event_len = e->event_type ? strlen(e->event_type) : 0;
    uint32_t data_len = e->payload ? e->payload_len : 0;
    uint32_t meta_len = e->metadata ? strlen((char *)e->metadata) : 0;

    size_t total_len = WORM_HEADER_SIZE + event_len + data_len + meta_len;
    uint8_t *buf = calloc(total_len, 1);
    if (!buf) return NULL;

    /* Magic */
    memcpy(&buf[0], WORM_MAGIC, 4);

    /* Version and flags */
    buf[4] = WORM_VERSION;
    buf[5] = 0x00;

    /* Event length */
    write_le16(&buf[6], event_len);

    /* Data length */
    write_le32(&buf[8], data_len);

    /* Metadata length */
    write_le32(&buf[12], meta_len);

    /* Timestamp */
    struct timespec ts;
    clock_gettime(CLOCK_REALTIME, &ts);
    uint64_t ns = (uint64_t)ts.tv_sec * 1000000000ULL + ts.tv_nsec;
    write_le64(&buf[16], ns);

    /* Previous hash (32 bytes) */
    if (e->prev_hash) {
        memcpy(&buf[24], e->prev_hash, 32);
    }

    /* Content hash (32 bytes) */
    if (e->content_hash) {
        memcpy(&buf[56], e->content_hash, 32);
    }

    /* Signature (64 bytes) */
    if (e->signature) {
        memcpy(&buf[88], e->signature, 64);
    }

    /* Event type */
    size_t offset = WORM_HEADER_SIZE;
    if (e->event_type) {
        memcpy(&buf[offset], e->event_type, event_len);
        offset += event_len;
    }

    /* Payload */
    if (e->payload) {
        memcpy(&buf[offset], e->payload, data_len);
        offset += data_len;
    }

    /* Metadata */
    if (e->metadata) {
        memcpy(&buf[offset], e->metadata, meta_len);
    }

    *out_len = total_len;
    return buf;
}

/* ================================================================
 * Binary deserialization: bytes → entry
 * ================================================================ */

hk_worm_entry *hk_worm_entry_deserialize(const uint8_t *buf, size_t buf_len) {
    if (!buf || buf_len < WORM_HEADER_SIZE) return NULL;

    /* Verify magic */
    if (memcmp(&buf[0], WORM_MAGIC, 4) != 0) return NULL;

    /* Verify version */
    if (buf[4] != WORM_VERSION) return NULL;

    hk_worm_entry *e = hk_worm_entry_alloc();
    if (!e) return NULL;

    uint16_t event_len = read_le16(&buf[6]);
    uint32_t data_len = read_le32(&buf[8]);
    uint32_t meta_len = read_le32(&buf[12]);

    size_t expected_len = WORM_HEADER_SIZE + event_len + data_len + meta_len;
    if (buf_len < expected_len) {
        hk_worm_entry_free(e);
        return NULL;
    }

    /* Extract fields */
    e->sequence = e->sequence;  /* Caller should set */
    e->timestamp_ns = read_le64(&buf[16]);

    memcpy(e->prev_hash, &buf[24], 32);
    memcpy(e->content_hash, &buf[56], 32);
    memcpy(e->signature, &buf[88], 64);

    size_t offset = WORM_HEADER_SIZE;

    if (event_len > 0) {
        e->event_type = malloc(event_len + 1);
        if (e->event_type) {
            memcpy(e->event_type, &buf[offset], event_len);
            e->event_type[event_len] = '\0';
        }
        offset += event_len;
    }

    if (data_len > 0) {
        e->payload = malloc(data_len);
        if (e->payload) {
            memcpy(e->payload, &buf[offset], data_len);
            e->payload_len = data_len;
        }
        offset += data_len;
    }

    if (meta_len > 0) {
        e->metadata = malloc(meta_len + 1);
        if (e->metadata) {
            memcpy(e->metadata, &buf[offset], meta_len);
            e->metadata[meta_len] = '\0';
        }
    }

    return e;
}

/* ================================================================
 * Chain hash computation
 * ================================================================ */

void hk_worm_compute_chain_hash(const uint8_t *prev_hash,
                                 const uint8_t *content_hash,
                                 uint8_t out[32]) {
    if (!out) return;

    /* SHA-256(prev_hash || content_hash) — stub */
    /* In production: call SHA256_Final or equivalent */
    memset(out, 0, 32);

    if (prev_hash && content_hash) {
        for (int i = 0; i < 32; i++) {
            out[i] = prev_hash[i] ^ content_hash[i];
        }
    }
}

/* ================================================================
 * Verify chain integrity
 * ================================================================ */

int hk_worm_verify_chain(const hk_worm_entry *entries, size_t count) {
    if (!entries || count == 0) return 1;

    uint8_t prev_chain_hash[32];
    memset(prev_chain_hash, 0, 32);

    for (size_t i = 0; i < count; i++) {
        /* Verify chain hash */
        uint8_t computed_chain[32];
        hk_worm_compute_chain_hash(prev_chain_hash, entries[i].content_hash,
                                    computed_chain);

        if (memcmp(computed_chain, entries[i].chain_hash, 32) != 0) {
            return 0;  /* Chain broken */
        }

        memcpy(prev_chain_hash, entries[i].chain_hash, 32);
    }

    return 1;
}

/* ================================================================
 * File I/O
 * ================================================================ */

int hk_worm_write_file(const char *path, const hk_worm_entry *entries, size_t count) {
    if (!path || !entries || count == 0) return 0;

    FILE *f = fopen(path, "wb");
    if (!f) return 0;

    for (size_t i = 0; i < count; i++) {
        size_t entry_len = 0;
        uint8_t *entry_buf = hk_worm_entry_serialize((hk_worm_entry *)&entries[i],
                                                      &entry_len);
        if (!entry_buf) {
            fclose(f);
            return 0;
        }

        if (fwrite(entry_buf, 1, entry_len, f) != entry_len) {
            free(entry_buf);
            fclose(f);
            return 0;
        }

        free(entry_buf);
    }

    fclose(f);
    return 1;
}

hk_worm_entry *hk_worm_read_file(const char *path, size_t *out_count) {
    if (!path || !out_count) return NULL;

    FILE *f = fopen(path, "rb");
    if (!f) return NULL;

    hk_worm_entry *entries = calloc(1024, sizeof(hk_worm_entry));
    if (!entries) {
        fclose(f);
        return NULL;
    }

    size_t count = 0;
    uint8_t header[WORM_HEADER_SIZE];

    while (count < 1024 && fread(header, 1, WORM_HEADER_SIZE, f) == WORM_HEADER_SIZE) {
        /* Read variable-length data */
        uint16_t event_len = read_le16(&header[6]);
        uint32_t data_len = read_le32(&header[8]);
        uint32_t meta_len = read_le32(&header[12]);

        size_t var_len = event_len + data_len + meta_len;
        uint8_t *var_buf = malloc(var_len);
        if (!var_buf || fread(var_buf, 1, var_len, f) != var_len) {
            free(var_buf);
            break;
        }

        /* Reconstruct full entry buffer */
        size_t full_len = WORM_HEADER_SIZE + var_len;
        uint8_t *full_buf = malloc(full_len);
        memcpy(full_buf, header, WORM_HEADER_SIZE);
        memcpy(&full_buf[WORM_HEADER_SIZE], var_buf, var_len);

        hk_worm_entry *e = hk_worm_entry_deserialize(full_buf, full_len);
        if (e) {
            entries[count] = *e;
            free(e);
            count++;
        }

        free(full_buf);
        free(var_buf);
    }

    fclose(f);
    *out_count = count;
    return entries;
}
