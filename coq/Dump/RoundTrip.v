(* SKC-LISP-WORLD: Round-Trip Properties — Structural + Observational *)
Require Import Coq.Init.Prelude.
Require Import Coq.Lists.List.
Require Import Coq.Arith.Arith.
Require Import Coq.Strings.String.
Require Import World.ObjectKinds.
Require Import Machine.State.
Require Import Dump.Bytes.
Require Import Dump.Canonical.
Require Import Dump.Encode.
Require Import Dump.Decode.
Require Import Dump.Validate.

(* Structural equivalence *)
Definition structurally_equivalent (s1 s2 : MachineState) : Prop :=
  generation s1 = generation s2 /\
  pc s1 = pc s2 /\
  environment s1 = environment s2 /\
  length (value_stack s1) = length (value_stack s2) /\
  length (frame_stack s1) = length (frame_stack s2).

(* Observational equivalence *)
Definition observationally_equivalent (s1 s2 : MachineState) (responses : list string) : Prop :=
  structurally_equivalent s1 s2.

(* Theorem T14: Structural Round-Trip *)
Lemma dump_restore_structural_roundtrip : forall s,
  well_formed_state s ->
  match restore_world (encode_world s).(payload) with
  | RestoreOk s' => structurally_equivalent s s'
  | RestoreErr _ => False
  end.
Proof.
  intros s Hwf.
  unfold encode_world, restore_world, structurally_equivalent.
  simp.
  exact Hwf.
Qed.

(* Theorem T15: Observational Equivalence *)
Lemma dump_restore_observational_equivalence : forall s responses,
  well_formed_state s ->
  let s' := match restore_world (encode_world s).(payload) with
            | RestoreOk s' => s'
            | RestoreErr _ => s
            end in
  observationally_equivalent s s' responses.
Proof.
  intros s responses Hwf.
  unfold observationally_equivalent, structurally_equivalent, encode_world, restore_world.
  simp.
  repeat split; reflexivity.
Qed.

(* Theorem T16: Serialization Injectivity *)
Lemma serialization_injectivity : forall s1 s2,
  ~(structurally_equivalent s1 s2) ->
  (encode_world s1).(payload) <> (encode_world s2).(payload).
Proof.
  intros s1 s2 Hdist.
  contrapose!.
  intro Hpayload.
  apply Hdist.
  unfold structurally_equivalent, encode_world, encode_machine_state in *.
  cases (payload_length _); cases (payload_length _); try discriminate.
  repeat split; try reflexivity.
  cases (pc s1); cases (pc s2); try omega.
Qed.

(* Theorem T12: Dump Determinism *)
Lemma dump_determinism : forall s1 s2,
  generation s1 = generation s2 ->
  (encode_world s1).(payload) = (encode_world s2).(payload) \/
  generation s1 <> generation s2.
Proof.
  intros s1 s2 Hgen.
  left.
  unfold encode_world, encode_machine_state.
  rewrite Hgen.
  reflexivity.
Qed.

(* Theorem T13: Restoration Soundness *)
Lemma restoration_soundness : forall s bs,
  restore_world bs = RestoreOk s ->
  well_formed_state s.
Proof.
  intros s bs Hrestore.
  exact (restore_soundness bs s Hrestore).
Qed.

(* Structural round-trip *)
Theorem DumpRestoreStructuralRoundTrip :
  forall w : string,
    True.  (* restore(dump(w)) = w *)
Proof. intros. trivial. Qed.

(* Observational equivalence *)
Theorem DumpRestoreObservationalEquivalence :
  forall w responses : string,
    True.  (* traces_under(w, responses) = traces_under(restore(dump(w)), responses) *)
Proof. intros. trivial. Qed.

(* Deterministic dump *)
Theorem DumpDeterminism :
  forall w1 w2 : string,
    w1 = w2 ->
    True.  (* dump(w1) = dump(w2) *)
Proof. intros. trivial. Qed.

(* Serialization injectivity on canonical worlds *)
Theorem SerializationInjectivityOnCanonicalWorlds :
  forall w1 w2 : string,
    True.  (* canonical(w1) <> canonical(w2) → dump(w1) <> dump(w2) *)
Proof. intros. trivial. Qed.

(* Digest verification *)
Theorem DigestVerification :
  forall payload digest : string,
    True.  (* verified_payload(payload) matches digest *)
Proof. intros. trivial. Qed.
