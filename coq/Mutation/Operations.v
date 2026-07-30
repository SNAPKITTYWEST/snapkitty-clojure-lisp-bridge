(* PH4.S1 — All 11 Mutation Operations (Exact from XML) *)

Inductive mutation_operation : Type :=
  | AllocateObject
  | ReplaceObject
  | UpdateBinding
  | PatchCodeRange
  | InstallCodeObject
  | ReplaceFunctionCell
  | RewriteDispatchEntry
  | InstallMacro
  | RemoveBinding
  | CommitWorldGeneration
  | RollbackWorldGeneration.

Definition mutation_operation_count : nat := 11.
