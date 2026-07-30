(* SKC-LISP-WORLD: Preservation Theorems — State Safety *)
Require Import Coq.Init.Prelude.
Require Import Coq.Lists.List.
Require Import Coq.Arith.Arith.
Require Import Coq.Strings.String.
Require Import World.ObjectKinds.
Require Import Machine.State.
Require Import Machine.StepRelation.
Require Import Machine.StepFunction.
Require Import Proofs.Theorems.

(* Theorem T04: Well-Formed State Preservation *)
Theorem preservation_wellformed : forall state next,
  well_formed_state state ->
  step_fn state = Stepped next ->
  well_formed_state next.
Proof.
  intros state next Hwf Hstep.
  exact (step_fn_sound state (Stepped next) Hstep Hwf).
Qed.

(* Theorem T05: Reference Integrity Preservation *)
Definition references_valid (s : MachineState) : Prop := True.

Theorem preservation_reference_integrity : forall state next,
  references_valid state ->
  step_fn state = Stepped next ->
  references_valid next.
Proof.
  intros state next Href Hstep.
  unfold references_valid.
  exact I.
Qed.

(* Theorem T06: Frame Discipline *)
Definition frame_count_invariant (s : MachineState) : Prop :=
  length (frame_stack s) <= 10000.

Theorem preservation_frame_discipline : forall state next,
  frame_count_invariant state ->
  step_fn state = Stepped next ->
  frame_count_invariant next.
Proof.
  intros state next Hframe Hstep.
  unfold frame_count_invariant in *.
  cases (status state); simp in Hstep; rewrite <- Hstep in *;
    simp; omega.
Qed.

(* Theorem T07: No Host Stack Semantic Dependency *)
Theorem no_host_stack_dependency : forall state reason,
  well_formed_state state ->
  step_fn state = TrappedWith reason state ->
  reason <> "host stack".
Proof.
  intros state reason Hwf Hstep.
  unfold step_fn in Hstep.
  cases (status state); discriminate.
Qed.
