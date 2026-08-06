use wasm_bindgen::prelude::*;
use hyperkitty_core::Glyph;
use hyperkitty_qra::{next_glyph, evolve_witness, validate_canonical_exhaustion, Q};
use hyperkitty_sla::{Ledger, LedgerSum, hash_symbol, ledger_from_glyph};
use hyperkitty_qlg::{vec3_from_glyph, K_QLG};
use hyperkitty_isomorphism::{reconcile, validate_full_reconciliation, ReconciliationCertificate};

// ── Glyph helpers ──────────────────────────────────────────────────────────

fn glyph_from_index(i: u8) -> Option<Glyph> {
    Glyph::by_index(i as usize)
}

fn glyph_name(g: Glyph) -> &'static str {
    match g {
        Glyph::Pi => "Pi", Glyph::Gamma => "Gamma", Glyph::Delta => "Delta",
        Glyph::Omega => "Omega", Glyph::Lambda => "Lambda", Glyph::Psi => "Psi",
    }
}

// ── QRA: tensor dispatch ───────────────────────────────────────────────────

/// Q[current][previous] — deterministic next glyph index.
/// Returns 255 for invalid input.
#[wasm_bindgen]
pub fn qra_next(current: u8, previous: u8) -> u8 {
    let curr = match glyph_from_index(current) { Some(g) => g, None => return 255 };
    let prev = match glyph_from_index(previous) { Some(g) => g, None => return 255 };
    next_glyph(curr, prev).index() as u8
}

/// Return glyph name for an index (0-5).
#[wasm_bindgen]
pub fn glyph_name_for(index: u8) -> String {
    match glyph_from_index(index) {
        Some(g) => glyph_name(g).to_string(),
        None => "Unknown".to_string(),
    }
}

/// Wire byte encoding for a glyph index.
#[wasm_bindgen]
pub fn glyph_wire_byte(index: u8) -> u8 {
    match glyph_from_index(index) {
        Some(g) => g.to_byte(),
        None => 0,
    }
}

/// Evolve a 3-glyph witness one step. Input/output as flat [curr0,prev0, curr1,prev1, curr2,prev2].
/// Returns [next0, next1, next2] (3 bytes) or empty on bad input.
#[wasm_bindgen]
pub fn qra_evolve_witness(w0: u8, w1: u8, w2: u8) -> Vec<u8> {
    let g0 = match glyph_from_index(w0) { Some(g) => g, None => return vec![] };
    let g1 = match glyph_from_index(w1) { Some(g) => g, None => return vec![] };
    let g2 = match glyph_from_index(w2) { Some(g) => g, None => return vec![] };
    let w = evolve_witness(&[g0, g1, g2]);
    w.iter().map(|g| g.index() as u8).collect()
}

/// Returns true if canonical witness [Pi,Gamma,Delta] exhausts in exactly 2 steps.
#[wasm_bindgen]
pub fn qra_validate_exhaustion() -> bool {
    validate_canonical_exhaustion()
}

/// Full 6x6 Q tensor as flat 36-byte array (row-major).
#[wasm_bindgen]
pub fn qra_tensor() -> Vec<u8> {
    Q.iter().flat_map(|row| row.iter().map(|&v| v as u8)).collect()
}

// ── SLA: ledger operations ─────────────────────────────────────────────────

/// Create a balanced ledger. Returns 24-byte encoding [s:8][delta:8][omega:8].
#[wasm_bindgen]
pub fn sla_ledger_new(s: u32, delta: i32, omega: i32) -> Vec<u8> {
    let l = Ledger::new(s as u64, delta as i64, omega as i64);
    l.encode().to_vec()
}

/// Compose two 24-byte encoded ledgers. Returns 24-byte result or empty on omega mismatch.
#[wasm_bindgen]
pub fn sla_compose(a: &[u8], b: &[u8]) -> Vec<u8> {
    if a.len() < 24 || b.len() < 24 { return vec![]; }
    let la = Ledger::decode(a[0..24].try_into().unwrap());
    let lb = Ledger::decode(b[0..24].try_into().unwrap());
    match la.compose(&lb) {
        Some(lc) => lc.encode().to_vec(),
        None => vec![],
    }
}

/// Evolve a 24-byte ledger by a 24-byte increment. Returns result or empty on violation.
#[wasm_bindgen]
pub fn sla_evolve(base: &[u8], increment: &[u8]) -> Vec<u8> {
    if base.len() < 24 || increment.len() < 24 { return vec![]; }
    let lb = Ledger::decode(base[0..24].try_into().unwrap());
    let li = Ledger::decode(increment[0..24].try_into().unwrap());
    match lb.evolve(li) {
        Some(next) => next.encode().to_vec(),
        None => vec![],
    }
}

/// Is a 24-byte encoded ledger balanced?
#[wasm_bindgen]
pub fn sla_is_balanced(encoded: &[u8]) -> bool {
    if encoded.len() < 24 { return false; }
    Ledger::decode(encoded[0..24].try_into().unwrap()).is_balanced()
}

/// Hash a symbol string to u64, returned as hex string.
#[wasm_bindgen]
pub fn sla_hash_symbol(s: &str) -> String {
    format!("{:016x}", hash_symbol(s))
}

/// Get the canonical SLA ledger for a glyph index. Returns 24-byte encoding.
#[wasm_bindgen]
pub fn sla_ledger_for_glyph(index: u8) -> Vec<u8> {
    match glyph_from_index(index) {
        Some(g) => ledger_from_glyph(g).encode().to_vec(),
        None => vec![],
    }
}

/// Accumulate multiple 24-byte ledgers. Returns 4-byte wire frame.
#[wasm_bindgen]
pub fn sla_accumulate(ledgers_flat: &[u8]) -> Vec<u8> {
    if ledgers_flat.len() % 24 != 0 { return vec![]; }
    let mut sum = LedgerSum::new();
    let chunks = ledgers_flat.chunks_exact(24);
    for chunk in chunks {
        let l = Ledger::decode(chunk.try_into().unwrap());
        if sum.push(l).is_none() { return vec![]; }
    }
    sum.encode_wire_frame().to_vec()
}

// ── QLG: geometry ──────────────────────────────────────────────────────────

/// Get the QLG sphere point [x, y, z] for a glyph index.
#[wasm_bindgen]
pub fn qlg_point_for_glyph(index: u8) -> Vec<i32> {
    match glyph_from_index(index) {
        Some(g) => {
            let v = vec3_from_glyph(g);
            vec![v.x as i32, v.y as i32, v.z as i32]
        }
        None => vec![],
    }
}

/// Norm squared of the QLG point for a glyph (should always be 1).
#[wasm_bindgen]
pub fn qlg_norm_sq(index: u8) -> i32 {
    match glyph_from_index(index) {
        Some(g) => vec3_from_glyph(g).norm_sq() as i32,
        None => -1,
    }
}

// ── Reconciler: K_QLG = ω_SLA = target_QRA ────────────────────────────────

/// Reconcile a glyph through all three layers.
/// Returns JSON string: {glyph, qlg_norm_sq, sla_balanced, sla_omega, qra_target, valid}
#[wasm_bindgen]
pub fn reconcile_glyph(index: u8) -> String {
    match glyph_from_index(index) {
        None => r#"{"error":"invalid glyph index"}"#.to_string(),
        Some(g) => match reconcile(g) {
            None => r#"{"error":"reconciliation failed"}"#.to_string(),
            Some(cert) => format!(
                r#"{{"glyph":"{}","qlg_norm_sq":{},"sla_balanced":{},"sla_omega":{},"qra_target":"{}","valid":{}}}"#,
                glyph_name(cert.glyph),
                cert.qlg_norm_sq,
                cert.sla_balanced,
                cert.sla_omega,
                glyph_name(cert.qra_target),
                cert.is_valid()
            ),
        },
    }
}

/// Validate the full tripartite isomorphism K_QLG = ω_SLA = target_QRA for all 6 glyphs.
#[wasm_bindgen]
pub fn validate_isomorphism() -> bool {
    validate_full_reconciliation()
}

/// Get reconciliation JSON for all 6 glyphs.
#[wasm_bindgen]
pub fn reconcile_all_json() -> String {
    let certs: Vec<String> = Glyph::all().iter().filter_map(|&g| {
        reconcile(g).map(|cert| format!(
            r#"{{"glyph":"{}","qlg_norm_sq":{},"sla_balanced":{},"sla_omega":{},"qra_target":"{}","valid":{}}}"#,
            glyph_name(cert.glyph),
            cert.qlg_norm_sq,
            cert.sla_balanced,
            cert.sla_omega,
            glyph_name(cert.qra_target),
            cert.is_valid()
        ))
    }).collect();
    format!("[{}]", certs.join(","))
}

// ── Agent routing: wire 16 agents A-P through QRA ─────────────────────────

/// Route a message from agent `from_idx` (0-15) to the next agent via QRA.
/// Returns {next_agent, glyph_current, glyph_next, wire_byte, worm_id}
#[wasm_bindgen]
pub fn agent_route(from_idx: u8, prev_glyph_idx: u8) -> String {
    // Map agent 0-15 into glyph space 0-5 (mod 6)
    let curr_glyph_idx = (from_idx % 6) as u8;
    let next_glyph_idx = qra_next(curr_glyph_idx, prev_glyph_idx % 6);
    // Map next glyph back to agent index (glyph 3=Omega → terminal, glyph 4=Lambda → identity)
    let next_agent = if next_glyph_idx == 3 {
        // Omega: converge to agent 0 (absorber)
        0u8
    } else {
        // Map glyph index to agent, wrapping within 16
        (next_glyph_idx as u16 * 3 % 16) as u8
    };
    let wire = glyph_wire_byte(next_glyph_idx);
    // Simple WORM ID: xor of inputs + agent index (deterministic)
    let worm_id = ((from_idx as u32) << 16) | ((prev_glyph_idx as u32) << 8) | next_agent as u32;
    format!(
        r#"{{"from_agent":{},"next_agent":{},"glyph_current":"{}","glyph_next":"{}","wire_byte":"0x{:02X}","worm_id":"0x{:06X}","absorbed":{}}}"#,
        from_idx,
        next_agent,
        glyph_name_for(curr_glyph_idx),
        glyph_name_for(next_glyph_idx),
        wire,
        worm_id,
        next_glyph_idx == 3
    )
}
