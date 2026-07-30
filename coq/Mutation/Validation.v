(* SKC-LISP-WORLD: Mutation Validation Gate *)
Require Import Coq.Init.Prelude.
Require Import Coq.Arith.Arith.
Require Import Coq.Lists.List.
Require Import Coq.Strings.String.
Require Import Machine.State.
Require Import Mutation.Event.
Require Import World.ObjectKinds.
Require Import Coq.omega.Omega.
Lemma generation_monotonicity : forall e,
  mutation_gate_valid e = true ->
  generation_before e < generation_after e.
Proof.
  intros e Hvalid.
  unfold mutation_gate_valid in Hvalid.
  simp in Hvalid.
  unfold generation_advances in *.
  cases (generation_after e > generation_before e); try discriminate.
  omega.
Qed.
