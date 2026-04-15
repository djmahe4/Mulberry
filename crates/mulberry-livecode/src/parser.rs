use crate::ast::PatternNode;
use crate::error::LiveCodeError;
use crate::lexer::{Lexer, Token};

/// Recursive-descent parser for the pattern mini-language.
///
/// Grammar (informal):
/// ```text
/// pattern    = sequence ("|" transform)*
/// sequence   = atom+
/// atom       = note ("*" number)?
///            | rest
///            | "[" sequence "]" ("*" number)?
/// transform  = ident number?
/// ```
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    /// Parse the token stream into a [`PatternNode`] AST.
    pub fn parse(&mut self) -> Result<PatternNode, LiveCodeError> {
        let seq = self.parse_sequence()?;
        let result = self.parse_transforms(seq)?;

        if !matches!(self.peek(), Some(Token::Eof) | None) {
            return Err(LiveCodeError::ParseError {
                position: self.pos,
                message: format!("Unexpected token: {:?}", self.peek()),
            });
        }

        Ok(result)
    }

    // ── private helpers ──────────────────────────────────────────────

    fn parse_sequence(&mut self) -> Result<PatternNode, LiveCodeError> {
        let mut elements = Vec::new();

        loop {
            match self.peek() {
                Some(Token::Note(_)) | Some(Token::Rest) | Some(Token::LBracket) => {
                    elements.push(self.parse_atom()?);
                }
                _ => break,
            }
        }

        if elements.is_empty() {
            return Err(LiveCodeError::ParseError {
                position: self.pos,
                message: "Expected at least one pattern element".to_string(),
            });
        }

        if elements.len() == 1 {
            Ok(elements.remove(0))
        } else {
            Ok(PatternNode::Sequence(elements))
        }
    }

    fn parse_atom(&mut self) -> Result<PatternNode, LiveCodeError> {
        match self.peek().cloned() {
            Some(Token::Note(name)) => {
                self.advance();
                let node = PatternNode::Note(name);
                self.maybe_repeat(node)
            }
            Some(Token::Rest) => {
                self.advance();
                Ok(PatternNode::Rest)
            }
            Some(Token::LBracket) => self.parse_subdivision(),
            other => Err(LiveCodeError::ParseError {
                position: self.pos,
                message: format!("Unexpected token: {:?}", other),
            }),
        }
    }

    fn parse_subdivision(&mut self) -> Result<PatternNode, LiveCodeError> {
        self.advance(); // consume '['

        let inner = self.parse_sequence()?;

        match self.peek() {
            Some(Token::RBracket) => self.advance(),
            _ => {
                return Err(LiveCodeError::ParseError {
                    position: self.pos,
                    message: "Expected ']'".to_string(),
                });
            }
        }

        let elements = match inner {
            PatternNode::Sequence(elems) => elems,
            other => vec![other],
        };

        let node = PatternNode::Subdivision(elements);
        self.maybe_repeat(node)
    }

    /// If the next token is `*`, consume it and wrap `node` in a `Repeat`.
    fn maybe_repeat(&mut self, node: PatternNode) -> Result<PatternNode, LiveCodeError> {
        if matches!(self.peek(), Some(Token::Star)) {
            self.advance(); // consume '*'
            match self.peek() {
                Some(Token::Number(n)) => {
                    let n = *n as u32;
                    self.advance();
                    Ok(PatternNode::Repeat(Box::new(node), n))
                }
                _ => Err(LiveCodeError::ParseError {
                    position: self.pos,
                    message: "Expected number after '*'".to_string(),
                }),
            }
        } else {
            Ok(node)
        }
    }

    /// Parse zero or more trailing `| transform [arg]` chains.
    fn parse_transforms(&mut self, node: PatternNode) -> Result<PatternNode, LiveCodeError> {
        let mut current = node;

        while matches!(self.peek(), Some(Token::Pipe)) {
            self.advance(); // consume '|'

            let name = match self.peek().cloned() {
                Some(Token::Ident(name)) => {
                    self.advance();
                    name
                }
                _ => {
                    return Err(LiveCodeError::ParseError {
                        position: self.pos,
                        message: "Expected transform name after '|'".to_string(),
                    });
                }
            };

            let arg = if let Some(Token::Number(n)) = self.peek() {
                let n = *n;
                self.advance();
                Some(n)
            } else {
                None
            };

            current = PatternNode::Transform {
                pattern: Box::new(current),
                name,
                arg,
            };
        }

        Ok(current)
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) {
        if self.pos < self.tokens.len() {
            self.pos += 1;
        }
    }
}

/// Parse a pattern string into an AST.
pub fn parse_pattern(input: &str) -> Result<PatternNode, LiveCodeError> {
    let mut lexer = Lexer::new(input);
    let tokens = lexer.tokenize()?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_sequence() {
        let ast = parse_pattern("c4 e4 g4").unwrap();
        match ast {
            PatternNode::Sequence(elems) => {
                assert_eq!(elems.len(), 3);
                assert!(matches!(&elems[0], PatternNode::Note(n) if n == "c4"));
                assert!(matches!(&elems[1], PatternNode::Note(n) if n == "e4"));
                assert!(matches!(&elems[2], PatternNode::Note(n) if n == "g4"));
            }
            _ => panic!("Expected Sequence, got {:?}", ast),
        }
    }

    #[test]
    fn test_parse_with_rest() {
        let ast = parse_pattern("c4 ~ e4").unwrap();
        match ast {
            PatternNode::Sequence(elems) => {
                assert_eq!(elems.len(), 3);
                assert!(matches!(&elems[1], PatternNode::Rest));
            }
            _ => panic!("Expected Sequence"),
        }
    }

    #[test]
    fn test_parse_subdivision() {
        let ast = parse_pattern("c4 [e4 g4]").unwrap();
        match ast {
            PatternNode::Sequence(elems) => {
                assert_eq!(elems.len(), 2);
                assert!(matches!(&elems[1], PatternNode::Subdivision(_)));
            }
            _ => panic!("Expected Sequence"),
        }
    }

    #[test]
    fn test_parse_repeat() {
        let ast = parse_pattern("c4*3 e4").unwrap();
        match ast {
            PatternNode::Sequence(elems) => {
                assert_eq!(elems.len(), 2);
                assert!(matches!(&elems[0], PatternNode::Repeat(_, 3)));
            }
            _ => panic!("Expected Sequence"),
        }
    }

    #[test]
    fn test_parse_transform() {
        let ast = parse_pattern("c4 e4 | fast 2").unwrap();
        match ast {
            PatternNode::Transform { name, arg, .. } => {
                assert_eq!(name, "fast");
                assert_eq!(arg, Some(2.0));
            }
            _ => panic!("Expected Transform"),
        }
    }

    #[test]
    fn test_parse_chained_transforms() {
        let ast = parse_pattern("c4 e4 | fast 2 | rev").unwrap();
        match ast {
            PatternNode::Transform {
                name, arg, pattern, ..
            } => {
                assert_eq!(name, "rev");
                assert_eq!(arg, None);
                assert!(matches!(
                    *pattern,
                    PatternNode::Transform {
                        name: ref n,
                        arg: Some(2.0),
                        ..
                    } if n == "fast"
                ));
            }
            _ => panic!("Expected Transform"),
        }
    }

    #[test]
    fn test_parse_single_note() {
        let ast = parse_pattern("c4").unwrap();
        assert!(matches!(ast, PatternNode::Note(ref n) if n == "c4"));
    }
}
