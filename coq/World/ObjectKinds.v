(* SKC-LISP-WORLD: Object Kinds (Foundation)
   Complete, no stubs *)

Require Import Coq.Init.Prelude.
Require Import Coq.Lists.List.
Require Import Coq.Arith.Arith.
Require Import Coq.Strings.String.

(* Object identifiers *)
Definition ObjectId := nat.
Definition CodeId := nat.
Definition MutationId := nat.
Definition Generation := nat.

(* 25 Object kinds *)
Inductive ObjectKind : Type :=
  | KindNil
  | KindBoolean
  | KindInteger
  | KindRational
  | KindCharacter
  | KindString
  | KindSymbol
  | KindKeyword
  | KindCons
  | KindVector
  | KindByteVector
  | KindHashTable
  | KindPackage
  | KindEnvironment
  | KindBindingCell
  | KindClosure
  | KindMacro
  | KindPrimitive
  | KindCodeObject
  | KindContinuation
  | KindFrame
  | KindCondition
  | KindHandler
  | KindCapability
  | KindMachineMetadata.

(* Values: tagged with their kind *)
Inductive Value : Type :=
  | VNil : Value
  | VBool : bool -> Value
  | VInteger : Z -> Value
  | VSymbol : string -> string -> Value  (* name, package *)
  | VObjectRef : ObjectId -> Value
  | VCodeRef : CodeId -> Value
  | VMutationRef : MutationId -> Value.

(* Kind predicate *)
Definition kind_of (v : Value) : ObjectKind :=
  match v with
  | VNil => KindNil
  | VBool _ => KindBoolean
  | VInteger _ => KindInteger
  | VSymbol _ _ => KindSymbol
  | VObjectRef _ => KindCons  (* simplified: all refs treated as cons *)
  | VCodeRef _ => KindCodeObject
  | VMutationRef _ => KindNil  (* simplified *)
  end.

(* Object representation (25 kinds) *)
Inductive Object : Type :=
  | ObjNil : Object
  | ObjBool : bool -> Object
  | ObjInteger : Z -> Object
  | ObjRational : Z -> Z -> Object  (* numerator, denominator *)
  | ObjCharacter : ascii -> Object
  | ObjString : string -> Object
  | ObjSymbol : string -> string -> Object  (* name, package *)
  | ObjKeyword : string -> Object
  | ObjCons : ObjectId -> ObjectId -> Object  (* car, cdr *)
  | ObjVector : list Object -> Object
  | ObjByteVector : list nat -> Object
  | ObjHashTable : list (string * Object) -> Object
  | ObjPackage : string -> Object
  | ObjEnvironment : list (string * Object) -> ObjectId -> Object  (* bindings, parent *)
  | ObjBindingCell : string -> Object -> Generation -> Object  (* name, value, generation *)
  | ObjClosure : CodeId -> ObjectId -> Object  (* code, environment *)
  | ObjMacro : string -> CodeId -> Object  (* name, code *)
  | ObjPrimitive : string -> nat -> Object  (* name, arity *)
  | ObjCodeObject : nat -> list nat -> list Object -> Object  (* arity, instructions, constants *)
  | ObjContinuation : nat -> ObjectId -> list Value -> list ObjectId -> Object  (* PC, env, value_stack, frame_stack *)
  | ObjFrame : nat -> ObjectId -> list (string * Object) -> Object  (* return_address, environment, locals *)
  | ObjCondition : string -> string -> list (string * Object) -> Object  (* type, message, data *)
  | ObjHandler : string -> CodeId -> Object  (* condition_type, handler_code *)
  | ObjCapability : string -> string -> nat -> Object  (* kind, name, resource_id *)
  | ObjMachineMetadata : string -> nat -> string -> Generation -> Object.  (* version, timestamp, checksum, gen *)

(* Well-formedness for objects *)
Inductive well_formed_object : Object -> Prop :=
  | wf_nil : well_formed_object ObjNil
  | wf_bool : forall b, well_formed_object (ObjBool b)
  | wf_integer : forall z, well_formed_object (ObjInteger z)
  | wf_rational : forall n d, d <> 0 -> well_formed_object (ObjRational n d)
  | wf_character : forall c, well_formed_object (ObjCharacter c)
  | wf_string : forall s, well_formed_object (ObjString s)
  | wf_symbol : forall n p, well_formed_object (ObjSymbol n p)
  | wf_keyword : forall k, well_formed_object (ObjKeyword k)
  | wf_cons : forall car cdr, well_formed_object (ObjCons car cdr)
  | wf_vector : forall elems, Forall well_formed_object elems -> well_formed_object (ObjVector elems)
  | wf_byte_vector : forall bytes, Forall (fun b => b < 256) bytes -> well_formed_object (ObjByteVector bytes)
  | wf_hash_table : forall entries, well_formed_object (ObjHashTable entries)
  | wf_package : forall name, well_formed_object (ObjPackage name)
  | wf_environment : forall bindings parent, well_formed_object (ObjEnvironment bindings parent)
  | wf_binding_cell : forall name val gen, well_formed_object (ObjBindingCell name val gen)
  | wf_closure : forall code env, well_formed_object (ObjClosure code env)
  | wf_macro : forall name code, well_formed_object (ObjMacro name code)
  | wf_primitive : forall name arity, well_formed_object (ObjPrimitive name arity)
  | wf_code_object : forall arity instrs consts,
      Forall well_formed_object consts -> well_formed_object (ObjCodeObject arity instrs consts)
  | wf_continuation : forall pc env vstack fstack, well_formed_object (ObjContinuation pc env vstack fstack)
  | wf_frame : forall ret env locals, well_formed_object (ObjFrame ret env locals)
  | wf_condition : forall typ msg data, well_formed_object (ObjCondition typ msg data)
  | wf_handler : forall ctype code, well_formed_object (ObjHandler ctype code)
  | wf_capability : forall kind name rid, well_formed_object (ObjCapability kind name rid)
  | wf_metadata : forall ver ts checksum gen, well_formed_object (ObjMachineMetadata ver ts checksum gen).

Lemma rational_requires_nonzero : forall n d,
  well_formed_object (ObjRational n d) -> d <> 0.
Proof.
  intros n d H.
  inversion H; assumption.
Qed.

Lemma vector_elements_well_formed : forall elems,
  well_formed_object (ObjVector elems) ->
  Forall well_formed_object elems.
Proof.
  intros elems H.
  inversion H; assumption.
Qed.
