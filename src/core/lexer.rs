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
/// use regviz::core::lexer::lex;
/// use regviz::core::tokens::{TokenKind, Token};
///
/// let input = "ab";
/// let output = lex(input);
/// assert!(output.unwrap() == vec![
///     Token::new(TokenKind::Char('a'), 0),
///     Token::new(TokenKind::Char('b'), 1),
///     Token::new(TokenKind::Eos, 2),
/// ]);
/// ```
pub fn lex(input: &str) -> Result<Vec<Token>, LexError> {
    let mut tokens = Vec::new();
    let mut iter = input.chars().enumerate();

    while let Some((idx, ch)) = iter.next() {
        let kind = match ch {
            '\\' => {
                let (_, next) = iter.next().ok_or(LexError {
                    at: idx,
                    kind: crate::errors::LexErrorKind::DanglingEscape,
                })?;
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
        tokens.push(Token::new(kind, tokens.len()));
    }

    tokens.push(Token::new(TokenKind::Eos, tokens.len()));
    Ok(tokens)
}

mod tests {
    use super::*;

    #[test]
    fn test_all_symbols() {
        let input = r"\|*+?()a";
        let output = lex(input).unwrap();
        assert_eq!(
            output,
            vec![
                Token::new(TokenKind::Char('|'), 0),
                Token::new(TokenKind::Star, 1),
                Token::new(TokenKind::Plus, 2),
                Token::new(TokenKind::QMark, 3),
                Token::new(TokenKind::LParen, 4),
                Token::new(TokenKind::RParen, 5),
                Token::new(TokenKind::Char('a'), 6),
                Token::new(TokenKind::Eos, 7),
            ]
        );
    }

    #[test]
    fn test_dangling_escape() {
        let input = r"\";
        let output = lex(input);
        assert!(output.is_err());
        let err = output.err().unwrap();
        assert_eq!(err.at, 0);
        match err.kind {
            crate::errors::LexErrorKind::DanglingEscape => {}
            _ => panic!("Expected DanglingEscape error"),
        }
    }

    #[test]
    fn test_multiple_escapes() {
        let input = r"\a\b\c\*\\";
        let output = lex(input).unwrap();
        assert_eq!(
            output,
            vec![
                Token::new(TokenKind::Char('a'), 0),
                Token::new(TokenKind::Char('b'), 1),
                Token::new(TokenKind::Char('c'), 2),
                Token::new(TokenKind::Char('*'), 3),
                Token::new(TokenKind::Char('\\'), 4),
                Token::new(TokenKind::Eos, 5),
            ]
        );
    }
}
