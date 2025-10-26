use std::fmt::Display;

use crate::errors::{LexError, LexErrorKind};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Token {
    /// A literal character
    Literal(char),
    /// A special operator token
    Op(OpToken),
    /// '('
    LParen,
    /// ')'
    RParen,
    /// End of input
    Eof,
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Token::Literal(c) => write!(f, "{}", c),
            Token::Op(op) => write!(f, "{}", op),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::Eof => write!(f, "<EOF>"),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum OpToken {
    /// '+' operator (alternation)
    Plus,
    /// '*' operator (Kleene star)
    Star,
    /// '.' operator (concatenation)
    Dot,
}

impl Display for OpToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbol = match self {
            OpToken::Plus => "+",
            OpToken::Star => "*",
            OpToken::Dot => ".",
        };
        write!(f, "{}", symbol)
    }
}

#[derive(Debug)]
pub struct Lexer {
    /// Stack of tokens with their original char indices in the input string.
    tokens: Vec<(Token, usize)>,
    /// Total number of characters processed from the input. Used to return along with [`Token::Eof`] in [`Lexer::peek`] and [`Lexer::advance`].
    num_chars: usize,
}

impl Lexer {
    /// Processes the input string into a sequence of tokens, stored in the [`Lexer`] instance.
    /// Returns a [`LexError`] if the input contains invalid characters.
    pub fn new(input: &str) -> Result<Self, LexError> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().enumerate();

        while let Some((mut idx, ch)) = chars.next() {
            let token = match ch {
                // Escape character, treat next character as literal
                '\\' => {
                    if let Some((next_idx, next_ch)) = chars.next() {
                        // Update idx to point to the escaped character
                        idx = next_idx;
                        Token::Literal(next_ch)
                    } else {
                        return Err(LexError {
                            at: idx,
                            kind: LexErrorKind::DanglingEscape,
                        });
                    }
                }
                '.' => Token::Op(OpToken::Dot),
                '+' => Token::Op(OpToken::Plus),
                '*' => Token::Op(OpToken::Star),
                '(' => Token::LParen,
                ')' => Token::RParen,
                // Skip whitespace
                c if c.is_ascii_whitespace() => continue,
                // Only allow alphanumeric literals
                c if c.is_ascii_alphanumeric() => Token::Literal(c),
                c => {
                    return Err(LexError {
                        at: idx,
                        kind: LexErrorKind::InvalidCharacter(c),
                    });
                }
            };

            // Store token with its original index
            tokens.push((token, idx));
        }

        tokens.reverse();
        Ok(Self {
            tokens,
            num_chars: input.chars().count(),
        })
    }

    /// Advance to the next token and return it.
    /// If there are no more tokens, return [`Token::Eof`].
    pub fn advance(&mut self) -> (Token, usize) {
        self.tokens.pop().unwrap_or((Token::Eof, self.num_chars))
    }

    /// Peek at the next token without consuming it.
    /// If there are no more tokens, return [`Token::Eof`].
    pub fn peek(&self) -> (Token, usize) {
        self.tokens
            .last()
            .copied()
            .unwrap_or((Token::Eof, self.num_chars))
    }
}
