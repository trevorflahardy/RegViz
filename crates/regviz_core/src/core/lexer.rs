use std::fmt::Display;

use crate::errors::{LexError, LexErrorKind};

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Token {
    /// Epsilon
    Epsilon,
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
            Token::Epsilon => write!(f, "ε"),
            Token::Literal(c) => write!(f, "{c}"),
            Token::Op(op) => write!(f, "{op}"),
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
    /// '?' operator (optional)
    Opt,
}

impl Display for OpToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let symbol = match self {
            OpToken::Plus => "+",
            OpToken::Star => "*",
            OpToken::Dot => ".",
            OpToken::Opt => "?",
        };
        write!(f, "{symbol}")
    }
}

type CharIndex = usize;

#[derive(Debug)]
pub struct Lexer<'a> {
    /// Index of the current character (the one returned by `self.chars.peek()`)
    char_pos: CharIndex,
    /// Character iterator
    chars: std::iter::Peekable<std::str::Chars<'a>>,
    /// Peeked token cache
    peeked: Option<(Token, CharIndex)>,
}

impl<'a> Lexer<'a> {
    /// Creates a new lexer for the given input string.
    pub fn new(input: &'a str) -> Self {
        Self {
            char_pos: 0,
            chars: input.chars().peekable(),
            peeked: None,
        }
    }

    /// Peek at the next token without consuming it.
    /// If there are no more tokens, return `Token::Eof`.
    pub fn peek(&mut self) -> Result<(Token, CharIndex), LexError> {
        match self.peeked {
            Some(peeked) => Ok(peeked),
            None => {
                let token = self.lex_next()?;
                self.peeked = Some(token);
                Ok(token)
            }
        }
    }

    /// Advance to the next token and return it.
    /// If there are no more tokens, return `Token::Eof`.
    pub fn advance(&mut self) -> Result<(Token, CharIndex), LexError> {
        match self.peeked.take() {
            Some(peeked) => Ok(peeked),
            None => self.lex_next(),
        }
    }

    /// Internal method to lex the next token from the input.
    fn lex_next(&mut self) -> Result<(Token, CharIndex), LexError> {
        // Skip whitespace
        while let Some(&ch) = self.chars.peek() {
            if !ch.is_ascii_whitespace() {
                break;
            }
            self.chars.next();
            self.char_pos += 1;
        }

        // Get next character
        let Some(ch) = self.chars.next() else {
            // End of input
            return Ok((Token::Eof, self.char_pos));
        };

        let token_char_idx = self.char_pos;
        self.char_pos += 1;

        let token = match ch {
            // Escape character, treat next character as literal
            '\\' => {
                if let Some(next_ch) = self.chars.next() {
                    // The token's char index should point to the escaped character
                    let token_char_idx = self.char_pos;
                    self.char_pos += 1;

                    let token = match next_ch {
                        // Check for epsilon escape
                        'e' => Token::Epsilon,
                        // Treat next character as literal
                        _ => Token::Literal(next_ch),
                    };

                    return Ok((token, token_char_idx));
                } else {
                    return Err(LexError {
                        at: token_char_idx,
                        kind: LexErrorKind::DanglingEscape,
                    });
                }
            }
            '.' => Token::Op(OpToken::Dot),
            '+' => Token::Op(OpToken::Plus),
            '*' => Token::Op(OpToken::Star),
            '?' => Token::Op(OpToken::Opt),
            '(' => Token::LParen,
            ')' => Token::RParen,
            // Allow any character as a literal (including emoji, unicode, etc.)
            c => Token::Literal(c),
        };

        Ok((token, token_char_idx))
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
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.advance(), Ok((Token::Literal('a'), 0)));
        assert_eq!(lexer.advance(), Ok((Token::Op(OpToken::Plus), 1)));
        assert_eq!(lexer.advance(), Ok((Token::Literal('b'), 2)));
        assert_eq!(lexer.advance(), Ok((Token::Op(OpToken::Star), 3)));
        assert_eq!(lexer.advance(), Ok((Token::Literal('c'), 4)));
        assert_eq!(lexer.advance(), Ok((Token::Eof, 5)));
    }
    #[test]
    fn test_lexer_with_parentheses() {
        let input = "(a.b)+c";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.advance(), Ok((Token::LParen, 0)));
        assert_eq!(lexer.advance(), Ok((Token::Literal('a'), 1)));
        assert_eq!(lexer.advance(), Ok((Token::Op(OpToken::Dot), 2)));
        assert_eq!(lexer.advance(), Ok((Token::Literal('b'), 3)));
        assert_eq!(lexer.advance(), Ok((Token::RParen, 4)));
        assert_eq!(lexer.advance(), Ok((Token::Op(OpToken::Plus), 5)));
        assert_eq!(lexer.advance(), Ok((Token::Literal('c'), 6)));
        assert_eq!(lexer.advance(), Ok((Token::Eof, 7)));
    }
    #[test]
    fn test_lexer_with_escape() {
        let input = r"a\+b\*c";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.advance(), Ok((Token::Literal('a'), 0)));
        assert_eq!(lexer.advance(), Ok((Token::Literal('+'), 2)));
        assert_eq!(lexer.advance(), Ok((Token::Literal('b'), 3)));
        assert_eq!(lexer.advance(), Ok((Token::Literal('*'), 5)));
        assert_eq!(lexer.advance(), Ok((Token::Literal('c'), 6)));
        assert_eq!(lexer.advance(), Ok((Token::Eof, 7)));
    }
    #[test]
    fn test_lexer_special_characters() {
        let input = "a+b$c";
        let mut lexer = Lexer::new(input);
        // All characters including special ones like $ are now allowed
        assert_eq!(lexer.advance(), Ok((Token::Literal('a'), 0)));
        assert_eq!(lexer.advance(), Ok((Token::Op(OpToken::Plus), 1)));
        assert_eq!(lexer.advance(), Ok((Token::Literal('b'), 2)));
        assert_eq!(lexer.advance(), Ok((Token::Literal('$'), 3)));
        assert_eq!(lexer.advance(), Ok((Token::Literal('c'), 4)));
        assert_eq!(lexer.advance(), Ok((Token::Eof, 5)));
    }

    #[test]
    fn test_lexer_emoji() {
        let input = "😀+🎉*💯";
        let mut lexer = Lexer::new(input);
        assert_eq!(lexer.advance(), Ok((Token::Literal('😀'), 0)));
        assert_eq!(lexer.advance(), Ok((Token::Op(OpToken::Plus), 1)));
        assert_eq!(lexer.advance(), Ok((Token::Literal('🎉'), 2)));
        assert_eq!(lexer.advance(), Ok((Token::Op(OpToken::Star), 3)));
        assert_eq!(lexer.advance(), Ok((Token::Literal('💯'), 4)));
        assert_eq!(lexer.advance(), Ok((Token::Eof, 5)));
    }
    #[test]
    fn test_lexer_dangling_escape() {
        let input = r"a+b\";
        let mut lexer = Lexer::new(input);
        // First three tokens should succeed
        assert_eq!(lexer.advance(), Ok((Token::Literal('a'), 0)));
        assert_eq!(lexer.advance(), Ok((Token::Op(OpToken::Plus), 1)));
        assert_eq!(lexer.advance(), Ok((Token::Literal('b'), 2)));
        // Fourth token should fail
        let result = lexer.advance();
        assert_eq!(
            result,
            Err(LexError {
                at: 3,
                kind: LexErrorKind::DanglingEscape,
            })
        );
    }
}
