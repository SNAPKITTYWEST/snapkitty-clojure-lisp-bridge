//! Witness evolution: algebraic token lifetime

use super::glyph::Glyph;
use super::tensor::Tensor;
use crate::Result;
use hyperkitty_core::Error;

/// Witness: w = (w₀, w₁, w₂) where each wᵢ ∈ {glyphs}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Witness([Glyph; 3]);

impl Witness {
    /// Canonical witness: [Π, Γ, Δ]
    pub fn canonical() -> Self {
        Witness([Glyph::Pi, Glyph::Gamma, Glyph::Delta])
    }

    /// Invalid fixed point: [Λ, Λ, Λ] (rejected at issuance)
    pub fn lambda_loop() -> Self {
        Witness([Glyph::Lambda, Glyph::Lambda, Glyph::Lambda])
    }

    /// Create witness, rejecting invalid fixed points
    pub fn new(w0: Glyph, w1: Glyph, w2: Glyph) -> Result<Self> {
        let witness = Witness([w0, w1, w2]);

        // Reject [Λ, Λ, Λ] fixed point
        if witness == Self::lambda_loop() {
            return Err(Error::WitnessExhaustion(
                "Witness [Λ, Λ, Λ] rejected: invalid fixed point".to_string(),
            ));
        }

        Ok(witness)
    }

    /// Evolve: w' = [Q(w₀, w₁), Q(w₁, w₂), Q(w₂, w₀)]
    pub fn evolve(&self) -> Self {
        Witness([
            Tensor::lookup(self.0[0], self.0[1]),
            Tensor::lookup(self.0[1], self.0[2]),
            Tensor::lookup(self.0[2], self.0[0]),
        ])
    }

    /// Check if exhausted: w = [Ω, Ω, Ω]
    pub fn is_exhausted(&self) -> bool {
        self.0[0] == Glyph::Omega
            && self.0[1] == Glyph::Omega
            && self.0[2] == Glyph::Omega
    }

    /// Steps to exhaustion (or None if infinite)
    pub fn steps_to_exhaustion(&self) -> Option<usize> {
        let mut w = *self;
        for steps in 0..1000 {
            if w.is_exhausted() {
                return Some(steps);
            }
            w = w.evolve();
        }
        None
    }

    pub fn as_tuple(&self) -> (Glyph, Glyph, Glyph) {
        (self.0[0], self.0[1], self.0[2])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_exhausts() {
        let w = Witness::canonical();
        let steps = w.steps_to_exhaustion();
        assert_eq!(steps, Some(2)); // Reaches [Ω,Ω,Ω] in 2 steps
    }

    #[test]
    fn lambda_loop_rejected() {
        let result = Witness::new(Glyph::Lambda, Glyph::Lambda, Glyph::Lambda);
        assert!(result.is_err());
    }

    #[test]
    fn valid_creation() {
        let w = Witness::new(Glyph::Pi, Glyph::Gamma, Glyph::Delta).unwrap();
        assert_eq!(w, Witness::canonical());
    }

    #[test]
    fn evolution_trace() {
        let w0 = Witness::canonical();
        let w1 = w0.evolve();
        assert_ne!(w0, w1);
        let w2 = w1.evolve();
        assert!(w2.is_exhausted());
    }
}
