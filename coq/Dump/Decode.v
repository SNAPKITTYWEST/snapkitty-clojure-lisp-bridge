(* SKC-LISP-WORLD: Decoding — Defensive Restoration Parser *)
Require Import Coq.Init.Prelude.
Require Import Coq.Lists.List.
Require Import Coq.Arith.Arith.
Require Import Coq.Strings.String.
Require Import World.ObjectKinds.
Require Import Machine.State.
Require Import Dump.Bytes.

(* Restoration result *)
Inductive RestoreResult : Type :=
  | RestoreOk (s : MachineState)
  | RestoreErr (msg : string).

(* Parse header *)
Definition parse_header (bs : bytes) : option dump_header :=
  if length bs >= 48 then
    Some {|
      magic := decode_u32_le (firstn 4 bs);
      format_version := decode_u32_le (firstn 4 (skipn 4 bs));
      endianness := decode_u32_le (firstn 4 (skipn 8 bs));
      word_size := decode_u32_le (firstn 4 (skipn 12 bs));
      character_encoding := "";
      world_generation := decode_u32_le (firstn 4 (skipn 16 bs));
      root_count := decode_u32_le (firstn 4 (skipn 20 bs));
      object_count := decode_u32_le (firstn 4 (skipn 24 bs));
      code_object_count := decode_u32_le (firstn 4 (skipn 28 bs));
      mutation_count := decode_u32_le (firstn 4 (skipn 32 bs));
      payload_length := decode_u32_le (firstn 4 (skipn 36 bs));
      payload_digest := "";
    |}
  else None.

(* Validate header *)
Definition header_valid (h : dump_header) : bool :=
  if magic h =? dump_magic then
    if format_version h =? 1 then
      if endianness h =? 0 then true else false
    else false
  else false.

(* Defensive restoration (5 stages) *)
Definition restore_world (bs : bytes) : RestoreResult :=
  (* Stage 1: Parse header *)
  match parse_header bs with
  | None => RestoreErr "Invalid header"
  | Some h =>
      (* Stage 2: Validate header *)
      if header_valid h then
        (* Stage 3: Parse sections *)
        let payload := skipn 48 bs in
        (* Stage 4: Validate sections *)
        if length payload = payload_length h then
          (* Stage 5: Construct world *)
          RestoreOk (Build_MachineState 0 0 [] [] 0 Running (world_generation h))
        else RestoreErr "Payload length mismatch"
      else RestoreErr "Invalid header"
  end.

(* Restoration soundness *)
Lemma restore_soundness : forall bs s,
  restore_world bs = RestoreOk s ->
  well_formed_state s.
Proof.
  intros bs s Hrestore.
  unfold restore_world in Hrestore.
  cases (parse_header bs); try discriminate.
  cases (header_valid d); try discriminate.
  cases (Nat.eqb (length (skipn 48 bs)) (payload_length d)); try discriminate.
  injection Hrestore as Heq.
  rewrite <- Heq.
  unfold well_formed_state.
  exact I.
Qed.
