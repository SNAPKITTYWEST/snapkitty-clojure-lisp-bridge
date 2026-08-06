//! AST node types with inverted security model

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeType {
    // Structural (weight=1): carry authority
    Intent,
    Constraint,
    Language,
    Entity,
    Route,
    ProofRequirement,
    StateTransition,
    // Payload (weight=0): cannot control routing
    Word,
    PathRef,
    Number,
    FunctionRef,
    StringLiteral,
    BinaryPayload,
}

impl NodeType {
    pub fn is_structural(&self) -> bool {
        matches!(
            self,
            NodeType::Intent
                | NodeType::Constraint
                | NodeType::Language
                | NodeType::Entity
                | NodeType::Route
                | NodeType::ProofRequirement
                | NodeType::StateTransition
        )
    }

    pub fn is_payload(&self) -> bool {
        !self.is_structural()
    }
}

/// AST Node with weight and metadata
#[derive(Debug, Clone)]
pub struct AstNode {
    pub node_type: NodeType,
    pub label: String,
    pub metadata: Vec<(String, String)>,
}

impl AstNode {
    pub fn new(node_type: NodeType, label: &str) -> Self {
        AstNode {
            node_type,
            label: label.to_string(),
            metadata: Vec::new(),
        }
    }

    /// Get node weight: 1.0 for structural, 0.0 for payload
    pub fn weight(&self) -> f64 {
        if self.node_type.is_structural() {
            1.0
        } else {
            0.0
        }
    }

    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.push((key.to_string(), value.to_string()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn structural_weight() {
        let node = AstNode::new(NodeType::Intent, "test");
        assert_eq!(node.weight(), 1.0);
    }

    #[test]
    fn payload_weight() {
        let node = AstNode::new(NodeType::Word, "hello");
        assert_eq!(node.weight(), 0.0);
    }
}
