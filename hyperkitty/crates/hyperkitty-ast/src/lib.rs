//! Inverted AST Security Model
//!
//! Payload nodes (weight=0) cannot control routing authority.
//! Structural nodes (weight=1) carry authority.

pub mod nodes;
pub mod weights;
pub mod validation;

pub use hyperkitty_core::{Error, Result};
pub use nodes::{AstNode, NodeType};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_creation() {
        let node = AstNode::new(NodeType::Intent, "test");
        assert!(node.weight() > 0.0);
    }
}
