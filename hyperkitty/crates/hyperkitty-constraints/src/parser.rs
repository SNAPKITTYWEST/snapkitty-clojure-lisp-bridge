use crate::ast::{ConstraintProgram, Constraint, Requirement, OtherwiseAction};
use crate::lexer::Token;

pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn current(&self) -> Token {
        self.tokens
            .get(self.pos)
            .cloned()
            .unwrap_or(Token::Eof)
    }

    fn peek(&self) -> Token {
        self.tokens
            .get(self.pos + 1)
            .cloned()
            .unwrap_or(Token::Eof)
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }

    fn expect(&mut self, expected: Token) -> hyperkitty_core::Result<()> {
        if std::mem::discriminant(&self.current()) == std::mem::discriminant(&expected) {
            self.advance();
            Ok(())
        } else {
            Err(hyperkitty_core::Error::SyntaxError(
                format!("Expected {:?}, got {:?}", expected, self.current())
            ))
        }
    }

    pub fn parse(&mut self) -> hyperkitty_core::Result<ConstraintProgram> {
        let mut program = ConstraintProgram::new();

        while self.current() != Token::Eof {
            let constraint = self.parse_constraint()?;
            program.add_constraint(constraint);
        }

        Ok(program)
    }

    fn parse_constraint(&mut self) -> hyperkitty_core::Result<Constraint> {
        self.expect(Token::Validity)?;

        // Parse name: identifier in parentheses
        self.expect(Token::LParen)?;
        let name = match self.current() {
            Token::Ident(s) => {
                let n = s.clone();
                self.advance();
                n
            }
            _ => return Err(hyperkitty_core::Error::SyntaxError("Expected constraint name".to_string())),
        };
        self.expect(Token::RParen)?;

        // Parse parameter: usually a variable name or message
        let param = match self.current() {
            Token::Ident(s) => {
                let p = s.clone();
                self.advance();
                p
            }
            Token::String(s) => {
                let p = s.clone();
                self.advance();
                p
            }
            _ => {
                return Err(hyperkitty_core::Error::SyntaxError("Expected constraint parameter".to_string()));
            }
        };

        // Parse body: { require ...; otherwise ... }
        self.expect(Token::LBrace)?;

        let mut constraint = Constraint::new(name, param, OtherwiseAction::Reject);

        // Parse require statements
        while self.current() != Token::Otherwise && self.current() != Token::RBrace {
            self.expect(Token::Require)?;
            let req = self.parse_requirement()?;
            constraint.add_requirement(req);
            self.expect(Token::Semicolon)?;
        }

        // Parse otherwise clause
        if self.current() == Token::Otherwise {
            self.advance();
            let action = match self.current() {
                Token::Reject => {
                    self.advance();
                    OtherwiseAction::Reject
                }
                Token::Accept => {
                    self.advance();
                    OtherwiseAction::Accept
                }
                _ => return Err(hyperkitty_core::Error::SyntaxError(
                    "Expected 'reject' or 'accept' after 'otherwise'".to_string()
                )),
            };
            constraint.otherwise = action;
            self.expect(Token::Semicolon)?;
        }

        self.expect(Token::RBrace)?;

        Ok(constraint)
    }

    fn parse_requirement(&mut self) -> hyperkitty_core::Result<Requirement> {
        match self.current() {
            Token::Ident(s) => {
                let name = s.clone();
                self.advance();
                // Check if this is a check() call or simple predicate
                if self.current() == Token::LParen {
                    self.advance();
                    self.expect(Token::RParen)?;
                    Ok(Requirement::Check(name))
                } else {
                    Ok(Requirement::Predicate(name))
                }
            }
            _ => Err(hyperkitty_core::Error::SyntaxError(
                "Expected requirement predicate or check".to_string()
            )),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    fn lex(source: &str) -> hyperkitty_core::Result<Vec<Token>> {
        let mut lexer = Lexer::new(source);
        lexer.tokenize()
    }

    #[test]
    fn test_parse_simple_constraint() -> hyperkitty_core::Result<()> {
        let source = r#"validity(V1) msg { require always_true(); otherwise reject; }"#;
        let tokens = lex(source)?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse()?;
        assert_eq!(program.constraints.len(), 1);
        assert_eq!(program.constraints[0].name, "V1");
        Ok(())
    }

    #[test]
    fn test_parse_multiple_constraints() -> hyperkitty_core::Result<()> {
        let source = r#"
            validity(V1) msg1 { require p1(); otherwise reject; }
            validity(V2) msg2 { require p2(); otherwise accept; }
        "#;
        let tokens = lex(source)?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse()?;
        assert_eq!(program.constraints.len(), 2);
        assert_eq!(program.constraints[0].otherwise, OtherwiseAction::Reject);
        assert_eq!(program.constraints[1].otherwise, OtherwiseAction::Accept);
        Ok(())
    }

    #[test]
    fn test_parse_multiple_requirements() -> hyperkitty_core::Result<()> {
        let source = r#"validity(V) msg {
            require p1();
            require p2();
            require p3();
            otherwise reject;
        }"#;
        let tokens = lex(source)?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse()?;
        assert_eq!(program.constraints[0].requires.len(), 3);
        Ok(())
    }

    #[test]
    fn test_parse_string_parameter() -> hyperkitty_core::Result<()> {
        let source = r#"validity(V) "error message" { require p(); otherwise reject; }"#;
        let tokens = lex(source)?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse()?;
        assert_eq!(program.constraints[0].param, "error message");
        Ok(())
    }

    #[test]
    fn test_parse_requirement_variations() -> hyperkitty_core::Result<()> {
        let source = r#"validity(V) msg {
            require check1();
            require check2();
            otherwise reject;
        }"#;
        let tokens = lex(source)?;
        let mut parser = Parser::new(tokens);
        let program = parser.parse()?;
        let reqs = &program.constraints[0].requires;
        assert_eq!(reqs.len(), 2);
        match &reqs[0] {
            Requirement::Check(name) => assert_eq!(name, "check1"),
            _ => panic!("Expected Check requirement"),
        }
        Ok(())
    }
}
