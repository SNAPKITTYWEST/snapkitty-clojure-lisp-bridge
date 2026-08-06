use hyperkitty_core::{Glyph, MAX_ENTROPY};
use hyperkitty_qlg::{K_QLG, vec3_from_glyph};

// ── Core object: Λ = (s, δ, ι, ω)  where  ι = −δ  always ──────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Ledger {
    pub s: u64,      // symbolic state hash
    pub delta: i64,  // debit
    pub iota: i64,   // credit — enforced == -delta
    pub omega: i64,  // invariant witness (conserved)
}

impl Ledger {
    /// Canonical constructor: iota is always forced to −delta.
    pub const fn new(s: u64, delta: i64, omega: i64) -> Self {
        Self { s, delta, iota: -delta, omega }
    }

    /// Identity element: zero debit, keeps omega.
    pub const fn identity(omega: i64) -> Self {
        Self::new(0, 0, omega)
    }

    /// Reconciliation operator R(Λ) = δ + ι. Must be 0 for a valid ledger.
    pub fn reconciliation(&self) -> i64 {
        self.delta + self.iota
    }

    pub fn is_balanced(&self) -> bool {
        self.reconciliation() == 0
    }

    /// Binary composition Λa ⊕ Λb.
    /// Requires ωa == ωb (conserved invariant). Returns None on mismatch.
    pub fn compose(&self, other: &Self) -> Option<Self> {
        if self.omega != other.omega {
            return None;
        }
        Some(Self {
            s: self.s.wrapping_add(other.s),
            delta: self.delta + other.delta,
            iota: self.iota + other.iota,   // = -(delta_a + delta_b)
            omega: self.omega,
        })
    }

    /// Evolution step: Λ_{t+1} = L(Λ_t) = Λ_t ⊕ ΔΛ_t
    /// ΔΛ must be balanced and have delta_omega = 0.
    pub fn evolve(&self, increment: Ledger) -> Option<Self> {
        if increment.delta + increment.iota != 0 {
            return None; // increment violates balance
        }
        if increment.omega != 0 {
            return None; // increment must not shift invariant
        }
        let new_s = self.s.wrapping_add(increment.s);
        let new_delta = self.delta + increment.delta;
        Some(Ledger::new(new_s, new_delta, self.omega))
    }

    /// 24-byte canonical wire encoding: [s:8][delta:8][omega:8]
    pub fn encode(&self) -> [u8; 24] {
        let mut out = [0u8; 24];
        out[0..8].copy_from_slice(&self.s.to_le_bytes());
        out[8..16].copy_from_slice(&self.delta.to_le_bytes());
        out[16..24].copy_from_slice(&self.omega.to_le_bytes());
        out
    }

    pub fn decode(bytes: &[u8; 24]) -> Self {
        let s = u64::from_le_bytes(bytes[0..8].try_into().unwrap());
        let delta = i64::from_le_bytes(bytes[8..16].try_into().unwrap());
        let omega = i64::from_le_bytes(bytes[16..24].try_into().unwrap());
        Self::new(s, delta, omega)
    }
}

// ── Geometric accumulator: global sum Σ Λ_i ────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LedgerSum {
    pub s_acc: u64,
    pub delta_acc: i64,
    pub iota_acc: i64,  // always == -delta_acc
    pub omega: i64,
    pub count: usize,
}

impl LedgerSum {
    pub fn new() -> Self {
        Self::default()
    }

    /// Push a ledger point into the accumulator.
    /// Returns None if the point violates balance or mismatches omega.
    pub fn push(&mut self, p: Ledger) -> Option<()> {
        if !p.is_balanced() {
            return None;
        }
        if self.count == 0 {
            self.omega = p.omega;
        } else if p.omega != self.omega {
            return None;
        }
        self.s_acc = self.s_acc.wrapping_add(p.s);
        self.delta_acc += p.delta;
        self.iota_acc = -self.delta_acc;
        self.count += 1;
        Some(())
    }

    /// Global reconciliation check: Σδ + Σι = 0
    pub fn is_balanced(&self) -> bool {
        self.delta_acc + self.iota_acc == 0
    }

    /// 4-byte compact wire frame matching the HK-OS constraint DSL spec.
    /// [prim:1][quad_op:1][lambda_local:1][terminal:1]
    pub fn encode_wire_frame(&self) -> [u8; 4] {
        let inv_flag: u8 = ((self.omega & 0xFF) as u8) << 4;
        let terminal = 0x0A | inv_flag;
        let primitives = (self.delta_acc as u8 & 0x0F)
            | ((self.delta_acc as u8).wrapping_neg() & 0xF0);
        [primitives, 0x0F, 0xFF, terminal]
    }
}

// ── Stable symbol hash (FNV-1a inspired, deterministic) ────────────────────

pub fn hash_symbol(s: &str) -> u64 {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    h
}

// ── Glyph → Ledger canonical mapping ───────────────────────────────────────

pub fn ledger_from_glyph(g: Glyph) -> Ledger {
    let v = vec3_from_glyph(g);
    // delta = x-component of the sphere point; omega = K_QLG = 1
    Ledger::new(g.index() as u64, v.x, K_QLG)
}

// ── Certificate ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SLACertificate {
    pub ledger: Ledger,
    pub is_balanced: bool,
    pub balance_value: i64,
    pub omega_preserved: bool,
    pub omega: i64,
}

impl SLACertificate {
    pub fn new(l: Ledger) -> Self {
        Self {
            is_balanced: l.is_balanced(),
            balance_value: l.reconciliation(),
            omega_preserved: true,
            omega: l.omega,
            ledger: l,
        }
    }
    pub fn validate(&self) -> bool {
        self.is_balanced && self.omega_preserved
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn balance_axiom_enforced_at_construction() {
        let l = Ledger::new(1, 5, 42);
        assert_eq!(l.iota, -5);
        assert!(l.is_balanced());
        assert_eq!(l.reconciliation(), 0);
    }

    #[test]
    fn composition_preserves_balance() {
        let a = Ledger::new(1, 3, 42);
        let b = Ledger::new(2, -2, 42);
        let c = a.compose(&b).unwrap();
        assert!(c.is_balanced());
        assert_eq!(c.delta, 1);
        assert_eq!(c.omega, 42);
    }

    #[test]
    fn composition_rejects_omega_mismatch() {
        let a = Ledger::new(1, 3, 42);
        let b = Ledger::new(2, -2, 99);
        assert!(a.compose(&b).is_none());
    }

    #[test]
    fn evolve_preserves_omega() {
        let base = Ledger::new(1, 5, 42);
        let inc = Ledger::new(2, 3, 0); // delta_omega = 0
        let next = base.evolve(inc).unwrap();
        assert_eq!(next.omega, 42);
        assert!(next.is_balanced());
    }

    #[test]
    fn evolve_rejects_bad_increment() {
        let base = Ledger::new(1, 5, 42);
        // manually construct an unbalanced increment
        let bad = Ledger { s: 1, delta: 3, iota: 0, omega: 0 }; // iota != -delta
        assert!(base.evolve(bad).is_none());
    }

    #[test]
    fn ledger_sum_geometric_accumulation() {
        const INV: i64 = 0x42;
        let p1 = Ledger::new(1, 5, INV);
        let p2 = Ledger::new(2, -3, INV);
        let p3 = Ledger::new(3, 2, INV);
        let mut sum = LedgerSum::new();
        assert!(sum.push(p1).is_some());
        assert!(sum.push(p2).is_some());
        assert!(sum.push(p3).is_some());
        assert_eq!(sum.delta_acc + sum.iota_acc, 0);
        assert_eq!(sum.omega, INV);
        assert_eq!(sum.count, 3);
        let frame = sum.encode_wire_frame();
        assert_eq!(frame[2], 0xFF); // Lambda local flag
        assert_eq!(frame[3] & 0x0F, 0x0A); // Omega terminal glyph
    }

    #[test]
    fn wire_roundtrip() {
        let l = Ledger::new(0xdeadbeef, -7, 99);
        let enc = l.encode();
        let dec = Ledger::decode(&enc);
        assert_eq!(dec, l);
    }

    #[test]
    fn all_glyphs_produce_balanced_ledgers() {
        for g in Glyph::all() {
            let l = ledger_from_glyph(g);
            assert!(l.is_balanced(), "{g} ledger not balanced");
            assert_eq!(l.omega, K_QLG);
        }
    }
}
