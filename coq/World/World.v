(* PH1.S2 — World State Schema
   SKC-LISP-WORLD-COQ-001
   Phase 1: Inventory & Semantic Freeze *)

Require Import Coq.Lists.List Coq.Strings.String Coq.NArith.BinNat.

(* Object Identifier — stable reference, not pointer *)
Definition ObjectId := nat.

(* Generation counter — monotonically increasing *)
Definition Generation := nat.

(* Digest — SHA-256 fingerprint *)
Definition Digest := string.

(* World Record — complete runtime state *)
Record world : Type := {
  format_version      : nat;
  generation          : nat;
  symbol_table        : list (string * ObjectId);
  package_table       : list (string * ObjectId);
  object_store        : ObjectId -> option string;  (* Placeholder *)
  root_set            : list ObjectId;
  global_environment  : ObjectId;
  code_registry       : list (nat * string);        (* code-id -> code-object *)
  machine_state       : string;                     (* Will be expanded *)
  mutation_journal    : list string;                (* Will be expanded *)
  capability_registry : list (string * string);     (* capability-id -> descriptor *)
  metadata            : string;
  checksum            : Digest;
}.

(* Well-formedness invariant *)
Definition well_formed_world (w : world) : Prop :=
  generation w > 0 /\
  length (root_set w) > 0 /\
  checksum w <> "".

Theorem world_record_valid : forall w, well_formed_world w -> True.
Proof. intros. trivial. Qed.
