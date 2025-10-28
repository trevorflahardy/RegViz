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

type CharIndex = usize;

#[derive(Debug)]
pub struct Lexer {
    /// Stack of tokens with their original char indices in the input string.
    tokens: Vec<(Token, CharIndex)>,
    /// Total number of characters processed from the input. Used to return along with [`Token::Eof`] in [`Lexer::peek`] and [`Lexer::advance`].
    num_chars: CharIndex,
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
    pub fn advance(&mut self) -> (Token, CharIndex) {
        self.tokens.pop().unwrap_or((Token::Eof, self.num_chars))
    }

    /// Peek at the next token without consuming it.
    /// If there are no more tokens, return [`Token::Eof`].
    pub fn peek(&self) -> (Token, CharIndex) {
        self.tokens
            .last()
            .copied()
            .unwrap_or((Token::Eof, self.num_chars))
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        core::lexer::{Lexer, OpToken, Token},
        errors::{LexError, LexErrorKind},
    };

    #[test]
    fn test_lexer_basic() {
        let input = "a+b*c";
        let mut lexer = Lexer::new(input).unwrap();

        assert_eq!(lexer.advance(), (Token::Literal('a'), 0));
        assert_eq!(lexer.advance(), (Token::Op(OpToken::Plus), 1));
        assert_eq!(lexer.advance(), (Token::Literal('b'), 2));
        assert_eq!(lexer.advance(), (Token::Op(OpToken::Star), 3));
        assert_eq!(lexer.advance(), (Token::Literal('c'), 4));
        assert_eq!(lexer.advance(), (Token::Eof, 5));
    }

    #[test]
    fn test_lexer_with_parentheses() {
        let input = "(a.b)+c";
        let mut lexer = Lexer::new(input).unwrap();

        assert_eq!(lexer.advance(), (Token::LParen, 0));
        assert_eq!(lexer.advance(), (Token::Literal('a'), 1));
        assert_eq!(lexer.advance(), (Token::Op(OpToken::Dot), 2));
        assert_eq!(lexer.advance(), (Token::Literal('b'), 3));
        assert_eq!(lexer.advance(), (Token::RParen, 4));
        assert_eq!(lexer.advance(), (Token::Op(OpToken::Plus), 5));
        assert_eq!(lexer.advance(), (Token::Literal('c'), 6));
        assert_eq!(lexer.advance(), (Token::Eof, 7));
    }

    #[test]
    fn test_lexer_with_escape() {
        let input = r"a\+b\*c";
        let mut lexer = Lexer::new(input).unwrap();

        assert_eq!(lexer.advance(), (Token::Literal('a'), 0));
        assert_eq!(lexer.advance(), (Token::Literal('+'), 2));
        assert_eq!(lexer.advance(), (Token::Literal('b'), 3));
        assert_eq!(lexer.advance(), (Token::Literal('*'), 5));
        assert_eq!(lexer.advance(), (Token::Literal('c'), 6));
        assert_eq!(lexer.advance(), (Token::Eof, 7));
    }

    #[test]
    fn test_lexer_invalid_character() {
        let input = "a+b$c";
        let result = Lexer::new(input);
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(
            err,
            LexError {
                at: 3,
                kind: LexErrorKind::InvalidCharacter('$'),
            }
        );
    }

    #[test]
    fn test_lexer_dangling_escape() {
        let input = r"a+b\";
        let result = Lexer::new(input);
        assert!(result.is_err());
        let err = result.err().unwrap();
        assert_eq!(
            err,
            LexError {
                at: 3,
                kind: LexErrorKind::DanglingEscape,
            }
        );
    }
}
