/// SoulVM bytecode — the instruction set consumed by the Cranelift JIT.
/// Stack-based at the source level; the JIT compiler lifts it to SSA.

use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Instruction {
    // ── Constants ────────────────────────────────────────────────────────────
    I64Const(i64),
    F64Const(f64),

    // ── Integer arithmetic (pop 2, push 1) ───────────────────────────────────
    I64Add,
    I64Sub,
    I64Mul,
    I64DivS, // signed divide
    I64And,
    I64Or,
    I64Xor,
    I64Shl,
    I64ShrS,

    // ── Integer comparisons (pop 2, push i64: 1=true 0=false) ────────────────
    I64Eq,
    I64Ne,
    I64LtS,
    I64GtS,
    I64LeS,
    I64GeS,

    // ── Float arithmetic ─────────────────────────────────────────────────────
    F64Add,
    F64Sub,
    F64Mul,
    F64Div,

    // ── Locals ───────────────────────────────────────────────────────────────
    LocalGet(u32),
    LocalSet(u32),

    // ── Control flow ─────────────────────────────────────────────────────────
    /// Unconditional jump to instruction index
    Br(usize),
    /// Pop i64; jump if non-zero
    BrIf(usize),
    /// Pop i64; jump if zero
    BrIfZ(usize),
    /// Call a function by index in the SoulFunc table
    Call(u32),
    /// Pop top-of-stack and return it
    Return,
    /// Push nothing, used as a jump target
    Nop,

    // ── GC heap ──────────────────────────────────────────────────────────────
    /// Allocate an object: (size_bytes: u32, type_tag: u32) → GcPtr (i64)
    Alloc { size: u32, type_tag: u32 },
    /// Load i64 from GcPtr + offset
    Load { offset: u32 },
    /// Store i64 at GcPtr + offset; pops value then ptr
    Store { offset: u32 },

    // ── Capability-gated operations ───────────────────────────────────────────
    /// Assert the caller holds cap_slot with at least `rights` bits set.
    /// Traps if the check fails; no stack effect on success.
    CapCheck { cap_slot: u32, rights: u8 },
    /// Invoke the endpoint at cap_slot, passing top-of-stack as the argument.
    CapInvoke { cap_slot: u32 },
}

/// Type of a value on the SoulVM operand stack.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum ValType {
    I64,
    F64,
}

/// A compiled function ready for JIT translation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SoulFunc {
    pub name:    String,
    pub params:  Vec<ValType>,
    pub returns: Vec<ValType>,
    pub locals:  Vec<ValType>, // extra locals beyond params
    pub body:    Vec<Instruction>,
}

impl SoulFunc {
    pub fn new(name: impl Into<String>, params: Vec<ValType>, returns: Vec<ValType>) -> Self {
        Self { name: name.into(), params, returns, locals: vec![], body: vec![] }
    }

    pub fn push(&mut self, instr: Instruction) -> &mut Self {
        self.body.push(instr);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_add_func() {
        let mut f = SoulFunc::new("add", vec![ValType::I64, ValType::I64], vec![ValType::I64]);
        f.push(Instruction::LocalGet(0))
         .push(Instruction::LocalGet(1))
         .push(Instruction::I64Add)
         .push(Instruction::Return);
        assert_eq!(f.body.len(), 4);
    }
}
