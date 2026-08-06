//! AST validation and dangerous pattern detection

use super::nodes::AstNode;
use crate::Result;
use hyperkitty_core::Error;

/// Validate AST structure before construction
pub fn validate(nodes: &[AstNode], edges: &[(usize, usize)]) -> Result<()> {
    // Check for dangerous patterns
    for &(from_idx, to_idx) in edges {
        if from_idx >= nodes.len() || to_idx >= nodes.len() {
            return Err(Error::InvalidAST("Edge index out of bounds".to_string()));
        }

        let from = &nodes[from_idx];
        let to = &nodes[to_idx];

        if super::weights::has_dangerous_pattern(from, to) {
            return Err(Error::InvalidAST(format!(
                "Dangerous pattern: {} → {}",
                from.label, to.label
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::nodes::NodeType;

    #[test]
    fn valid_ast() {
        let nodes = vec![
            AstNode::new(NodeType::Intent, "route"),
            AstNode::new(NodeType::Word, "hello"),
        ];
        let edges = vec![(0, 1)]; // structural → payload (OK)
        assert!(validate(&nodes, &edges).is_ok());
    }

    #[test]
    fn invalid_ast_dangerous() {
        let nodes = vec![
            AstNode::new(NodeType::Word, "hello"),
            AstNode::new(NodeType::Intent, "route"),
        ];
        let edges = vec![(0, 1)]; // payload → structural (BLOCKED)
        assert!(validate(&nodes, &edges).is_err());
    }
}
