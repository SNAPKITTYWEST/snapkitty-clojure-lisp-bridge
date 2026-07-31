use silverback::capability::{Cap, CapPtr, CapType, Rights};

use crate::{BifrostBridge, Instruction, SoulFunc, SoulRuntime, ValType};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GenesisSoulReport {
    pub soul_id: u64,
    pub result: i64,
    pub chain_height: u64,
}

/// Spawn the first host-side genesis soul and seal its first execution.
///
/// This is the pre-QEMU Genesis gate. It exercises the same contract the
/// bare-metal path must satisfy next: Silverback grants a root capability,
/// SoulVM executes a genesis function, and Bifrost records the resulting JIT
/// plus capability transfer events on a verified chain.
pub fn spawn_genesis_soul(
    soul_id: u64,
    bridge: BifrostBridge,
) -> Result<GenesisSoulReport, String> {
    let mut runtime = SoulRuntime::new(soul_id)?;
    runtime.attach_bifrost(bridge.clone());

    let root_cap = Cap {
        cap_type: CapType::Endpoint,
        object_id: soul_id,
        rights: Rights::ALL,
        badge: 0,
        sealed: false,
    };
    runtime
        .cspace
        .insert(CapPtr(0), root_cap)
        .map_err(|e| format!("{e:?}"))?;

    let mut func = SoulFunc::new("genesis_ping", vec![], vec![ValType::I64]);
    func.push(Instruction::I64Const(1)).push(Instruction::Return);

    let compiled = runtime.compile(&func)?;
    let entry: extern "C" fn() -> i64 = unsafe { core::mem::transmute(compiled.code) };
    let result = entry();

    let exported_cap = Cap {
        cap_type: CapType::Endpoint,
        object_id: soul_id + 1,
        rights: Rights::READ | Rights::INVOKE,
        badge: 1,
        sealed: false,
    };
    runtime
        .mint_cap(CapPtr(0), CapPtr(1), exported_cap)
        .map_err(|e| format!("{e:?}"))?;

    let chain_height = bridge
        .with_bifrost(|b| b.dag.height().unwrap_or(0))
        .ok_or_else(|| "bifrost bridge lock poisoned".to_string())?;

    Ok(GenesisSoulReport {
        soul_id,
        result,
        chain_height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use bifrost::seal::Bifrost;

    #[test]
    fn genesis_soul_seals_first_execution() {
        let dir = tempfile::tempdir().unwrap();
        let bridge = BifrostBridge::new(Bifrost::open(dir.path()).unwrap());

        let report = spawn_genesis_soul(1, bridge).unwrap();

        assert_eq!(report.soul_id, 1);
        assert_eq!(report.result, 1);
        assert_eq!(report.chain_height, 2);
    }
}
