/*
 * hyperkitty_bus.c — Full sovereign bus implementation
 * Pure C99, POSIX sockets, pthread mutex + cond, topic matching,
 * bounded queues, per-connection reader threads.
 *
 * Wire format (JSON):
 *   {"type":"...","from":"...","to":"...","topic":"...","corr":NNN,"body":"..."}
 */
#include "hyperkitty/bus.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <errno.h>
#include <time.h>

#ifdef _WIN32
  /* Windows stubs — Unix domain sockets via Winsock2 */
  #include <winsock2.h>
  #include <afunix.h>
  typedef SOCKET hk_sock_t;
  #define HK_INVALID_SOCK INVALID_SOCKET
  static void hk_close_sock(SOCKET s) { closesocket(s); }
#else
  #include <unistd.h>
  #include <fcntl.h>
  #include <sys/socket.h>
  #include <sys/un.h>
  typedef int hk_sock_t;
  #define HK_INVALID_SOCK (-1)
  static void hk_close_sock(int s) { close(s); }
#endif

/* =========================================================
 *  Encode / Decode
 * ========================================================= */

int hk_msg_encode(const hk_message_t *m, char *out, size_t sz) {
    if (!m || !out || sz == 0) return -1;
    return snprintf(out, sz,
        "{\"type\":\"%s\",\"from\":\"%s\",\"to\":\"%s\","
        "\"topic\":\"%s\",\"corr\":%llu,\"body\":\"%s\"}",
        m->type, m->from, m->to, m->topic,
        (unsigned long long)m->corr, m->body);
}

/* Extract a JSON string value: {"key":"VALUE"} → copies VALUE into dst.
 * Returns pointer after closing quote, or NULL on failure. */
static const char *extract_str(const char *p, const char *end,
                                const char *key, char *dst, size_t dst_sz) {
    char needle[128];
    snprintf(needle, sizeof(needle), "\"%s\":\"", key);
    const char *found = strstr(p, needle);
    if (!found || found >= end) return NULL;
    const char *val = found + strlen(needle);
    const char *closing = strchr(val, '"');
    if (!closing || closing >= end) return NULL;
    size_t len = (size_t)(closing - val);
    if (len >= dst_sz) len = dst_sz - 1;
    memcpy(dst, val, len);
    dst[len] = '\0';
    return closing + 1;
}

/* Extract a JSON uint64 value: {"key":NNN} */
static const char *extract_u64(const char *p, const char *end,
                                const char *key, uint64_t *out) {
    char needle[128];
    snprintf(needle, sizeof(needle), "\"%s\":", key);
    const char *found = strstr(p, needle);
    if (!found || found >= end) return NULL;
    const char *val = found + strlen(needle);
    char *endptr = NULL;
    *out = (uint64_t)strtoull(val, &endptr, 10);
    return endptr;
}

int hk_msg_decode(const char *data, size_t len, hk_message_t *out) {
    if (!data || !out || len == 0) return -1;
    const char *end = data + len;
    memset(out, 0, sizeof(*out));

    extract_str(data, end, "type",  out->type,  sizeof(out->type));
    extract_str(data, end, "from",  out->from,  sizeof(out->from));
    extract_str(data, end, "to",    out->to,    sizeof(out->to));
    extract_str(data, end, "topic", out->topic, sizeof(out->topic));
    extract_u64(data, end, "corr",  &out->corr);

    /* body may contain escaped content — extract between "body":" and final " */
    const char *body_needle = "\"body\":\"";
    const char *bp = strstr(data, body_needle);
    if (bp && bp < end) {
        bp += strlen(body_needle);
        const char *be = bp;
        /* scan for closing quote, skip \\" escapes */
        while (be < end && *be != '\0') {
            if (*be == '\\' && *(be+1) == '"') { be += 2; continue; }
            if (*be == '"') break;
            be++;
        }
        size_t blen = (size_t)(be - bp);
        if (blen >= sizeof(out->body)) blen = sizeof(out->body) - 1;
        memcpy(out->body, bp, blen);
        out->body[blen] = '\0';
    }
    return 0;
}

/* =========================================================
 *  Topic matching — exact or wildcard with '*'
 * ========================================================= */
static bool topic_matches(const char *pattern, const char *topic) {
    if (strcmp(pattern, "*") == 0) return true;
    /* simple prefix wildcard: "agents.*" matches "agents.quark" */
    size_t plen = strlen(pattern);
    if (plen > 0 && pattern[plen-1] == '*') {
        return strncmp(pattern, topic, plen-1) == 0;
    }
    return strcmp(pattern, topic) == 0;
}

/* =========================================================
 *  Connection queue helpers (caller holds conn->q_lock)
 * ========================================================= */
static bool q_push(hk_conn_t *c, const hk_message_t *m) {
    if (c->q_count >= HK_QUEUE_MAX) return false;
    c->queue[c->q_tail].msg  = *m;
    c->queue[c->q_tail].used = true;
    c->q_tail = (c->q_tail + 1) % HK_QUEUE_MAX;
    c->q_count++;
    return true;
}

static bool q_pop(hk_conn_t *c, hk_message_t *out) {
    if (c->q_count == 0) return false;
    *out = c->queue[c->q_head].msg;
    c->queue[c->q_head].used = false;
    c->q_head = (c->q_head + 1) % HK_QUEUE_MAX;
    c->q_count--;
    return true;
}

/* =========================================================
 *  Bus: init / destroy
 * ========================================================= */
int hk_bus_init(hk_bus_t *bus, const char *sock_path) {
    if (!bus) return -1;
    memset(bus, 0, sizeof(*bus));
    pthread_mutex_init(&bus->lock, NULL);
    bus->server_fd = (int)HK_INVALID_SOCK;
    bus->running   = false;

#ifdef _WIN32
    WSADATA wsd;
    WSAStartup(MAKEWORD(2,2), &wsd);
#endif

    if (!sock_path) { bus->running = true; return 0; }  /* in-process mode */

    /* Create Unix domain socket server */
#ifdef _WIN32
    SOCKET sfd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (sfd == INVALID_SOCKET) return -1;
#else
    int sfd = socket(AF_UNIX, SOCK_STREAM, 0);
    if (sfd < 0) return -1;
    /* allow reuse */
    int opt = 1; setsockopt(sfd, SOL_SOCKET, SO_REUSEADDR, &opt, sizeof(opt));
#endif

    struct sockaddr_un addr;
    memset(&addr, 0, sizeof(addr));
    addr.sun_family = AF_UNIX;
    strncpy(addr.sun_path, sock_path, sizeof(addr.sun_path)-1);
#ifndef _WIN32
    unlink(sock_path);
#endif

    if (bind(sfd, (struct sockaddr *)&addr, sizeof(addr)) != 0) {
        hk_close_sock(sfd);
        return -1;
    }
    if (listen(sfd, 32) != 0) {
        hk_close_sock(sfd);
        return -1;
    }

    bus->server_fd = (int)sfd;
    bus->running   = true;
    return 0;
}

void hk_bus_destroy(hk_bus_t *bus) {
    if (!bus) return;
    bus->running = false;

    pthread_mutex_lock(&bus->lock);
    for (uint32_t i = 0; i < bus->conn_count; i++) {
        hk_conn_t *c = bus->conns[i];
        if (!c) continue;
        c->active = false;
        pthread_cond_broadcast(&c->q_cond);
        if (c->fd != -1) { hk_close_sock(c->fd); c->fd = -1; }
    }
    pthread_mutex_unlock(&bus->lock);

    /* brief wait for reader threads */
    for (uint32_t i = 0; i < bus->conn_count; i++) {
        hk_conn_t *c = bus->conns[i];
        if (c && c->thread) pthread_join(c->thread, NULL);
        if (c) {
            pthread_mutex_destroy(&c->q_lock);
            pthread_cond_destroy(&c->q_cond);
            free(c);
        }
    }
    bus->conn_count = 0;

    if (bus->server_fd != -1 && bus->server_fd != (int)HK_INVALID_SOCK) {
        hk_close_sock(bus->server_fd);
        bus->server_fd = -1;
    }
    if (bus->accept_thread) pthread_join(bus->accept_thread, NULL);
    pthread_mutex_destroy(&bus->lock);
}

/* =========================================================
 *  Bus: connect (in-process) / disconnect
 * ========================================================= */
int hk_bus_connect(hk_bus_t *bus, const char *id, hk_conn_t **out) {
    if (!bus || !id || !out) return -1;

    hk_conn_t *c = calloc(1, sizeof(hk_conn_t));
    if (!c) return -1;

    strncpy(c->id, id, HK_MAX_ID_LEN - 1);
    c->fd     = -1;
    c->active = true;
    c->bus    = bus;
    pthread_mutex_init(&c->q_lock, NULL);
    pthread_cond_init(&c->q_cond, NULL);

    pthread_mutex_lock(&bus->lock);
    if (bus->conn_count >= HK_MAX_CONNS) {
        pthread_mutex_unlock(&bus->lock);
        free(c);
        return -1;
    }
    bus->conns[bus->conn_count++] = c;
    pthread_mutex_unlock(&bus->lock);

    *out = c;
    return 0;
}

void hk_bus_disconnect(hk_bus_t *bus, hk_conn_t *conn) {
    if (!bus || !conn) return;
    pthread_mutex_lock(&bus->lock);
    for (uint32_t i = 0; i < bus->conn_count; i++) {
        if (bus->conns[i] == conn) {
            bus->conns[i] = bus->conns[--bus->conn_count];
            break;
        }
    }
    pthread_mutex_unlock(&bus->lock);
    conn->active = false;
    pthread_cond_broadcast(&conn->q_cond);
}

/* =========================================================
 *  Bus: subscribe / unsubscribe
 * ========================================================= */
int hk_bus_subscribe(hk_conn_t *conn, const char *topic) {
    if (!conn || !topic) return -1;
    pthread_mutex_lock(&conn->q_lock);
    if (conn->sub_count >= HK_MAX_SUBS_PER_CONN) {
        pthread_mutex_unlock(&conn->q_lock);
        return -1;
    }
    /* check duplicate */
    for (uint32_t i = 0; i < conn->sub_count; i++) {
        if (conn->subs[i].active &&
            strcmp(conn->subs[i].topic, topic) == 0) {
            pthread_mutex_unlock(&conn->q_lock);
            return 0;
        }
    }
    strncpy(conn->subs[conn->sub_count].topic, topic, HK_MAX_TOPIC_LEN - 1);
    conn->subs[conn->sub_count].active = true;
    conn->sub_count++;
    pthread_mutex_unlock(&conn->q_lock);
    return 0;
}

int hk_bus_unsubscribe(hk_conn_t *conn, const char *topic) {
    if (!conn || !topic) return -1;
    pthread_mutex_lock(&conn->q_lock);
    for (uint32_t i = 0; i < conn->sub_count; i++) {
        if (conn->subs[i].active &&
            strcmp(conn->subs[i].topic, topic) == 0) {
            conn->subs[i].active = false;
            /* compact */
            conn->subs[i] = conn->subs[--conn->sub_count];
            break;
        }
    }
    pthread_mutex_unlock(&conn->q_lock);
    return 0;
}

/* =========================================================
 *  Bus: publish — fan-out to all matching subscribers
 * ========================================================= */
int hk_bus_publish(hk_bus_t *bus, const hk_message_t *msg) {
    if (!bus || !msg) return -1;

    int delivered = 0;
    pthread_mutex_lock(&bus->lock);
    bus->total_published++;

    for (uint32_t i = 0; i < bus->conn_count; i++) {
        hk_conn_t *c = bus->conns[i];
        if (!c || !c->active) continue;

        bool matches = false;

        /* direct address match */
        if (msg->to[0] != '\0' && strcmp(msg->to, c->id) == 0) {
            matches = true;
        } else if (msg->to[0] == '\0' || strcmp(msg->to, "*") == 0) {
            /* broadcast — check topic subscriptions */
            pthread_mutex_lock(&c->q_lock);
            for (uint32_t s = 0; s < c->sub_count && !matches; s++) {
                if (c->subs[s].active &&
                    topic_matches(c->subs[s].topic, msg->topic)) {
                    matches = true;
                }
            }
            pthread_mutex_unlock(&c->q_lock);
        }

        if (matches) {
            pthread_mutex_lock(&c->q_lock);
            if (q_push(c, msg)) {
                delivered++;
                bus->total_routed++;
                pthread_cond_signal(&c->q_cond);
            } else {
                c->drops++;
                bus->total_dropped++;
            }
            pthread_mutex_unlock(&c->q_lock);
        }
    }
    pthread_mutex_unlock(&bus->lock);
    return delivered > 0 ? 0 : -1;
}

/* =========================================================
 *  Bus: route — deliver to single best-match conn
 * ========================================================= */
int hk_bus_route(hk_bus_t *bus, const hk_message_t *msg) {
    if (!bus || !msg) return -1;

    pthread_mutex_lock(&bus->lock);
    hk_conn_t *best = NULL;

    /* prefer direct address */
    for (uint32_t i = 0; i < bus->conn_count; i++) {
        hk_conn_t *c = bus->conns[i];
        if (!c || !c->active) continue;
        if (msg->to[0] != '\0' && strcmp(msg->to, c->id) == 0) {
            best = c;
            break;
        }
    }

    /* fallback: pick conn subscribed to topic with smallest queue */
    if (!best) {
        uint32_t min_q = UINT32_MAX;
        for (uint32_t i = 0; i < bus->conn_count; i++) {
            hk_conn_t *c = bus->conns[i];
            if (!c || !c->active) continue;
            pthread_mutex_lock(&c->q_lock);
            for (uint32_t s = 0; s < c->sub_count; s++) {
                if (c->subs[s].active &&
                    topic_matches(c->subs[s].topic, msg->topic)) {
                    if (c->q_count < min_q) {
                        min_q = c->q_count;
                        best  = c;
                    }
                    break;
                }
            }
            pthread_mutex_unlock(&c->q_lock);
        }
    }

    int rc = -1;
    if (best) {
        pthread_mutex_lock(&best->q_lock);
        if (q_push(best, msg)) {
            bus->total_routed++;
            pthread_cond_signal(&best->q_cond);
            rc = 0;
        } else {
            best->drops++;
            bus->total_dropped++;
        }
        pthread_mutex_unlock(&best->q_lock);
    }
    pthread_mutex_unlock(&bus->lock);
    return rc;
}

/* =========================================================
 *  Bus: recv — blocking with optional timeout
 * ========================================================= */
int hk_bus_recv(hk_conn_t *conn, hk_message_t *out, uint32_t timeout_ms) {
    if (!conn || !out) return -1;

    pthread_mutex_lock(&conn->q_lock);

    if (timeout_ms == 0) {
        /* non-blocking */
        bool got = q_pop(conn, out);
        pthread_mutex_unlock(&conn->q_lock);
        return got ? 0 : 1;
    }

    /* timed wait */
    struct timespec deadline;
    clock_gettime(CLOCK_REALTIME, &deadline);
    deadline.tv_sec  += (time_t)(timeout_ms / 1000);
    deadline.tv_nsec += (long)((timeout_ms % 1000) * 1000000L);
    if (deadline.tv_nsec >= 1000000000L) {
        deadline.tv_sec++;
        deadline.tv_nsec -= 1000000000L;
    }

    while (conn->q_count == 0 && conn->active) {
        int r = pthread_cond_timedwait(&conn->q_cond, &conn->q_lock, &deadline);
        if (r == ETIMEDOUT) break;
    }

    bool got = q_pop(conn, out);
    pthread_mutex_unlock(&conn->q_lock);
    return got ? 0 : 1;
}

/* =========================================================
 *  Stats
 * ========================================================= */
void hk_bus_get_stats(const hk_bus_t *bus, hk_bus_stats_t *out) {
    if (!bus || !out) return;
    /* const cast — stats read is atomic enough for monitoring */
    hk_bus_t *b = (hk_bus_t *)(uintptr_t)bus;
    pthread_mutex_lock(&b->lock);
    out->total_published    = b->total_published;
    out->total_routed       = b->total_routed;
    out->total_dropped      = b->total_dropped;
    out->active_connections = b->conn_count;
    pthread_mutex_unlock(&b->lock);
}
