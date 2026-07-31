/// BifrostBridge — seals soulvm events into the bifrost audit chain.
///
/// `compile()` → `JitCompile` event (SoulIR bytes sealed in WORM_FS first).
/// `mint_cap()` → `CapTransfer` event (zero policy_cid; verify_chain skips WORM
///                check for zero CIDs, so this is valid for runtime-originated mints).
///
/// All errors are swallowed: cap / JIT operations must never fail because of audit.

use std::sync::{Arc, Mutex};

use bifrost::{
    error::BifrostError,
    event::{Cid, EventPayload},
    seal::Bifrost,
};
use silverback::capability::Cap;

use crate::bytecode::SoulFunc;

/// Clone is cheap: shares the same Arc<Mutex<Bifrost>>.
#[derive(Clone)]
pub struct BifrostBridge(Arc<Mutex<Bifrost>>);

impl BifrostBridge {
    /// Wrap a `Bifrost` instance. Seals genesis automatically if the chain is empty.
    ///
    /// The node's verifying key is sealed into WORM_FS here (via `Bifrost::open`
    /// in the normal path, or explicitly here for key-imported chains).
    pub fn new(mut bifrost: Bifrost) -> Self {
        if bifrost.dag.head().ok().flatten().is_none() {
            let _ = bifrost.seal_genesis();
        }
        // Belt-and-suspenders: ensure the pubkey is in WORM even for chains
        // opened with `open_with_key` in tests.
        let pk = bifrost.pubkey();
        match bifrost.worm.seal_blob(&pk.0, None) {
            Ok(_) | Err(bifrost::error::BifrostError::WormViolation { .. }) => {}
            Err(e) => eprintln!("BifrostBridge: seal_pubkey failed: {e}"),
        }
        Self(Arc::new(Mutex::new(bifrost)))
    }

    /// Return the CID of this node's sealed verifying key in WORM_FS.
    pub fn pubkey_cid(&self) -> Option<Cid> {
        self.0.lock().ok().map(|b| b.pubkey_cid())
    }

    /// Seal a `JitCompile` event for the given function.
    ///
    /// Serialises `func` to JSON, seals the bytes into WORM_FS to satisfy
    /// `seal_jit_compile`'s precondition, then commits the event to the chain.
    pub fn seal_jit_compile(&self, func: &SoulFunc) {
        let Ok(mut b) = self.0.lock() else { return };
        let Ok(bytes) = serde_json::to_vec(func) else { return };

        let soulir_cid = match b.worm.seal_blob(&bytes, None) {
            Ok(c) => c,
            // Already sealed (same func compiled twice) — recompute CID from bytes.
            Err(BifrostError::WormViolation { .. }) => Cid::of(&bytes),
            Err(_) => return,
        };
        // wasm_cid is a deterministic proxy: blake3(soulir_bytes || ":native").
        // A production runtime would seal the actual machine-code bytes here.
        let wasm_cid = Cid::of(&[bytes.as_slice(), b":native"].concat());
        let gas = func.body.len() as u64 * 10; // 10 gas units per instruction
        let _ = b.seal_jit_compile(soulir_cid, wasm_cid, 1, gas);
    }

    /// Seal a `CapTransfer` event for a capability minted within `soul_id`'s CSpace.
    ///
    /// Uses `seal_payload` (not `seal_cap_transfer`) so the zero policy_cid is
    /// accepted — runtime mints are rights-gated at the kernel level.
    pub fn seal_cap_transfer(&self, soul_id: u64, cap: &Cap) {
        let Ok(mut b) = self.0.lock() else { return };
        let soul_cid = Cid::of(&soul_id.to_le_bytes());
        let obj_cid  = Cid::of(&cap.object_id.to_le_bytes());
        let cap_hash = Cid::of(&cap_bytes(cap));
        let _ = b.seal_payload(EventPayload::CapTransfer {
            from:       soul_cid,
            to:         obj_cid,
            cap_hash,
            policy_cid: Cid([0u8; 32]), // zero = genesis-class; verify_chain skips WORM check
        });
    }

    /// Run `f` with exclusive access to the underlying `Bifrost`.
    /// Returns `None` if the mutex is poisoned.
    pub fn with_bifrost<T>(&self, f: impl FnOnce(&mut Bifrost) -> T) -> Option<T> {
        self.0.lock().ok().map(|mut b| f(&mut *b))
    }

    /// Expose the shared chain lock so external components (e.g. bifrost-observer)
    /// can hold a reference without depending on soulvm.
    pub fn shared_chain(&self) -> Arc<Mutex<Bifrost>> {
        self.0.clone()
    }
}

/// Stable 17-byte encoding of a `Cap` for CID computation.
/// Fields: object_id (8) + rights bits (1) + badge (8).
fn cap_bytes(cap: &Cap) -> [u8; 17] {
    let mut b = [0u8; 17];
    b[0..8].copy_from_slice(&cap.object_id.to_le_bytes());
    b[8] = cap.rights.0;
    b[9..17].copy_from_slice(&cap.badge.to_le_bytes());
    b
}
