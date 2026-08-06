//! Parser: Lexer and tokenization for constraint DSL

pub mod lexer;
pub mod tokens;

pub use hyperkitty_core::{Error, Result};
pub use tokens::Token;

pub fn tokenize(input: &str) -> Result<Vec<Token>> {
    lexer::lex(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_simple() {
        let tokens = tokenize("require balance").unwrap();
        assert!(!tokens.is_empty());
    }
}
