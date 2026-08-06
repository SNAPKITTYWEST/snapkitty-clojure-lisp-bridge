#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    // Keywords
    Validity,
    Require,
    Deny,
    Otherwise,
    Reject,
    Accept,

    // Identifiers and literals
    Ident(String),
    Number(String),
    String(String),

    // Operators and delimiters
    LParen,
    RParen,
    LBrace,
    RBrace,
    Semicolon,
    Colon,
    Comma,
    Arrow,

    // Special
    Eof,
}

pub struct Lexer {
    source: String,
    pos: usize,
    line: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            pos: 0,
            line: 1,
        }
    }

    fn current_char(&self) -> Option<char> {
        self.source[self.pos..].chars().next()
    }

    fn peek_char(&self, offset: usize) -> Option<char> {
        self.source[self.pos + offset..].chars().next()
    }

    fn advance(&mut self) {
        if let Some(ch) = self.current_char() {
            if ch == '\n' {
                self.line += 1;
            }
            self.pos += ch.len_utf8();
        }
    }

    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.current_char() {
            if ch.is_whitespace() {
                self.advance();
            } else if ch == '#' {
                // Comment: skip to end of line
                while let Some(c) = self.current_char() {
                    self.advance();
                    if c == '\n' {
                        break;
                    }
                }
            } else {
                break;
            }
        }
    }

    fn read_string(&mut self, quote: char) -> hyperkitty_core::Result<String> {
        self.advance(); // consume opening quote
        let mut result = String::new();
        while let Some(ch) = self.current_char() {
            if ch == quote {
                self.advance(); // consume closing quote
                return Ok(result);
            } else if ch == '\\' {
                self.advance();
                match self.current_char() {
                    Some('n') => result.push('\n'),
                    Some('t') => result.push('\t'),
                    Some('r') => result.push('\r'),
                    Some('\\') => result.push('\\'),
                    Some('"') => result.push('"'),
                    Some('\'') => result.push('\''),
                    Some(c) => result.push(c),
                    None => return Err(hyperkitty_core::Error::LexerError("Unterminated string escape".to_string())),
                }
                self.advance();
            } else {
                result.push(ch);
                self.advance();
            }
        }
        Err(hyperkitty_core::Error::LexerError("Unterminated string".to_string()))
    }

    fn read_number(&mut self) -> String {
        let mut result = String::new();
        while let Some(ch) = self.current_char() {
            if ch.is_ascii_digit() || ch == '.' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        result
    }

    fn read_ident(&mut self) -> String {
        let mut result = String::new();
        while let Some(ch) = self.current_char() {
            if ch.is_alphanumeric() || ch == '_' {
                result.push(ch);
                self.advance();
            } else {
                break;
            }
        }
        result
    }

    pub fn tokenize(&mut self) -> hyperkitty_core::Result<Vec<Token>> {
        let mut tokens = vec![];

        loop {
            self.skip_whitespace();

            match self.current_char() {
                None => {
                    tokens.push(Token::Eof);
                    break;
                }
                Some('(') => {
                    tokens.push(Token::LParen);
                    self.advance();
                }
                Some(')') => {
                    tokens.push(Token::RParen);
                    self.advance();
                }
                Some('{') => {
                    tokens.push(Token::LBrace);
                    self.advance();
                }
                Some('}') => {
                    tokens.push(Token::RBrace);
                    self.advance();
                }
                Some(';') => {
                    tokens.push(Token::Semicolon);
                    self.advance();
                }
                Some(':') => {
                    tokens.push(Token::Colon);
                    self.advance();
                }
                Some(',') => {
                    tokens.push(Token::Comma);
                    self.advance();
                }
                Some('=') if self.peek_char(1) == Some('>') => {
                    tokens.push(Token::Arrow);
                    self.advance();
                    self.advance();
                }
                Some('"') => {
                    let s = self.read_string('"')?;
                    tokens.push(Token::String(s));
                }
                Some('\'') => {
                    let s = self.read_string('\'')?;
                    tokens.push(Token::String(s));
                }
                Some(ch) if ch.is_ascii_digit() => {
                    let num = self.read_number();
                    tokens.push(Token::Number(num));
                }
                Some(ch) if ch.is_alphabetic() || ch == '_' => {
                    let ident = self.read_ident();
                    let token = match ident.as_str() {
                        "validity" => Token::Validity,
                        "require" => Token::Require,
                        "deny" => Token::Deny,
                        "otherwise" => Token::Otherwise,
                        "reject" => Token::Reject,
                        "accept" => Token::Accept,
                        _ => Token::Ident(ident),
                    };
                    tokens.push(token);
                }
                Some(ch) => {
                    return Err(hyperkitty_core::Error::LexerError(
                        format!("Unexpected character: {} at line {}", ch, self.line)
                    ));
                }
            }
        }

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("validity require deny otherwise reject accept");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Validity);
        assert_eq!(tokens[1], Token::Require);
        assert_eq!(tokens[2], Token::Deny);
        assert_eq!(tokens[3], Token::Otherwise);
        assert_eq!(tokens[4], Token::Reject);
        assert_eq!(tokens[5], Token::Accept);
    }

    #[test]
    fn test_identifiers() {
        let mut lexer = Lexer::new("foo bar_baz _private");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Ident("foo".to_string()));
        assert_eq!(tokens[1], Token::Ident("bar_baz".to_string()));
        assert_eq!(tokens[2], Token::Ident("_private".to_string()));
    }

    #[test]
    fn test_numbers() {
        let mut lexer = Lexer::new("42 3.14 0");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::Number("42".to_string()));
        assert_eq!(tokens[1], Token::Number("3.14".to_string()));
        assert_eq!(tokens[2], Token::Number("0".to_string()));
    }

    #[test]
    fn test_strings() {
        let mut lexer = Lexer::new("\"hello\" 'world'");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::String("hello".to_string()));
        assert_eq!(tokens[1], Token::String("world".to_string()));
    }

    #[test]
    fn test_operators_and_delimiters() {
        let mut lexer = Lexer::new("( ) { } ; : , =>");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0], Token::LParen);
        assert_eq!(tokens[1], Token::RParen);
        assert_eq!(tokens[2], Token::LBrace);
        assert_eq!(tokens[3], Token::RBrace);
        assert_eq!(tokens[4], Token::Semicolon);
        assert_eq!(tokens[5], Token::Colon);
        assert_eq!(tokens[6], Token::Comma);
        assert_eq!(tokens[7], Token::Arrow);
    }
}
