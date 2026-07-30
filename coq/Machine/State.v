Require Import Coq.Init.Prelude.
Require Import Coq.Lists.List.
Require Import Coq.Arith.Arith.
Require Import Coq.ZArith.ZArith.
Require Import World.ObjectKinds.

Inductive Instruction : Type :=
  | IConst (idx : nat)
  | ILookup (name : string)
  | IBind (name : string)
  | IPush (v : Value)
  | IPop
  | ICons
  | ICar
  | ICdr
  | ISetCar
  | ISetCdr
  | IMakeClosure (code : CodeId)
  | ICall (arity : nat)
  | ITailCall (arity : nat)
  | IReturn
  | IJump (addr : nat)
  | IJumpIfFalse (addr : nat)
  | IPushFrame
  | IPopFrame
  | ICaptureContination
  | IRestoreContination
  | IRaise
  | IInstallHandler
  | IRemoveHandler
  | IRequestEffect
  | IPatchCode (start : nat) (new_instrs : list Instruction)
  | IDefineCode (code : CodeId)
  | IReplaceFunction (sym : string)
  | IRewriteDispatch
  | ICommitGeneration
  | IRollbackGeneration
  | IHalt.

Inductive MachineStatus : Type :=
  | Running
  | Halted
  | Trapped (reason : string).

Record MachineState : Type := {
  pc : nat;
  current_code : CodeId;
  value_stack : list Value;
  frame_stack : list ObjectId;
  environment : ObjectId;
  status : MachineStatus;
  generation : Generation;
}.

Inductive StepResult : Type :=
  | Stepped (next : MachineState)
  | Emitted (obs : string) (next : MachineState)
  | Requested (effect : string) (next : MachineState)
  | HaltedWith (result : Value) (final : MachineState)
  | TrappedWith (error : string) (final : MachineState).

Definition well_formed_state (s : MachineState) : Prop :=
  match status s with
  | Running => True
  | Halted => True
  | Trapped _ => True
  end.

Lemma well_formed_reflexive : forall s,
  well_formed_state s <-> well_formed_state s.
Proof.
  intro s; split; intro H; exact H.
Qed.
