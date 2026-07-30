(* SKC-LISP-WORLD: Preservation Theorems — State Safety *)
Require Import Coq.Init.Prelude.
Require Import Coq.Arith.Arith.
Require Import Coq.Lists.List.
Require Import Coq.Strings.String.
Require Import Machine.State.
Require Import Machine.StepFunction.
Require Import Machine.StepRelation.
Require Import Proofs.Theorems.
Require Import World.ObjectKinds.
Require Import Coq.omega.Omega.
Theorem no_host_stack_dependency : forall state reason,
  well_formed_state state ->
  step_fn state = TrappedWith reason state ->
  reason <> "host stack".
Proof.
  intros state reason Hwf Hstep.
  unfold step_fn in Hstep.
  cases (status state); discriminate.
Qed.
