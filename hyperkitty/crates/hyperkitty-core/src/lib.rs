//! HyperKitty Core - Foundational types and utilities

use std::fmt;

pub mod error;
pub mod types;
pub mod canonical;

pub use error::{Error, Result};
pub use types::*;

/// Central invariant: Probability proposes, Proof disposes.
pub const CENTRAL_INVARIANT: &str = "REPLACE PROBABILITY WITH PROOF";

/// Maximum allowed entropy for validity (nats)
pub const MAX_ENTROPY: f64 = 0.20;

/// Golden ratio
pub const GOLDEN_RATIO: f64 = 1.61803398874989484820458683436563811772030917980576;
/// Inverse golden ratio
pub const GOLDEN_RATIO_INV: f64 = 0.61803398874989484820458683436563811772030917980576;
/// Square of inverse golden ratio
pub const GOLDEN_RATIO_INV_SQ: f64 = 0.38196601125010515179541316563436188227969082019424;

/// The six glyph symbols
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Glyph {
    Pi, Gamma, Delta, Omega, Lambda, Psi,
}

impl Glyph {
    pub const fn to_byte(self) -> u8 {
        match self {
            Glyph::Pi => 0x01, Glyph::Gamma => 0x03, Glyph::Delta => 0x04,
            Glyph::Omega => 0x0A, Glyph::Lambda => 0xFF, Glyph::Psi => 0x0B,
        }
    }
    pub fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(Glyph::Pi), 0x03 => Some(Glyph::Gamma), 0x04 => Some(Glyph::Delta),
            0x0A => Some(Glyph::Omega), 0xFF => Some(Glyph::Lambda), 0x0B => Some(Glyph::Psi),
            _ => None,
        }
    }
    pub fn all() -> [Self; 6] {
        [Glyph::Pi, Glyph::Gamma, Glyph::Delta, Glyph::Omega, Glyph::Lambda, Glyph::Psi]
    }
    pub const fn index(self) -> usize {
        match self {
            Glyph::Pi => 0, Glyph::Gamma => 1, Glyph::Delta => 2,
            Glyph::Omega => 3, Glyph::Lambda => 4, Glyph::Psi => 5,
        }
    }
    pub fn by_index(idx: usize) -> Option<Self> {
        match idx { 0 => Some(Glyph::Pi), 1 => Some(Glyph::Gamma), 2 => Some(Glyph::Delta),
                     3 => Some(Glyph::Omega), 4 => Some(Glyph::Lambda), 5 => Some(Glyph::Psi), _ => None }
    }
    pub fn is_absorber(self) -> bool { matches!(self, Glyph::Omega) }
    pub fn is_identity(self) -> bool { matches!(self, Glyph::Lambda) }
}

impl fmt::Display for Glyph {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Glyph::Pi => write!(f, "Pi"), Glyph::Gamma => write!(f, "Gamma"),
            Glyph::Delta => write!(f, "Delta"), Glyph::Omega => write!(f, "Omega"),
            Glyph::Lambda => write!(f, "Lambda"), Glyph::Psi => write!(f, "Psi"),
        }
    }
}

pub fn float_eq(a: f64, b: f64, epsilon: f64) -> bool { (a - b).abs() < epsilon }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_glyph_encoding() {
        for g in Glyph::all() {
            let byte = g.to_byte();
            assert_eq!(Glyph::from_byte(byte), Some(g));
        }
    }
    #[test] fn test_glyph_index() {
        for (i, &g) in Glyph::all().iter().enumerate() {
            assert_eq!(g.index(), i);
            assert_eq!(Glyph::by_index(i), Some(g));
        }
    }
    #[test] fn test_absorber_identity() {
        assert!(Glyph::Omega.is_absorber());
        assert!(Glyph::Lambda.is_identity());
    }
    #[test] fn test_golden_ratio_invariants() {
        assert!(float_eq(GOLDEN_RATIO_INV + GOLDEN_RATIO_INV_SQ, 1.0, 1e-15));
        assert!(float_eq(GOLDEN_RATIO_INV_SQ, GOLDEN_RATIO_INV * GOLDEN_RATIO_INV, 1e-15));
    }
}
