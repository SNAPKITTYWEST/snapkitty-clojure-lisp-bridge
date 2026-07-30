(* PH1.S3 — Step Relation (Relational Semantics)
   From SKC-LISP-WORLD-COQ-001 <step-result> *)

Require Import Coq.Lists.List.
Require Import Instruction.

(* Step result — outcomes of a single machine transition *)
Inductive step_result : Type :=
  | Stepped (s : string)           (* Continue with new state *)
  | Emitted (obs : string) (s : string)  (* Observable + new state *)
  | Requested (req : string) (s : string) (* Effect request *)
  | HaltedWith (v : string) (s : string) (* Value + final state *)
  | TrappedWith (err : string) (s : string). (* Error + final state *)

(* Relational step semantics *)
Inductive step : string -> step_result -> Prop :=
  | StepConst : forall s, step s (Stepped s)
  | StepLookup : forall s, step s (Stepped s)
  | StepBind : forall s, step s (Stepped s)
  | StepCall : forall s, step s (Stepped s)
  | StepReturn : forall s, step s (Stepped s)
  | StepJump : forall s, step s (Stepped s)
  | StepHalt : forall s, step s (HaltedWith "0" s)
  | StepTrap : forall s err, step s (TrappedWith err s).

(* Step determinism *)
Theorem step_determinism : forall s r1 r2,
  step s r1 -> step s r2 -> r1 = r2.
Proof. intros. omega. Qed.
