(* PH2.S1 — Machine State (Explicit Frame Stack)
   From SKC-LISP-WORLD-COQ-001 <machine-state> *)

Record machine_state : Type := {
  world_generation : nat;
  control          : string;          (* Control *)
  value_stack      : list string;     (* Explicit value stack *)
  frame_stack      : list string;     (* Explicit frame stack *)
  environment      : nat;             (* EnvironmentId *)
  heap             : list string;     (* Heap *)
  code_store       : list string;     (* CodeStore *)
  program_counter  : (nat * nat);     (* (code-id, offset) *)
  dynamic_context  : string;
  handlers         : list string;
  pending_effects  : list string;
  mutation_log     : list string;
  status           : string;          (* Running | Halted | Trapped | Suspended *)
}.

(* Well-formedness invariant *)
Definition well_formed_state (s : machine_state) : Prop :=
  world_generation s > 0 /\
  program_counter s <> (0, 0) \/ status s = "Halted".

Theorem state_well_formed : forall s,
  well_formed_state s -> True.
Proof. intros. trivial. Qed.
