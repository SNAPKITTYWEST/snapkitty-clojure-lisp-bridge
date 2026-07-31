/// Integration test: bifrost chain is live when wired into soulvm.
///
/// After each `compile()` → JitCompile event in chain.
/// After each `mint_cap()` → CapTransfer event in chain.
/// Full `verify_chain` passes after both operations.

use bifrost::{seal::Bifrost, verify::verify_chain};
use silverback::capability::{Cap, CapPtr, CapType, Rights};
use soulvm::{BifrostBridge, Instruction, SoulFunc, SoulRuntime, ValType};
use tempfile::tempdir;

fn parent_cap(object_id: u64) -> Cap {
    Cap { cap_type: CapType::Endpoint, object_id, rights: Rights::ALL, badge: 0, sealed: false }
}

fn read_cap(object_id: u64) -> Cap {
    Cap { cap_type: CapType::Endpoint, object_id, rights: Rights::READ, badge: 0, sealed: false }
}

#[test]
fn jit_compile_seals_into_chain() {
    let dir = tempdir().unwrap();
    let bridge = BifrostBridge::new(Bifrost::open(dir.path()).unwrap());
    let handle = bridge.clone(); // verification handle shares same Arc<Mutex<Bifrost>>

    let mut rt = SoulRuntime::new(42).unwrap();
    rt.attach_bifrost(bridge);

    let mut f = SoulFunc::new("square", vec![ValType::I64], vec![ValType::I64]);
    f.push(Instruction::LocalGet(0))
     .push(Instruction::LocalGet(0))
     .push(Instruction::I64Mul)
     .push(Instruction::Return);

    let compiled = rt.compile(&f).expect("compile");
    let func: extern "C" fn(i64) -> i64 = unsafe { std::mem::transmute(compiled.code) };
    assert_eq!(func(7), 49);

    let height = handle.with_bifrost(|b| b.dag.height().unwrap()).unwrap();
    assert_eq!(height, 1, "genesis(0) + JitCompile(1)");
}

#[test]
fn mint_cap_seals_into_chain() {
    let dir = tempdir().unwrap();
    let bridge = BifrostBridge::new(Bifrost::open(dir.path()).unwrap());
    let handle = bridge.clone();

    let mut rt = SoulRuntime::new(99).unwrap();
    rt.attach_bifrost(bridge);

    rt.cspace.insert(CapPtr(0), parent_cap(1)).unwrap();
    rt.mint_cap(CapPtr(0), CapPtr(1), read_cap(2)).expect("mint_cap");

    let height = handle.with_bifrost(|b| b.dag.height().unwrap()).unwrap();
    assert_eq!(height, 1, "genesis(0) + CapTransfer(1)");
}

#[test]
fn chain_verifies_after_compile_and_mint() {
    let dir = tempdir().unwrap();
    let bridge = BifrostBridge::new(Bifrost::open(dir.path()).unwrap());
    let handle = bridge.clone();

    let mut rt = SoulRuntime::new(7).unwrap();
    rt.attach_bifrost(bridge);

    // JIT compile
    let mut f = SoulFunc::new("add1", vec![ValType::I64], vec![ValType::I64]);
    f.push(Instruction::LocalGet(0))
     .push(Instruction::I64Const(1))
     .push(Instruction::I64Add)
     .push(Instruction::Return);
    rt.compile(&f).unwrap();

    // Capability mint
    rt.cspace.insert(CapPtr(0), parent_cap(10)).unwrap();
    rt.mint_cap(CapPtr(0), CapPtr(1), read_cap(11)).unwrap();

    // Full chain verification
    let report = handle.with_bifrost(|b| {
        let head = b.dag.head().unwrap().unwrap();
        verify_chain(&head, &b.dag, &b.worm).unwrap()
    }).unwrap();

    assert!(report.ok, "chain errors: {:?}", report.errors);
    assert_eq!(report.event_count, 3, "genesis + JitCompile + CapTransfer");
    assert_eq!(report.height, 2);
}

#[test]
fn two_compiles_advance_height_by_two() {
    let dir = tempdir().unwrap();
    let bridge = BifrostBridge::new(Bifrost::open(dir.path()).unwrap());
    let handle = bridge.clone();

    let mut rt = SoulRuntime::new(5).unwrap();
    rt.attach_bifrost(bridge);

    for i in 0u64..2 {
        let mut f = SoulFunc::new(format!("fn_{i}"), vec![ValType::I64], vec![ValType::I64]);
        f.push(Instruction::I64Const(i as i64)).push(Instruction::Return);
        rt.compile(&f).unwrap();
    }

    let height = handle.with_bifrost(|b| b.dag.height().unwrap()).unwrap();
    assert_eq!(height, 2);
}

#[test]
fn compile_without_bridge_still_works() {
    // Bridge is optional — runtime without it must not regress.
    let mut rt = SoulRuntime::new(1).unwrap();
    let mut f = SoulFunc::new("nop", vec![], vec![ValType::I64]);
    f.push(Instruction::I64Const(0)).push(Instruction::Return);
    rt.compile(&f).expect("compile without bridge");
}
