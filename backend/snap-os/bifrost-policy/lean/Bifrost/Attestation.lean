-- Bifrost.Attestation — formal spec of epoch attestation validity.
--
-- An attestation chains epochs together and binds each epoch root to a
-- specific kernel measurement (TPM PCRs, dm-verity, bootloader hash).

import Bifrost.State

namespace Bifrost

-- ── KernelMeasurement ────────────────────────────────────────────────────────

/-- Hardware measurement binding an attestation to a specific kernel + firmware.
    All fields are blake3 digests (32 bytes). -/
structure KernelMeasurement where
  pcrDigest      : ByteArray  -- blake3(TPM PCR 0..7)
  dmVerityRoot   : ByteArray  -- dm-verity root hash of /worm
  bootloaderHash : ByteArray  -- blake3(bootloader.elf)
  deriving Repr, BEq, Inhabited

-- ── AttestationPayload ───────────────────────────────────────────────────────

/-- The content that is signed in an epoch attestation. -/
structure AttestationPayload where
  epoch              : UInt64
  epochRootCid       : Cid
  timestampMs        : UInt64
  kernelMeasurement  : KernelMeasurement
  prevAttestationCid : Option Cid
  deriving Repr, BEq, Inhabited

-- ── valid_attestation ────────────────────────────────────────────────────────

/-- An attestation payload is valid given:
    1. The `epochRootCid` matches the DAG root recorded for this epoch in state.
    2. The `kernelMeasurement` matches expected golden values (configurable;
       modelled as an oracle `expectedMeasurement` here).
    3. `prevAttestationCid` is either absent (first attestation) or is present
       in the state's attestation chain.

    Signature verification is excluded from this Prop because it requires
    access to the genesis key (a side-effect / oracle), which is handled in
    the Rust `bifrost-attest::verify_attestation` function. -/
def validAttestationPayload
    (p : AttestationPayload)
    (s : State)
    (expectedMsmt : KernelMeasurement) : Prop :=
  -- 1. Epoch root is recorded in state
  s.epochRoot p.epoch = some p.epochRootCid ∧
  -- 2. Kernel measurement matches golden values
  p.kernelMeasurement == expectedMsmt ∧
  -- 3. Chain of attestations is non-forking (prev must be sealed or absent)
  (match p.prevAttestationCid with
   | none     => True
   | some cid => s.wormSealed cid = true)

end Bifrost
