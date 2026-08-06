//! Conflict Registry — expert conflict declaration and querying.
//!
//! Stores pairwise conflicts between experts. When two experts are declared
//! conflicting, at most one may be active at any time.

use std::collections::HashSet;

/// Expert identifier — a unique 64-bit tag for each expert in the system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ExpertId(pub u64);

impl ExpertId {
    /// Create a new ExpertId
    pub fn new(id: u64) -> Self {
        ExpertId(id)
    }
}

/// Registry of pairwise conflicts between experts.
///
/// A conflict between A and B means at most one of {A, B} may be active
/// in any valid route configuration.
#[derive(Debug, Clone)]
pub struct ConflictRegistry {
    /// Stored as canonical pairs (min, max) to avoid duplicates
    conflicts: HashSet<(ExpertId, ExpertId)>,
}

impl ConflictRegistry {
    /// Create an empty conflict registry
    pub fn new() -> Self {
        ConflictRegistry {
            conflicts: HashSet::new(),
        }
    }

    /// Canonicalize a pair so (min, max) is always stored
    fn canonical(a: ExpertId, b: ExpertId) -> (ExpertId, ExpertId) {
        if a.0 <= b.0 {
            (a, b)
        } else {
            (b, a)
        }
    }

    /// Register a conflict between two experts.
    ///
    /// Declaring a conflict with oneself is a no-op.
    pub fn register_conflict(&mut self, a: ExpertId, b: ExpertId) {
        if a == b {
            return;
        }
        let pair = Self::canonical(a, b);
        self.conflicts.insert(pair);
    }

    /// Check whether two experts are in conflict.
    pub fn are_conflicting(&self, a: ExpertId, b: ExpertId) -> bool {
        if a == b {
            return false;
        }
        let pair = Self::canonical(a, b);
        self.conflicts.contains(&pair)
    }

    /// Find all conflicting pairs within a set of experts.
    ///
    /// Returns pairs in deterministic order (sorted by canonical pair).
    pub fn find_conflicts(&self, experts: &[ExpertId]) -> Vec<(ExpertId, ExpertId)> {
        let mut found = Vec::new();
        let n = experts.len();
        for i in 0..n {
            for j in (i + 1)..n {
                if self.are_conflicting(experts[i], experts[j]) {
                    let pair = Self::canonical(experts[i], experts[j]);
                    found.push(pair);
                }
            }
        }
        found.sort();
        found.dedup();
        found
    }

    /// Return the number of registered conflicts
    pub fn len(&self) -> usize {
        self.conflicts.len()
    }

    /// Check if the registry is empty
    pub fn is_empty(&self) -> bool {
        self.conflicts.is_empty()
    }
}

impl Default for ConflictRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_and_query() {
        let mut reg = ConflictRegistry::new();
        let a = ExpertId(1);
        let b = ExpertId(2);
        let c = ExpertId(3);

        reg.register_conflict(a, b);

        assert!(reg.are_conflicting(a, b));
        assert!(reg.are_conflicting(b, a)); // symmetric
        assert!(!reg.are_conflicting(a, c));
        assert!(!reg.are_conflicting(b, c));
    }

    #[test]
    fn test_self_conflict_is_noop() {
        let mut reg = ConflictRegistry::new();
        let a = ExpertId(1);
        reg.register_conflict(a, a);
        assert!(!reg.are_conflicting(a, a));
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn test_find_conflicts_in_set() {
        let mut reg = ConflictRegistry::new();
        let a = ExpertId(1);
        let b = ExpertId(2);
        let c = ExpertId(3);
        let d = ExpertId(4);

        reg.register_conflict(a, b);
        reg.register_conflict(c, d);

        let experts = vec![a, b, c, d];
        let conflicts = reg.find_conflicts(&experts);
        assert_eq!(conflicts.len(), 2);
        assert!(conflicts.contains(&(a, b)));
        assert!(conflicts.contains(&(c, d)));
    }

    #[test]
    fn test_find_conflicts_subset() {
        let mut reg = ConflictRegistry::new();
        let a = ExpertId(1);
        let b = ExpertId(2);
        let c = ExpertId(3);

        reg.register_conflict(a, b);
        reg.register_conflict(a, c);
        reg.register_conflict(b, c);

        // Only query subset {a, c}
        let conflicts = reg.find_conflicts(&[a, c]);
        assert_eq!(conflicts.len(), 1);
        assert!(conflicts.contains(&(a, c)));
    }

    #[test]
    fn test_no_conflicts() {
        let reg = ConflictRegistry::new();
        let experts = vec![ExpertId(1), ExpertId(2), ExpertId(3)];
        let conflicts = reg.find_conflicts(&experts);
        assert!(conflicts.is_empty());
    }

    #[test]
    fn test_duplicate_registration() {
        let mut reg = ConflictRegistry::new();
        let a = ExpertId(1);
        let b = ExpertId(2);

        reg.register_conflict(a, b);
        reg.register_conflict(a, b);
        reg.register_conflict(b, a);

        assert_eq!(reg.len(), 1);
        assert!(reg.are_conflicting(a, b));
    }

    #[test]
    fn test_many_conflicts() {
        let mut reg = ConflictRegistry::new();
        // Register conflicts between consecutive experts
        for i in 0..10 {
            reg.register_conflict(ExpertId(i), ExpertId(i + 1));
        }
        assert_eq!(reg.len(), 10);
        assert!(reg.are_conflicting(ExpertId(0), ExpertId(1)));
        assert!(reg.are_conflicting(ExpertId(9), ExpertId(10)));
        assert!(!reg.are_conflicting(ExpertId(0), ExpertId(2)));
    }
}
