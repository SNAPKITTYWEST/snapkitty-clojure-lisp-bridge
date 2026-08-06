//! Token types for constraint DSL

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    // Keywords
    Require,
    Balance,
    Invariant,
    Entropy,
    Proof,
    Runtime,
    Identity,
    Phase,
    Otherwise,
    Reject,
    // Identifiers and literals
    Identifier(String),
    Number(i64),
    String(String),
    // Operators
    LessThanEq,
    Equals,
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Semicolon,
    Comma,
    // Special
    Eof,
}

impl Token {
    pub fn is_keyword(&self) -> bool {
        matches!(
            self,
            Token::Require
                | Token::Balance
                | Token::Invariant
                | Token::Entropy
                | Token::Proof
                | Token::Runtime
                | Token::Identity
                | Token::Phase
                | Token::Otherwise
                | Token::Reject
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn token_classification() {
        assert!(Token::Require.is_keyword());
        assert!(!Token::Identifier("test".to_string()).is_keyword());
    }
}
