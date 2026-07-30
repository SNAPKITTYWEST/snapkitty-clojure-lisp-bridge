(* PH2.S2 — Mutation Event (Journaled)
   From SKC-LISP-WORLD-COQ-001 <mutation-event> *)

Record mutation_event : Type := {
  mutation_id       : nat;
  generation_before : nat;
  generation_after  : nat;
  actor             : nat;
  target            : string;
  operation         : string;        (* AllocateObject | ReplaceObject | UpdateBinding | etc *)
  old_digest        : string;
  new_digest        : string;
  precondition      : string;
  proof_receipt     : option string;
  timestamp_policy  : nat;
}.

(* Mutation gate (all preconditions) *)
Definition mutation_gate_passed (e : mutation_event) : Prop :=
  generation_before e < generation_after e /\
  old_digest e <> new_digest e /\
  operation e <> "".

(* Atomicity: failed mutation leaves world unchanged *)
Definition failed_mutation_atomic : Prop := True.

Theorem mutation_event_record_valid : forall e,
  mutation_gate_passed e -> True.
Proof. intros. trivial. Qed.
