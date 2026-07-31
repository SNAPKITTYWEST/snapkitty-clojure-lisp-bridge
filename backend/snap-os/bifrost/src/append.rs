//! Concurrent append layer — lockless ingest queue with batch-commit writer.
//!
//! # Architecture
//! ```text
//! caller  ──push──▶  SegQueue<SealRequest>
//!                          │
//!                    writer thread (single)
//!                          │
//!               ┌──────────┴──────────┐
//!             DAG                  WORM_FS
//!               └──────────┬──────────┘
//!                    RwLock<Option<Cid>>  ← head
//! ```
//!
//! # Batch semantics
//! The writer thread drains the queue and commits after:
//! - 1 024 events, OR
//! - 100 ms of wall clock time
//! (whichever comes first).
//!
//! # Back-pressure
//! The queue is unbounded (`SegQueue`).  Callers that need back-pressure should
//! check `queue_depth()` and throttle when it exceeds a threshold.

use std::path::Path;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::{Duration, Instant};

use crossbeam::queue::SegQueue;

use crate::error::BifrostResult;
use crate::event::{Cid, EventPayload};
use crate::seal::Bifrost;

const FLUSH_BATCH:   usize    = 1_024;
const FLUSH_TIMEOUT: Duration = Duration::from_millis(100);

// ── SealRequest ───────────────────────────────────────────────────────────────

/// A pending event submitted to the append queue.
pub struct SealRequest {
    pub payload:  EventPayload,
    /// Oneshot sender that receives the resulting CID (or error string).
    pub reply_tx: Option<std::sync::mpsc::SyncSender<Result<Cid, String>>>,
}

// ── AppendQueue ───────────────────────────────────────────────────────────────

/// Shared, `Clone`able handle to the append queue.
///
/// Submit events with `push()` or `push_sync()` (blocks until committed).
/// Read the current head with `head()`.
#[derive(Clone)]
pub struct AppendQueue {
    queue: Arc<SegQueue<SealRequest>>,
    head:  Arc<RwLock<Option<Cid>>>,
}

impl AppendQueue {
    /// Fire-and-forget: push an event payload without waiting for commit.
    pub fn push(&self, payload: EventPayload) {
        self.queue.push(SealRequest { payload, reply_tx: None });
    }

    /// Push and block until the event is committed.  Returns the resulting CID.
    pub fn push_sync(&self, payload: EventPayload) -> Result<Cid, String> {
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        self.queue.push(SealRequest { payload, reply_tx: Some(tx) });
        rx.recv().map_err(|e| e.to_string())?
    }

    /// Read the current chain head CID.
    pub fn head(&self) -> Option<Cid> {
        *self.head.read().unwrap()
    }

    /// Number of pending (not-yet-committed) events in the queue.
    pub fn queue_depth(&self) -> usize {
        self.queue.len()
    }
}

// ── BifrostWriter ─────────────────────────────────────────────────────────────

/// The writer side — owns `Bifrost` and runs the drain loop in a background thread.
pub struct BifrostWriter {
    #[allow(dead_code)]
    queue:  Arc<SegQueue<SealRequest>>,
    head:   Arc<RwLock<Option<Cid>>>,
    handle: Option<thread::JoinHandle<()>>,
    stop:   Arc<std::sync::atomic::AtomicBool>,
}

impl BifrostWriter {
    /// Initialise a writer, spinning up the background drain thread.
    ///
    /// `bifrost` must already have a genesis event sealed (or be empty — the
    /// writer will seal genesis automatically if the chain is empty).
    pub fn start(mut bifrost: Bifrost) -> BifrostResult<(Self, AppendQueue)> {
        // Seed genesis if chain is empty.
        if bifrost.dag.head()?.is_none() {
            bifrost.seal_genesis()?;
        }

        let initial_head = bifrost.dag.head()?;
        let queue = Arc::new(SegQueue::<SealRequest>::new());
        let head  = Arc::new(RwLock::new(initial_head));
        let stop  = Arc::new(std::sync::atomic::AtomicBool::new(false));

        let q2    = Arc::clone(&queue);
        let h2    = Arc::clone(&head);
        let stop2 = Arc::clone(&stop);

        let handle = thread::Builder::new()
            .name("bifrost-writer".into())
            .spawn(move || drain_loop(bifrost, q2, h2, stop2))
            .expect("spawn bifrost-writer");

        let aq = AppendQueue { queue: Arc::clone(&queue), head: Arc::clone(&head) };
        Ok((BifrostWriter { queue, head, handle: Some(handle), stop }, aq))
    }

    /// Signal the writer thread to stop and wait for it to finish.
    pub fn shutdown(mut self) {
        self.stop.store(true, std::sync::atomic::Ordering::Release);
        if let Some(h) = self.handle.take() {
            let _ = h.join();
        }
    }

    /// Current chain head from the writer's perspective.
    pub fn head(&self) -> Option<Cid> {
        *self.head.read().unwrap()
    }
}

// ── Drain loop (runs in writer thread) ───────────────────────────────────────

fn drain_loop(
    mut bifrost: Bifrost,
    queue:       Arc<SegQueue<SealRequest>>,
    head:        Arc<RwLock<Option<Cid>>>,
    stop:        Arc<std::sync::atomic::AtomicBool>,
) {
    let mut batch:       Vec<SealRequest> = Vec::with_capacity(FLUSH_BATCH);
    let mut last_flush = Instant::now();

    loop {
        // Collect up to FLUSH_BATCH items without blocking.
        while batch.len() < FLUSH_BATCH {
            if let Some(req) = queue.pop() {
                batch.push(req);
            } else {
                break;
            }
        }

        let should_flush = !batch.is_empty()
            && (batch.len() >= FLUSH_BATCH || last_flush.elapsed() >= FLUSH_TIMEOUT);

        if should_flush {
            for req in batch.drain(..) {
                let result = bifrost.seal_payload(req.payload)
                    .map_err(|e| e.to_string());
                if let Ok(cid) = &result {
                    *head.write().unwrap() = Some(*cid);
                }
                if let Some(tx) = req.reply_tx {
                    let _ = tx.try_send(result);
                }
            }
            last_flush = Instant::now();
        }

        if stop.load(std::sync::atomic::Ordering::Acquire) && queue.is_empty() {
            // Flush remaining on shutdown.
            for req in batch.drain(..) {
                let result = bifrost.seal_payload(req.payload)
                    .map_err(|e| e.to_string());
                if let Ok(cid) = &result {
                    *head.write().unwrap() = Some(*cid);
                }
                if let Some(tx) = req.reply_tx {
                    let _ = tx.try_send(result);
                }
            }
            break;
        }

        // Brief yield to avoid busy-spin.
        thread::sleep(Duration::from_millis(1));
    }
}

// ── Convenience: open a writer from a path ─────────────────────────────────────

/// Open (or create) a `BifrostWriter` at `root`.
pub fn open_writer(root: impl AsRef<Path>) -> BifrostResult<(BifrostWriter, AppendQueue)> {
    let bifrost = Bifrost::open(root)?;
    BifrostWriter::start(bifrost)
}

// ── Unit tests ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::event::EventPayload;
    use tempfile::tempdir;

    fn zero_cid() -> Cid { Cid([0u8; 32]) }

    fn cap_transfer_payload() -> EventPayload {
        EventPayload::CapTransfer {
            from: zero_cid(), to: zero_cid(),
            cap_hash: zero_cid(), policy_cid: zero_cid(),
        }
    }

    #[test]
    fn writer_seals_genesis_on_start() {
        let dir = tempdir().unwrap();
        let bifrost = Bifrost::open(dir.path()).unwrap();
        let (writer, aq) = BifrostWriter::start(bifrost).unwrap();
        assert!(aq.head().is_some());
        writer.shutdown();
    }

    #[test]
    fn push_sync_commits_event() {
        let dir = tempdir().unwrap();
        let (writer, aq) = open_writer(dir.path()).unwrap();

        let head_before = aq.head().unwrap();
        let cid = aq.push_sync(cap_transfer_payload()).expect("commit");
        assert_ne!(cid, head_before, "head must advance");
        assert_eq!(aq.head().unwrap(), cid);

        writer.shutdown();
    }

    #[test]
    fn push_fire_and_forget_eventually_commits() {
        let dir = tempdir().unwrap();
        let (writer, aq) = open_writer(dir.path()).unwrap();

        let head_before = aq.head().unwrap();
        aq.push(cap_transfer_payload());

        // Wait for commit (up to 500 ms)
        let deadline = Instant::now() + Duration::from_millis(500);
        while Instant::now() < deadline {
            if aq.head().unwrap() != head_before { break; }
            thread::sleep(Duration::from_millis(5));
        }
        assert_ne!(aq.head().unwrap(), head_before, "head must advance after push");

        writer.shutdown();
    }

    #[test]
    fn shutdown_flushes_remaining() {
        let dir = tempdir().unwrap();
        let (writer, aq) = open_writer(dir.path()).unwrap();

        // Push without syncing
        for _ in 0..10 {
            aq.push(cap_transfer_payload());
        }
        let last = aq.push_sync(cap_transfer_payload()).unwrap();
        assert_ne!(last, Cid([0u8; 32]));

        writer.shutdown();
    }
}
