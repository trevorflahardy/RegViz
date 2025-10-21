use crate::core::tokens::{Token, TokenKind};
use crate::errors::LexError;

/// Converts a regular-expression source string into a sequence of tokens.
pub fn lex(input: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();
    let mut iter = input.char_indices().peekable();

    while let Some((idx, ch)) = iter.next() {
        let column = idx + 1;
        let token = make_token(ch, column, &mut iter)?;
        tokens.push(token);
    }

    tokens.push(Token::new(TokenKind::Eos, input.len() + 1));
    Ok(tokens)
}

fn make_token<'a, I>(ch: char, column: usize, iter: &mut I) -> Result<Token, LexError>
where
    I: Iterator<Item = (usize, char)>,
{
    let kind = match ch {
        '\\' => {
            let (_, next) = iter
                .next()
                .ok_or_else(|| LexError::new(column, "dangling escape"))?;
            TokenKind::Char(next)
        }
        '|' => TokenKind::Or,
        '*' => TokenKind::Star,
        '+' => TokenKind::Plus,
        '?' => TokenKind::QMark,
        '(' => TokenKind::LParen,
        ')' => TokenKind::RParen,
        _ => TokenKind::Char(ch),
    };
    Ok(Token::new(kind, column))
}
