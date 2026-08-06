//! Jacobian Lens: Condition number and dead path detection

pub mod condition;
pub mod deadpaths;

pub use hyperkitty_core::{Error, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn jacobian_loads() {
        // Placeholder
    }
}
