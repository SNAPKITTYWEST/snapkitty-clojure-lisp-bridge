-- Bifrost.State — runtime state consumed by the policy decision procedure.

import Bifrost.Event

namespace Bifrost

-- ── State ────────────────────────────────────────────────────────────────────

/-- Snapshot of runtime state at the point a new event is being evaluated.

    In production this is reconstructed from the bifrost DAG + WORM_FS;
    in the Lean spec it is an abstract first-class value. -/
structure State where
  /-- CIDs of blobs currently sealed in WORM_FS. -/
  worm        : List Cid
  /-- Capability map: cap_hash → owner pubkey. -/
  caps        : List (Cid × PubKey)
  /-- Current chain head (None before genesis). -/
  head        : Option Cid
  /-- Epoch roots: epoch_number → epoch_root_cid. -/
  epochRoots  : List (UInt64 × Cid)
  /-- Genesis key fingerprint (blake3 of the verifying key bytes). -/
  genesisKeyCid : Option Cid
  deriving Inhabited

namespace State

/-- True iff `cid` is present in WORM_FS. -/
def wormSealed (s : State) (cid : Cid) : Bool :=
  s.worm.any (· == cid)

/-- Look up the owner of a capability by its content-hash. -/
def capOwner (s : State) (capHash : Cid) : Option PubKey :=
  (s.caps.find? fun (c, _) => c == capHash).map (·.2)

/-- True iff the capability identified by `capHash` has a registered owner. -/
def capExists (s : State) (capHash : Cid) : Bool :=
  (s.capOwner capHash).isSome

/-- Look up the epoch root CID for a given epoch. -/
def epochRoot (s : State) (epoch : UInt64) : Option Cid :=
  (s.epochRoots.find? fun (e, _) => e == epoch).map (·.2)

/-- Empty initial state (before genesis). -/
def empty : State :=
  { worm := [], caps := [], head := none, epochRoots := [], genesisKeyCid := none }

end State

end Bifrost
