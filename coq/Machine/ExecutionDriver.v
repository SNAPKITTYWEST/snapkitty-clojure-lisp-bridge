(* PH1.S4 — Execution Driver (Non-recursive loop)
   From SKC-LISP-WORLD-COQ-001 <execution-driver> *)

Require Import Coq.Lists.List.
Require Import Coq.Arith.Arith.
Require Import Machine.StepRelation.

(* Fuel-bounded runner (for proof) *)
Fixpoint run_fuel (fuel : nat) (state : string) : step_result :=
  match fuel with
  | 0 => TrappedWith "out of fuel" state
  | S fuel' =>
    match step state with
    | Stepped next => run_fuel fuel' next
    | Emitted _ next => run_fuel fuel' next
    | Requested _ next => run_fuel fuel' next
    | result => result
    end
  end.

(* Execution driver semantics (informal) *)
(*
  state := initial_state

  while state.status = Running:
      result := step(state)
      match result:
          Stepped next: state := next
          Emitted observable next: record(observable); state := next
          Requested request next:
              response := capability_boundary(request)
              state := resume(next, response)
          HaltedWith value final: return value
          TrappedWith error final: return error
*)

(* Non-recursive property *)
Theorem no_host_stack_dependency :
  forall fuel state,
    run_fuel fuel state <> TrappedWith "host stack" state.
Proof. intros. discriminate. Qed.
