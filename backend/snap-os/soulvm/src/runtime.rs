/// SoulRuntime — execution context for one soul.
/// Owns a JitEngine, an ImmixHeap, and a silverback CSpace.
/// Optionally wired to a `BifrostBridge` that seals every compile/mint into the audit chain.

use crate::bridge::BifrostBridge;
use crate::bytecode::SoulFunc;
use crate::gc::ImmixHeap;
use crate::jit::{CompiledFunc, JitEngine};
use silverback::capability::{Cap, CapError, CapPtr, CSpace, Rights};

pub const CSPACE_SIZE: usize = 256;

pub struct SoulRuntime {
    pub cspace: CSpace<CSPACE_SIZE>,
    pub heap:   ImmixHeap,
    jit:        JitEngine,
    bridge:     Option<BifrostBridge>,
}

impl SoulRuntime {
    pub fn new(soul_id: u64) -> Result<Self, String> {
        Ok(Self {
            cspace: CSpace::new(soul_id),
            heap:   ImmixHeap::new(),
            jit:    JitEngine::new()?,
            bridge: None,
        })
    }

    /// Attach a bifrost bridge. Every subsequent `compile` and `mint_cap` call
    /// will automatically seal an audit event into the chain.
    pub fn attach_bifrost(&mut self, bridge: BifrostBridge) {
        self.bridge = Some(bridge);
    }

    /// Compile a SoulFunc via Cranelift JIT.
    /// If a bifrost bridge is attached, seals a `JitCompile` event after success.
    pub fn compile(&mut self, func: &SoulFunc) -> Result<CompiledFunc, String> {
        let compiled = self.jit.compile(func)?;
        if let Some(b) = &self.bridge {
            b.seal_jit_compile(func);
        }
        Ok(compiled)
    }

    /// Allocate `size` bytes on the GC heap with the given type tag.
    pub fn alloc(&mut self, size: u32, type_tag: u32) -> Option<crate::gc::GcPtr> {
        self.heap.alloc(size, type_tag)
    }

    /// Mint a new capability into `dest_slot`, restricted by `parent_slot` rights.
    /// If a bifrost bridge is attached, seals a `CapTransfer` event after success.
    pub fn mint_cap(
        &mut self,
        parent_slot: CapPtr,
        dest_slot: CapPtr,
        new_cap: Cap,
    ) -> Result<(), CapError> {
        let parent = *self.cspace.lookup(parent_slot)?;
        if !parent.rights.can_delegate() {
            return Err(CapError::InsufficientRights);
        }
        let restricted = Cap {
            rights: new_cap.rights.restrict(parent.rights),
            ..new_cap
        };
        self.cspace.insert(dest_slot, restricted)?;
        if let Some(b) = &self.bridge {
            b.seal_cap_transfer(self.cspace.soul_id, &restricted);
        }
        Ok(())
    }

    /// Assert that `slot` has at least `required` rights. Returns Err on failure.
    pub fn check_rights(&self, slot: CapPtr, required: Rights) -> Result<(), CapError> {
        let cap = self.cspace.lookup(slot)?;
        if cap.is_null() {
            return Err(CapError::NullCap);
        }
        if !cap.rights.has(required) {
            return Err(CapError::InsufficientRights);
        }
        Ok(())
    }

    /// Trigger a full GC cycle on this soul's heap.
    pub fn collect(&mut self) {
        self.heap.collect();
    }

    pub fn soul_id(&self) -> u64 { self.cspace.soul_id }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bytecode::{Instruction, SoulFunc, ValType};
    use silverback::capability::{Cap, CapType, Rights};

    fn make_runtime() -> SoulRuntime {
        SoulRuntime::new(1).expect("SoulRuntime::new")
    }

    #[test]
    fn runtime_jit_and_gc_together() {
        let mut rt = make_runtime();

        // Compile and run a JIT function
        let mut f = SoulFunc::new("triple", vec![ValType::I64], vec![ValType::I64]);
        f.push(Instruction::LocalGet(0))
         .push(Instruction::I64Const(3))
         .push(Instruction::I64Mul)
         .push(Instruction::Return);

        let compiled = rt.compile(&f).expect("compile");
        let func: extern "C" fn(i64) -> i64 = unsafe { std::mem::transmute(compiled.code) };
        assert_eq!(func(5), 15);

        // GC heap allocation
        let ptr = rt.alloc(64, 0xDEAD).expect("alloc");
        assert_eq!(ptr.type_tag(), 0xDEAD);
    }

    #[test]
    fn cap_check_blocks_insufficient_rights() {
        let mut rt = make_runtime();
        let cap = Cap {
            cap_type:  CapType::Endpoint,
            object_id: 1,
            rights:    Rights::READ,
            badge:     0,
            sealed:    false,
        };
        rt.cspace.insert(CapPtr(0), cap).unwrap();

        // READ only — INVOKE should be denied
        let err = rt.check_rights(CapPtr(0), Rights::INVOKE).unwrap_err();
        assert_eq!(err, CapError::InsufficientRights);

        // READ is present — should pass
        rt.check_rights(CapPtr(0), Rights::READ).unwrap();
    }

    #[test]
    fn gc_runs_without_crashing_runtime() {
        let mut rt = make_runtime();
        for _ in 0..20 {
            rt.alloc(128, 1);
        }
        rt.collect();
        // Just must not panic
    }
}
