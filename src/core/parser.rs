use crate::core::ast::Ast;
use crate::core::tokens::{Token, TokenKind};
use crate::errors::{ParseError, ParseErrorKind};

pub fn parse(tokens: &[Token]) -> Result<Ast, ParseError> {
    let mut parser = Parser { tokens, pos: 0 };
    let ast = parser.parse_regex()?;
    parser.expect(TokenKind::Eos)?;
    Ok(ast)
}

struct Parser<'a> {
    tokens: &'a [Token],
    pos: usize,
}

impl<'a> Parser<'a> {
    fn parse_regex(&mut self) -> Result<Ast, ParseError> {
        self.parse_alt()
    }

    fn parse_alt(&mut self) -> Result<Ast, ParseError> {
        let mut node = self.parse_concat()?;
        while self.matches(TokenKind::Or) {
            let rhs = self.parse_concat()?;
            node = Ast::alt(node, rhs);
        }
        Ok(node)
    }

    fn parse_concat(&mut self) -> Result<Ast, ParseError> {
        let mut nodes = Vec::new();
        while self.can_start_atom() {
            nodes.push(self.parse_repeat()?);
        }
        match nodes.len() {
            0 => Err(self.error_here(ParseErrorKind::EmptyAlternative)),
            1 => Ok(nodes.remove(0)),
            _ => {
                let mut it = nodes.into_iter();
                let mut acc = it.next().expect("checked len");
                for node in it {
                    acc = Ast::concat(acc, node);
                }
                Ok(acc)
            }
        }
    }

    fn parse_repeat(&mut self) -> Result<Ast, ParseError> {
        let mut node = self.parse_atom()?;
        loop {
            let apply_fn: Option<fn(Ast) -> Ast> = match self.peek_kind() {
                Some(TokenKind::Star) => Some(Ast::star),
                Some(TokenKind::Plus) => Some(Ast::plus),
                Some(TokenKind::QMark) => Some(Ast::opt),
                _ => None,
            };
            if let Some(apply) = apply_fn {
                self.pos += 1;
                node = apply(node);
            } else {
                break;
            }
        }
        Ok(node)
    }

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

    fn can_start_atom(&self) -> bool {
        matches!(
            self.peek_kind(),
            Some(TokenKind::Char(_)) | Some(TokenKind::LParen)
        )
    }

    fn matches(&mut self, kind: TokenKind) -> bool {
        if self.peek_kind() == Some(kind) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, kind: TokenKind) -> Result<(), ParseError> {
        if self.peek_kind() == Some(kind) {
            self.pos += 1;
            Ok(())
        } else {
            let err = match self.peek() {
                Some(tok) => ParseError::new(
                    tok.pos,
                    ParseErrorKind::UnexpectedToken {
                        found: tok.kind.to_string(),
                    },
                ),
                None => ParseError::new(self.last_column(), ParseErrorKind::UnexpectedEos),
            };
            Err(err)
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
        if let Some(last) = self.tokens.last() {
            last.pos
        } else {
            0
        }
    }
}
