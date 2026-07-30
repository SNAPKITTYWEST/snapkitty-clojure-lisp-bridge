(* PH1.S3 — Instruction Set
   From SKC-LISP-WORLD-COQ-001 <instruction-set> *)

Inductive instruction : Type :=
  | CONST                (* Load constant *)
  | LOOKUP               (* Resolve binding *)
  | BIND                 (* Create/replace binding *)
  | PUSH                 (* Push value stack *)
  | POP                  (* Pop value stack *)
  | CONS                 (* Allocate cons *)
  | CAR | CDR            (* Access cons *)
  | SET_CAR | SET_CDR    (* Mutate cons *)
  | MAKE_CLOSURE
  | CALL
  | TAIL_CALL
  | RETURN
  | JUMP
  | JUMP_IF_FALSE
  | PUSH_FRAME
  | POP_FRAME
  | CAPTURE_CONTINUATION
  | RESTORE_CONTINUATION
  | RAISE
  | INSTALL_HANDLER
  | REMOVE_HANDLER
  | REQUEST_EFFECT
  | PATCH_CODE           (* Validated code patch *)
  | DEFINE_CODE
  | REPLACE_FUNCTION
  | REWRITE_DISPATCH
  | COMMIT_GENERATION    (* New world generation *)
  | ROLLBACK_GENERATION
  | HALT.

Definition instruction_count : nat := 30.
