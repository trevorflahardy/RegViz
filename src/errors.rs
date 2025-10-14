use std::fmt::{self, Display, Formatter};

use thiserror::Error;

#[derive(Debug, Error, Clone)]
#[error("{message} at column {column}")]
pub struct LexError {
    pub column: usize,
    pub message: String,
}

impl LexError {
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

#[derive(Debug, Clone)]
pub struct ParseError {
    pub column: usize,
    pub kind: ParseErrorKind,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} at column {}", self.kind, self.column)
    }
}

impl std::error::Error for ParseError {}

impl ParseError {
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
