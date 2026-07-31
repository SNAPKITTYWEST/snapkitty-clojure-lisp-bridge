# Browser↔Backend Interlock: Tau Prolog + Clojure + Assembly

**Date:** 2026-07-31  
**Status:** Phase Integration Strategy

---

## The Three-Layer Interlock

### Layer 1: Browser (Tau Prolog - Symbolic)
Prolog runs pure logic, generates bytecode IR

### Layer 2: Backend (Clojure - Compiled)  
Clojure parses bytecode, compiles to native via Cranelift

### Layer 3: Assembly/Bytecode (Interlock Protocol)
Common language both understand

---

## Bytecode Assembly Spec

```
PUSH <value>           — Push value onto stack
LIST-CONS              — Create list cons cell
UNIFY <var> <term>     — Unify variable with term
RETURN <value>         — Return result
SEAL <cid>             — Bifrost seal result
SIGNATURE <ed25519>    — Sign with key
```

---

## Integration into 9-Phase Build

Each phase:
- Browser: emit/consume bytecode (Tau Prolog)
- Backend: parse/compile/execute (Clojure→Cranelift)
- Proof: bytecode execution trace (sealed to WORM)

---

## Implementation Tasks

### Task 1: Tau Prolog Bytecode Emitter
Browser-side Tau Prolog → bytecode IR

### Task 2: Clojure Bytecode Parser  
Backend-side bytecode IR → JVM execution

### Task 3: Cranelift JIT
JVM→native x86-64 compilation

### Task 4: Browser Verification
Verify result matches Tau Prolog query

---

## Concrete Example

Browser: `?- append([1,2], [3], Z).`  
Browser emits bytecode → Backend compiles → Backend executes → Result sealed  
Browser verifies proof → Display Z = [1,2,3]

---

**Status:** Ready to implement  
**Estimated time:** 7 hours to production
