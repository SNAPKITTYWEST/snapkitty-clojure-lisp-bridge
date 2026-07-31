-- Bifrost.Event — inductive event types and supporting primitives.
--
-- Mirrors the Rust types in bifrost/src/event.rs.
-- All byte arrays are represented as ByteArray (variable-length) rather than
-- fixed-size arrays; the constraint that Cid is 32 bytes is enforced by
-- the smart constructors `Cid.mk` and checked in `Cid.isValid`.

namespace Bifrost

-- ── Cid ─────────────────────────────────────────────────────────────────────

/-- Content Identifier: blake3 hash (32 bytes). -/
structure Cid where
  bytes : ByteArray
  deriving Repr, BEq, Hashable, Inhabited

namespace Cid
/-- The all-zeros CID used as a genesis / null sentinel. -/
def zero : Cid := ⟨ByteArray.mk (Array.mkArray 32 0)⟩

def isZero (c : Cid) : Bool :=
  c.bytes == zero.bytes
end Cid

-- ── PubKey ───────────────────────────────────────────────────────────────────

/-- Ed25519 verifying key (32 bytes). -/
structure PubKey where
  bytes : ByteArray
  deriving Repr, BEq, Inhabited

-- ── Signature ────────────────────────────────────────────────────────────────

/-- Ed25519 signature (64 bytes). -/
structure Signature where
  bytes : ByteArray
  deriving Repr, BEq, Inhabited

-- ── Rights ───────────────────────────────────────────────────────────────────

/-- Capability rights bitfield — matches silverback::Rights. -/
structure Rights where
  bits : UInt8
  deriving Repr, BEq, Inhabited

namespace Rights
def READ     : Rights := ⟨0b00001⟩
def WRITE    : Rights := ⟨0b00010⟩
def INVOKE   : Rights := ⟨0b00100⟩
def DELEGATE : Rights := ⟨0b01000⟩
def SEAL     : Rights := ⟨0b10000⟩
def ALL      : Rights := ⟨0b11111⟩
def NONE     : Rights := ⟨0⟩

def has (r mask : Rights) : Bool := (r.bits &&& mask.bits) == mask.bits
def canDelegate (r : Rights) : Bool := r.has DELEGATE
def restrict (r mask : Rights) : Rights := ⟨r.bits &&& mask.bits⟩
end Rights

-- ── OptLevel ─────────────────────────────────────────────────────────────────

/-- Cranelift optimisation level (0 = none, 1 = speed, 2 = speed_and_size). -/
abbrev OptLevel := UInt8

-- ── Gas ──────────────────────────────────────────────────────────────────────

/-- Fuel units estimated for a JIT-compiled function's execution budget. -/
abbrev Gas := UInt64

-- ── Event ────────────────────────────────────────────────────────────────────

/-- A typed event that can be sealed into the bifrost Merkle-DAG.

    Variants mirror `EventPayload` in bifrost/src/event.rs.
    The `attestation` variant is introduced here; it does not exist in the
    current Rust code but is the target of Iteration 2 (`bifrost-attest`). -/
inductive Event where
  /-- A silverback capability was transferred between souls. -/
  | capTransfer (from to : PubKey) (capHash : Cid) (policyCid : Cid)
  /-- A SoulIR function was JIT-compiled by the Cranelift engine. -/
  | jitCompile  (souliirCid wasmCid : Cid) (optLevel : OptLevel) (gas : Gas)
  /-- An epoch attestation was sealed (introduced in bifrost-attest). -/
  | attestation (epoch : UInt64) (rootCid : Cid) (sig : Signature)
  /-- Lossless session handoff from one agent to another.
      `contextCid` is the WORM-sealed full session state.
      `summaryCid` is an optional lossy human-readable blob (also WORM-sealed).
      `schemaVersion` lets receivers reject formats they don't understand.
      This is the protocol primitive that replaces lossy context summarisation. -/
  | contextHandoff (fromAgent toAgent : PubKey)
                   (contextCid : Cid) (epoch : UInt64)
                   (schemaVersion : UInt32) (summaryCid : Option Cid)
  deriving Repr, Inhabited

end Bifrost
