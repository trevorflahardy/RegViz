use crate::core::ast::Ast;
use crate::core::tokens::{Token, TokenKind};
use crate::errors::{ParseError, ParseErrorKind};

/// Converts a token stream into an [`Ast`] using a Pratt-style recursive-descent
/// parser for regular expressions.
pub fn parse(tokens: &[Token]) -> Result<Ast, ParseError> {
    let mut parser = Parser::new(tokens);
    let ast = parser.parse_regex()?;
    parser.expect(TokenKind::Eos)?;
    Ok(ast)
}

/// Stateful parser over a token slice.
struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    pub fn new(tokens: &'a [Token]) -> Self {
        Self { tokens, pos: 0 }
    }

    /// Parses a full regular expression, covering alternation and concatenation.
    fn parse_regex(&mut self) -> Result<Ast, ParseError> {
        self.parse_alt()
    }

    /// Parses an alternation (`lhs | rhs`).
    fn parse_alt(&mut self) -> Result<Ast, ParseError> {
        let mut node = self.parse_concat()?;
        while self.matches(TokenKind::Or) {
            let rhs = self.parse_concat()?;
            node = Ast::alt(node, rhs);
        }
        Ok(node)
    }

    /// Parses implicit concatenation of atoms.
    fn parse_concat(&mut self) -> Result<Ast, ParseError> {
        let mut nodes = Vec::new();
        while self.can_start_atom() {
            nodes.push(self.parse_repeat()?);
        }
        match nodes.len() {
            0 => {
                if matches!(
                    self.peek_kind(),
                    Some(TokenKind::Star | TokenKind::Plus | TokenKind::QMark)
                ) {
                    Err(self.error_here(ParseErrorKind::MisplacedPostfix))
                } else {
                    Err(self.error_here(ParseErrorKind::EmptyAlternative))
                }
            }
            1 => Ok(nodes.remove(0)),
            _ => Ok(chain_concat(nodes)),
        }
    }

    /// Parses unary postfix operators (`*`, `+`, `?`).
    fn parse_repeat(&mut self) -> Result<Ast, ParseError> {
        let mut node = self.parse_atom()?;
        while let Some(apply) = self.next_repetition() {
            node = apply(node);
        }
        Ok(node)
    }

    /// Determines whether the current token may begin an atom.
    fn can_start_atom(&self) -> bool {
        matches!(
            self.peek_kind(),
            Some(TokenKind::Char(_)) | Some(TokenKind::LParen)
        )
    }

    /// Parses a single atom (literal or grouped sub-expression).
    fn parse_atom(&mut self) -> Result<Ast, ParseError> {
        match self.peek_kind() {
            Some(TokenKind::Char(c)) => {
                let _ = self.advance();
                Ok(Ast::Char(c))
            }
            Some(TokenKind::LParen) => {
                let _ = self.advance();
                let node = self.parse_regex()?;
                self.expect(TokenKind::RParen)?;
                Ok(node)
            }
            Some(TokenKind::RParen) => {
                Err(self.error_here(ParseErrorKind::UnexpectedToken { found: ")".into() }))
            }
            Some(TokenKind::Eos) => Err(self.error_here(ParseErrorKind::UnexpectedEos)),
            Some(other) => Err(self.error_here(ParseErrorKind::UnexpectedToken {
                found: other.to_string(),
            })),
            None => Err(self.error_here(ParseErrorKind::UnexpectedEos)),
        }
    }

    /// Returns and consumes the next repetition operator, if any.
    fn next_repetition(&mut self) -> Option<fn(Ast) -> Ast> {
        let kind = match self.peek_kind() {
            Some(kind @ (TokenKind::Star | TokenKind::Plus | TokenKind::QMark)) => kind,
            Some(TokenKind::RParen | TokenKind::Or | TokenKind::Eos) | None => return None,
            Some(TokenKind::Char(_) | TokenKind::LParen) => return None,
        };

        self.advance();
        let apply = match kind {
            TokenKind::Star => Ast::star,
            TokenKind::Plus => Ast::plus,
            TokenKind::QMark => Ast::opt,
            _ => unreachable!("filtered above"),
        };
        Some(apply)
    }

    /// Consumes the next token if it matches the provided kind.
    fn matches(&mut self, kind: TokenKind) -> bool {
        if self.peek_kind() == Some(kind) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    /// Consumes the next token or reports a detailed error.
    fn expect(&mut self, kind: TokenKind) -> Result<(), ParseError> {
        if self.peek_kind() == Some(kind) {
            self.pos += 1;
            Ok(())
        } else {
            Err(self.unexpected_token_error())
        }
    }

    fn unexpected_token_error(&self) -> ParseError {
        match self.peek() {
            Some(tok) => ParseError::new(
                tok.pos,
                ParseErrorKind::UnexpectedToken {
                    found: tok.kind.to_string(),
                },
            ),
            None => ParseError::new(self.last_column(), ParseErrorKind::UnexpectedEos),
        }
    }

    fn peek(&self) -> Option<&'a Token> {
        self.tokens.get(self.pos)
    }

    fn peek_kind(&self) -> Option<TokenKind> {
        self.peek().map(|tok| tok.kind)
    }

    fn advance(&mut self) -> Option<&'a Token> {
        let token = self.peek();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    fn error_here(&self, kind: ParseErrorKind) -> ParseError {
        let column = self
            .peek()
            .map(|t| t.pos)
            .unwrap_or_else(|| self.last_column());
        ParseError::new(column, kind)
    }

    fn last_column(&self) -> usize {
        self.tokens.last().map(|tok| tok.pos).unwrap_or_default()
    }
}

fn chain_concat(nodes: Vec<Ast>) -> Ast {
    let mut it = nodes.into_iter();
    let mut acc = it.next().expect("chain_concat requires a non-empty vector");
    for node in it {
        acc = Ast::concat(acc, node);
    }
    acc
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::lexer;

    #[test]
    fn test_alteration() -> () {
        let input = "a|b";
        let tokens = lexer::lex(input).unwrap();
        let ast = parse(&tokens).unwrap();

        assert_eq!(
            ast,
            Ast::Alt(Box::new(Ast::Char('a')), Box::new(Ast::Char('b'))),
        )
    }

    #[test]
    fn test_concatenation() -> () {
        let input = "ab";
        let tokens = lexer::lex(input).unwrap();
        let ast = parse(&tokens).unwrap();
        assert_eq!(
            ast,
            Ast::Concat(Box::new(Ast::Char('a')), Box::new(Ast::Char('b'))),
        )
    }

    #[test]
    fn test_star() -> () {
        let input = "a*";
        let tokens = lexer::lex(input).unwrap();
        let ast = parse(&tokens).unwrap();
        assert_eq!(ast, Ast::Star(Box::new(Ast::Char('a'))))
    }

    #[test]
    fn test_plus() -> () {
        let input = "b+";
        let tokens = lexer::lex(input).unwrap();
        let ast = parse(&tokens).unwrap();
        assert_eq!(ast, Ast::Plus(Box::new(Ast::Char('b'))))
    }

    #[test]
    fn test_opt() -> () {
        let input = "c?";
        let tokens = lexer::lex(input).unwrap();
        let ast = parse(&tokens).unwrap();
        assert_eq!(ast, Ast::Opt(Box::new(Ast::Char('c'))))
    }

    #[test]
    fn test_grouping() -> () {
        let input = "(a|b)c";
        let tokens = lexer::lex(input).unwrap();
        let ast = parse(&tokens).unwrap();
        assert_eq!(
            ast,
            Ast::Concat(
                Box::new(Ast::Alt(Box::new(Ast::Char('a')), Box::new(Ast::Char('b')))),
                Box::new(Ast::Char('c')),
            ),
        )
    }

    #[test]
    fn test_grouping_optional() -> () {
        let input = "(ab)?";
        let tokens = lexer::lex(input).unwrap();
        let ast = parse(&tokens).unwrap();
        assert_eq!(
            ast,
            Ast::Opt(Box::new(Ast::Concat(
                Box::new(Ast::Char('a')),
                Box::new(Ast::Char('b')),
            ))),
        )
    }

    #[test]
    fn test_grouping_star() -> () {
        let input = "(a|b)*";
        let tokens = lexer::lex(input).unwrap();
        let ast = parse(&tokens).unwrap();
        assert_eq!(
            ast,
            Ast::Star(Box::new(Ast::Alt(
                Box::new(Ast::Char('a')),
                Box::new(Ast::Char('b')),
            ))),
        )
    }

    #[test]
    fn test_nested_grouping_alteration() -> () {
        let input = "a|(b|c)";
        let tokens = lexer::lex(input).unwrap();
        let ast = parse(&tokens).unwrap();
        assert_eq!(
            ast,
            Ast::Alt(
                Box::new(Ast::Char('a')),
                Box::new(Ast::Alt(Box::new(Ast::Char('b')), Box::new(Ast::Char('c')),)),
            ),
        )
    }

    #[test]
    fn test_nested_grouping_concatenation() -> () {
        let input = "(ab)c";
        let tokens = lexer::lex(input).unwrap();
        let ast = parse(&tokens).unwrap();
        assert_eq!(
            ast,
            Ast::Concat(
                Box::new(Ast::Concat(
                    Box::new(Ast::Char('a')),
                    Box::new(Ast::Char('b')),
                )),
                Box::new(Ast::Char('c')),
            ),
        )
    }

    #[test]
    #[rustfmt::skip]
    fn test_complex_expression() -> () {
        let input = "(a|b)*abb";
        let tokens = lexer::lex(input).unwrap();
        let ast = parse(&tokens).unwrap();
        assert_eq!(
            ast,
            Ast::Concat(
                Box::new(Ast::Concat(
                    Box::new(Ast::Concat(
                        Box::new(Ast::Star(
                            Box::new(Ast::Alt(
                                Box::new(Ast::Char('a')),
                                Box::new(Ast::Char('b')),
                            ))
                        )),
                        Box::new(Ast::Char('a')),
                    )),
                    Box::new(Ast::Char('b')),
                )),
                Box::new(Ast::Char('b')),
            ),
        )
    }
}
