use crate::error::LiveCodeError;

/// Tokens produced by the lexer.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// A note name like "c4", "eb3", "f#5"
    Note(String),
    /// A number literal
    Number(f64),
    /// Rest symbol: ~
    Rest,
    /// Open bracket [
    LBracket,
    /// Close bracket ]
    RBracket,
    /// Repeat operator *
    Star,
    /// Pipe for transforms |
    Pipe,
    /// Identifier (for transform names: fast, slow, rev, etc.)
    Ident(String),
    /// String literal in double quotes
    StringLit(String),
    /// End of input
    Eof,
}

/// Tokenizer for the live-code pattern mini-language.
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    pub fn new(input: &str) -> Self {
        Lexer {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    /// Tokenize the entire input into a list of tokens.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, LiveCodeError> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace_and_commas();

            if self.pos >= self.input.len() {
                tokens.push(Token::Eof);
                break;
            }

            let ch = self.input[self.pos];

            match ch {
                '~' => {
                    tokens.push(Token::Rest);
                    self.pos += 1;
                }
                '[' => {
                    tokens.push(Token::LBracket);
                    self.pos += 1;
                }
                ']' => {
                    tokens.push(Token::RBracket);
                    self.pos += 1;
                }
                '*' => {
                    tokens.push(Token::Star);
                    self.pos += 1;
                }
                '|' => {
                    tokens.push(Token::Pipe);
                    self.pos += 1;
                }
                '"' => {
                    let s = self.read_string_lit()?;
                    tokens.push(Token::StringLit(s));
                }
                c if c.is_ascii_digit() => {
                    tokens.push(self.read_number()?);
                }
                c if c.is_ascii_alphabetic() => {
                    if let Some(note) = self.try_read_note() {
                        tokens.push(Token::Note(note));
                    } else {
                        let ident = self.read_ident();
                        tokens.push(Token::Ident(ident));
                    }
                }
                _ => {
                    return Err(LiveCodeError::LexError {
                        position: self.pos,
                        message: format!("Unexpected character: '{}'", ch),
                    });
                }
            }
        }

        Ok(tokens)
    }

    fn skip_whitespace_and_commas(&mut self) {
        while self.pos < self.input.len()
            && (self.input[self.pos].is_whitespace() || self.input[self.pos] == ',')
        {
            self.pos += 1;
        }
    }

    /// Try to read a note token: [a-g][#b]?[0-9]+
    ///
    /// Returns `Some(note_string)` on success, `None` (with position unchanged) on failure.
    fn try_read_note(&mut self) -> Option<String> {
        let saved = self.pos;

        if self.pos >= self.input.len() || !matches!(self.input[self.pos], 'a'..='g') {
            return None;
        }
        self.pos += 1;

        // Optional accidental: # always, or b only when followed by a digit
        if self.pos < self.input.len() {
            if self.input[self.pos] == '#' {
                self.pos += 1;
            } else if self.input[self.pos] == 'b'
                && self.pos + 1 < self.input.len()
                && self.input[self.pos + 1].is_ascii_digit()
            {
                self.pos += 1;
            }
        }

        // Must have at least one octave digit
        let digit_start = self.pos;
        while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
            self.pos += 1;
        }

        if self.pos > digit_start {
            // Reject if immediately followed by alphabetic chars (part of a longer word)
            if self.pos < self.input.len() && self.input[self.pos].is_ascii_alphabetic() {
                self.pos = saved;
                return None;
            }
            let note: String = self.input[saved..self.pos].iter().collect();
            Some(note)
        } else {
            self.pos = saved;
            None
        }
    }

    fn read_ident(&mut self) -> String {
        let start = self.pos;
        while self.pos < self.input.len()
            && (self.input[self.pos].is_ascii_alphabetic() || self.input[self.pos] == '_')
        {
            self.pos += 1;
        }
        self.input[start..self.pos].iter().collect()
    }

    fn read_number(&mut self) -> Result<Token, LiveCodeError> {
        let start = self.pos;

        while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
            self.pos += 1;
        }

        // Optional decimal part
        if self.pos < self.input.len()
            && self.input[self.pos] == '.'
            && self.pos + 1 < self.input.len()
            && self.input[self.pos + 1].is_ascii_digit()
        {
            self.pos += 1; // consume '.'
            while self.pos < self.input.len() && self.input[self.pos].is_ascii_digit() {
                self.pos += 1;
            }
        }

        let num_str: String = self.input[start..self.pos].iter().collect();
        let value = num_str.parse::<f64>().map_err(|_| LiveCodeError::LexError {
            position: start,
            message: format!("Invalid number: '{}'", num_str),
        })?;

        Ok(Token::Number(value))
    }

    fn read_string_lit(&mut self) -> Result<String, LiveCodeError> {
        let start = self.pos;
        self.pos += 1; // consume opening "

        let content_start = self.pos;
        while self.pos < self.input.len() && self.input[self.pos] != '"' {
            self.pos += 1;
        }

        if self.pos >= self.input.len() {
            return Err(LiveCodeError::LexError {
                position: start,
                message: "Unterminated string literal".to_string(),
            });
        }

        let content: String = self.input[content_start..self.pos].iter().collect();
        self.pos += 1; // consume closing "
        Ok(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_simple_pattern() {
        let mut lexer = Lexer::new("c4 e4 g4");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Note("c4".into()),
                Token::Note("e4".into()),
                Token::Note("g4".into()),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_tokenize_with_rest() {
        let mut lexer = Lexer::new("c4 ~ e4");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Note("c4".into()),
                Token::Rest,
                Token::Note("e4".into()),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_tokenize_subdivision() {
        let mut lexer = Lexer::new("c4 [e4 g4]");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Note("c4".into()),
                Token::LBracket,
                Token::Note("e4".into()),
                Token::Note("g4".into()),
                Token::RBracket,
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_tokenize_with_transform() {
        let mut lexer = Lexer::new("c4 e4 | fast 2");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Note("c4".into()),
                Token::Note("e4".into()),
                Token::Pipe,
                Token::Ident("fast".into()),
                Token::Number(2.0),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_tokenize_repeat() {
        let mut lexer = Lexer::new("c4*3");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Note("c4".into()),
                Token::Star,
                Token::Number(3.0),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_tokenize_sharps_and_flats() {
        let mut lexer = Lexer::new("c#4 eb3 f#5");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Note("c#4".into()),
                Token::Note("eb3".into()),
                Token::Note("f#5".into()),
                Token::Eof,
            ]
        );
    }

    #[test]
    fn test_ident_not_confused_with_note() {
        let mut lexer = Lexer::new("fast slow rev");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Ident("fast".into()),
                Token::Ident("slow".into()),
                Token::Ident("rev".into()),
                Token::Eof,
            ]
        );
    }
}
