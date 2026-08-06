//! Weight computation and edge rules

use super::nodes::{AstNode, NodeType};

/// Edge weight rules for inverted AST
pub fn edge_weight(from: &AstNode, to: &AstNode) -> f64 {
    match (from.node_type.is_payload(), to.node_type.is_structural()) {
        // payload → structural = 0.0 (BLOCKED)
        (true, true) => 0.0,
        // structural → payload = 0.1 (muted)
        (false, true) => 0.1,
        // structural → structural = computed routing weight
        (false, false) => 0.5,
        // payload → payload = 0.0
        (true, false) => 0.0,
    }
}

/// Verify dangerous patterns (payload trying to control routing)
pub fn has_dangerous_pattern(from: &AstNode, to: &AstNode) -> bool {
    // Dangerous: payload node pointing to structural authority node
    from.node_type.is_payload() && to.node_type.is_structural()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn payload_cannot_reach_structural() {
        let payload = AstNode::new(NodeType::Word, "hello");
        let structural = AstNode::new(NodeType::Intent, "route");
        assert_eq!(edge_weight(&payload, &structural), 0.0);
        assert!(has_dangerous_pattern(&payload, &structural));
    }

    #[test]
    fn structural_can_reach_payload() {
        let structural = AstNode::new(NodeType::Intent, "route");
        let payload = AstNode::new(NodeType::Word, "hello");
        assert_eq!(edge_weight(&structural, &payload), 0.1);
        assert!(!has_dangerous_pattern(&structural, &payload));
    }
}
