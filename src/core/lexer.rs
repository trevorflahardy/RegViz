use crate::core::tokens::{Token, TokenKind};
use crate::errors::{LexError, LexErrorKind};

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
    let mut token_kinds = Vec::new();
    let mut iter = input.chars().enumerate();

    while let Some((idx, ch)) = iter.next() {
        let kind = match ch {
            '\\' => {
                let (_, next) = iter.next().ok_or(LexError {
                    at: idx,
                    kind: LexErrorKind::DanglingEscape,
                })?;
                TokenKind::Char(validate_char(next, idx + 1)?)
            }
            '|' => TokenKind::Or,
            '*' => TokenKind::Star,
            '+' => TokenKind::Plus,
            '?' => TokenKind::QMark,
            '(' => TokenKind::LParen,
            ')' => TokenKind::RParen,
            _ => TokenKind::Char(validate_char(ch, idx)?),
        };
        token_kinds.push(kind);
    }

    token_kinds.push(TokenKind::Eos);

    // Convert TokenKinds to Tokens with positions
    let tokens = token_kinds
        .into_iter()
        .enumerate()
        .map(|(idx, kind)| Token::new(kind, idx))
        .collect();

    Ok(tokens)
}

/// Validates a character.
/// Invalid characters include control characters and whitespace.
/// Returns the character if valid, otherwise returns a LexError.
///
/// # Arguments
/// - `ch` (`char`) - The character to validate.
/// - `pos` (`usize`) - The position of the character in the input string.
fn validate_char(ch: char, pos: usize) -> Result<char, LexError> {
    if ch.is_control() {
        Err(LexError {
            at: pos,
            kind: LexErrorKind::InvalidCharacter(ch, "control characters are not allowed"),
        })
    } else if ch.is_whitespace() {
        Err(LexError {
            at: pos,
            kind: LexErrorKind::InvalidCharacter(ch, "whitespace characters are not allowed"),
        })
    } else {
        Ok(ch)
    }
}

mod tests {
    use super::*;

    #[test]
    fn test_lexer_simple() {
        let input = "a";
        let tokens = lex(input).unwrap();
        assert_eq!(tokens.len(), 2); // Char('a'), Eos
        assert_eq!(tokens[0].kind, TokenKind::Char('a'));
        assert_eq!(tokens[1].kind, TokenKind::Eos);
    }

    #[test]
    fn test_lexer_complex() {
        let input = "a|b*";
        let tokens = lex(input).unwrap();
        assert_eq!(tokens.len(), 5); // Char('a'), Or, Char('b'), Star, Eos
        assert_eq!(tokens[0].kind, TokenKind::Char('a'));
        assert_eq!(tokens[1].kind, TokenKind::Or);
        assert_eq!(tokens[2].kind, TokenKind::Char('b'));
        assert_eq!(tokens[3].kind, TokenKind::Star);
        assert_eq!(tokens[4].kind, TokenKind::Eos);
    }

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
        let input = r"abc\";
        let output = lex(input);
        assert!(output.is_err());
        let err = output.err().unwrap();
        assert_eq!(err.at, 3);
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

    #[test]
    fn test_validate_valid_chars() {
        // Valid character: ASCII, Unicode, CJK characters, etc.
        assert_eq!(validate_char('a', 0).unwrap(), 'a');
        assert_eq!(validate_char('字', 0).unwrap(), '字');
        assert_eq!(validate_char('😊', 0).unwrap(), '😊');
        assert_eq!(validate_char('1', 0).unwrap(), '1');
        assert_eq!(validate_char('#', 0).unwrap(), '#');
        assert_eq!(validate_char('-', 0).unwrap(), '-');
        assert_eq!(validate_char('_', 0).unwrap(), '_');
        assert_eq!(validate_char('Z', 0).unwrap(), 'Z');
    }

    #[test]
    fn test_validate_invalid_chars() {
        // Control characters
        let control_chars = ['\x00', '\x1F', '\x7F', '\n', '\r', '\t'];
        for &ch in &control_chars {
            let err = validate_char(ch, 0).err().unwrap();
            assert_eq!(err.at, 0);
            match err.kind {
                LexErrorKind::InvalidCharacter(c, _) => assert_eq!(c, ch),
                _ => panic!("Expected InvalidCharacter error"),
            }
        }

        // Whitespace characters
        let whitespace_chars = [' ', '\t', '\n', '\r', '\x0B', '\x0C'];
        for &ch in &whitespace_chars {
            let err = validate_char(ch, 1).err().unwrap();
            assert_eq!(err.at, 1);
            match err.kind {
                LexErrorKind::InvalidCharacter(c, _) => assert_eq!(c, ch),
                _ => panic!("Expected InvalidCharacter error"),
            }
        }
    }
}
