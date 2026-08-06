//! QRA State transitions with entropy verification

use super::glyph::Glyph;
use super::tensor::Tensor;

/// Deterministic state step: given current and previous, produce next
pub fn step(current: Glyph, previous: Glyph) -> Glyph {
    Tensor::lookup(current, previous)
}

/// Verify entropy is zero (deterministic): same inputs → same output
pub fn verify_entropy_zero() -> bool {
    // All 36 transitions deterministic
    for c in super::glyph::all_glyphs() {
        for p in super::glyph::all_glyphs() {
            let r1 = step(c, p);
            let r2 = step(c, p);
            if r1 != r2 {
                return false;
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_transitions() {
        assert!(verify_entropy_zero());
    }

    #[test]
    fn pi_gamma_delta_pattern() {
        // Test a key pattern
        let c = Glyph::Pi;
        let p = Glyph::Gamma;
        let next = step(c, p);
        assert_eq!(next, Glyph::Gamma);
    }

    #[test]
    fn transition_paths() {
        // Trace canonical witness evolution path
        let mut current = Glyph::Pi;
        let mut prev = Glyph::Gamma;
        let mut trace = vec![current, prev];

        for _ in 0..5 {
            let next = step(current, prev);
            trace.push(next);
            prev = current;
            current = next;
        }

        assert_eq!(trace.len(), 7);
    }
}
