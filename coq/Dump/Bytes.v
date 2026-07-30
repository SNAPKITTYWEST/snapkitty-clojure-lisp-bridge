(* SKC-LISP-WORLD: Dump Format — Byte-level Serialization *)
Require Import Coq.Init.Prelude.
Require Import Coq.Arith.Arith.
Require Import Coq.Lists.List.
Require Import Coq.Strings.String.
Require Import Machine.State.
Require Import World.ObjectKinds.
Require Import Coq.omega.Omega.
Definition dump_is_deterministic : Prop :=
  forall w1 w2,
    generation w1 = generation w2 ->
    canonical_world_bytes w1 = canonical_world_bytes w2.

Lemma dump_deterministic :
  dump_is_deterministic.
Proof.
  unfold dump_is_deterministic, canonical_world_bytes.
  intros w1 w2 Hgen.
  rewrite Hgen.
  reflexivity.
Qed.
