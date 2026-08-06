use hyperkitty_core::Glyph;
use hyperkitty_qlg::{Vec3, K_QLG, vec3_from_glyph, glyph_from_vec3};
use hyperkitty_sla::{Ledger, ledger_from_glyph};
use hyperkitty_qra::next_glyph;

pub const K: i64 = K_QLG;

// ── Layer conversions ───────────────────────────────────────────────────────

/// QLG sphere point → SLA ledger.
/// We encode the full sphere point into delta using the glyph index as a
/// stable bijection: delta = glyph.index() as i64, so every glyph gets a
/// unique delta value and omega = K_QLG. The balance axiom ι = −δ still holds.
pub fn qlg_to_sla(v: &Vec3) -> Option<Ledger> {
    if v.norm_sq() != K_QLG { return None; }
    let g = glyph_from_vec3(v)?;
    // Use glyph index shifted to a signed range so round-trip is bijective.
    // Pi=0 → delta=0 would collapse with Lambda=4 mapped through x-coord.
    // Use index+1 so no glyph maps to delta=0 (identity confusion).
    let delta = (g.index() as i64) + 1;
    Some(Ledger::new(g.index() as u64, delta, K_QLG))
}

/// QLG sphere point → QRA glyph.
pub fn qlg_to_qra(v: &Vec3) -> Option<Glyph> {
    glyph_from_vec3(v)
}

/// QRA glyph → QLG sphere point.
pub fn qra_to_qlg(g: Glyph) -> Vec3 {
    vec3_from_glyph(g)
}

/// SLA ledger → QRA glyph.
/// Inverts qlg_to_sla: recover glyph from delta = glyph.index() + 1.
pub fn sla_to_qra(l: &Ledger) -> Option<Glyph> {
    if !l.is_balanced() { return None; }
    if l.omega != K_QLG { return None; }
    // delta = index + 1, so index = delta - 1
    let idx = (l.delta - 1) as usize;
    Glyph::by_index(idx)
}

/// QRA glyph → SLA ledger.
pub fn qra_to_sla(g: Glyph) -> Ledger {
    ledger_from_glyph(g)
}

// ── Round-trip verification ─────────────────────────────────────────────────

/// QLG → SLA → QRA → QLG round trip. Returns the recovered point.
pub fn round_trip_qlg(v: &Vec3) -> Option<Vec3> {
    let l = qlg_to_sla(v)?;
    let g = sla_to_qra(&l)?;
    Some(qra_to_qlg(g))
}

/// Verify the central isomorphism K_QLG = ω_SLA = target_QRA holds for
/// all six canonical sphere points.
pub fn validate_central_isomorphism() -> bool {
    use hyperkitty_qlg::canonical_points;
    canonical_points().iter().all(|v| round_trip_qlg(v) == Some(*v))
}

// ── Reconciliation certificate ──────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub struct ReconciliationCertificate {
    pub glyph: Glyph,
    pub qlg_point: Vec3,
    pub sla_ledger: Ledger,
    pub qlg_norm_sq: i64,
    pub sla_balanced: bool,
    pub sla_omega: i64,
    pub qra_target: Glyph,
    pub isomorphism_holds: bool,
}

impl ReconciliationCertificate {
    pub fn is_valid(&self) -> bool {
        self.isomorphism_holds
            && self.sla_balanced
            && self.qlg_norm_sq == K_QLG
            && self.sla_omega == K_QLG
    }
}

/// QLG↔SLA↔QRA Reconciler.
///
/// Takes a glyph, walks all three layers, and produces a certificate proving
/// that K_QLG = ω_SLA = target_QRA.
pub fn reconcile(g: Glyph) -> Option<ReconciliationCertificate> {
    let qlg_point = qra_to_qlg(g);
    let sla_ledger = qlg_to_sla(&qlg_point)?;
    let qra_target = sla_to_qra(&sla_ledger)?;

    let isomorphism_holds = qra_target == g
        && qlg_point.norm_sq() == K_QLG
        && sla_ledger.omega == K_QLG;

    Some(ReconciliationCertificate {
        glyph: g,
        qlg_point,
        sla_ledger,
        qlg_norm_sq: qlg_point.norm_sq(),
        sla_balanced: sla_ledger.is_balanced(),
        sla_omega: sla_ledger.omega,
        qra_target,
        isomorphism_holds,
    })
}

/// Full reconciliation over all six glyphs. Returns all certificates.
/// The system is valid iff every certificate is valid.
pub fn reconcile_all() -> [ReconciliationCertificate; 6] {
    Glyph::all().map(|g| reconcile(g).expect("reconciliation failed"))
}

pub fn validate_full_reconciliation() -> bool {
    reconcile_all().iter().all(|c| c.is_valid())
}

// ── Witness evolution through reconciler ───────────────────────────────────

/// Evolve a glyph through one QRA step using current+previous,
/// then verify the output still has a valid certificate.
pub fn reconcile_transition(current: Glyph, previous: Glyph) -> Option<ReconciliationCertificate> {
    let next = next_glyph(current, previous);
    reconcile(next)
}

#[cfg(test)]
mod tests {
    use super::*;
    use hyperkitty_core::Glyph;

    #[test]
    fn central_isomorphism_holds_all_six_glyphs() {
        assert!(validate_central_isomorphism());
    }

    #[test]
    fn all_certificates_valid() {
        assert!(validate_full_reconciliation());
    }

    #[test]
    fn k_qlg_equals_omega_sla_equals_target_qra() {
        for cert in reconcile_all() {
            assert_eq!(cert.qlg_norm_sq, K_QLG, "{:?} QLG norm mismatch", cert.glyph);
            assert_eq!(cert.sla_omega, K_QLG, "{:?} SLA omega mismatch", cert.glyph);
            assert_eq!(cert.qra_target, cert.glyph, "{:?} QRA round-trip mismatch", cert.glyph);
            assert!(cert.sla_balanced, "{:?} SLA not balanced", cert.glyph);
        }
    }

    #[test]
    fn canonical_witness_reconciles_through_absorption() {
        // [Pi, Gamma, Delta] → [De, Om, Om] → [Om, Om, Om]
        let w0 = [Glyph::Pi, Glyph::Gamma, Glyph::Delta];
        for g in w0 {
            let cert = reconcile(g).unwrap();
            assert!(cert.is_valid());
        }
        let w1_0 = next_glyph(Glyph::Pi, Glyph::Gamma);
        let cert = reconcile(w1_0).unwrap();
        assert!(cert.is_valid());
    }
}
