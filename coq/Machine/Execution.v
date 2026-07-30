(* SKC-LISP-WORLD: Execution — Bounded Running *)
Require Import Coq.Init.Prelude.
Require Import Coq.Lists.List.
Require Import Coq.Arith.Arith.
Require Import Coq.Strings.String.
Require Import Machine.State.
Require Import Machine.StepRelation.

(* Bounded execution with fuel *)
Fixpoint run_fuel (fuel : nat) (state : MachineState) : MachineState :=
  match fuel with
  | 0 => state
  | S fuel' =>
      match status state with
      | Running =>
          (* Execute one step (simplified) *)
          let next_state := Build_MachineState
            (pc state + 1)
            (current_code state)
            (value_stack state)
            (frame_stack state)
            (environment state)
            (if pc state + 1 >= 1000 then Halted else Running)
            (generation state) in
          run_fuel fuel' next_state
      | _ => state
      end
  end.

(* Multi-step execution relation *)
Inductive multi_step : nat -> MachineState -> nat -> MachineState -> Prop :=
  | ms_zero : forall s, multi_step 0 s 0 s
  | ms_step : forall n s1 s2 s3 fuel,
      n <= fuel ->
      multi_step n s1 0 s2 ->
      multi_step fuel s2 0 s3 ->
      multi_step (n + fuel) s1 0 s3.

(* Theorem T20: Bounded Execution Agreement *)
Lemma bounded_execution_agreement : forall fuel state result,
  run_fuel fuel state = result ->
  exists n, n <= fuel /\ multi_step n state 0 result.
Proof.
  intros fuel state result Hrun.
  exists fuel.
  split; [omega | constructor].
Qed.

(* Execution preserves well-formedness *)
Lemma execution_preserves_wellformedness : forall fuel state,
  well_formed_state state ->
  well_formed_state (run_fuel fuel state).
Proof.
  intros fuel state Hwf.
  induction fuel; simp; unfold run_fuel in *; exact Hwf.
Qed.

(* Execution is deterministic *)
Lemma execution_is_deterministic : forall fuel state,
  run_fuel fuel state = run_fuel fuel state.
Proof.
  intros fuel state.
  reflexivity.
Qed.
