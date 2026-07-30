# SNAPKITTY CLOJURE LISP BRIDGE — MASTER BUILD SPECIFICATION

**Protocol:** SKC-LISP-WORLD-COQ-001  
**Author:** Ahmad Ali Parr (SnapKitty Collective)  
**Integration:** 5-Phase Auto-Addressed Build Plan  
**Target Model:** Claude Opus (Sonnet)  
**Assurance Policy:** Evidence or Silence  
**Execution Model:** Non-recursive explicit-state machine  
**Proof Target:** Coq (kernel-checked)

---

## MISSION SUMMARY

Construct a unified Lisp world-image system that:

1. **Freezes** complete executable state into deterministic world dumps
2. **Serializes** using stable object identifiers (no raw pointers)
3. **Restores** through defensive parsing + full validation
4. **Executes** via non-recursive explicit-state machine (no host call stack)
5. **Self-modifies** through typed, journaled mutation operations
6. **Mirrors** complete semantics in Coq for machine-checked proofs
7. **Proves** 20 critical theorems (determinism, preservation, round-trip equivalence)
8. **Eliminates** all Admitted/admit gaps — every theorem kernel-checked

---

## 5-PHASE INTEGRATION MAP

The 13-step execution plan (from XML protocol) is compressed into 5 phases with auto-addressing:

```
PHASE 1: Inventory & Freeze         (PH1.S1 - PH1.S5)
    ↓
PHASE 2: Explicit Machine & Mutation (PH2.S1 - PH2.S5)
    ↓
PHASE 3: Dump/Restore & Validation   (PH3.S1 - PH3.S5)
    ↓
PHASE 4: Coq Formalization & Proofs  (PH4.S1 - PH4.S5)
    ↓
PHASE 5: Audit & Release            (PH5.S1 - PH5.S5)
```

---

# PHASE 1: INVENTORY & SEMANTIC FREEZE

**Phase ID:** `PH1`  
**Duration:** ~4 hours  
**Objective:** Inspect live runtime, extract world model, write formal spec before changing code  
**Key Constraint:** R2 — Every step is an explicit transition `step : MachineState → StepResult`

## PH1.S1 — Runtime World Archaeology

**Address:** `PH1.S1`  
**Task:** Reconstruct complete Lisp runtime object graph

**Deliverables:**
- `PH1_WORLD_MANIFEST.edn` — Complete object-kind inventory
- `PH1_OBJECT_GRAPH.dot` — Heap topology (symbols, packages, closures, code)
- `PH1_DYNAMIC_STATE.txt` — Machine state snapshot (stack, frames, PC, handlers)
- `PH1_PACKAGE_MAP.json` — Symbol table + package namespaces

**Verification:**
- [ ] All 25 object kinds (nil, boolean, integer, cons, closure, macro, code-object, etc.) identified
- [ ] Symbol table exhaustively enumerated
- [ ] Closure environments traced to bindings
- [ ] Code objects located and fingerprinted
- [ ] Continuation/frame structure documented

**Non-Negotiable Rule:** R1 — Do not implement recursive evaluator. Only inventory existing state.

---

## PH1.S2 — Formal World State Schema

**Address:** `PH1.S2`  
**Task:** Write operational specification BEFORE changing implementation

**Deliverables:**
- `PH1_WORLD_SCHEMA.v` — Coq Record definition for World type
- `PH1_WORLD_INVARIANTS.txt` — Declared structural invariants
- `PH1_MACHINE_STATE_SCHEMA.v` — Coq Record for MachineState
- `PH1_ASSUMPTIONS.md` — External boundaries + capabilities list

**Core Coq Structure:**
```coq
Record world : Type := {
  format_version      : Version;
  generation          : nat;
  symbol_table        : SymbolTable;
  package_table       : PackageTable;
  object_store        : ObjectId → option Object;
  root_set            : list ObjectId;
  global_environment  : ObjectId;
  code_registry       : CodeId → CodeObject;
  machine_state       : MachineState;
  mutation_journal    : list MutationEvent;
  capability_registry : CapabilityId → CapabilityDescriptor;
  metadata            : WorldMetadata;
  checksum            : Digest;
}
```

**Verification:**
- [ ] Schema compiles under pinned Coq toolchain
- [ ] All object kinds have type definitions
- [ ] Stable reference model (ObjectId, not pointers)
- [ ] Mutation journal structure defined
- [ ] No recursive function definitions (only records + inductives)

**Non-Negotiable Rule:** R4 — Do not serialize raw pointers. Use ObjectId.

---

## PH1.S3 — Instruction Set & Step Semantics

**Address:** `PH1.S3`  
**Task:** Define the 30+ instruction set and single-step transition function

**Deliverables:**
- `PH1_INSTRUCTION_SET.v` — Coq Inductive type for all instructions
- `PH1_STEP_RELATION.v` — Relational semantics `step : MachineState → StepResult → Prop`
- `PH1_STEP_FUNCTION.v` — Executable `step_fn : MachineState → StepResult`
- `PH1_INSTRUCTION_DOCS.md` — Human-readable semantics for all 30 instructions

**Instruction Categories:**
- Constant load: CONST
- Variable lookup: LOOKUP, BIND
- Stack operations: PUSH, POP
- List operations: CONS, CAR, CDR, SET_CAR, SET_CDR
- Control flow: CALL, TAIL_CALL, RETURN, JUMP, JUMP_IF_FALSE
- Frame management: PUSH_FRAME, POP_FRAME
- Continuation: CAPTURE_CONTINUATION, RESTORE_CONTINUATION
- Exception: RAISE, INSTALL_HANDLER, REMOVE_HANDLER
- Effects: REQUEST_EFFECT
- Mutation: PATCH_CODE, DEFINE_CODE, REPLACE_FUNCTION, REWRITE_DISPATCH
- Generation: COMMIT_GENERATION, ROLLBACK_GENERATION
- Halt: HALT

**Verification:**
- [ ] 30+ instructions defined as Coq Inductive
- [ ] StepResult type includes: Stepped, Emitted, Requested, HaltedWith, TrappedWith
- [ ] Relational step relation has 30+ cases (one per instruction)
- [ ] Executable step_fn type-checks
- [ ] No implicit recursion on call stack

**Non-Negotiable Rule:** R2 — Every step is explicit. R3 — Calls use explicit continuation stack.

---

## PH1.S4 — Execution Driver Architecture

**Address:** `PH1.S4`  
**Task:** Design the non-recursive execution loop

**Deliverables:**
- `PH1_EXECUTION_DRIVER.v` — Coq model of execution loop
- `PH1_FUEL_BOUNDED_RUNNER.v` — Finite-fuel execution (for proof)
- `PH1_RUNTIME_EXECUTOR.cljs` — Actual executable loop (ClojureScript)
- `PH1_DRIVER_TESTS.edn` — Test vectors for driver

**Driver Pattern:**
```
state := initial_state

while state.status = Running:
    result := step(state)
    match result:
        Stepped next: state := next
        Emitted observable next: record(observable); state := next
        Requested request next: 
            response := capability_boundary(request)
            state := resume(next, response)
        HaltedWith value final: return value
        TrappedWith error final: return error
```

**Verification:**
- [ ] No call stack dependency
- [ ] Explicit frame stack in MachineState
- [ ] Capability boundary function isolated
- [ ] Driver loop terminates only on Halted or Trapped
- [ ] Observable events logged deterministically

**Non-Negotiable Rule:** R9 — Proof may use fuel/recursion, but runtime must not.

---

## PH1.S5 — External Capability Boundary

**Address:** `PH1.S5`  
**Task:** Define external resource handling (files, sockets, clocks, devices)

**Deliverables:**
- `PH1_CAPABILITY_MODEL.v` — Coq types for 8 capability kinds
- `PH1_RESTORATION_POLICIES.edn` — reject | detach | reopen | snapshot policies
- `PH1_CAPABILITY_REGISTRY.json` — Declared external capabilities
- `PH1_EFFECT_REQUEST.v` — Coq RequestEffect type

**Capability Kinds:**
```
- file              (reject/detach/reopen/snapshot)
- socket           (reject/reopen)
- clock            (detach/reopen)
- random-source    (detach)
- process          (reject)
- thread           (reject)
- device           (reject/snapshot)
- foreign-runtime  (reject/detach)
```

**Verification:**
- [ ] Each capability has explicit restoration policy
- [ ] RequestEffect type allows boundary queries
- [ ] No silent external state captured as pure values
- [ ] Dump format includes capability metadata section

**Non-Negotiable Rule:** R8 — External state is not silently frozen as heap state.

---

# PHASE 2: EXPLICIT MACHINE & MUTATION

**Phase ID:** `PH2`  
**Duration:** ~5 hours  
**Objective:** Replace recursive evaluator with explicit machine; implement controlled mutations  
**Key Constraint:** R5 — Every mutation produces typed event

## PH2.S1 — Machine State Conversion

**Address:** `PH2.S1`  
**Task:** Replace recursive call-stack evaluation with explicit state machine

**Deliverables:**
- `PH2_EXPLICIT_MACHINE.cljs` — Runtime machine implementation
- `PH2_FRAME_STACK.cljs` — Explicit frame representation (replaces call stack)
- `PH2_CONTINUATION_STORE.cljs` — Continuation capture/restore
- `PH2_STATE_INVARIANTS.edn` — Declared machine state invariants
- `PH2_MACHINE_TESTS.edn` — 30 test vectors (one per instruction)

**Frame Structure:**
```clojure
{:frame-id frame-id
 :return-address code-id
 :return-index instruction-offset
 :environment environment-id
 :saved-locals {symbol value ...}
 :saved-handlers [handler ...]
 :timestamp generation}
```

**Verification:**
- [ ] All recursive calls use explicit frame stack
- [ ] Frame lifecycle: PUSH_FRAME → update state → POP_FRAME
- [ ] Return address is code-id + offset (not host pointer)
- [ ] Tail calls reuse frame (no stack growth)
- [ ] Frame underflow detected before pop

**Non-Negotiable Rule:** R3 — Calls use explicit continuation stack.

---

## PH2.S2 — Mutation Event Model

**Address:** `PH2.S2`  
**Task:** Implement typed, validated, journaled mutations

**Deliverables:**
- `PH2_MUTATION_EVENT.v` — Coq MutationEvent Record
- `PH2_MUTATION_VALIDATOR.cljs` — Precondition checker
- `PH2_MUTATION_JOURNAL.cljs` — Append-only mutation log
- `PH2_GENERATION_MANAGER.cljs` — World generation tracking
- `PH2_MUTATION_TESTS.edn` — Mutation validation test vectors

**MutationEvent Schema:**
```coq
Record mutation_event : Type := {
  mutation_id         : MutationId;
  generation_before   : nat;
  generation_after    : nat;
  actor               : ActorId;
  target              : MutationTarget;
  operation           : MutationOperation;
  old_digest          : Digest;
  new_digest          : Digest;
  precondition        : PredicateId;
  proof_receipt       : option ProofReceipt;
  timestamp_policy    : LogicalClock;
}
```

**Allowed Mutations:**
1. AllocateObject — Create new heap object
2. ReplaceObject — Swap object reference
3. UpdateBinding — Change symbol binding
4. PatchCodeRange — Replace instruction sequence
5. InstallCodeObject — Register new code
6. ReplaceFunctionCell — Redirect function
7. RewriteDispatchEntry — Update method table
8. InstallMacro — Define macro
9. RemoveBinding — Delete binding
10. CommitWorldGeneration — Publish mutation batch
11. RollbackWorldGeneration — Undo to prior generation

**Mutation Gate (all must pass):**
1. Target exists or allocation explicit
2. Expected old digest matches current
3. Replacement object is well-formed
4. All referenced objects exist
5. Executable code passes validation
6. Protected invariants remain true
7. Mutation event appended before publication
8. Generation incremented on commit

**Verification:**
- [ ] Failed mutations leave world unchanged (atomicity)
- [ ] Every mutation creates journal entry
- [ ] Generation strictly increasing
- [ ] Old/new digests computed correctly
- [ ] No silent mutation

**Non-Negotiable Rule:** R5 — Every mutation produces typed event. R9 — R11 mutations only.

---

## PH2.S3 — Code Patching & Self-Modification

**Address:** `PH2.S3`  
**Task:** Implement safe code-object mutation (PATCH_CODE, REPLACE_FUNCTION, REWRITE_DISPATCH)

**Deliverables:**
- `PH2_CODE_VALIDATOR.v` — Coq validator for instruction sequences
- `PH2_PATCH_ENGINE.cljs` — Code patch application logic
- `PH2_DISPATCH_TABLE.cljs` — Mutable dispatch table
- `PH2_FUNCTION_CELL.cljs` — Symbol function slot replacement
- `PH2_PATCH_TESTS.edn` — 20 test vectors (valid + invalid patches)

**Patch Validation:**
- All opcodes are defined
- All jump targets within range
- All references resolvable
- No segment overlap
- Patch region is aligned
- No mid-instruction boundaries crossed

**Verification:**
- [ ] Invalid patches rejected before journal entry
- [ ] Valid patch succeeds atomically
- [ ] Code object well-formedness preserved
- [ ] Rollback restores prior code
- [ ] Executing patched code works correctly

---

## PH2.S4 — Mutation Replay & Rollback

**Address:** `PH2.S4`  
**Task:** Implement deterministic replay and transactional rollback

**Deliverables:**
- `PH2_REPLAY_ENGINE.v` — Coq replay semantics
- `PH2_ROLLBACK_VALIDATOR.cljs` — Rollback precondition checker
- `PH2_GENERATION_STORE.cljs` — Checkpoint generation storage
- `PH2_REPLAY_TESTS.edn` — Deterministic replay test vectors
- `PH2_ROLLBACK_TESTS.edn` — Rollback under various failures

**Replay Property:**
```coq
Theorem ReplayCorrectness :
  forall base_world mutations,
    let world1 := apply_mutations base_world mutations in
    let replayed := replay_mutations base_world mutations in
    canonical_world(world1) = canonical_world(replayed).
```

**Rollback Property:**
```coq
Theorem RollbackCorrectness :
  forall world gen_before,
    valid_rollback_target(world, gen_before) →
    let rolled_back := rollback_to_generation(world, gen_before) in
    observable_world(rolled_back) ≃ observable_world_at(gen_before).
```

**Verification:**
- [ ] Replay deterministic (same input → same output)
- [ ] Rollback atomicity (all-or-nothing)
- [ ] Generation consistency preserved
- [ ] Journal replay + rollback inverse operations
- [ ] No partial rollback state visible

---

## PH2.S5 — Machine & Mutation Integration Tests

**Address:** `PH2.S5`  
**Task:** End-to-end tests for explicit machine + mutations

**Deliverables:**
- `PH2_INTEGRATION_TESTS.edn` — 50 integration test vectors
- `PH2_TEST_RESULTS.txt` — Pass/fail report
- `PH2_PERFORMANCE_PROFILE.json` — Timing profile (step time, mutation cost)
- `PH2_INVARIANT_CHECK.log` — Automated invariant validation

**Test Scenarios:**
1. Simple arithmetic without mutation
2. Function call → return on explicit stack
3. Tail call (reuses frame)
4. Code patch + execution
5. Function redirect + call
6. Mutation journal recording
7. Rollback + replay
8. Nested frame stacks
9. Exception capture + handler
10. Continuation capture → restoration

**Verification:**
- [ ] All 50 tests pass
- [ ] Step time < 1μs per instruction (benchmark)
- [ ] Machine invariants hold after every step
- [ ] No implicit state mutations
- [ ] Deterministic results on repeated execution

---

# PHASE 3: DUMP / RESTORE & VALIDATION

**Phase ID:** `PH3`  
**Duration:** ~5 hours  
**Objective:** Build canonical world serializer and defensive restore parser  
**Key Constraint:** R4 — No raw pointers. R6 — Dump ≠ source tree. R7 — Prove equivalence separately.

## PH3.S1 — Canonical Dump Format Design

**Address:** `PH3.S1`  
**Task:** Define deterministic world serialization with stable identifiers

**Deliverables:**
- `PH3_DUMP_FORMAT.md` — Formal dump specification (10 sections)
- `PH3_CANONICALIZATION_RULES.v` — Coq canonicalization predicates
- `PH3_BYTE_LAYOUT.bin` — Binary format specification
- `PH3_FORMAT_REFERENCE.md` — Human-readable reference

**Dump Format (10 Sections):**

1. **Header** (64 bytes)
   - magic (4 bytes): "LISP"
   - format_version (2 bytes): 0x0100
   - endianness_marker (2 bytes): 0x1234
   - word_size_policy (1 byte)
   - character_encoding (1 byte): UTF-8
   - world_generation (8 bytes)
   - root_count (4 bytes)
   - object_count (4 bytes)
   - code_object_count (4 bytes)
   - mutation_count (4 bytes)
   - payload_length (8 bytes)
   - payload_digest (32 bytes): SHA-256

2. **Symbol & Package Registry**
   - Sorted by canonical string representation
   - No duplicate symbols
   - Package prefix hierarchy

3. **Object Table**
   - Sorted by ObjectId
   - One record per object
   - All references use ObjectId (not pointers)

4. **Environment Graph**
   - Environment binding records
   - Lexical scope chain
   - Dynamic binding cells

5. **Code Object Registry**
   - Code fingerprints
   - Instruction sequences
   - Relocation information

6. **Continuation & Frame State**
   - Continuation stack
   - Frame records (explicit)
   - Return addresses as (code-id, offset)

7. **Machine State Snapshot**
   - Control state
   - Value stack
   - Program counter
   - Dynamic context

8. **Mutation Journal**
   - All committed mutations
   - In chronological order
   - With generation markers

9. **Capability Descriptors**
   - 8 capability kinds
   - Restoration policies
   - Metadata per capability

10. **Root Set & Integrity Trailer**
    - Root ObjectIds
    - Section checksums
    - Final digest

**Canonicalization Rules (Required):**
- Fixed byte order (big-endian)
- Canonical integer encoding (two's complement)
- Unicode NFC normalization on strings
- Sort map entries by canonical serialized key
- Sort object records by ObjectId
- No allocation addresses
- No nondeterministic timestamps
- No uninitialized padding
- Reject duplicate ObjectIds
- Reject dangling references

**Verification:**
- [ ] Format spec compiles as Coq Record
- [ ] All 10 sections defined with fixed offsets
- [ ] No recursive references in header
- [ ] Endianness marker matches declared order
- [ ] Payload digest matches computed SHA-256
- [ ] All ObjectIds within bounds

**Non-Negotiable Rule:** R4 — No raw pointers. R6 — Dump must include runtime state, not just source.

---

## PH3.S2 — Dump Serializer Implementation

**Address:** `PH3.S2`  
**Task:** Implement deterministic dump function

**Deliverables:**
- `PH3_SERIALIZER.v` — Coq dump semantics `dump : World → ByteString`
- `PH3_SERIALIZER.cljs` — ClojureScript implementation
- `PH3_DUMP_TESTS.edn` — 30 test vectors (empty world, cyclic graphs, large worlds)
- `PH3_DETERMINISM_CHECK.edn` — Verify repeated dumps match

**Serializer Invariants:**
```coq
Theorem DumpDeterminism :
  forall w1 w2,
    canonical_world(w1) = canonical_world(w2) →
    dump(w1) = dump(w2).
```

**Verification:**
- [ ] Dump function total (always terminates)
- [ ] Repeated dumps of same world produce identical bytes
- [ ] No randomness or nondeterministic ordering
- [ ] Payload digest matches computed hash
- [ ] Format version written correctly
- [ ] All ObjectIds included exactly once

---

## PH3.S3 — Defensive Restore Parser

**Address:** `PH3.S3`  
**Task:** Implement defensive restore with complete validation

**Deliverables:**
- `PH3_RESTORE.v` — Coq restore semantics `restore : ByteString → Result World Error`
- `PH3_RESTORE.cljs` — ClojureScript parser
- `PH3_VALIDATE.cljs` — Multi-stage validation gates
- `PH3_RESTORE_TESTS.edn` — 40 test vectors (valid + corrupted dumps)

**Restoration Stages:**

1. **Parse untrusted bytes**
   - Read magic, version, endianness
   - Verify format compatibility
   - Read payload length
   - Allocate intermediate structure (no world yet)

2. **Validate structure**
   - Check all section lengths
   - Verify object count matches table size
   - Verify root set ObjectIds within range
   - Check reference validity

3. **Validate content**
   - Verify code objects (instruction validation)
   - Check environment chain
   - Verify continuation stack frames
   - Validate machine state fields

4. **Verify integrity**
   - Recompute payload digest
   - Compare against header digest
   - Reject on mismatch

5. **Publish world**
   - Only after all 4 stages pass
   - World never partially constructed

**Error Cases (Must Reject):**
- [ ] Invalid magic or version
- [ ] Corrupt length fields (integer overflow)
- [ ] Duplicate object identifiers
- [ ] Dangling references
- [ ] Invalid opcode sequences
- [ ] Digest mismatch
- [ ] Unknown capability kinds
- [ ] Misaligned sections

**Verification:**
- [ ] Malformed bytes never produce world
- [ ] All errors caught before world construction
- [ ] Payload digest verified
- [ ] ReferenceIntegrity preserved
- [ ] No partial restore visible

**Non-Negotiable Rule:** R7 — Prove equivalence separately (not just byte matching).

---

## PH3.S4 — Round-Trip Preservation Proofs

**Address:** `PH3.S4`  
**Task:** Prove dump/restore symmetry

**Deliverables:**
- `PH3_STRUCTURAL_ROUNDTRIP.v` — Coq proof of structure preservation
- `PH3_OBSERVATIONAL_ROUNDTRIP.v` — Coq proof of behavior equivalence
- `PH3_ROUNDTRIP_TESTS.edn` — 20 round-trip test vectors
- `PH3_ROUNDTRIP_REPORT.md` — Proof status report

**Structural Round-Trip Theorem:**
```coq
Theorem StructuralRoundTrip :
  forall w,
    well_formed_world(w) ∧ serializable(w) →
    match restore(dump(w)) with
    | Ok w' => canonical_world(w') = canonical_world(w)
    | Err _ => False
    end.
```

**Observational Equivalence Theorem:**
```coq
Theorem ObservationalEquivalence :
  forall w capability_responses,
    well_formed_world(w) →
    let w' := restore_world(dump(w)) in
    observational_behavior(w, capability_responses) =
    observational_behavior(w', capability_responses).
```

**Verification:**
- [ ] Both theorems kernel-checked in Coq
- [ ] No Admitted or admit in proofs
- [ ] Induction principles used explicitly
- [ ] Well-formedness preserved through each step
- [ ] Canonical equivalence proven by cases

---

## PH3.S5 — Determinism Verification

**Address:** `PH3.S5`  
**Task:** Verify dump is deterministic for canonical worlds

**Deliverables:**
- `PH3_DETERMINISM.v` — Coq determinism theorem
- `PH3_CANONICALIZATION.cljs` — Canonicalization function
- `PH3_COLLISION_TEST.edn` — 10 test vectors (should produce identical dumps)
- `PH3_DETERMINISM_REPORT.txt` — Verification report

**Determinism Theorem:**
```coq
Theorem DumpDeterminism :
  forall w1 w2,
    well_formed_world(w1) →
    well_formed_world(w2) →
    canonical_world(w1) = canonical_world(w2) →
    dump(w1) = dump(w2).
```

**Verification:**
- [ ] Repeated dumps of same world match byte-for-byte
- [ ] Different canonical worlds produce different dumps
- [ ] No randomness in serializer
- [ ] No floating-point in hashing
- [ ] All ObjectIds deterministically ordered

---

# PHASE 4: COQ FORMALIZATION & PROOFS

**Phase ID:** `PH4`  
**Duration:** ~6 hours  
**Objective:** Mirror machine in Coq and prove 20 critical theorems  
**Key Constraint:** R12 — All Coq files compile from clean environment

## PH4.S1 — Coq Type Definitions & Inductive Semantics

**Address:** `PH4.S1`  
**Task:** Define all Coq types and relational step semantics

**Deliverables:**
- `coq/World/Identifier.v` — ObjectId, CodeId, etc.
- `coq/World/Value.v` — Value type (nil, boolean, integer, symbol, etc.)
- `coq/World/Object.v` — Object type (cons, closure, macro, code-object)
- `coq/Machine/Instruction.v` — Instruction type (30+ constructors)
- `coq/Machine/State.v` — MachineState Record
- `coq/Machine/StepRelation.v` — Inductive step relation (30+ cases)
- `coq/Mutation/Event.v` — MutationEvent Record

**Coq Compilation:**
```bash
cd coq && coqc -Q . "" World/Identifier.v
cd coq && coqc -Q . "" Machine/StepRelation.v
```

**Verification:**
- [ ] All files compile without warnings
- [ ] No universe inconsistencies
- [ ] All inductives are well-formed
- [ ] Mutual recursion (if used) terminating
- [ ] No circular dependencies

---

## PH4.S2 — Executable Step Function & Correspondence

**Address:** `PH4.S2`  
**Task:** Define executable step_fn and prove soundness/completeness

**Deliverables:**
- `coq/Machine/StepFunction.v` — Executable `step_fn : machine_state → step_result`
- `coq/Proofs/Soundness.v` — Proof that executable matches relational
- `coq/Proofs/Completeness.v` — Proof that relational matches executable
- `PH4_CORRESPONDENCE_TESTS.edn` — Test vectors comparing both

**Correspondence Theorems:**
```coq
Theorem StepFunctionSoundness :
  forall s r,
    step_fn s = r →
    step s r.

Theorem StepFunctionCompleteness :
  forall s r,
    step s r →
    step_fn s = r.
```

**Verification:**
- [ ] Both theorems proven (no Admitted)
- [ ] Induction over instruction types
- [ ] Case split exhaustive
- [ ] Undefined cases provably unreachable
- [ ] No axioms used

---

## PH4.S3 — Core Preservation Theorems

**Address:** `PH4.S3`  
**Task:** Prove fundamental machine properties

**Deliverables:**
- `coq/Proofs/Determinism.v` — Theorem T01: Step determinism
- `coq/Proofs/Preservation.v` — Theorem T04: Well-formedness preservation
- `coq/Proofs/ReferenceIntegrity.v` — Theorem T05: No dangling references
- `coq/Proofs/FrameDiscipline.v` — Theorem T06: Frame invariants
- `coq/Proofs/NoHostStackDependency.v` — Theorem T07: Pure explicit state

**Determinism Theorem (T01):**
```coq
Theorem StepDeterminism :
  forall s r1 r2,
    step s r1 →
    step s r2 →
    r1 = r2.
```

**Preservation Theorem (T04):**
```coq
Theorem WellFormedStatePreservation :
  forall s s',
    well_formed_state(s) →
    step s (Stepped s') ∨ step s (Emitted _ s') →
    well_formed_state(s').
```

**Verification:**
- [ ] All 5 theorems kernel-checked
- [ ] No sorry, admit, or Admitted
- [ ] Induction structure explicit
- [ ] Case analysis exhaustive

---

## PH4.S4 — Mutation & Dump Theorems

**Address:** `PH4.S4`  
**Task:** Prove mutation safety and dump properties

**Deliverables:**
- `coq/Proofs/MutationJournalCompleteness.v` — Theorem T08
- `coq/Proofs/FailedMutationAtomicity.v` — Theorem T09
- `coq/Proofs/GenerationMonotonicity.v` — Theorem T11
- `coq/Proofs/DumpDeterminism.v` — Theorem T12
- `coq/Proofs/RestoreSoundness.v` — Theorem T13

**Mutation Journal Completeness (T08):**
```coq
Theorem MutationJournalCompleteness :
  forall w w',
    step_world w w' →
    mutable_change(w, w') →
    exists e, In e (mutation_journal w').
```

**Atomicity (T09):**
```coq
Theorem FailedMutationAtomicity :
  forall w w_rejected,
    mutation_attempt_fails(w, w_rejected) →
    canonical_world(w) = canonical_world(w_rejected).
```

**Verification:**
- [ ] All 5 theorems proven
- [ ] Mutation transitions traced through journal
- [ ] Generation strictly increasing
- [ ] Dump determinism via canonicalization

---

## PH4.S5 — Round-Trip & Observational Equivalence

**Address:** `PH4.S5`  
**Task:** Prove dump/restore preservation and behavior equivalence

**Deliverables:**
- `coq/Proofs/DumpRestoreRoundTrip.v` — Theorem T14: Structural preservation
- `coq/Proofs/ObservationalEquivalence.v` — Theorem T15: Behavior preservation
- `coq/Proofs/DigestVerification.v` — Theorem T17: Hash integrity
- `coq/Proofs/TraceReplay.v` — Theorem T19: Deterministic replay
- `coq/Proofs/BoundedExecutionAgreement.v` — Theorem T20: Fuel-bounded runner agrees

**Round-Trip (T14):**
```coq
Theorem DumpRestoreStructuralRoundTrip :
  forall w,
    well_formed_world(w) ∧ serializable(w) →
    match restore(dump(w)) with
    | Ok w' => canonical_world(w') = canonical_world(w)
    | Err _ => False
    end.
```

**Observational Equivalence (T15):**
```coq
Theorem DumpRestoreObservationalEquivalence :
  forall w responses,
    well_formed_world(w) →
    let w' := restore_world(dump(w)) in
    traces_under(w, responses) = traces_under(w', responses).
```

**Verification:**
- [ ] All 5 theorems fully proven
- [ ] Structural properties separated from behavioral
- [ ] Digest verification explicit
- [ ] Trace replay deterministic

---

# PHASE 5: AUDIT & RELEASE

**Phase ID:** `PH5`  
**Duration:** ~4 hours  
**Objective:** Verify no proof gaps, audit all claims, publish v1.0.0  
**Key Constraint:** R10, R11 — No hidden axioms. Every gap analyzed.

## PH5.S1 — Coq Gap Audit

**Address:** `PH5.S1`  
**Task:** Find and close all Admitted/admit occurrences

**Deliverables:**
- `PH5_ADMITTED_SEARCH.txt` — Complete grep output (should be empty)
- `coq/Proofs/GapAudit.v` — Audit report for each gap found
- `assurance/SORRY_REPORT.txt` — Final sorry/admit count (target: 0)
- `assurance/AXIOM_REPORT.txt` — All axioms and parameters documented

**Gap Search Pattern:**
```bash
grep -r "Admitted\|admit\|Axiom\|Parameter\|Hypothesis" coq/
```

**Gap Closure Checklist:**
- [ ] Find every sorry, Admitted, admit
- [ ] Classify: definition missing | lemma missing | induction invariant missing | assumption
- [ ] Reduce to smallest failing subgoal
- [ ] Search for finite counterexample
- [ ] Repair definition before adding proof
- [ ] Extract reusable intermediate lemmas
- [ ] Prove through kernel-checked tactics
- [ ] Verify no new gaps introduced

**Axiom Whitelist (Only allowed):**
- Extensionality (if required for function equality)
- Functional extensionality (if required)
- Univalence (if required for universe-level reasoning)
- Classical logic (if required)
- Any axiom must be DOCUMENTED with justification

**Verification:**
- [ ] grep finds 0 Admitted
- [ ] grep finds 0 admit
- [ ] All Parameter/Hypothesis listed in AXIOM_REPORT
- [ ] Each axiom justified in comments
- [ ] No hidden unsafe casts

---

## PH5.S2 — Counterexample Hunting

**Address:** `PH5.S2`  
**Task:** Attempt to falsify every theorem

**Deliverables:**
- `PH5_COUNTEREXAMPLE_SEARCH.md` — Search strategy + results
- `assurance/COUNTEREXAMPLES.md` — Any found (should be empty)
- `PH5_FALSE_THEOREM_REPAIRS.md` — Theorem statement fixes (if any)
- `PH5_HUNT_REPORT.txt` — Search coverage report

**Counterexample Search Targets (from XML):**
- [ ] Dangling object references
- [ ] Duplicate object identifiers
- [ ] Cyclic graphs mishandled by serialization
- [ ] Invalid code patch boundaries
- [ ] Mutation without journal entry
- [ ] Generation reuse
- [ ] Noncanonical map ordering
- [ ] Unicode normalization drift
- [ ] Restore before digest verification
- [ ] Capability identity confusion
- [ ] Stale continuation restoration
- [ ] Frame underflow
- [ ] Program-counter escape
- [ ] Integer overflow in section lengths
- [ ] Silent opcode reinterpretation across format versions

**Verification:**
- [ ] Each target searched exhaustively
- [ ] If counterexample found: theorem repaired (not hidden)
- [ ] If no counterexample: documented why search was thorough
- [ ] Coverage report shows search scope

---

## PH5.S3 — Quality Gate Verification

**Address:** `PH5.S3`  
**Task:** Verify all 10 quality gates pass

**Deliverables:**
- `assurance/QUALITY_GATES.md` — Gate-by-gate verification
- `assurance/GATE_EVIDENCE.txt` — Evidence for each gate
- `PH5_GATE_REPORT.txt` — Pass/fail summary

**Quality Gates (from XML):**

| Gate | Target | Evidence | Status |
|------|--------|----------|--------|
| G1 | No host-recursive evaluator in verified path | Grep: no recursive calls in step_fn | ✓ |
| G2 | Every call/return in explicit machine state | Proof: CALL/RETURN use frame stack | ✓ |
| G3 | Every committed mutation has journal record | Proof: MutationJournalCompleteness | ✓ |
| G4 | No raw process pointers in dump | Inspection: ObjectId only | ✓ |
| G5 | Dump bytes deterministic for canonical worlds | Proof: DumpDeterminism | ✓ |
| G6 | Restore validates all data before execution | Inspection: 5-stage validation | ✓ |
| G7 | Executable runtime & Coq vectors agree | Test vectors: all pass | ✓ |
| G8 | Coq compilation: no Admitted, no admit | Grep: find 0 | ✓ |
| G9 | All axioms in assumptions report | Report: complete list | ✓ |
| G10 | README claims match proof artifacts | Review: no overstating | ✓ |

**Verification:**
- [ ] All 10 gates have evidence
- [ ] No gate marked pass without evidence
- [ ] Evidence is inspectable and reproducible

---

## PH5.S4 — Documentation & Proof Status

**Address:** `PH5.S4`  
**Task:** Complete proof status catalog and produce final documentation

**Deliverables:**
- `assurance/PROOF_STATUS_CATALOG.csv` — All 20 theorems + status
- `ARCHITECTURE.md` — Complete system architecture (final)
- `ASSUMPTIONS.md` — All external boundaries (final)
- `README.md` — Production README (final)
- `REPRODUCIBILITY.md` — How to rebuild from scratch

**Proof Status Catalog Format:**
```csv
claim,formal_statement,source_file,status,dependencies,assumptions,kernel_checked,artifact_digest
T01_StepDeterminism,forall s r1 r2 step s r1 -> step s r2 -> r1 = r2,coq/Proofs/Determinism.v,PROVED,"[T02]","[]",YES,"abc123..."
...
```

**Allowed Status Values:**
- PROVED — Kernel-checked, no Admitted
- DISPROVED — Counterexample found; statement repaired
- CONDITIONAL — Proved under declared assumptions only
- EXTERNAL_ASSUMPTION — Boundary condition; not proved
- IMPLEMENTED_NOT_PROVED — Code exists; proof deferred (with justification)
- OPEN — Actively being worked (should be 0 for release)

**Verification:**
- [ ] All 20 theorems have status row
- [ ] No OPEN theorems for v1.0.0 release
- [ ] No IMPLEMENTED_NOT_PROVED for critical theorems
- [ ] EXTERNAL_ASSUMPTION entries justified
- [ ] Artifact digests match actual proofs

---

## PH5.S5 — Release & Artifact Certification

**Address:** `PH5.S5`  
**Task:** Tag v1.0.0 with complete artifact certification

**Deliverables:**
- `assurance/MACHINE_CHECK_REPORT.md` — Full Coq compilation report
- `assurance/TOOLCHAIN.lock` — Pinned Coq/Rocq versions
- `assurance/ARTIFACT_HASHES.txt` — SHA-256 of all proof files
- Git tag: `v1.0.0-coq-verified`
- GitHub Release with full documentation

**Machine Check Report Contents:**
```
Toolchain:
  Coq: 8.20.0
  Rocq: 0.7.0
  OCaml: 4.14.1

Compilation:
  Files: 47
  Definitions: 312
  Theorems: 20
  Proofs: 20/20 (100%)
  Admitted: 0
  Admitted theorems: 0

Time:
  Total: 156 seconds
  Per file: average 3.3 seconds

Memory:
  Peak: 2.1 GB
```

**Artifact Certification (SHA-256):**
```
coq/World/Identifier.v:              abc123...
coq/World/Value.v:                   def456...
coq/Machine/StepRelation.v:          ghi789...
coq/Proofs/Determinism.v:            jkl012...
... (all 47 files)
```

**Release Notes:**
- Version: 1.0.0-coq-verified
- Date: [execution date]
- 20 theorems proven
- 0 proof gaps
- 33 test vectors (all pass)
- Coq kernel-checked artifacts
- Non-recursive execution verified
- Dump/restore round-trip proven
- 5-phase build completed

**Verification:**
- [ ] All 47 Coq files compile from scratch
- [ ] `coqc -check` on all .vo files passes
- [ ] Hashes match artifact files
- [ ] Toolchain locked and reproducible
- [ ] Release notes accurate
- [ ] No overstated claims

---

## MASTER CHECKLIST (PH1-PH5)

**PH1 — Inventory & Freeze:**
- [ ] PH1.S1: World manifest complete
- [ ] PH1.S2: World schema in Coq
- [ ] PH1.S3: 30 instructions + step semantics
- [ ] PH1.S4: Execution driver architecture
- [ ] PH1.S5: Capability boundary model

**PH2 — Explicit Machine & Mutation:**
- [ ] PH2.S1: Frame stack replaces call stack
- [ ] PH2.S2: Mutation events implemented
- [ ] PH2.S3: Code patching works
- [ ] PH2.S4: Replay + rollback deterministic
- [ ] PH2.S5: 50 integration tests pass

**PH3 — Dump / Restore:**
- [ ] PH3.S1: Dump format canonicalized
- [ ] PH3.S2: Deterministic serializer
- [ ] PH3.S3: Defensive restore parser
- [ ] PH3.S4: Round-trip proofs
- [ ] PH3.S5: Determinism verified

**PH4 — Coq Formalization:**
- [ ] PH4.S1: All types defined in Coq
- [ ] PH4.S2: step_fn correspondence proven
- [ ] PH4.S3: 5 core preservation theorems
- [ ] PH4.S4: 5 mutation theorems
- [ ] PH4.S5: 5 round-trip theorems

**PH5 — Audit & Release:**
- [ ] PH5.S1: 0 Admitted/admit found
- [ ] PH5.S2: Counterexample search complete
- [ ] PH5.S3: All 10 quality gates pass
- [ ] PH5.S4: Proof status catalog complete
- [ ] PH5.S5: v1.0.0-coq-verified released

---

## SONNET INVOCATION

**To feed to Claude Opus (Sonnet):**

1. Save this file as `MASTER_BUILD_SPEC.md`
2. Provide original XML protocol as context
3. Invoke:

```
Model: Claude Opus (claude-opus-5)
Prompt: "Execute PHASE 1 (PH1.S1 through PH1.S5) using the Master Build Spec. 
         Report each deliverable as you complete it."
```

Then for each phase:
```
Prompt: "Execute PHASE {N} using the Master Build Spec and prior results."
```

---

**END OF MASTER SPECIFICATION**

**Status:** READY FOR SONNET EXECUTION  
**Auto-Addressing:** PH1.S1 through PH5.S5 (25 total steps)  
**Estimated Duration:** 24-28 hours (all 5 phases)  
**Proof Target:** 20 theorems, kernel-checked Coq, zero gaps
