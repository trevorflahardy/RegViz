use std::fmt::{self, Display, Formatter};

use thiserror::Error;

/// Error emitted by the lexer with a message and column position.
#[derive(Debug, Error, Clone)]
#[error("{message} at column {column}")]
pub struct LexError {
    /// Column at which the error occurred (1-indexed).
    pub column: usize,
    /// Human-readable error message.
    pub message: String,
}

impl LexError {
    /// Creates a new [`LexError`].
    #[must_use]
    pub fn new(column: usize, message: impl Into<String>) -> Self {
        Self {
            column,
            message: message.into(),
        }
    }
}

#[derive(Debug, Error, Clone)]
pub enum ParseErrorKind {
    #[error("unexpected end of input")]
    UnexpectedEos,
    #[error("unexpected token {found}")]
    UnexpectedToken { found: String },
    #[error("missing closing parenthesis")]
    MissingRParen,
    #[error("illegal postfix operator usage")]
    MisplacedPostfix,
    #[error("empty alternative")]
    EmptyAlternative,
}

/// Parser error annotated with the offending column and kind.
#[derive(Debug, Clone)]
pub struct ParseError {
    /// Column at which the parser reported the error.
    pub column: usize,
    /// Detailed categorization of the error.
    pub kind: ParseErrorKind,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} at column {}", self.kind, self.column)
    }
}

impl std::error::Error for ParseError {}

impl ParseError {
    /// Creates a new [`ParseError`].
    #[must_use]
    pub fn new(column: usize, kind: ParseErrorKind) -> Self {
        Self { column, kind }
    }
}

#[derive(Debug, Error, Clone)]
pub enum BuildError {
    #[error("lex error: {0}")]
    Lex(#[from] LexError),
    #[error("parse error: {0}")]
    Parse(#[from] ParseError),
}
