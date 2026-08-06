//! Symbolic Graph: AST to adjacency matrix conversion

pub mod adjacency;
pub mod connectivity;

pub use hyperkitty_core::{Error, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn graph_module_loads() {
        // Placeholder
    }
}
