use std::fmt::{self, Display, Formatter};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    Char(char),
    Or,
    Star,
    Plus,
    QMark,
    LParen,
    RParen,
    Eos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token {
    pub kind: TokenKind,
    pub pos: usize,
}

impl Token {
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
