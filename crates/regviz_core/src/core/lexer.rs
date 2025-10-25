use crate::core::tokens::{Token, TokenKind};
use crate::errors::LexError;

/// Lexical analysis on a given input string to a list of valid regex tokens.
///
/// # Arguments
///
/// - `input` (`&str`) - The input regular expression string.
///
/// # Returns
///
/// - `Result<Vec<Token>, LexError>` - A result containing a vector of tokens or a lexical error.
///
/// # Errors
///
/// Returns a `LexError` if the input contains invalid or unexpected characters.
///
/// # Examples
///
/// ```
/// use regviz_core::core::lexer::lex;
/// use regviz_core::core::tokens::{TokenKind, Token};
///
/// let input = "ab";
/// let output = lex(input);
/// assert!(output.unwrap() == vec![
///     Token::new(TokenKind::Char('a'), 1),
///     Token::new(TokenKind::Char('b'), 2),
///     Token::new(TokenKind::Eos, 3),
/// ]);
/// ```
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

/// Inner helper function to make a token from a given character.
///
/// # Arguments
///
/// - `ch` (`char`) - The character to tokenize.
/// - `column` (`usize`) - The column position of the character.
/// - `iter` (`&mut I`) - The iterator over the input characters. Used if lookahead is needed.
///
/// # Returns
///
/// - `Result<Token, LexError> where I: Iterator<Item = (usize, char)>,` - The resulting token or a lexical error.
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
