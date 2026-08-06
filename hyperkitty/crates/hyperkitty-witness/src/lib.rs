use hyperkitty_core::{Error, Glyph, Result};
use hyperkitty_qra::{next_glyph, is_absorber, is_identity, evolve_witness, canonical_witness};

/// Witness vector of exactly 3 glyphs
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Witness(pub [Glyph; 3]);

impl Witness {
    pub fn new(g0: Glyph, g1: Glyph, g2: Glyph) -> Self {
        Self([g0, g1, g2])
    }
    
    pub fn as_slice(&self) -> &[Glyph] {
        &self.0
    }
    
    /// Evolve the witness one step
    pub fn evolve(&self) -> Self {
        let evolved = evolve_witness(&self.0);
        Self([evolved[0], evolved[1], evolved[2]])
    }
    
    /// Check if witness is exhausted (all Omega)
    pub fn is_exhausted(&self) -> bool {
        self.0.iter().all(|&g| is_absorber(g))
    }
    
    /// Check if witness is at invalid fixed point
    pub fn is_invalid_fixed_point(&self) -> bool {
        self.0.iter().all(|&g| is_identity(g))
    }
    
    /// Exhaust witness until completion
    pub fn exhaust(mut self) -> WitnessHistory {
        let mut history = vec![self.clone()];
        loop {
            if self.is_exhausted() {
                return WitnessHistory { states: history, exhausted: true, fixed_point: false };
            }
            if self.is_invalid_fixed_point() {
                return WitnessHistory { states: history, exhausted: false, fixed_point: true };
            }
            self = self.evolve();
            history.push(self.clone());
        }
    }
}

/// History of witness evolution
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WitnessHistory {
    pub states: Vec<Witness>,
    pub exhausted: bool,
    pub fixed_point: bool,
}

/// Witness certificate for QLG-certified tokens
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WitnessCertificate {
    pub witness: Witness,
    pub qlg_certified: bool,
    pub issue_time: u64,
    pub expiry: u64,
}

impl WitnessCertificate {
    pub fn new(w: Witness, issue_time: u64) -> Self {
        // QLG-certified tokens have expiry through algebraic exhaustion
        // For now, set expiry based on exhaustion steps
        let expiry = issue_time + 1000; // Placeholder
        Self { witness: w, qlg_certified: true, issue_time, expiry }
    }
    
    pub fn is_valid(&self, current_time: u64) -> bool {
        current_time <= self.expiry && self.qlg_certified
    }
}

/// Canonical witness
pub fn canonical() -> Witness {
    Witness::new(Glyph::Pi, Glyph::Gamma, Glyph::Delta)
}

/// Validate canonical exhaustion
pub fn validate_canonical_exhaustion() -> bool {
    let w0 = canonical();
    let w1 = w0.evolve();
    let w2 = w1.evolve();
    w2.is_exhausted()
}
