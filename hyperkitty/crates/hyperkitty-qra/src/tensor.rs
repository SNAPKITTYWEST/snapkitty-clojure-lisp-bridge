//! QRA Routing Tensor: Deterministic 6×6 transition table

use super::glyph::Glyph;

/// Complete routing tensor Q[current][previous] = next_glyph
/// H = 0 nats (deterministic): same (current, previous) always yields same next
pub struct Tensor;

impl Tensor {
    /// Look up next glyph given current and previous
    pub fn lookup(current: Glyph, previous: Glyph) -> Glyph {
        Self::TABLE[current.index()][previous.index()]
    }

    /// Full routing table: Q[current][previous] = next
    const TABLE: [[Glyph; 6]; 6] = [
        // current=Pi (0)
        [
            Glyph::Pi,
            Glyph::Gamma,
            Glyph::Delta,
            Glyph::Omega,
            Glyph::Pi,
            Glyph::Psi,
        ],
        // current=Gamma (1)
        [
            Glyph::Gamma,
            Glyph::Delta,
            Glyph::Omega,
            Glyph::Lambda,
            Glyph::Gamma,
            Glyph::Psi,
        ],
        // current=Delta (2)
        [
            Glyph::Delta,
            Glyph::Omega,
            Glyph::Lambda,
            Glyph::Psi,
            Glyph::Delta,
            Glyph::Pi,
        ],
        // current=Omega (3) - ABSORBER: always outputs Omega
        [
            Glyph::Omega,
            Glyph::Omega,
            Glyph::Omega,
            Glyph::Omega,
            Glyph::Omega,
            Glyph::Omega,
        ],
        // current=Lambda (4) - LEFT IDENTITY: passes through previous
        [
            Glyph::Pi,
            Glyph::Gamma,
            Glyph::Delta,
            Glyph::Omega,
            Glyph::Lambda,
            Glyph::Psi,
        ],
        // current=Psi (5)
        [
            Glyph::Psi,
            Glyph::Pi,
            Glyph::Gamma,
            Glyph::Delta,
            Glyph::Psi,
            Glyph::Omega,
        ],
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn absorber_property() {
        // Omega absorbs: Q[Ω][j] = Ω for all j
        for prev in super::super::glyph::all_glyphs() {
            assert_eq!(Tensor::lookup(Glyph::Omega, prev), Glyph::Omega);
        }
    }

    #[test]
    fn identity_property() {
        // Lambda is left identity: Q[Λ][j] = j for all j
        for prev in super::super::glyph::all_glyphs() {
            assert_eq!(Tensor::lookup(Glyph::Lambda, prev), prev);
        }
    }

    #[test]
    fn determinism() {
        // Same inputs produce same output
        for _ in 0..100 {
            let c = Glyph::Gamma;
            let p = Glyph::Delta;
            let result1 = Tensor::lookup(c, p);
            let result2 = Tensor::lookup(c, p);
            assert_eq!(result1, result2);
        }
    }

    #[test]
    fn complete_table() {
        // All 36 transitions defined
        let mut count = 0;
        for c in super::super::glyph::all_glyphs() {
            for p in super::super::glyph::all_glyphs() {
                let _ = Tensor::lookup(c, p);
                count += 1;
            }
        }
        assert_eq!(count, 36);
    }
}
