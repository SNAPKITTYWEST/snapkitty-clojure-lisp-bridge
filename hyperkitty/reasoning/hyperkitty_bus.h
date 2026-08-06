/* hyperkitty_bus.h — Hyperkitty C-- Bus - Bare-metal twin of hc_bus.cpp */
#pragma once
#include <stdint.h>
#include <stddef.h>
#include <stdbool.h>
#include <pthread.h>

/* ---- constants ---- */
#define HK_QUEUE_MAX          256
#define HK_MAX_SUBS_PER_CONN  64
#define HK_MAX_TOPIC_LEN      128
#define HK_MAX_CONNS          256
#define HK_MAX_ID_LEN         64
#define HK_MAX_BODY           8192
#define HK_SOCK_PATH          "/tmp/hk.sock"

/* ---- wire message ---- */
typedef struct {
    char     type[32];
    char     from[64];
    char     to[64];
    char     topic[128];
    uint64_t corr;
    char     body[HK_MAX_BODY];
} hk_message_t;

/* ---- subscription slot ---- */
typedef struct {
    char  topic[HK_MAX_TOPIC_LEN];
    bool  active;
} hk_subscription_t;

/* ---- per-connection queue entry ---- */
typedef struct {
    hk_message_t msg;
    bool          used;
} hk_queue_entry_t;

/* ---- per-connection state ---- */
typedef struct hk_conn hk_conn_t;
struct hk_conn {
    char               id[HK_MAX_ID_LEN];
    int                fd;                            /* socket fd, -1 = closed */
    hk_subscription_t  subs[HK_MAX_SUBS_PER_CONN];
    uint32_t           sub_count;
    pthread_mutex_t    q_lock;
    pthread_cond_t     q_cond;
    hk_queue_entry_t   queue[HK_QUEUE_MAX];
    uint32_t           q_head;
    uint32_t           q_tail;
    uint32_t           q_count;
    pthread_t          thread;
    bool               active;
    struct hk_bus     *bus;
    /* stats */
    uint64_t           msgs_sent;
    uint64_t           msgs_recv;
    uint64_t           drops;
};

/* ---- bus instance ---- */
typedef struct hk_bus {
    pthread_mutex_t lock;
    hk_conn_t      *conns[HK_MAX_CONNS];
    uint32_t        conn_count;
    int             server_fd;
    pthread_t       accept_thread;
    bool            running;
    /* stats */
    uint64_t        total_published;
    uint64_t        total_routed;
    uint64_t        total_dropped;
} hk_bus_t;

/* ---- API ---- */
int  hk_msg_encode    (const hk_message_t *m, char *out, size_t out_sz);
int  hk_msg_decode    (const char *data, size_t len, hk_message_t *out);

int  hk_bus_init      (hk_bus_t *bus, const char *sock_path);
void hk_bus_destroy   (hk_bus_t *bus);

int  hk_bus_connect   (hk_bus_t *bus, const char *id, hk_conn_t **out);
void hk_bus_disconnect(hk_bus_t *bus, hk_conn_t *conn);

int  hk_bus_publish   (hk_bus_t *bus, const hk_message_t *msg);
int  hk_bus_subscribe (hk_conn_t *conn, const char *topic);
int  hk_bus_unsubscribe(hk_conn_t *conn, const char *topic);

/* Route: deliver msg to the best matching connected conn.
 * Returns 0 on delivery, -1 if no subscriber found. */
int  hk_bus_route     (hk_bus_t *bus, const hk_message_t *msg);

/* Poll next message from a connection's queue (timeout_ms=0 → non-blocking).
 * Returns 0 on success, 1 if queue empty, -1 on error. */
int  hk_bus_recv      (hk_conn_t *conn, hk_message_t *out, uint32_t timeout_ms);

/* Stats snapshot */
typedef struct {
    uint64_t total_published;
    uint64_t total_routed;
    uint64_t total_dropped;
    uint32_t active_connections;
} hk_bus_stats_t;

void hk_bus_get_stats(const hk_bus_t *bus, hk_bus_stats_t *out);
