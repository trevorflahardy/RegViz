use thiserror::Error;

/// Error emitted by the lexer with a message and column position.
#[derive(Debug, Error, Clone)]
#[error("{kind} at index {at}")]
pub struct LexError {
    /// Position of the character (0-indexed) in the input where the error occurred.
    pub at: usize,
    /// Detailed categorization of the error.
    pub kind: LexErrorKind,
}

#[derive(Debug, Error, Clone)]
pub enum LexErrorKind {
    #[error("dangling escape character")]
    DanglingEscape,
    #[error("invalid character '{0}': {1}")]
    InvalidCharacter(char, &'static str),
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
#[derive(Debug, Error, Clone)]
#[error("{kind} at column {column}")]
pub struct ParseError {
    /// Column at which the parser reported the error.
    pub column: usize,
    /// Detailed categorization of the error.
    pub kind: ParseErrorKind,
}

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
