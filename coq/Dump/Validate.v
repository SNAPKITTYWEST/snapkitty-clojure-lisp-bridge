(* SKC-LISP-WORLD: Validation — Complete Structural Checks *)
Require Import Coq.Init.Prelude.
Require Import Coq.Arith.Arith.
Require Import Coq.Lists.List.
Require Import Coq.Strings.String.
Require Import Dump.Bytes.
Require Import Dump.Decode.
Require Import Machine.State.
Require Import World.ObjectKinds.
Require Import Coq.omega.Omega.
Theorem validation_complete : forall h total_len,
  all_validations_pass h total_len = true ->
  validate_magic h = true /\
  validate_version h = true /\
  validate_endianness h = true /\
  validate_word_size h = true.
Proof.
  intros h total_len Hval.
  unfold all_validations_pass in Hval.
  repeat split; try omega.
Qed.
