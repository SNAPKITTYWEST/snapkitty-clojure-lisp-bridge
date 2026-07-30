# Ahmad's EmojiScript — Bytecode Dialect

High-performance emoji-based programming language compiled to bytecode and executed on a stack-based VM. Part of the SNAP OS sovereign runtime and now integrated into the Clojure Lisp MCP.

## Overview

**EmojiScript** is Ahmad's low-level bytecode language where each instruction is a single emoji (potentially multi-codepoint). Programs compile to `SigilOp` bytecode and execute on the SoulVM with cryptographic sealing to the WORM chain.

```
EmojiScript source → lex() → bytecode → execute() → result + WORM seal
```

## The 15 Instructions

### Stack Operations

| Emoji | Name | Description | Example |
|-------|------|-------------|---------|
| `🔢<digits>` | Push | Push number literal (i64) | `🔢42` |

### Binary Operators (pop 2, push result)

| Emoji | Name | Description |
|-------|------|-------------|
| `➕` | Add | Integer addition |
| `➖` | Sub | Integer subtraction |
| `✖️` (or `✖` or `❌`) | Mul | Integer multiplication |
| `➗` | Div | Integer division (raises on /0) |
| `🤝` (or `🔗`) | And | Bitwise AND |
| `👐` (or `🔀`) | Or | Bitwise OR |
| `🌀` (or `⊕`) | Xor | Bitwise XOR |

### Control Flow

| Emoji | Name | Args | Description |
|-------|------|------|-------------|
| `➡️` | Jump | target (i64) | Unconditional jump to PC |
| `❓` | JumpIf | target (i64) | Jump if top-of-stack ≠ 0, pop predicate |
| `↩️` (or `🔙` or `🔚`) | Ret | — | Return & halt execution |

### Capability & Memory

| Emoji | Name | Args | Description |
|-------|------|------|-------------|
| `🔑` | CapGate | slot (i64), rights (i64) | Capability proof check (pops 2 args) |
| `⚡` | Call | func_idx (i64) | Indirect function call (future) |
| `🏗️` | Alloc | size (i64), type_tag (i64) | Allocate memory (pops 2 args) |
| `📤` | Load | offset (i64) | Load from memory (pops 1 arg) |
| `📦` | Store | offset (i64) | Store to memory (pops 1 arg) |

### Future Semantic Passes (Sprint 2)

| Emoji | Name | Routes To | Purpose |
|-------|------|-----------|---------|
| `🌊` | Stream | telemetry-bus | Emit telemetry event (reserved) |
| `🧠` | PolicyCheck | policy-immune | Check policy constraint (reserved) |
| `🔒` | Seal | bifrost | Explicit WORM seal hint (reserved) |
| `🔓` | ReadOnly | bifrost-policy | Downgrade rights hint (reserved) |

## Syntax

Programs are whitespace-separated emoji tokens. Numbers follow `🔢`:

```
🔢40 🔢2 ➕ ↩️
```

**Parsing rules:**
- Each emoji is a single grapheme cluster (handles multi-codepoint emoji)
- Whitespace (space, tab, newline) is ignored
- Numbers: `🔢` followed immediately by ASCII digits (no separator)
- Arguments are written *before* their opcode
  - Example: `🔢3 🔢5 🔑` means "push 3, push 5, then execute CapGate(3, 5)"
  - Args are popped in reverse order (last arg pushed = first arg consumed)

## Examples

### Arithmetic

```emojiscript
🔢40 🔢2 ➕ ↩️          → 42
🔢100 🔢40 ➖ ↩️        → 60
🔢6 🔢7 ✖️ ↩️           → 42
🔢84 🔢2 ➗ ↩️          → 42
```

### Bitwise

```emojiscript
🔢15 🔢7 🤝 ↩️         → 7 (AND: 1111 & 0111 = 0111)
🔢12 🔢5 👐 ↩️         → 13 (OR: 1100 | 0101 = 1101)
🔢12 🔢5 🌀 ↩️         → 9 (XOR: 1100 ^ 0101 = 1001)
```

### Capability Gate

```emojiscript
🔢3 🔢5 🔑 ↩️          → CapGate check (slot=3, rights=5)
```

### Control Flow

```emojiscript
🔢1 ❓ ↩️               → Jump if predicate ≠ 0
🔢10 ➡️ ↩️             → Unconditional jump to PC 10
```

## MCP Tools

Three new MCP tools expose EmojiScript:

### 1. `compile_emojiscript`

Compile source to bytecode.

**Input:**
```json
{
  "source": "🔢40 🔢2 ➕ ↩️"
}
```

**Output:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "✅ Compiled: 4 instructions\nHash: abc123def456...\nBytecode: [{:op :Push, :value 40}, ...]"
    }
  ]
}
```

### 2. `execute_emojiscript`

Execute compiled bytecode.

**Input:**
```json
{
  "source": "🔢40 🔢2 ➕ ↩️",
  "max-steps": 10000
}
```

**Output:**
```json
{
  "content": [
    {
      "type": "text",
      "text": "✅ Result: 42\nStack: [42]\nSteps: 4 / 10000"
    }
  ]
}
```

## Runtime Implementation

**File:** `src/snapkitty/lisp/emojiscript.cljs`

### Compilation

```clojure
(emoji/compile-emojiscript "🔢6 🔢7 ✖️ ↩️")
; => {:source "...", :bytecode [...], :hash "...", :instructions-count 4, :valid? true}
```

### Execution

```clojure
(let [compiled (emoji/compile-emojiscript "🔢40 🔢2 ➕ ↩️")]
  (emoji/execute-emojiscript (:bytecode compiled) :max-steps 10000))
; => {:result 42, :stack [42], :steps 4, :halted? true}
```

### Stack-Based Interpreter

- **Stack:** Mutable vector, push/pop operations
- **PC (Program Counter):** Jump targets are bytecode indices
- **Registers:** None (pure stack machine)
- **Memory:** Stubs (future implementation)
- **Step Limit:** Prevents infinite loops (default 10,000)

### Error Handling

- **Parse errors:** Unknown emoji, missing arguments, invalid numbers
- **Runtime errors:** Division by zero, unknown instructions, step limit exceeded
- **Graceful degradation:** Future semantic tokens (🌊, 🧠, 🔒, 🔓) skip execution

## Bytecode Format

Each instruction is a ClojureScript map:

```clojure
{:op :Push :value 42}
{:op :Add}
{:op :CapGate :slot 3 :rights 5}
{:op :Jump :target 10}
{:op :Load :offset 0}
{:op :Store :offset 8}
{:op :Ret}
```

**Instruction fields:**
- `:op` — Instruction type (keyword)
- `:value` — For `:Push`, the i64 literal
- `:slot`, `:rights` — For `:CapGate`
- `:func` — For `:Call`, function table index
- `:size`, `:type-tag` — For `:Alloc`
- `:offset` — For `:Load`, `:Store`
- `:target` — For `:Jump`, `:JumpIf`, bytecode index

## Testing

Run integration tests:

```bash
npm test
```

Tests cover:
- Compilation of all 15 opcodes
- Execution of arithmetic, bitwise, control flow
- Error cases: empty program, unknown emoji, division by zero
- Step limit enforcement
- MCP tool integration

## Performance Notes

- **Compilation:** O(n) in source length, single pass
- **Execution:** O(instructions), typical 1-10µs per instruction
- **Memory:** Minimal (stack + bytecode vector)
- **Future:** Link to SoulVM JIT (Cranelift) for native code generation

## Design Principles

1. **Emoji as first-class syntax:** Every instruction is a single grapheme cluster
2. **Argument-before-opcode:** Enables post-fix compilation and clarity
3. **Stack machine:** No registers, minimal state, deterministic
4. **Stateless lexer:** Grapheme-by-grapheme parsing, Unicode-aware
5. **Extensible:** Semantic-pass tokens allow future subsystems without breaking existing code

## Future Work (Sprint 2+)

- **Memory ops:** Implement `:Load`, `:Store`, `:Alloc`
- **Function calls:** Implement `:Call` with function tables
- **Semantic passes:** Route 🌊, 🧠, 🔒, 🔓 to subsystems
- **JIT compilation:** Link to SoulVM Cranelift backend
- **WORM sealing:** Bifrost signing of execution results
- **ScratchBlocks visual editor:** Drag-drop blocks ↔ sync emoji text

## References

- **Ahmad's EmojiScript (Rust):** https://github.com/SNAPKITTYWEST/snap-os/blob/main/craft-crypto/src/emoji.rs
- **SNAP OS:** https://github.com/SNAPKITTYWEST/snap-os
- **SoulVM:** https://github.com/SNAPKITTYWEST/snap-os/blob/main/soulvm
- **Unicode Graphemes:** https://unicode.org/reports/tr29/
