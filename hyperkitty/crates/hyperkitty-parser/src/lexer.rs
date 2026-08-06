//! Lexical analyzer for constraint DSL

use super::tokens::Token;
use crate::Result;
use hyperkitty_core::Error;

pub fn lex(input: &str) -> Result<Vec<Token>> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            '(' => {
                tokens.push(Token::LeftParen);
                chars.next();
            }
            ')' => {
                tokens.push(Token::RightParen);
                chars.next();
            }
            '{' => {
                tokens.push(Token::LeftBrace);
                chars.next();
            }
            '}' => {
                tokens.push(Token::RightBrace);
                chars.next();
            }
            ';' => {
                tokens.push(Token::Semicolon);
                chars.next();
            }
            ',' => {
                tokens.push(Token::Comma);
                chars.next();
            }
            '=' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    tokens.push(Token::Equals);
                    chars.next();
                } else {
                    tokens.push(Token::Equals);
                }
            }
            '<' => {
                chars.next();
                if chars.peek() == Some(&'=') {
                    tokens.push(Token::LessThanEq);
                    chars.next();
                }
            }
            '"' => {
                chars.next();
                let mut s = String::new();
                while let Some(&c) = chars.peek() {
                    if c == '"' {
                        chars.next();
                        break;
                    }
                    s.push(c);
                    chars.next();
                }
                tokens.push(Token::String(s));
            }
            c if c.is_ascii_digit() => {
                let mut num = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() {
                        num.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                if let Ok(n) = num.parse::<i64>() {
                    tokens.push(Token::Number(n));
                }
            }
            c if c.is_alphabetic() || c == '_' => {
                let mut ident = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                let token = match ident.as_str() {
                    "require" => Token::Require,
                    "balance" => Token::Balance,
                    "invariant" => Token::Invariant,
                    "entropy" => Token::Entropy,
                    "proof" => Token::Proof,
                    "runtime" => Token::Runtime,
                    "identity" => Token::Identity,
                    "phase" => Token::Phase,
                    "otherwise" => Token::Otherwise,
                    "reject" => Token::Reject,
                    _ => Token::Identifier(ident),
                };
                tokens.push(token);
            }
            _ => {
                chars.next();
            }
        }
    }

    tokens.push(Token::Eof);
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lex_keywords() {
        let tokens = lex("require balance").unwrap();
        assert_eq!(tokens[0], Token::Require);
        assert_eq!(tokens[1], Token::Balance);
    }

    #[test]
    fn lex_number() {
        let tokens = lex("42").unwrap();
        assert_eq!(tokens[0], Token::Number(42));
    }

    #[test]
    fn lex_string() {
        let tokens = lex("\"hello\"").unwrap();
        assert!(matches!(tokens[0], Token::String(ref s) if s == "hello"));
    }
}
