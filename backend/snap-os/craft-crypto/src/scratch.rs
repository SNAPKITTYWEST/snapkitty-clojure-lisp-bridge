use serde::Deserialize;
use serde_json::Value;
use soulvm::SoulFunc;

use crate::{input_hash, BinOp, CraftError, Dialect, SigilOp, SigilProgram};

#[derive(Clone, Copy, Debug, Default)]
pub struct ScratchBlocks;

#[derive(Debug, Deserialize)]
struct Block {
    #[serde(rename = "type")]
    opcode: String,
    #[serde(default)]
    args: Vec<Value>,
    #[serde(default)]
    next: Option<Box<Block>>,
}

impl ScratchBlocks {
    pub fn compile(input: &str) -> Result<SoulFunc, CraftError> {
        let root: Block = serde_json::from_str(input)?;
        let mut ops = Vec::new();
        walk(&root, &mut ops)?;
        Ok(SigilProgram::new(ops).lower(format!("scratch_fn_{}", input_hash(input))))
    }
}

impl Dialect for ScratchBlocks {
    fn compile(&self, input: &str) -> Result<SoulFunc, CraftError> {
        Self::compile(input)
    }
}

fn walk(block: &Block, ops: &mut Vec<SigilOp>) -> Result<(), CraftError> {
    ops.push(match block.opcode.as_str() {
        "const" => SigilOp::Push(i64_arg(block, 0)?),
        "add" => SigilOp::BinOp(BinOp::Add),
        "sub" => SigilOp::BinOp(BinOp::Sub),
        "mul" => SigilOp::BinOp(BinOp::Mul),
        "div" => SigilOp::BinOp(BinOp::Div),
        "cap_check" => SigilOp::CapGate(u32_arg(block, 0)?, u8_arg(block, 1)?),
        "call" => SigilOp::Call(u32_arg(block, 0)?),
        "alloc" => SigilOp::Alloc(u32_arg(block, 0)?, u32_arg(block, 1)?),
        "load" => SigilOp::Load(u32_arg(block, 0)?),
        "store" => SigilOp::Store(u32_arg(block, 0)?),
        "return" => SigilOp::Ret,
        "jump" => SigilOp::Jump(usize_arg(block, 0)?),
        "jump_if" => SigilOp::JumpIf(usize_arg(block, 0)?),
        unknown => return Err(CraftError::UnknownBlock(unknown.to_owned())),
    });

    block.next.as_deref().map_or(Ok(()), |next| walk(next, ops))
}

fn value_arg(block: &Block, index: usize) -> Result<&Value, CraftError> {
    block
        .args
        .get(index)
        .ok_or_else(|| CraftError::InvalidBlockArgument {
            block: block.opcode.clone(),
            index,
        })
}

fn i64_arg(block: &Block, index: usize) -> Result<i64, CraftError> {
    value_arg(block, index)?
        .as_i64()
        .ok_or_else(|| invalid_arg(block, index))
}

fn u32_arg(block: &Block, index: usize) -> Result<u32, CraftError> {
    value_arg(block, index)?
        .as_u64()
        .and_then(|value| u32::try_from(value).ok())
        .ok_or_else(|| invalid_arg(block, index))
}

fn u8_arg(block: &Block, index: usize) -> Result<u8, CraftError> {
    value_arg(block, index)?
        .as_u64()
        .and_then(|value| u8::try_from(value).ok())
        .ok_or_else(|| invalid_arg(block, index))
}

fn usize_arg(block: &Block, index: usize) -> Result<usize, CraftError> {
    value_arg(block, index)?
        .as_u64()
        .and_then(|value| usize::try_from(value).ok())
        .ok_or_else(|| invalid_arg(block, index))
}

fn invalid_arg(block: &Block, index: usize) -> CraftError {
    CraftError::InvalidBlockArgument {
        block: block.opcode.clone(),
        index,
    }
}

#[cfg(test)]
mod tests {
    use soulvm::Instruction;

    use super::*;

    #[test]
    fn compiles_linked_blocks() {
        let input = r#"{
            "type":"const",
            "args":[7],
            "next":{
                "type":"const",
                "args":[6],
                "next":{
                    "type":"mul",
                    "args":[],
                    "next":{"type":"return","args":[],"next":null}
                }
            }
        }"#;
        let function = ScratchBlocks::compile(input).unwrap();
        assert_eq!(
            function.body,
            vec![
                Instruction::I64Const(7),
                Instruction::I64Const(6),
                Instruction::I64Mul,
                Instruction::Return,
            ]
        );
    }

    #[test]
    fn compiles_operands() {
        let input = r#"{
            "type":"cap_check",
            "args":[3,5],
            "next":{
                "type":"alloc",
                "args":[64,9],
                "next":{
                    "type":"jump_if",
                    "args":[2],
                    "next":null
                }
            }
        }"#;
        let function = ScratchBlocks::compile(input).unwrap();
        assert_eq!(
            function.body,
            vec![
                Instruction::CapCheck {
                    cap_slot: 3,
                    rights: 5,
                },
                Instruction::Alloc {
                    size: 64,
                    type_tag: 9,
                },
                Instruction::BrIf(2),
            ]
        );
    }

    #[test]
    fn rejects_unknown_block() {
        let error =
            ScratchBlocks::compile(r#"{"type":"paint","args":[],"next":null}"#).unwrap_err();
        assert!(matches!(error, CraftError::UnknownBlock(value) if value == "paint"));
    }
}
