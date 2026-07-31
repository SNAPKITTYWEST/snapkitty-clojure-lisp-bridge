//! Genesis fixture integration test.
//!
//! Exercises the full bifrost sealing pipeline:
//!   genesis → CapTransfer → JitCompile → verify_chain
//!
//! Run with: `cargo test -p bifrost --test genesis_fixture`
//! Or:       `cargo test -p bifrost --features genesis`

use bifrost::event::{Cid, EventPayload};
use bifrost::seal::Bifrost;
use bifrost::verify::verify_chain;

fn make_chain() -> (Bifrost, tempfile::TempDir) {
    let dir = tempfile::tempdir().unwrap();
    let mut b = Bifrost::open(dir.path()).unwrap();

    // ── 1. Genesis ─────────────────────────────────────────────────────────
    let genesis_cid = b.seal_genesis().unwrap();
    assert_eq!(b.dag.height().unwrap(), 0);
    assert_eq!(b.dag.head().unwrap().unwrap(), genesis_cid);

    // ── 2. Seal policy blob into WORM_FS ───────────────────────────────────
    let policy_src = include_bytes!("../policy/cap_transfer.pl");
    let policy_cid = b.worm.seal_blob(policy_src, None).unwrap();
    assert!(b.worm.is_sealed(&policy_cid));

    // ── 3. CapTransfer event ────────────────────────────────────────────────
    let soul_a = Cid::of(b"silverback_soul");
    let soul_b = Cid::of(b"koko_soul");
    let cap    = Cid::of(b"endpoint_cap_0");

    let ct_cid = b.seal_cap_transfer(soul_a, soul_b, cap, policy_cid).unwrap();
    assert_eq!(b.dag.height().unwrap(), 1);

    // Event must be retrievable from DAG.
    let ev = b.dag.get(&ct_cid).unwrap().unwrap();
    assert_eq!(ev.height, 1);
    assert_eq!(ev.prev, Some(genesis_cid));
    assert!(matches!(ev.payload, EventPayload::CapTransfer { .. }));

    // ── 4. Seal SoulIR blob ─────────────────────────────────────────────────
    let soulir_bytes = b"\x00\x01\x02 SoulVM bytecode";
    let soulir_cid   = b.worm.seal_blob(soulir_bytes, None).unwrap();

    // ── 5. JitCompile event ─────────────────────────────────────────────────
    let wasm_cid = Cid::of(b"cranelift_native_output");
    let jit_cid  = b.seal_jit_compile(soulir_cid, wasm_cid, 1, 55_000).unwrap();
    assert_eq!(b.dag.height().unwrap(), 2);

    let jit_ev = b.dag.get(&jit_cid).unwrap().unwrap();
    assert_eq!(jit_ev.height, 2);
    assert_eq!(jit_ev.prev, Some(ct_cid));

    if let EventPayload::JitCompile { opt_level, gas_estimate, .. } = jit_ev.payload {
        assert_eq!(opt_level, 1);
        assert_eq!(gas_estimate, 55_000);
    } else {
        panic!("expected JitCompile payload");
    }

    (b, dir)
}

#[test]
fn genesis_chain_verifies_clean() {
    let (b, _dir) = make_chain();
    let head   = b.dag.head().unwrap().unwrap();
    let report = verify_chain(&head, &b.dag, &b.worm).unwrap();

    assert!(
        report.ok,
        "chain verification failed:\n{}",
        report.errors.join("\n")
    );
    assert_eq!(report.event_count, 3); // genesis + CapTransfer + JitCompile
    assert_eq!(report.height, 2);
    assert_eq!(report.chain[0], head); // newest first
}

#[test]
fn all_events_have_valid_cids() {
    let (b, _dir) = make_chain();
    let head = b.dag.head().unwrap().unwrap();
    let chain = b.dag.walk_from(&head).unwrap();

    for cid in &chain {
        let ev = b.dag.get(cid).unwrap().unwrap();
        ev.verify_cid().unwrap_or_else(|e| {
            panic!("CID mismatch at height {}: {e}", ev.height);
        });
    }
}

#[test]
fn all_events_have_valid_sigs() {
    let (b, _dir) = make_chain();
    let head = b.dag.head().unwrap().unwrap();
    let chain = b.dag.walk_from(&head).unwrap();

    for cid in &chain {
        let ev = b.dag.get(cid).unwrap().unwrap();
        ev.verify_sig().unwrap_or_else(|e| {
            panic!("sig invalid at height {}: {e}", ev.height);
        });
    }
}

#[test]
fn worm_contains_all_event_blobs() {
    let (b, _dir) = make_chain();
    let head = b.dag.head().unwrap().unwrap();
    let chain = b.dag.walk_from(&head).unwrap();

    for cid in &chain {
        let ev = b.dag.get(cid).unwrap().unwrap();
        let ev_bytes = serde_json::to_vec(&ev).unwrap();
        let ev_blob_cid = Cid::of(&ev_bytes);
        assert!(
            b.worm.is_sealed(&ev_blob_cid),
            "event blob not in WORM_FS at height {}", ev.height
        );
    }
}

#[test]
fn worm_read_verifies_integrity() {
    let dir = tempfile::tempdir().unwrap();
    let worm = bifrost::worm::WormFs::open(dir.path()).unwrap();

    let data = b"immutable payload";
    let cid  = worm.seal_blob(data, None).unwrap();
    let back = worm.read_blob(&cid).unwrap();
    assert_eq!(data.as_ref(), back.as_slice());
}

#[test]
fn concurrent_append_grows_chain() {
    use bifrost::append::open_writer;
    use std::time::{Duration, Instant};
    use std::thread;

    let dir = tempfile::tempdir().unwrap();
    let (writer, aq) = open_writer(dir.path()).unwrap();

    let head_before = aq.head().unwrap();

    // Push 5 events synchronously
    for _ in 0..5 {
        aq.push_sync(EventPayload::CapTransfer {
            from: Cid([0u8; 32]), to: Cid([0u8; 32]),
            cap_hash: Cid([0u8; 32]), policy_cid: Cid([0u8; 32]),
        }).expect("push_sync");
    }

    let head_after = aq.head().unwrap();
    assert_ne!(head_before, head_after, "head must advance");

    writer.shutdown();
}
