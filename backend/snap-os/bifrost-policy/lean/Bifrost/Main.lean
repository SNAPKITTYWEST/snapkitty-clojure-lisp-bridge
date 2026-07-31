-- Bifrost.Main — package entry + export surface for lean4-rust extraction.
--
-- When `lean4-rust` extraction is wired into the cargo build.rs, this file
-- drives the export: `lean --run BifrostPolicy.export` emits `bifrost_policy.rs`
-- containing a `policy_decide` function that replaces the hand-written stub in
-- bifrost-policy/src/lib.rs.
--
-- Until then, this file serves as the authoritative cross-reference between
-- the Lean spec and the Rust implementation.

import Bifrost.Policy
import Bifrost.Attestation

namespace Bifrost

-- ── Re-export for extraction ──────────────────────────────────────────────────

/-- The sole exported symbol for lean4-rust extraction.
    Signature must match Rust: `fn policy_decide(event: &Event, worm_cids: &[Cid]) → bool` -/
def policyDecide := @decide

-- ── Correctness summary ───────────────────────────────────────────────────────
-- The following list tracks which theorems have sorry-free proofs:
--
--  ✗ decide_sound   — proof obligation (Week 3)
--  ✗ decide_complete — not yet stated (Week 4+)
--  ✗ validAttestationPayload — stated, no decide counterpart yet
--
-- Add `#check @decide_sound` after removing the sorry to verify.

end Bifrost
