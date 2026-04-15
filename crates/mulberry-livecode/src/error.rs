use thiserror::Error;

/// Errors that can occur during live-code parsing and evaluation.
#[derive(Debug, Error, Clone)]
pub enum LiveCodeError {
    #[error("Lexer error at position {position}: {message}")]
    LexError { position: usize, message: String },

    #[error("Parse error at position {position}: {message}")]
    ParseError { position: usize, message: String },

    #[error("Evaluation error: {0}")]
    EvalError(String),

    #[error("Unknown note name: {0}")]
    UnknownNote(String),
}
