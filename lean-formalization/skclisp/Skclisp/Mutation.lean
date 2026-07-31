-- SKC-LISP-WORLD: Mutation Module (M03)
-- Foundation: Mutation model + replay/rollback relations
-- Mirrors Coq formalization from coq/Mutation/
-- Equivalence target: Lean M03 ≡ Coq Mutation

import Std
import Skclisp.Basic
import Skclisp.Machine

namespace Skclisp.Mutation

-- ============================================================================
-- Mutation Event (mirrors Coq Event.v)
-- ============================================================================

structure MutationEvent where
  eventId : Nat
  timestamp : Nat
  actor : Nat
  target : Nat
  generationBefore : Nat
  generationAfter : Nat
  deriving Repr, DecidableEq

def isValidEvent (e : MutationEvent) : Prop :=
  e.generationAfter = e.generationBefore + 1 &&
  e.actor < 1024 &&
  e.target < 1024

-- ============================================================================
-- Mutation Journal (mirrors Coq Journal.v)
-- ============================================================================

structure MutationJournal where
  events : Array MutationEvent
  lastGeneration : Nat
  sealed : Bool
  deriving Repr, DecidableEq

def emptyJournal : MutationJournal where
  events := #[]
  lastGeneration := 0
  sealed := false

def appendEvent (j : MutationJournal) (e : MutationEvent) : Option MutationJournal :=
  if j.sealed then
    none
  else if ¬isValidEvent e then
    none
  else if e.generationBefore ≠ j.lastGeneration then
    none
  else
    some {
      events := j.events.push e
      lastGeneration := e.generationAfter
      sealed := false
    }

-- ============================================================================
-- Replay Relation (mirrors Coq Replay.v)
-- ============================================================================

inductive ReplayResult where
  | ok (s : MachineState) (j : MutationJournal)
  | error (msg : String)

def replayEvent (s : MachineState) (j : MutationJournal) (e : MutationEvent) : ReplayResult :=
  if ¬isValidEvent e then
    .error "invalid event"
  else if e.generationBefore ≠ j.lastGeneration then
    .error "generation mismatch"
  else if e.actor ≥ s.stack.size then
    .error "actor out of bounds"
  else
    match appendEvent j e with
    | none => .error "cannot append event"
    | some j' => .ok { s with generation := e.generationAfter } j'

def replayJournal (s : MachineState) (j : MutationJournal) : ReplayResult :=
  if j.events.size = 0 then
    .ok s j
  else
    let rec loop (s : MachineState) (j : MutationJournal) (idx : Nat) : ReplayResult :=
      if idx ≥ j.events.size then
        .ok s j
      else
        let e := j.events[idx]!
        match replayEvent s j e with
        | .error msg => .error msg
        | .ok s' j' => loop s' j' (idx + 1)
    loop s j 0

-- ============================================================================
-- Rollback Relation (mirrors Coq Rollback.v)
-- ============================================================================

inductive RollbackResult where
  | ok (s : MachineState) (j : MutationJournal)
  | error (msg : String)

def rollbackToGeneration (s : MachineState) (j : MutationJournal) (gen : Nat) : RollbackResult :=
  if gen > j.lastGeneration then
    .error "target generation in future"
  else if j.events.size = 0 then
    .ok s j
  else
    let rec findCutoff (idx : Nat) : Option Nat :=
      if idx ≥ j.events.size then
        some 0
      else if j.events[idx]!.generationAfter > gen then
        some idx
      else
        findCutoff (idx + 1)
    match findCutoff 0 with
    | none => .error "cutoff not found"
    | some cutoff =>
      let newEvents := j.events.extract 0 cutoff
      let newGen := if cutoff = 0 then 0 else j.events[cutoff - 1]!.generationAfter
      .ok { s with generation := newGen } {
        events := newEvents
        lastGeneration := newGen
        sealed := false
      }

-- ============================================================================
-- Validation Gate (mirrors Coq Validation.v)
-- ============================================================================

def validateMutation (e : MutationEvent) (j : MutationJournal) : Bool :=
  isValidEvent e &&
  e.generationBefore = j.lastGeneration &&
  e.actor < 1024 &&
  e.target < 1024

-- ============================================================================
-- Equivalence Target (Lean M03 ≡ Coq Mutation)
-- ============================================================================

-- T08: Mutation journal completeness
-- Theorem mutationJournalCompleteness : ∀ (e : MutationEvent) (j : MutationJournal),
--   isValidEvent e →
--   e.generationBefore = j.lastGeneration →
--   ∃ (j' : MutationJournal), appendEvent j e = some j'

-- T09: Replay determinism
-- Theorem replayDeterminism : ∀ (s : MachineState) (j : MutationJournal) (r1 r2 : ReplayResult),
--   replayJournal s j = r1 →
--   replayJournal s j = r2 →
--   r1 = r2

-- T10: Rollback soundness
-- Theorem rollbackSoundness : ∀ (s : MachineState) (j : MutationJournal) (gen : Nat),
--   gen ≤ j.lastGeneration →
--   ∃ (s' : MachineState) (j' : MutationJournal),
--     rollbackToGeneration s j gen = RollbackResult.ok s' j'

-- T11: Generation monotonicity
-- Theorem generationMonotonicity : ∀ (e1 e2 : MutationEvent) (j : MutationJournal),
--   appendEvent j e1 = some j' →
--   appendEvent j' e2 = some j'' →
--   e1.generationAfter < e2.generationAfter

-- [Proofs deferred to Coq equivalence proof]

end Skclisp.Mutation
