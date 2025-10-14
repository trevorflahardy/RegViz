use crate::core::tokens::{Token, TokenKind};
use crate::errors::LexError;

pub fn lex(input: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();
    let mut iter = input.char_indices().peekable();

    while let Some((idx, ch)) = iter.next() {
        let column = idx + 1;
        let token = match ch {
            '\\' => {
                let (_, next) = iter
                    .next()
                    .ok_or_else(|| LexError::new(column, "dangling escape"))?;
                Token::new(TokenKind::Char(next), column)
            }
            '|' => Token::new(TokenKind::Or, column),
            '*' => Token::new(TokenKind::Star, column),
            '+' => Token::new(TokenKind::Plus, column),
            '?' => Token::new(TokenKind::QMark, column),
            '(' => Token::new(TokenKind::LParen, column),
            ')' => Token::new(TokenKind::RParen, column),
            _ => Token::new(TokenKind::Char(ch), column),
        };
        tokens.push(token);
    }

    tokens.push(Token::new(TokenKind::Eos, input.len() + 1));
    Ok(tokens)
}
