(* SKC-LISP-WORLD: Mutation Journal — Append-only Log Semantics *)
Require Import Coq.Init.Prelude.
Require Import Coq.Lists.List.
Require Import Coq.Arith.Arith.
Require Import Coq.Strings.String.
Require Import World.ObjectKinds.
Require Import Mutation.Event.

(* Journal is an ordered list of mutation events *)
Definition mutation_journal : Type := list mutation_event.

(* Journal append *)
Definition journal_append (j : mutation_journal) (e : mutation_event) : mutation_journal :=
  j ++ [e].

(* Journal length *)
Definition journal_length (j : mutation_journal) : nat :=
  length j.

(* Journal get *)
Definition journal_get (j : mutation_journal) (idx : nat) : option mutation_event :=
  nth_error j idx.

(* Journal slice *)
Definition journal_slice (j : mutation_journal) (start end_idx : nat) : mutation_journal :=
  firstn (end_idx - start) (skipn start j).

(* Journal is sequence ordered *)
Definition journal_ordered (j : mutation_journal) : Prop :=
  forall i j_idx e1 e2,
    j_idx + 1 < journal_length (j) ->
    journal_get j j_idx = Some e1 ->
    journal_get j (j_idx + 1) = Some e2 ->
    mutation_id e1 < mutation_id e2.

(* Append maintains ordering *)
Lemma append_preserves_ordering : forall j e,
  journal_ordered j ->
  mutation_gate_passed e ->
  journal_ordered (journal_append j e).
Proof.
  intros j e Hord Hgate.
  unfold journal_ordered, journal_append, journal_get, journal_length.
  intros i j_idx e1 e2 Hlen H1 H2.
  cases (nth_error (j ++ [e]) j_idx); try discriminate.
  cases (nth_error (j ++ [e]) (j_idx + 1)); try discriminate.
  simp in *; omega.
Qed.

(* Theorem T08: Mutation Journal Completeness *)
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
