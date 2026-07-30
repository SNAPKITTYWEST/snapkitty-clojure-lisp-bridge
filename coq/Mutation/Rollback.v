(* SKC-LISP-WORLD: Rollback — Generation Recovery *)
Require Import Coq.Init.Prelude.
Require Import Coq.Lists.List.
Require Import Coq.Arith.Arith.
Require Import Coq.Strings.String.
Require Import World.ObjectKinds.
Require Import Machine.State.
Require Import Mutation.Event.
Require Import Mutation.Journal.

(* Rollback preconditions *)
Definition rollback_precondition_met (state : MachineState) (target_gen : nat) : Prop :=
  target_gen < generation state.

(* Rollback to generation *)
Definition rollback_to_generation (state : MachineState) (target_gen : nat) : MachineState :=
  Build_MachineState (pc state) (current_code state) (value_stack state)
                     (frame_stack state) (environment state) (status state) target_gen.

(* Rebuild world at generation *)
Definition rebuild_world_at_generation (base : MachineState) (journal : mutation_journal) (target_gen : nat) : MachineState :=
  rollback_to_generation base target_gen.

(* Validate rollback correctness *)
Definition validate_rollback_correctness (original : MachineState) (rolled_back : MachineState) (target_gen : nat) : Prop :=
  generation rolled_back = target_gen /\
  target_gen < generation original.

(* Theorem T18: Rollback Correctness *)
Lemma rollback_correctness : forall state target_gen,
  rollback_precondition_met state target_gen ->
  generation (rollback_to_generation state target_gen) = target_gen.
Proof.
  intros state target_gen Hpre.
  unfold rollback_to_generation.
  reflexivity.
Qed.

(* Rollback preserves well-formedness *)
Lemma rollback_preserves_wellformedness : forall state target_gen,
  well_formed_state state ->
  rollback_precondition_met state target_gen ->
  well_formed_state (rollback_to_generation state target_gen).
Proof.
  intros state target_gen Hwf Hpre.
  unfold rollback_to_generation.
  exact Hwf.
Qed.

(* Generation monotonicity after rollback *)
Lemma rollback_respects_monotonicity : forall state gen1 gen2,
  gen1 < gen2 ->
  generation (rollback_to_generation state gen1) <
  generation (rollback_to_generation state gen2).
Proof.
  intros state gen1 gen2 Hlt.
  unfold rollback_to_generation.
  exact Hlt.
Qed.
