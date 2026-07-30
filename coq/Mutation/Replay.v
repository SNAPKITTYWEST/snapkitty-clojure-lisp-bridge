(* SKC-LISP-WORLD: Trace Replay — Deterministic Re-execution *)
Require Import Coq.Init.Prelude.
Require Import Coq.Arith.Arith.
Require Import Coq.Lists.List.
Require Import Coq.Strings.String.
Require Import Machine.State.
Require Import Mutation.Event.
Require Import Mutation.Journal.
Require Import World.ObjectKinds.
Require Import Coq.omega.Omega.
Lemma replay_correctness : forall base journal final_gen,
  journal_length journal > 0 ->
  (match journal_get journal (journal_length journal - 1) with
   | Some e => generation_after e = final_gen
   | None => False
   end) ->
  generation (replay_mutations base journal) = final_gen.
Proof.
  intros base journal final_gen Hlen Hfinal.
  unfold replay_mutations.
  rewrite Hlen.
  simpl in Hfinal.
  cases (journal_get journal _).
  - injection Hfinal as H.
    rewrite <- H.
    reflexivity.
  - exact Hfinal.
Qed.
