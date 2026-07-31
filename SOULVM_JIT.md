# SoulVM JIT Integration

**Status:** Phase 3D Architecture (Ready for Implementation)  
**Date:** 2026-07-30  
**Target:** Runtime bytecode verification + native code generation

---

## Overview

**What:** Link Lean/Coq formal proofs directly to runtime bytecode generation.  
**Why:** Every EmojiScript program verification → Cranelift native code (proven correct).  
**How:** 3-stage pipeline: Lean theorems → proof certificates → JIT code generation.

---

## Architecture

### Stage 1: Proof Extraction (Lean → Certificates)

**Input:** `Skclisp.Equivalence.lean` (proven theorems)

**Output:** Proof certificates (binary format):
```
Certificate struct:
  - theorem_id: u32                    (T01-T11 proof ID)
  - proof_hash: [u8; 32]              (Blake3 of proof)
  - theorems_covered: u32             (bitmask: T01-T11)
  - machine_state_invariants: u32     (validity checks)
  - cranelift_backend: String         ("x86_64" | "aarch64")
  - optimization_level: u8            (0=none, 1=basic, 2=aggressive)
```

### Stage 2: Proof Certificate Validation

**Input:** Proof certificate + EmojiScript bytecode

**Validation checks:**
1. Blake3(certificate) matches on-chain WORM record ✓
2. Theorem ID covers the bytecode operations ✓
3. Machine state invariants apply to execution context ✓
4. Cranelift backend available for target architecture ✓

**Output:** Validated proof token (allows JIT codegen)

### Stage 3: JIT Code Generation (Bytecode → Native)

**Input:** EmojiScript bytecode + proof token

**Cranelift pipeline:**
```
EmojiScript bytecode
  ↓ [parse emoji→SigilOp]
Intermediate representation (IR)
  ↓ [apply proof invariants]
Verified IR (with theorem bounds)
  ↓ [Cranelift backend]
Machine code (x64 or ARM64)
  ↓ [seal with Ed25519]
Signed native code block
  ↓ [memory map + execute]
Runtime execution (proven correct)
```

---

## Integration Points

### 1. EmojiScript Compiler Hook

```clojure
(defn compile-with-proof [source proof-certificate]
  "Compile EmojiScript to native code using proof certificate"
  (let [bytecode (emoji/compile-emojiscript source)
        validity (validate-proof-cert proof-certificate bytecode)]
    (if (:valid? validity)
      (jit-compile-cranelift bytecode proof-certificate)
      (throw (ex-info "Proof does not cover bytecode" validity)))))
```

### 2. MCP Tooling

New MCP tool: `compile_with_proof`

```json
{
  "name": "compile_with_proof",
  "description": "Compile EmojiScript to native code with formal proof",
  "parameters": {
    "source": "EmojiScript source code",
    "proof_certificate": "Blake3 + Ed25519 signed proof (base64)",
    "target": "x86_64 or aarch64"
  },
  "returns": {
    "native_code": "Cranelift-generated machine code (base64)",
    "proof_id": "Lean theorem ID used (T01-T11)",
    "performance_estimate": "estimated execution time (ns)"
  }
}
```

### 3. WORM Ledger Sealing

Every JIT compilation requires WORM entry:

```
Event struct:
  - timestamp: u64
  - source_hash: Blake3(bytecode)
  - proof_id: u32
  - native_code_hash: Blake3(machine code)
  - actor_signature: Ed25519
  - status: "compiled" | "verified" | "executed"
```

---

## Proof Invariants Enforced by JIT

### T01: Step Determinism
- Native code generation is deterministic
- Same bytecode → identical machine code
- Verified: Blake3 hash before/after compilation must match

### T04: State Preservation
- Every native instruction preserves machine state validity
- Stack bounds checked at entry + exit
- Heap invariants maintained throughout execution

### T08: Mutation Journal Completeness
- Every mutation recorded to WORM before commit
- Generation counter verified at boundary
- Rollback points marked in machine code

### T10: Rollback Soundness
- Native code includes rollback jump targets
- State snapshots at generation boundaries
- Recovery code verified by proof certificate

---

## Implementation Roadmap

### Phase 3D-1: Proof Certificate Format (2h)

- Define binary proof certificate struct (20 lines)
- Blake3 + Ed25519 signing/verification (30 lines)
- Serialization → base64 for MCP transport (15 lines)

**Artifacts:**
- `Skclisp/ProofCertificate.lean` (65 lines)
- `skclisp_proof_certificate.ts` (Node.js bindings, 40 lines)

### Phase 3D-2: Cranelift Backend Wiring (4h)

- Cranelift dependency integration (10 lines package.json)
- EmojiScript → Cranelift IR translation (80 lines)
- x86_64 + aarch64 target support (50 lines)

**Artifacts:**
- `native/cranelift-backend.rs` (150 lines)
- `src/snapkitty/lisp/jit.cljs` (120 lines)

### Phase 3D-3: Proof Invariant Enforcement (3h)

- T01-T04 checks compiled into native code (40 lines)
- T08-T10 mutation/rollback instrumentation (30 lines)
- Performance profiling + optimization (20 lines)

**Artifacts:**
- `native/jit-verification.rs` (90 lines)
- Test suite + benchmarks (50 lines)

### Phase 3D-4: WORM Integration (2h)

- Ledger entry generation after compilation (25 lines)
- Proof certificate → WORM link (15 lines)
- Rollback coordination (20 lines)

**Artifacts:**
- `src/snapkitty/lisp/jit-ledger.cljs` (60 lines)
- Integration tests (30 lines)

**Total Phase 3D:** ~11 hours, 645 lines code + tests

---

## Security Model

### Proof-of-Correctness Execution

1. **Proof Certificate** → Blake3-signed theorem ID
2. **Bytecode** → Compile to IR
3. **Invariant Check** → Does proof cover all ops?
4. **Code Generation** → Cranelift → native
5. **Sealing** → Ed25519 sign machine code
6. **Execution** → Verified native code only

**Threat model:** If the proof certificate matches the WORM record, we execute knowing:
- Lean theorem verified the operation semantics
- Machine state invariants are enforced
- No mutation occurs without logging
- Rollback points are reachable

### Attack Surface

Protected:
- ✅ Proof substitution (Ed25519 signature)
- ✅ Bytecode modification (Blake3 check before compile)
- ✅ Code injection (proof certificate required for JIT)
- ✅ State corruption (T04 preservation enforced)

Assumptions:
- ❌ Proof certificate is honest (must be from trusted Lean verifier)
- ❌ Cranelift is bug-free (compiler bugs will pass through)
- ❌ Private keys are not compromised (Ed25519 verification only)

---

## Performance Target

| Operation | Baseline | With JIT |
|-----------|----------|----------|
| Compile bytecode | 2.5µs | 15µs (compile phase) |
| Execute (interpreted) | 500ns/op | — |
| Execute (native) | — | 50ns/op (10x faster) |
| Proof verification | — | 1.2µs |
| WORM ledger seal | 2.1µs | 2.1µs (unchanged) |

**Breakeven:** ~300 operations before native execution pays off.

---

## Next Phase (Phase 4: Production Hardening)

- Distributed JIT compilation (load balancing across 200 repos)
- Hardware acceleration profiles (AVX-512, SVE for ARM)
- Proof certificate caching + precompilation
- End-to-end benchmarking (proof→code→execution)

---

**Built by:** Jessica (SnapKittyWest) + Claude Code  
**Theorem Foundation:** Lean 4 M02-M03 (Coq-equivalent)  
**Runtime:** Cranelift IR → x86_64/aarch64 native  
**Security:** Ed25519 proof certificates + Blake3 validation  
**Status:** ARCHITECTURE COMPLETE, READY FOR PHASE 3D-1

