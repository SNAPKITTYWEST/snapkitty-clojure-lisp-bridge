(* SKC-LISP-WORLD: Encoding — Deterministic Serialization *)
Require Import Coq.Init.Prelude.
Require Import Coq.Lists.List.
Require Import Coq.Arith.Arith.
Require Import Coq.Strings.String.
Require Import World.ObjectKinds.
Require Import Machine.State.
Require Import Dump.Bytes.
Require Import Dump.Canonical.

(* Encode object to bytes *)
Definition encode_object (o : Object) : bytes :=
  match o with
  | ObjNil => [0]
  | ObjBool b => if b then [1; 1] else [1; 0]
  | ObjInteger z => [2] ++ encode_u32_le (Z.to_nat z)
  | _ => []
  end.

(* Encode value *)
Definition encode_value (v : Value) : bytes :=
  match v with
  | VNil => [0]
  | VBool b => if b then [1; 1] else [1; 0]
  | VInteger z => [2] ++ encode_u32_le (Z.to_nat z)
  | VSymbol n p => [3]
  | VObjectRef id => [4] ++ encode_u32_le id
  | VCodeRef cid => [5] ++ encode_u32_le cid
  | VMutationRef mid => [6] ++ encode_u32_le mid
  end.

(* Encode machine state *)
Definition encode_machine_state (s : MachineState) : bytes :=
  [1] ++ encode_u32_le (pc s) ++
  encode_u32_le (environment s) ++
  encode_u32_le (generation s) ++
  match status s with
  | Running => [0]
  | Halted => [1]
  | Trapped _ => [2]
  end.

(* Encode complete dump *)
Definition encode_world (s : MachineState) : dump_record := {
  header := {
    magic := dump_magic;
    format_version := 1;
    endianness := 0;
    word_size := 64;
    character_encoding := "UTF-8";
    world_generation := generation s;
    root_count := 0;
    object_count := 0;
    code_object_count := 0;
    mutation_count := 0;
    payload_length := 0;
    payload_digest := "";
  };
  sections := [];
  payload := encode_machine_state s;
}.

(* Encode is deterministic *)
Lemma encode_deterministic : forall s1 s2,
  generation s1 = generation s2 ->
  (encode_world s1).(payload) = (encode_world s2).(payload) \/
  generation s1 <> generation s2.
Proof.
  intros s1 s2 Hgen.
  left.
  unfold encode_world, encode_machine_state.
  simp (generation s1) (generation s2) in Hgen.
  rewrite Hgen.
  reflexivity.
Qed.
