pub mod emoji;
pub mod poet;
pub mod scratch;
pub mod sigil;

use soulvm::{BifrostBridge, Instruction, SoulFunc, ValType};
use thiserror::Error;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Xor,
    // Float ops (Nemotron extension)
    FAdd,
    FSub,
    FMul,
    FDiv,
}

/// Nemotron alias — same type, two names.
pub type BinOpKind = BinOp;

#[derive(Clone, Debug, PartialEq)]  // no Eq: PushF(f64) is not Eq
pub enum SigilOp {
    Push(i64),
    PushF(f64),  // Nemotron extension: float constant
    BinOp(BinOp),
    Jump(usize),
    JumpIf(usize),
    CapGate(u32, u8),
    Call(u32),
    Ret,
    Alloc(u32, u32),
    Load(u32),
    Store(u32),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SigilProgram {
    pub ops: Vec<SigilOp>,
}

impl SigilProgram {
    pub fn new(ops: Vec<SigilOp>) -> Self {
        Self { ops }
    }

    pub fn lower(self, name: impl Into<String>) -> SoulFunc {
        SoulFunc {
            name: name.into(),
            params: Vec::new(),
            returns: vec![ValType::I64],
            locals: Vec::new(),
            body: self.ops.into_iter().map(lower_op).collect(),
        }
    }
}

pub trait Dialect: Send + Sync {
    fn compile(&self, input: &str) -> Result<SoulFunc, CraftError>;
}

#[derive(Debug, Error)]
pub enum CraftError {
    #[error("invalid emoji: {0}")]
    InvalidEmoji(String),
    #[error("empty program")]
    EmptyProgram,
    #[error("invalid numeric literal: {0}")]
    InvalidNumber(String),
    #[error("unknown Scratch block: {0}")]
    UnknownBlock(String),
    #[error("missing or invalid argument {index} for block {block}")]
    InvalidBlockArgument { block: String, index: usize },
    #[error("invalid Scratch JSON: {0}")]
    InvalidJson(#[from] serde_json::Error),
}

pub fn compile_and_seal<D: Dialect>(
    dialect: &D,
    input: &str,
    bridge: &BifrostBridge,
) -> Result<SoulFunc, CraftError> {
    dialect.compile(input).map(|func| {
        bridge.seal_jit_compile(&func);
        func
    })
}

pub(crate) fn input_hash(input: &str) -> String {
    let hash = input
        .as_bytes()
        .iter()
        .fold(0xcbf29ce484222325u64, |hash, byte| {
            (hash ^ u64::from(*byte)).wrapping_mul(0x100000001b3)
        });
    format!("{hash:016x}")
}

fn lower_op(op: SigilOp) -> Instruction {
    match op {
        SigilOp::Push(value)         => Instruction::I64Const(value),
        SigilOp::PushF(value)        => Instruction::F64Const(value),
        SigilOp::BinOp(BinOp::Add)  => Instruction::I64Add,
        SigilOp::BinOp(BinOp::Sub)  => Instruction::I64Sub,
        SigilOp::BinOp(BinOp::Mul)  => Instruction::I64Mul,
        SigilOp::BinOp(BinOp::Div)  => Instruction::I64DivS,
        SigilOp::BinOp(BinOp::And)  => Instruction::I64And,
        SigilOp::BinOp(BinOp::Or)   => Instruction::I64Or,
        SigilOp::BinOp(BinOp::Xor)  => Instruction::I64Xor,
        SigilOp::BinOp(BinOp::FAdd) => Instruction::F64Add,
        SigilOp::BinOp(BinOp::FSub) => Instruction::F64Sub,
        SigilOp::BinOp(BinOp::FMul) => Instruction::F64Mul,
        SigilOp::BinOp(BinOp::FDiv) => Instruction::F64Div,
        SigilOp::Jump(target)        => Instruction::Br(target),
        SigilOp::JumpIf(target)      => Instruction::BrIf(target),
        SigilOp::CapGate(slot, rights) => Instruction::CapCheck { cap_slot: slot, rights },
        SigilOp::Call(function)      => Instruction::Call(function),
        SigilOp::Ret                 => Instruction::Return,
        SigilOp::Alloc(size, type_tag) => Instruction::Alloc { size, type_tag },
        SigilOp::Load(offset)        => Instruction::Load { offset },
        SigilOp::Store(offset)       => Instruction::Store { offset },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lowers_all_sigil_ops() {
        let program = SigilProgram::new(vec![
            SigilOp::Push(1),
            SigilOp::BinOp(BinOp::Add),
            SigilOp::Jump(2),
            SigilOp::JumpIf(3),
            SigilOp::CapGate(4, 5),
            SigilOp::Call(6),
            SigilOp::Alloc(7, 8),
            SigilOp::Load(9),
            SigilOp::Store(10),
            SigilOp::Ret,
        ]);
        let function = program.lower("all");
        assert_eq!(function.params, Vec::<ValType>::new());
        assert_eq!(function.returns, vec![ValType::I64]);
        assert_eq!(function.body.len(), 10);
    }
}
