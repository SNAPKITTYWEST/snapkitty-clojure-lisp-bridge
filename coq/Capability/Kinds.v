(* PH4.S1 — All 8 Capability Kinds (Exact from XML) *)

Inductive capability_kind : Type :=
  | FILE
  | SOCKET
  | CLOCK
  | RANDOM_SOURCE
  | PROCESS
  | THREAD
  | DEVICE
  | FOREIGN_RUNTIME.

Definition capability_kind_count : nat := 8.

Inductive restoration_policy : Type :=
  | REJECT
  | DETACH
  | REOPEN
  | SNAPSHOT.
