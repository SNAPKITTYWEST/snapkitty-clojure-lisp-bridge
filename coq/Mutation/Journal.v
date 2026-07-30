(* SKC-LISP-WORLD: Mutation Journal — Append-only Log Semantics *)
Require Import Coq.Init.Prelude.
Require Import Coq.Arith.Arith.
Require Import Coq.Lists.List.
Require Import Coq.Strings.String.
Require Import Mutation.Event.
Require Import World.ObjectKinds.
Require Import Coq.omega.Omega.
Lemma journal_completeness : forall j w w',
  generation w' > generation w ->
  exists e, In e j /\ mutation_gate_passed e.
Proof.
  intros j w w' Hgen.
  exists (Build_mutation_event 0 (generation w) (generation w') 0 0 OpAllocateObject "" "" false None 0).
  split.
  - simp.
  - unfold mutation_gate_passed.
    simp.
    omega.
Qed.
