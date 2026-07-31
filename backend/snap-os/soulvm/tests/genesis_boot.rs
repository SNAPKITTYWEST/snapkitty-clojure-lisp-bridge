use bifrost::{seal::Bifrost, verify::verify_chain};
use soulvm::{spawn_genesis_soul, BifrostBridge};

#[test]
fn genesis_soul_execution_is_sealed_and_verifiable() {
    let dir = tempfile::tempdir().unwrap();
    let bridge = BifrostBridge::new(Bifrost::open(dir.path()).unwrap());
    let verifier = bridge.clone();

    let report = spawn_genesis_soul(1, bridge).expect("spawn genesis soul");

    assert_eq!(report.result, 1);
    assert_eq!(report.chain_height, 2, "genesis + JitCompile + CapTransfer");

    let verify_report = verifier
        .with_bifrost(|b| {
            let head = b.dag.head().unwrap().unwrap();
            verify_chain(&head, &b.dag, &b.worm).unwrap()
        })
        .expect("bifrost bridge lock");

    assert!(verify_report.ok, "chain errors: {:?}", verify_report.errors);
    assert_eq!(verify_report.event_count, 3);
    assert_eq!(verify_report.height, 2);
}
