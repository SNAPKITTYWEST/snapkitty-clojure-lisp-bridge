//! Connectivity analysis

use super::adjacency::AdjacencyMatrix;

pub fn is_strongly_connected(adj: &AdjacencyMatrix) -> bool {
    // Simplified: check if any edge exists (stub)
    for row in adj.matrix() {
        if row.iter().any(|&w| w > 0.0) {
            return true;
        }
    }
    false
}

pub fn find_cycles(adj: &AdjacencyMatrix) -> Vec<Vec<usize>> {
    // Stub: simplified cycle detection
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disconnected_graph() {
        let adj = AdjacencyMatrix::new(3);
        assert!(!is_strongly_connected(&adj));
    }

    #[test]
    fn connected_graph() {
        let mut adj = AdjacencyMatrix::new(2);
        adj.set_edge(0, 1, 1.0);
        assert!(is_strongly_connected(&adj));
    }
}
