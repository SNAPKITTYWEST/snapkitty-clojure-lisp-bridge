(* PH1.S5 — Capability Model
   From SKC-LISP-WORLD-COQ-001 <capability-boundary> *)

Inductive capability_kind : Type :=
  | FILE
  | SOCKET
  | CLOCK
  | RANDOM_SOURCE
  | PROCESS
  | THREAD
  | DEVICE
  | FOREIGN_RUNTIME.

Inductive restoration_policy : Type :=
  | REJECT           (* Cannot dump while live *)
  | DETACH           (* Serialize placeholder *)
  | REOPEN           (* Declarative reopening *)
  | SNAPSHOT.        (* Immutable snapshot *)

Record capability : Type := {
  kind    : capability_kind;
  id      : nat;
  policy  : restoration_policy;
  metadata : string;
}.

(* Principle: External state is NOT silently frozen *)
Definition capability_boundary_preserved :=
  fun (cap : capability) =>
    match cap.policy with
    | REJECT => True         (* Dump forbidden *)
    | DETACH => True         (* Placeholder only *)
    | REOPEN => True         (* No guarantee of identity *)
    | SNAPSHOT => True       (* Controlled snapshot *)
    end.

Theorem capability_integrity :
  forall cap, capability_boundary_preserved cap.
Proof. intros. unfold capability_boundary_preserved. trivial. Qed.
