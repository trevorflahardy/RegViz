use thiserror::Error;

use crate::core::lexer::{OpToken, Token};

/// Error emitted by the lexer with a message and position.
#[derive(Debug, Error, Clone, PartialEq)]
#[error("{kind} at index {at}")]
pub struct LexError {
    /// Position of the character (0-indexed) in the input where the error occurred.
    pub at: usize,
    /// Detailed categorization of the error.
    pub kind: LexErrorKind,
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum LexErrorKind {
    #[error("dangling escape character")]
    DanglingEscape,
    #[error("invalid character '{0}'. Only alphanumeric characters are allowed")]
    InvalidCharacter(char),
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum ParseErrorKind {
    #[error("unexpected end of input")]
    UnexpectedEof,
    #[error("expected an expression after the operator '{0}'")]
    UnexpectedPrefixOperator(OpToken),
    #[error("expected closing parenthesis but found token '{other}'")]
    MismatchedLeftParen { other: Token },
    #[error("found closing parenthesis with no matching opening parenthesis")]
    RightParenWithoutLeft,
    #[error("found empty parentheses '()' which is not allowed")]
    EmptyParentheses,
}

/// Parser error annotated with the .
#[derive(Debug, Error, Clone, PartialEq)]
#[error("{kind} at index {at}")]
pub struct ParseError {
    /// Position (0-indexed) in the input where the error occurred.
    pub at: usize,
    /// Detailed categorization of the error.
    pub kind: ParseErrorKind,
}

#[derive(Debug, Error, Clone, PartialEq)]
pub enum BuildError {
    #[error("[lex error] {0}")]
    Lex(#[from] LexError),
    #[error("[parse error] {0}")]
    Parse(#[from] ParseError),
}
