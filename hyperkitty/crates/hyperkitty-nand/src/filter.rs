//! NAND Filter — routing conflict resolution engine.
//!
//! Takes a set of expert activations, applies the conflict registry,
//! and outputs only the admitted (non-suppressed) experts.
//!
//! Conflict rule: If `w_A > 0 AND w_B > 0 AND conflict(A, B)`
//! then suppress `argmin(w_A, w_B)`.
//!
//! Deterministic tie-breaking: when weights are equal, suppress the
//! expert with the LOWER ExpertId.

use crate::conflicts::{ConflictRegistry, ExpertId};

/// An expert activation with associated routing weight.
#[derive(Debug, Clone)]
pub struct ExpertActivation {
    /// The expert's identifier
    pub id: ExpertId,
    /// Routing weight (must be > 0 to be considered active)
    pub weight: f64,
}

impl ExpertActivation {
    /// Create a new expert activation
    pub fn new(id: ExpertId, weight: f64) -> Self {
        ExpertActivation { id, weight }
    }
}

/// The NAND Filter: applies conflict suppression to a set of expert activations.
///
/// Given a conflict registry and a set of active experts, deterministically
/// removes experts that violate conflict constraints.
#[derive(Debug, Clone)]
pub struct NANDFilter {
    registry: ConflictRegistry,
}

impl NANDFilter {
    /// Create a new NAND filter backed by the given conflict registry.
    pub fn new(registry: ConflictRegistry) -> Self {
        NANDFilter { registry }
    }

    /// Filter expert activations, suppressing conflicting experts.
    ///
    /// For each conflicting pair (A, B) where both have weight > 0:
    /// - Suppress the one with lower weight
    /// - If weights are equal, suppress the one with LOWER ExpertId
    ///
    /// Processing order is deterministic: conflicts are resolved in
    /// sorted order of expert pairs, and suppression cascades.
    ///
    /// Returns only the admitted (non-suppressed) experts, preserving
    /// their original activation weights.
    pub fn filter(&self, activations: &[ExpertActivation]) -> Vec<ExpertActivation> {
        if activations.is_empty() {
            return Vec::new();
        }

        // Collect only active experts (weight > 0)
        let mut active: Vec<&ExpertActivation> = activations
            .iter()
            .filter(|a| a.weight > 0.0)
            .collect();

        // Sort by ExpertId for deterministic processing
        active.sort_by_key(|a| a.id);

        // Track which experts are suppressed
        let mut suppressed: std::collections::HashSet<ExpertId> = std::collections::HashSet::new();

        // Collect all expert IDs for conflict finding
        let expert_ids: Vec<ExpertId> = active.iter().map(|a| a.id).collect();
        let conflicts = self.registry.find_conflicts(&expert_ids);

        // Resolve each conflict deterministically
        for (id_a, id_b) in &conflicts {
            // Skip if either is already suppressed
            if suppressed.contains(id_a) || suppressed.contains(id_b) {
                continue;
            }

            // Find weights
            let w_a = active
                .iter()
                .find(|a| a.id == *id_a)
                .map(|a| a.weight)
                .unwrap_or(0.0);
            let w_b = active
                .iter()
                .find(|a| a.id == *id_b)
                .map(|a| a.weight)
                .unwrap_or(0.0);

            // Suppress argmin(w_A, w_B)
            // Tie-breaking: equal weights => suppress LOWER ExpertId
            if w_a < w_b {
                suppressed.insert(*id_a);
            } else if w_b < w_a {
                suppressed.insert(*id_b);
            } else {
                // Equal weights: suppress the one with lower ExpertId
                // Since canonical pairs have id_a < id_b, suppress id_a
                suppressed.insert(*id_a);
            }
        }

        // Return non-suppressed activations in original order
        activations
            .iter()
            .filter(|a| a.weight > 0.0 && !suppressed.contains(&a.id))
            .map(|a| ExpertActivation::new(a.id, a.weight))
            .collect()
    }

    /// Get a reference to the underlying conflict registry
    pub fn registry(&self) -> &ConflictRegistry {
        &self.registry
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_registry_with_conflict(a: u64, b: u64) -> ConflictRegistry {
        let mut reg = ConflictRegistry::new();
        reg.register_conflict(ExpertId(a), ExpertId(b));
        reg
    }

    #[test]
    fn test_no_conflicts_passes_all() {
        let reg = ConflictRegistry::new();
        let filter = NANDFilter::new(reg);

        let activations = vec![
            ExpertActivation::new(ExpertId(1), 0.8),
            ExpertActivation::new(ExpertId(2), 0.6),
            ExpertActivation::new(ExpertId(3), 0.9),
        ];

        let result = filter.filter(&activations);
        assert_eq!(result.len(), 3);
    }

    #[test]
    fn test_conflict_suppresses_lower_weight() {
        let reg = make_registry_with_conflict(1, 2);
        let filter = NANDFilter::new(reg);

        let activations = vec![
            ExpertActivation::new(ExpertId(1), 0.8),
            ExpertActivation::new(ExpertId(2), 0.3), // lower weight, gets suppressed
        ];

        let result = filter.filter(&activations);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, ExpertId(1));
    }

    #[test]
    fn test_conflict_suppresses_higher_id_when_lower_weight() {
        let reg = make_registry_with_conflict(1, 2);
        let filter = NANDFilter::new(reg);

        let activations = vec![
            ExpertActivation::new(ExpertId(1), 0.3), // lower weight
            ExpertActivation::new(ExpertId(2), 0.8),
        ];

        let result = filter.filter(&activations);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, ExpertId(2));
    }

    #[test]
    fn test_equal_weights_suppresses_lower_id() {
        let reg = make_registry_with_conflict(1, 2);
        let filter = NANDFilter::new(reg);

        let activations = vec![
            ExpertActivation::new(ExpertId(1), 0.5),
            ExpertActivation::new(ExpertId(2), 0.5),
        ];

        let result = filter.filter(&activations);
        assert_eq!(result.len(), 1);
        // Equal weights: suppress lower ExpertId (1), keep higher (2)
        assert_eq!(result[0].id, ExpertId(2));
    }

    #[test]
    fn test_zero_weight_not_active() {
        let reg = make_registry_with_conflict(1, 2);
        let filter = NANDFilter::new(reg);

        let activations = vec![
            ExpertActivation::new(ExpertId(1), 0.8),
            ExpertActivation::new(ExpertId(2), 0.0), // not active
        ];

        let result = filter.filter(&activations);
        // ExpertId(2) has weight 0, so no conflict triggers
        // Only ExpertId(1) passes (weight > 0)
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, ExpertId(1));
    }

    #[test]
    fn test_multiple_conflicts() {
        let mut reg = ConflictRegistry::new();
        reg.register_conflict(ExpertId(1), ExpertId(2));
        reg.register_conflict(ExpertId(3), ExpertId(4));
        let filter = NANDFilter::new(reg);

        let activations = vec![
            ExpertActivation::new(ExpertId(1), 0.9),
            ExpertActivation::new(ExpertId(2), 0.4),
            ExpertActivation::new(ExpertId(3), 0.3),
            ExpertActivation::new(ExpertId(4), 0.7),
        ];

        let result = filter.filter(&activations);
        assert_eq!(result.len(), 2);
        // (1,2): suppress 2 (lower weight)
        // (3,4): suppress 3 (lower weight)
        let ids: Vec<ExpertId> = result.iter().map(|a| a.id).collect();
        assert!(ids.contains(&ExpertId(1)));
        assert!(ids.contains(&ExpertId(4)));
    }

    #[test]
    fn test_cascade_suppression() {
        // If A conflicts with B and B conflicts with C,
        // and A suppresses B, then B-C conflict doesn't fire
        let mut reg = ConflictRegistry::new();
        reg.register_conflict(ExpertId(1), ExpertId(2));
        reg.register_conflict(ExpertId(2), ExpertId(3));
        let filter = NANDFilter::new(reg);

        let activations = vec![
            ExpertActivation::new(ExpertId(1), 0.9),
            ExpertActivation::new(ExpertId(2), 0.5), // suppressed by conflict with 1
            ExpertActivation::new(ExpertId(3), 0.4), // NOT suppressed because 2 is already gone
        ];

        let result = filter.filter(&activations);
        assert_eq!(result.len(), 2);
        let ids: Vec<ExpertId> = result.iter().map(|a| a.id).collect();
        assert!(ids.contains(&ExpertId(1)));
        assert!(ids.contains(&ExpertId(3)));
    }

    #[test]
    fn test_determinism() {
        let mut reg = ConflictRegistry::new();
        reg.register_conflict(ExpertId(1), ExpertId(2));
        reg.register_conflict(ExpertId(3), ExpertId(4));
        let filter = NANDFilter::new(reg);

        let activations = vec![
            ExpertActivation::new(ExpertId(1), 0.7),
            ExpertActivation::new(ExpertId(2), 0.7),
            ExpertActivation::new(ExpertId(3), 0.5),
            ExpertActivation::new(ExpertId(4), 0.5),
        ];

        // Run multiple times — must always produce same output
        let result1 = filter.filter(&activations);
        let result2 = filter.filter(&activations);
        let result3 = filter.filter(&activations);

        let ids1: Vec<ExpertId> = result1.iter().map(|a| a.id).collect();
        let ids2: Vec<ExpertId> = result2.iter().map(|a| a.id).collect();
        let ids3: Vec<ExpertId> = result3.iter().map(|a| a.id).collect();

        assert_eq!(ids1, ids2);
        assert_eq!(ids2, ids3);

        // Verify the specific deterministic outcome:
        // (1,2) equal weights -> suppress lower ID (1), keep 2
        // (3,4) equal weights -> suppress lower ID (3), keep 4
        assert_eq!(ids1, vec![ExpertId(2), ExpertId(4)]);
    }

    #[test]
    fn test_empty_activations() {
        let reg = make_registry_with_conflict(1, 2);
        let filter = NANDFilter::new(reg);
        let result = filter.filter(&[]);
        assert!(result.is_empty());
    }

    #[test]
    fn test_single_activation_no_suppression() {
        let reg = make_registry_with_conflict(1, 2);
        let filter = NANDFilter::new(reg);

        let activations = vec![ExpertActivation::new(ExpertId(1), 0.9)];
        let result = filter.filter(&activations);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, ExpertId(1));
    }

    #[test]
    fn test_negative_weight_treated_as_inactive() {
        let reg = make_registry_with_conflict(1, 2);
        let filter = NANDFilter::new(reg);

        let activations = vec![
            ExpertActivation::new(ExpertId(1), 0.8),
            ExpertActivation::new(ExpertId(2), -0.5), // negative = not active
        ];

        let result = filter.filter(&activations);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].id, ExpertId(1));
    }

    #[test]
    fn test_preserves_original_weights() {
        let reg = make_registry_with_conflict(1, 2);
        let filter = NANDFilter::new(reg);

        let activations = vec![
            ExpertActivation::new(ExpertId(1), 0.9),
            ExpertActivation::new(ExpertId(2), 0.3),
            ExpertActivation::new(ExpertId(3), 0.7),
        ];

        let result = filter.filter(&activations);
        // ExpertId(2) suppressed, 1 and 3 remain
        assert_eq!(result.len(), 2);
        let expert_1 = result.iter().find(|a| a.id == ExpertId(1)).unwrap();
        let expert_3 = result.iter().find(|a| a.id == ExpertId(3)).unwrap();
        assert!((expert_1.weight - 0.9).abs() < f64::EPSILON);
        assert!((expert_3.weight - 0.7).abs() < f64::EPSILON);
    }
}
