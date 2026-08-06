pub mod spin_factor;
pub mod idempotent;
pub mod density;
pub mod topology;
pub mod convergence;

pub use spin_factor::SpinFactor;
pub use idempotent::PrimitiveIdempotents;
pub use density::DensityMatrix;
pub use topology::{Topology, find_best_topology};
pub use convergence::ConvergenceTrace;

use hyperkitty_core::{GOLDEN_RATIO_INV, GOLDEN_RATIO_INV_SQ, float_eq};

pub fn validate_coefficient_invariants() -> bool {
    float_eq(GOLDEN_RATIO_INV + GOLDEN_RATIO_INV_SQ, 1.0, 1e-15)
        && float_eq(GOLDEN_RATIO_INV_SQ, GOLDEN_RATIO_INV * GOLDEN_RATIO_INV, 1e-15)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coefficient_invariants_hold() {
        assert!(validate_coefficient_invariants());
    }
}
