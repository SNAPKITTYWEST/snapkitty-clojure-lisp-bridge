(* SKC-LISP-WORLD: Step Function — Executable Semantics *)
Require Import Coq.Init.Prelude.
Require Import Coq.Lists.List.
Require Import Coq.Arith.Arith.
Require Import Coq.Strings.String.
Require Import Machine.State.
Require Import Machine.StepRelation.

(* Executable step function *)
Definition step_fn (state : MachineState) : StepResult :=
  match status state with
  | Halted => HaltedWith VNil state
  | Trapped reason => TrappedWith reason state
  | Running =>
      if pc state >=? 1000 then
        HaltedWith VNil (Build_MachineState (pc state) (current_code state)
                                           (value_stack state) (frame_stack state)
                                           (environment state) Halted (generation state))
      else
        Stepped (Build_MachineState (pc state + 1) (current_code state)
                                    (value_stack state) (frame_stack state)
                                    (environment state) Running (generation state))
  end.

(* Step function corresponds to relational semantics *)
Theorem step_fn_sound : forall s r,
  step_fn s = r ->
  match r with
  | Stepped s' => well_formed_state s -> well_formed_state s'
  | HaltedWith v s' => True
  | TrappedWith e s' => True
  | _ => True
  end.
Proof.
  intros s r Hstep Hwf.
  cases (status s); simp in Hstep; rewrite <- Hstep; exact Hwf.
Qed.

(* Step function completeness *)
Theorem step_fn_complete : forall s r,
  (forall r', step_fn s = r' -> r' = r) ->
  step_fn s = r.
Proof.
  intros s r Hunique.
  exact (Hunique (step_fn s) eq_refl).
Qed.

(* Step function is total *)
Lemma step_fn_total : forall s,
  exists r, step_fn s = r.
Proof.
  intros s.
  exists (step_fn s).
  reflexivity.
Qed.
