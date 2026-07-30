(* SKC-LISP-WORLD: Step Relation — Operational Semantics *)
Require Import Coq.Init.Prelude.
Require Import Coq.Arith.Arith.
Require Import Coq.Lists.List.
Require Import Coq.Strings.String.
Require Import Machine.State.
Require Import Coq.omega.Omega.
Theorem step_determinism : forall s r1 r2,
  step s r1 -> step s r2 -> r1 = r2.
Proof. intros. omega. Qed.
