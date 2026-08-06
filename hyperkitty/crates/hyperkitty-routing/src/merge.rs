//! Output merging (Stage 11)

pub struct MergePolicy {
    merge_all: bool,
    dedup: bool,
}

impl MergePolicy {
    pub fn new() -> Self {
        MergePolicy {
            merge_all: true,
            dedup: true,
        }
    }

    pub fn merge(&self, results: Vec<String>) -> String {
        let deduplicated: std::collections::HashSet<_> = if self.dedup {
            results.into_iter().collect()
        } else {
            results.into_iter().collect()
        };

        deduplicated.into_iter().collect::<Vec<_>>().join(";")
    }
}

impl Default for MergePolicy {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merge_results() {
        let policy = MergePolicy::new();
        let results = vec!["a".to_string(), "b".to_string()];
        let merged = policy.merge(results);
        assert!(!merged.is_empty());
    }
}
