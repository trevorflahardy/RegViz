use crate::errors::{LexError, LexErrorKind};

#[derive(Debug, PartialEq, Eq)]
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

#[derive(Debug, PartialEq, Eq)]
pub enum OpToken {
    /// '+' operator (alternation)
    Plus,
    /// '*' operator (Kleene star)
    Star,
    /// '.' operator (concatenation)
    Dot,
}

#[derive(Debug)]
pub struct Lexer {
    tokens: Vec<Token>,
}

impl Lexer {
    pub fn new(input: &str) -> Result<Self, LexError> {
        let mut tokens = Vec::new();
        let mut chars = input.chars().enumerate();

        while let Some((idx, ch)) = chars.next() {
            let token = match ch {
                // Escape character, treat next character as literal
                '\\' => {
                    if let Some((_, next_ch)) = chars.next() {
                        Token::Literal(next_ch)
                    } else {
                        return Err(LexError {
                            at: idx,
                            kind: LexErrorKind::DanglingEscape,
                        });
                    }
                }
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
            tokens.push(token);
        }

        tokens.reverse();
        Ok(Self { tokens })
    }

    fn next(&mut self) -> Token {
        self.tokens.pop().unwrap_or(Token::Eof)
    }

    fn peek(&self) -> &Token {
        self.tokens.last().unwrap_or(&Token::Eof)
    }
}
