-- SKC-LISP-WORLD: Primitive Types (M01)
-- Foundation: typed values, identifiers, generations, error codes

import Std

namespace SKCLisp

-- All identifiers are natural numbers (no separate type aliases to avoid instance issues)
-- Semantics carried by function names and documentation

-- Object identifier operations
def object_id_zero : Nat := 0
def object_id_succ (oid : Nat) : Nat := Nat.succ oid
def object_id_lt (a b : Nat) : Prop := a < b
def object_id_le (a b : Nat) : Prop := a ≤ b

-- Code identifier operations
def code_id_zero : Nat := 0
def code_id_succ (cid : Nat) : Nat := Nat.succ cid

-- Mutation identifier operations
def mutation_id_zero : Nat := 0
def mutation_id_succ (mid : Nat) : Nat := Nat.succ mid

-- Generation operations (Invariant I12: Generation monotonicity)
def generation_zero : Nat := 0
def generation_succ (g : Nat) : Nat := Nat.succ g
def generation_lt (a b : Nat) : Prop := a < b
def generation_le (a b : Nat) : Prop := a ≤ b

-- Byte sequence type for serialization
def ByteSequence := List UInt8

-- Error codes for mutation and validation failures
inductive ErrorCode : Type where
  | NoError : ErrorCode
  | InvalidReference : ErrorCode
  | OutOfRange : ErrorCode
  | TypeMismatch : ErrorCode
  | PermissionDenied : ErrorCode
  | StateInvalid : ErrorCode
  | CorruptedData : ErrorCode
  | NotFound : ErrorCode
  | AlreadyExists : ErrorCode
  | Overflow : ErrorCode
  | Underflow : ErrorCode
  | ZeroDivision : ErrorCode
  | DeadlockDetected : ErrorCode
  | TimeoutExpired : ErrorCode
  | OperationAborted : ErrorCode
  | UnknownError : ErrorCode

-- Check if error code represents failure
def is_error (ec : ErrorCode) : Bool :=
  match ec with
  | ErrorCode.NoError => false
  | _ => true

-- Boolean operations
def bool_not (b : Bool) : Bool := !b
def bool_and (a b : Bool) : Bool := a && b
def bool_or (a b : Bool) : Bool := a || b
def bool_xor (a b : Bool) : Bool := a != b

-- Numeric value types
def IntValue := Int
def UIntValue := Nat

-- Character and byte types
def CharValue := Char
def ByteValue := UInt8

-- String and symbol names
def SymbolName := String

-- Machine execution status
inductive MachineStatus : Type where
  | Running : MachineStatus
  | Halted : MachineStatus
  | Trapped : String → MachineStatus

-- Validation result type (generic over success type)
inductive ValidationResult (α : Type) : Type where
  | Valid : α → ValidationResult α
  | Invalid : ErrorCode → String → ValidationResult α

-- Mutation operation result
inductive MutationResult : Type where
  | Accepted : Nat → Nat → MutationResult  -- mutation_id, generation
  | Rejected : ErrorCode → String → MutationResult

-- Journal sequence number (strictly increasing within generation)
def JournalSeqNo := Nat

-- Capability kind enumeration (8 kinds)
inductive CapabilityKind : Type where
  | FileIO : CapabilityKind
  | SocketIO : CapabilityKind
  | Clock : CapabilityKind
  | RandomSource : CapabilityKind
  | ProcessControl : CapabilityKind
  | ThreadControl : CapabilityKind
  | DeviceAccess : CapabilityKind
  | ForeignRuntime : CapabilityKind

-- Capability restoration policy
inductive RestorationPolicy : Type where
  | Rebuild : RestorationPolicy
  | Reconnect : RestorationPolicy
  | Fresh : RestorationPolicy
  | Deny : RestorationPolicy

-- Instruction opcodes (30 operations)
inductive Opcode : Type where
  | OpConst : Opcode
  | OpLookup : Opcode
  | OpBind : Opcode
  | OpPush : Opcode
  | OpPop : Opcode
  | OpCall : Opcode
  | OpTailCall : Opcode
  | OpReturn : Opcode
  | OpJump : Opcode
  | OpJumpIfFalse : Opcode
  | OpJumpIfTrue : Opcode
  | OpMakeArray : Opcode
  | OpMakeRecord : Opcode
  | OpArrayRef : Opcode
  | OpArraySet : Opcode
  | OpRecordRef : Opcode
  | OpRecordSet : Opcode
  | OpClosure : Opcode
  | OpContinuation : Opcode
  | OpApply : Opcode
  | OpMutate : Opcode
  | OpCommit : Opcode
  | OpRollback : Opcode
  | OpReplay : Opcode
  | OpPatchCode : Opcode
  | OpCapabilityOp : Opcode
  | OpDump : Opcode
  | OpRestore : Opcode
  | OpAssert : Opcode
  | OpHalt : Opcode

-- Value tags for the tagged value representation (25 kinds)
inductive ValueTag : Type where
  | TagNil : ValueTag
  | TagBoolean : ValueTag
  | TagInteger : ValueTag
  | TagCharacter : ValueTag
  | TagSymbol : ValueTag
  | TagArray : ValueTag
  | TagRecord : ValueTag
  | TagString : ValueTag
  | TagByteSequence : ValueTag
  | TagObjectRef : ValueTag
  | TagClosure : ValueTag
  | TagCodeObject : ValueTag
  | TagContinuation : ValueTag
  | TagFrame : ValueTag
  | TagEnvironment : ValueTag
  | TagMutationRecord : ValueTag
  | TagCapability : ValueTag
  | TagWorldDump : ValueTag
  | TagJournal : ValueTag
  | TagGeneration : ValueTag
  | TagUnknown : ValueTag
  | TagTrap : ValueTag
  | TagChannel : ValueTag
  | TagPromise : ValueTag
  | TagCustom : String → ValueTag

-- Tagged value payload structure
structure TaggedValue where
  tag : ValueTag
  payload : ByteSequence
  metadata : String  -- for future extensions

-- Value construction helpers
def mk_nil_value : TaggedValue := {
  tag := ValueTag.TagNil
  payload := []
  metadata := ""
}

def mk_bool_value (b : Bool) : TaggedValue := {
  tag := ValueTag.TagBoolean
  payload := if b then [1] else [0]
  metadata := ""
}

def mk_integer_value (n : Int) : TaggedValue := {
  tag := ValueTag.TagInteger
  payload := []  -- simplified: no actual encoding here
  metadata := ""
}

-- Allocation counter for fresh ID generation
structure AllocationCounter where
  next_oid : Nat
  next_cid : Nat
  next_mid : Nat

def mk_allocation_counter : AllocationCounter := {
  next_oid := object_id_zero
  next_cid := code_id_zero
  next_mid := mutation_id_zero
}

-- Allocate next object ID
def allocate_object (counter : AllocationCounter) : Nat × AllocationCounter :=
  let new_oid := counter.next_oid
  let new_counter := { counter with next_oid := object_id_succ counter.next_oid }
  (new_oid, new_counter)

-- Allocate next code ID
def allocate_code (counter : AllocationCounter) : Nat × AllocationCounter :=
  let new_cid := counter.next_cid
  let new_counter := { counter with next_cid := code_id_succ counter.next_cid }
  (new_cid, new_counter)

-- Allocate next mutation ID
def allocate_mutation (counter : AllocationCounter) : Nat × AllocationCounter :=
  let new_mid := counter.next_mid
  let new_counter := { counter with next_mid := mutation_id_succ counter.next_mid }
  (new_mid, new_counter)

-- Lemmas and proofs for invariants

-- Invariant I01: Unique Object Identifiers - succ is injective
lemma object_id_succ_injective {a b : Nat} (h : object_id_succ a = object_id_succ b) : a = b :=
  Nat.succ_injective h

-- Invariant I01: succ never produces zero
lemma object_id_succ_ne_zero {oid : Nat} : object_id_succ oid ≠ 0 :=
  Nat.succ_ne_zero oid

-- Invariant I12: Generation Monotonicity - generations strictly increase
lemma generation_strictly_increasing {g : Nat} : generation_lt g (generation_succ g) := by
  unfold generation_lt generation_succ
  omega

-- Generation successor is unique (not equal)
lemma generation_succ_ne {g : Nat} : ¬(generation_eq g (generation_succ g)) := by
  unfold generation_eq generation_succ
  omega

-- Allocation always produces fresh IDs
lemma allocate_object_fresh {counter : AllocationCounter} : object_id_lt counter.next_oid (allocate_object counter).2.next_oid := by
  simp [allocate_object, object_id_lt]
  omega

-- Allocation counter advances monotonically
lemma allocation_counter_increasing {counter : AllocationCounter} : counter.next_oid < (allocate_object counter).2.next_oid := by
  simp [allocate_object]
  omega

-- Bool negation is involutive (not (not x) = x)
lemma bool_not_involutive {b : Bool} : bool_not (bool_not b) = b := by
  cases b <;> rfl

-- Bool and is associative
lemma bool_and_assoc {a b c : Bool} : bool_and (bool_and a b) c = bool_and a (bool_and b c) := by
  cases a <;> cases b <;> cases c <;> rfl

-- Bool or is associative
lemma bool_or_assoc {a b c : Bool} : bool_or (bool_or a b) c = bool_or a (bool_or b c) := by
  cases a <;> cases b <;> cases c <;> rfl

-- Error codes are finite (16 variants)
def error_codes : List ErrorCode :=
  [ErrorCode.NoError, ErrorCode.InvalidReference, ErrorCode.OutOfRange,
   ErrorCode.TypeMismatch, ErrorCode.PermissionDenied, ErrorCode.StateInvalid,
   ErrorCode.CorruptedData, ErrorCode.NotFound, ErrorCode.AlreadyExists,
   ErrorCode.Overflow, ErrorCode.Underflow, ErrorCode.ZeroDivision,
   ErrorCode.DeadlockDetected, ErrorCode.TimeoutExpired, ErrorCode.OperationAborted,
   ErrorCode.UnknownError]

-- Capability kinds are finite (8 variants)
def capability_kinds : List CapabilityKind :=
  [CapabilityKind.FileIO, CapabilityKind.SocketIO, CapabilityKind.Clock,
   CapabilityKind.RandomSource, CapabilityKind.ProcessControl,
   CapabilityKind.ThreadControl, CapabilityKind.DeviceAccess,
   CapabilityKind.ForeignRuntime]

-- Opcodes are finite (30 operations)
def opcodes : List Opcode :=
  [Opcode.OpConst, Opcode.OpLookup, Opcode.OpBind, Opcode.OpPush,
   Opcode.OpPop, Opcode.OpCall, Opcode.OpTailCall, Opcode.OpReturn,
   Opcode.OpJump, Opcode.OpJumpIfFalse, Opcode.OpJumpIfTrue,
   Opcode.OpMakeArray, Opcode.OpMakeRecord, Opcode.OpArrayRef,
   Opcode.OpArraySet, Opcode.OpRecordRef, Opcode.OpRecordSet,
   Opcode.OpClosure, Opcode.OpContinuation, Opcode.OpApply,
   Opcode.OpMutate, Opcode.OpCommit, Opcode.OpRollback, Opcode.OpReplay,
   Opcode.OpPatchCode, Opcode.OpCapabilityOp, Opcode.OpDump,
   Opcode.OpRestore, Opcode.OpAssert, Opcode.OpHalt]

-- Value tags are finite
def value_tags : List ValueTag :=
  [ValueTag.TagNil, ValueTag.TagBoolean, ValueTag.TagInteger,
   ValueTag.TagCharacter, ValueTag.TagSymbol, ValueTag.TagArray,
   ValueTag.TagRecord, ValueTag.TagString, ValueTag.TagByteSequence,
   ValueTag.TagObjectRef, ValueTag.TagClosure, ValueTag.TagCodeObject,
   ValueTag.TagContinuation, ValueTag.TagFrame, ValueTag.TagEnvironment,
   ValueTag.TagMutationRecord, ValueTag.TagCapability, ValueTag.TagWorldDump,
   ValueTag.TagJournal, ValueTag.TagGeneration, ValueTag.TagUnknown,
   ValueTag.TagTrap, ValueTag.TagChannel, ValueTag.TagPromise]

-- Count error codes (lemma)
lemma error_code_count : error_codes.length = 16 := by rfl

-- Count capabilities (lemma)
lemma capability_kind_count : capability_kinds.length = 8 := by rfl

-- Count opcodes (lemma)
lemma opcode_count : opcodes.length = 30 := by rfl

-- Count value tags (lemma)
lemma value_tag_count : value_tags.length = 25 := by rfl

-- Validation result functor instance
instance : Functor ValidationResult where
  map f r := match r with
    | ValidationResult.Valid a => ValidationResult.Valid (f a)
    | ValidationResult.Invalid err msg => ValidationResult.Invalid err msg

-- Byte sequence operations
def byte_sequence_length (bs : ByteSequence) : Nat := bs.length

def byte_sequence_append (bs1 bs2 : ByteSequence) : ByteSequence :=
  List.append bs1 bs2

def byte_sequence_empty : ByteSequence := []

def byte_sequence_is_empty (bs : ByteSequence) : Bool :=
  bs.isEmpty

-- Lemma: byte sequence is associative
lemma byte_sequence_append_assoc : ∀ (a b c : ByteSequence),
  byte_sequence_append (byte_sequence_append a b) c =
  byte_sequence_append a (byte_sequence_append b c) := by
  intros a b c
  simp [byte_sequence_append, List.append_assoc]

-- Lemma: byte sequence empty is identity
lemma byte_sequence_append_empty : ∀ bs : ByteSequence,
  byte_sequence_append bs byte_sequence_empty = bs := by
  intro bs
  simp [byte_sequence_append, byte_sequence_empty, List.append_nil]

-- Status matching lemma
lemma machine_status_eq_running : ∀ s : MachineStatus,
  s = MachineStatus.Running ∨ s ≠ MachineStatus.Running := by
  intro s
  cases s <;> simp

end SKCLisp
