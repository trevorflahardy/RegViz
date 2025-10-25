use std::fmt::{self, Display, Formatter};

/// The different kinds of tokens produced by the [`lexer`](super::lexer).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    /// A literal character.
    Char(char),
    /// The alternation operator (`|`).
    Or,
    /// The Kleene star operator (`*`).
    Star,
    /// The one-or-more operator (`+`).
    Plus,
    /// The zero-or-one operator (`?`).
    QMark,
    /// An opening parenthesis (`(`).
    LParen,
    /// A closing parenthesis (`)`).
    RParen,
    /// End-of-stream marker appended by the lexer.
    Eos,
}

/// A lexical token annotated with the column in the original source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    /// The kind of token that was found.
    pub kind: TokenKind,
    /// The (0-indexed) column at which the token begins.
    pub pos: usize,
}

impl Token {
    /// Creates a new [`Token`] instance.
    #[must_use]
    pub fn new(kind: TokenKind, pos: usize) -> Self {
        Self { kind, pos }
    }
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            TokenKind::Char(c) => write!(f, "'{}'", c),
            TokenKind::Or => write!(f, "'|'"),
            TokenKind::Star => write!(f, "'*'"),
            TokenKind::Plus => write!(f, "'+'"),
            TokenKind::QMark => write!(f, "'?'"),
            TokenKind::LParen => write!(f, "'('"),
            TokenKind::RParen => write!(f, "')'"),
            TokenKind::Eos => write!(f, "<eos>"),
        }
    }
}
