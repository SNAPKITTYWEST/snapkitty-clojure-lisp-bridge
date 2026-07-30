(* PH4.S1 — All 25 Object Kinds (Exact from XML) *)

Inductive object_kind : Type :=
  | nil
  | boolean
  | integer
  | rational
  | character
  | string
  | symbol
  | keyword
  | cons
  | vector
  | byte_vector
  | hash_table
  | package
  | environment
  | binding_cell
  | closure
  | macro
  | primitive
  | code_object
  | continuation
  | frame
  | condition
  | handler
  | capability
  | machine_metadata.

Definition object_kind_count : nat := 25.

Inductive value : Type :=
  | VNil
  | VBool (b : bool)
  | VInt (n : nat)
  | VSymbol (s : string)
  | VCons (car cdr : value)
  | VClosure
  | VCodeObject
  | VContinuation
  | VFrame.
