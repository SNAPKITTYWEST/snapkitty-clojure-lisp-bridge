use soulvm::SoulFunc;
use unicode_segmentation::UnicodeSegmentation;

use crate::{input_hash, BinOp, CraftError, Dialect, SigilOp, SigilProgram};

/// Nemotron's token layer — exposes the EmojiScript AST for tooling and
/// future semantic passes (🌊 stream, 🧠 policy) that don't compile directly
/// to SigilOp until those organs (telemetry-bus, policy-immune) exist.
#[derive(Debug, Clone, PartialEq)]
pub enum EmojiToken {
    Number(i64),          // 🔢<digits>
    NumberF(f64),         // 🔣<digits.fraction>  (reserved, Sprint 2)
    BinOp(BinOp),         // ➕ ➖ ✖️ ➗ 🌀 🤝 👐
    CapGate(u32, u8),     // 🔑  (consumes two preceding Number tokens)
    Call(u32),            // ⚡
    Alloc(u32, u32),      // 🏗️
    Load(u32),            // 📤
    Store(u32),           // 📦
    Jump(usize),          // ➡️
    JumpIf(usize),        // ❓
    Ret,                  // ↩️
    Stream,               // 🌊 — routes to telemetry-bus (Sprint 2)
    PolicyCheck,          // 🧠 — routes to policy-immune  (Sprint 2)
    Seal,                 // 🔒 — explicit Bifrost seal hint (Sprint 2)
    ReadOnly,             // 🔓 — downgrade rights hint      (Sprint 2)
}

impl EmojiToken {
    /// Lower to a `SigilOp`.  Returns `None` for semantic-only tokens
    /// (Stream, PolicyCheck, Seal, ReadOnly) that route to other organs.
    pub fn to_sigil_op(&self) -> Option<SigilOp> {
        match self {
            Self::Number(n)          => Some(SigilOp::Push(*n)),
            Self::NumberF(f)         => Some(SigilOp::PushF(*f)),
            Self::BinOp(k)           => Some(SigilOp::BinOp(*k)),
            Self::CapGate(s, r)      => Some(SigilOp::CapGate(*s, *r)),
            Self::Call(f)            => Some(SigilOp::Call(*f)),
            Self::Alloc(sz, tt)      => Some(SigilOp::Alloc(*sz, *tt)),
            Self::Load(o)            => Some(SigilOp::Load(*o)),
            Self::Store(o)           => Some(SigilOp::Store(*o)),
            Self::Jump(t)            => Some(SigilOp::Jump(*t)),
            Self::JumpIf(t)          => Some(SigilOp::JumpIf(*t)),
            Self::Ret                => Some(SigilOp::Ret),
            Self::Stream
            | Self::PolicyCheck
            | Self::Seal
            | Self::ReadOnly         => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct EmojiScript;

impl EmojiScript {
    pub fn compile(input: &str) -> Result<SoulFunc, CraftError> {
        let ops = lex(input)?;
        Ok(SigilProgram::new(ops).lower(format!("emoji_fn_{}", input_hash(input))))
    }
}

impl Dialect for EmojiScript {
    fn compile(&self, input: &str) -> Result<SoulFunc, CraftError> {
        Self::compile(input)
    }
}

/// Pop the most-recently-pushed `SigilOp::Push(n)` and return its value.
/// Args are written before their opcode in source: `🔢3 🔢5 🔑` = cap_check(3,5).
fn pop_push_val(ops: &mut Vec<SigilOp>) -> Result<i64, CraftError> {
    match ops.last() {
        Some(SigilOp::Push(_)) => {
            if let Some(SigilOp::Push(n)) = ops.pop() { Ok(n) } else { unreachable!() }
        }
        _ => Err(CraftError::InvalidEmoji("🔢<arg> required before this opcode".into())),
    }
}

fn lex(input: &str) -> Result<Vec<SigilOp>, CraftError> {
    let graphemes: Vec<&str> = UnicodeSegmentation::graphemes(input, true).collect();
    let mut ops: Vec<SigilOp> = Vec::new();
    let mut idx = 0;

    while idx < graphemes.len() {
        let g = graphemes[idx];
        if g.chars().all(char::is_whitespace) { idx += 1; continue; }

        // ── numeric literal: 🔢<digits> ───────────────────────────────────────
        if g == "🔢" {
            let start = idx + 1;
            let mut end = start;
            let mut digits = String::new();
            while let Some(c) = graphemes.get(end) {
                if c.len() == 1 && c.as_bytes().first().is_some_and(u8::is_ascii_digit) {
                    digits.push_str(c);
                    end += 1;
                } else { break; }
            }
            if digits.is_empty() {
                return Err(CraftError::InvalidNumber(digits));
            }
            let n = digits.parse::<i64>().map_err(|_| CraftError::InvalidNumber(digits))?;
            ops.push(SigilOp::Push(n));
            idx = end;
            continue;
        }

        // ── zero-arg ops ──────────────────────────────────────────────────────
        let zero_arg: Option<SigilOp> = match g {
            "➕"              => Some(SigilOp::BinOp(BinOp::Add)),
            "➖"              => Some(SigilOp::BinOp(BinOp::Sub)),
            "✖" | "✖️" | "❌" => Some(SigilOp::BinOp(BinOp::Mul)),
            "➗"              => Some(SigilOp::BinOp(BinOp::Div)),
            "🤝" | "🔗"      => Some(SigilOp::BinOp(BinOp::And)),
            "👐" | "🔀"      => Some(SigilOp::BinOp(BinOp::Or)),
            "🌀" | "⊕"       => Some(SigilOp::BinOp(BinOp::Xor)), // ⚡ = Call
            "↩" | "↩️" | "🔙" | "🔚" => Some(SigilOp::Ret),
            _ => None,
        };
        if let Some(op) = zero_arg {
            ops.push(op);
            idx += 1;
            continue;
        }

        // ── arg-consuming ops (args are preceding 🔢 pushes, popped in reverse) ─
        let arg_op: Result<SigilOp, CraftError> = match g {
            // 🔢<slot> 🔢<rights> 🔑  →  pop rights, pop slot
            "🔑" => {
                let rights = pop_push_val(&mut ops)? as u8;
                let slot   = pop_push_val(&mut ops)? as u32;
                Ok(SigilOp::CapGate(slot, rights))
            }
            // 🔢<func_idx> ⚡
            "⚡" => {
                let func = pop_push_val(&mut ops)? as u32;
                Ok(SigilOp::Call(func))
            }
            // 🔢<size> 🔢<type_tag> 🏗️
            "🏗️" => {
                let type_tag = pop_push_val(&mut ops)? as u32;
                let size     = pop_push_val(&mut ops)? as u32;
                Ok(SigilOp::Alloc(size, type_tag))
            }
            // 🔢<offset> 📤
            "📤" => Ok(SigilOp::Load(pop_push_val(&mut ops)? as u32)),
            // 🔢<offset> 📦
            "📦" => Ok(SigilOp::Store(pop_push_val(&mut ops)? as u32)),
            // 🔢<target> ➡️
            "➡️" => Ok(SigilOp::Jump(pop_push_val(&mut ops)? as usize)),
            // 🔢<target> ❓
            "❓" => Ok(SigilOp::JumpIf(pop_push_val(&mut ops)? as usize)),

            other => Err(CraftError::InvalidEmoji(other.to_owned())),
        };
        ops.push(arg_op?);
        idx += 1;
    }

    if ops.is_empty() { Err(CraftError::EmptyProgram) } else { Ok(ops) }
}

#[cfg(test)]
mod tests {
    use soulvm::{Instruction, ValType};
    use super::*;

    #[test]
    fn compiles_arithmetic_program() {
        let f = EmojiScript::compile("🔢40 🔢2 ➕ ↩️").unwrap();
        assert!(f.name.starts_with("emoji_fn_"));
        assert_eq!(f.params, Vec::<ValType>::new());
        assert_eq!(f.returns, vec![ValType::I64]);
        assert_eq!(f.body, vec![
            Instruction::I64Const(40),
            Instruction::I64Const(2),
            Instruction::I64Add,
            Instruction::Return,
        ]);
    }

    #[test]
    fn compiles_cap_check() {
        // 🔢3 🔢5 🔑 → CapCheck { cap_slot: 3, rights: 5 }
        let f = EmojiScript::compile("🔢3 🔢5 🔑 ↩️").unwrap();
        assert_eq!(f.body, vec![
            Instruction::CapCheck { cap_slot: 3, rights: 5 },
            Instruction::Return,
        ]);
    }

    #[test]
    fn compiles_alloc_and_call() {
        // 🔢64 🔢9 🏗️  →  Alloc { size: 64, type_tag: 9 }
        // 🔢2 ⚡         →  Call(2)
        let f = EmojiScript::compile("🔢64 🔢9 🏗️ 🔢2 ⚡ ↩️").unwrap();
        assert_eq!(f.body, vec![
            Instruction::Alloc { size: 64, type_tag: 9 },
            Instruction::Call(2),
            Instruction::Return,
        ]);
    }

    #[test]
    fn rejects_unknown_grapheme() {
        assert!(matches!(
            EmojiScript::compile("🥷"),
            Err(CraftError::InvalidEmoji(_))
        ));
    }

    #[test]
    fn rejects_empty_program() {
        assert!(matches!(
            EmojiScript::compile("   \n\t"),
            Err(CraftError::EmptyProgram)
        ));
    }
}
