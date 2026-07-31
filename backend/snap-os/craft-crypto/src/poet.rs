/// PoetCode — Shakespearean verse compiles to SoulFunc.
///
/// Rules:
///   - Each line = one statement
///   - "Speak" at start of line → Return
///   - "Guard" at start → CapCheck (slot = ordinal after "slot", rights = 0x01)
///   - "Push" followed by a number word or Arabic numeral → I64Const
///   - "Add" / "Subtract" / "Multiply" / "Divide" anywhere → arithmetic op
///   - Number words: one=1 … ten=10; Arabic numerals also accepted
///   - "Thus" at end is decorative — ignored

use soulvm::{Instruction, SoulFunc, ValType};

use crate::{input_hash, CraftError, Dialect};

#[derive(Clone, Copy, Debug, Default)]
pub struct PoetCode;

impl Dialect for PoetCode {
    fn compile(&self, input: &str) -> Result<SoulFunc, CraftError> {
        let lines: Vec<&str> = input
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .collect();

        if lines.is_empty() {
            return Err(CraftError::EmptyProgram);
        }

        let mut body: Vec<Instruction> = Vec::new();

        for line in &lines {
            let lower = line.to_lowercase();
            let words: Vec<&str> = lower.split_whitespace().collect();

            if words.is_empty() {
                continue;
            }

            if words[0] == "speak" {
                body.push(Instruction::Return);
                continue;
            }

            if words[0] == "guard" {
                let slot = slot_ordinal(&words);
                body.push(Instruction::CapCheck { cap_slot: slot, rights: 0x01 });
                continue;
            }

            // Scan for Push + number pairs first
            let mut i = 0;
            while i < words.len() {
                if words[i] == "push" {
                    if let Some(n) = words.get(i + 1).and_then(|w| parse_number(w)) {
                        body.push(Instruction::I64Const(n));
                        i += 2;
                        continue;
                    }
                }
                // Arithmetic keywords
                match words[i] {
                    "add"      => body.push(Instruction::I64Add),
                    "subtract" => body.push(Instruction::I64Sub),
                    "multiply" => body.push(Instruction::I64Mul),
                    "divide"   => body.push(Instruction::I64DivS),
                    _          => {}
                }
                i += 1;
            }
        }

        Ok(SoulFunc {
            name:    format!("poet_{}", input_hash(input)),
            params:  Vec::new(),
            returns: vec![ValType::I64],
            locals:  Vec::new(),
            body,
        })
    }
}

fn parse_number(word: &str) -> Option<i64> {
    match word.trim_end_matches([',', '.', ';', '!', '?']) {
        "one"   => Some(1),
        "two"   => Some(2),
        "three" => Some(3),
        "four"  => Some(4),
        "five"  => Some(5),
        "six"   => Some(6),
        "seven" => Some(7),
        "eight" => Some(8),
        "nine"  => Some(9),
        "ten"   => Some(10),
        w       => w.parse::<i64>().ok(),
    }
}

fn slot_ordinal(words: &[&str]) -> u32 {
    for (i, w) in words.iter().enumerate() {
        if *w == "slot" {
            if let Some(next) = words.get(i + 1) {
                return match next.trim_end_matches([',', '.']) {
                    "first"  | "one"   | "1" => 0,
                    "second" | "two"   | "2" => 1,
                    "third"  | "three" | "3" => 2,
                    "fourth" | "four"  | "4" => 3,
                    s => s.parse::<u32>().unwrap_or(0),
                };
            }
        }
    }
    0
}

#[cfg(test)]
mod tests {
    use super::*;
    use soulvm::Instruction;

    fn compile(src: &str) -> Vec<Instruction> {
        PoetCode.compile(src).unwrap().body
    }

    #[test]
    fn compiles_add_couplet() {
        let src = "Push two push three and add\nSpeak the sum";
        let body = compile(src);
        assert_eq!(body, vec![
            Instruction::I64Const(2),
            Instruction::I64Const(3),
            Instruction::I64Add,
            Instruction::Return,
        ]);
    }

    #[test]
    fn rejects_empty_verse() {
        let err = PoetCode.compile("").unwrap_err();
        assert!(matches!(err, CraftError::EmptyProgram));
    }

    #[test]
    fn guard_line_emits_cap_check() {
        let body = compile("Guard the gate at slot the first");
        assert_eq!(body, vec![Instruction::CapCheck { cap_slot: 0, rights: 0x01 }]);
    }
}
