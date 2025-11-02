use std::fmt::Display;

use crate::{
    core::lexer::{Lexer, OpToken, Token},
    errors::{BuildError, ParseError, ParseErrorKind},
};

/// An abstract syntax tree for a regular expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ast {
    /// Epsilon (empty string).
    Epsilon,
    /// A literal character.
    Atom(char),
    /// Concatenation of two expressions.
    Concat(Box<Ast>, Box<Ast>),
    /// Alternation between two expressions.
    Alt(Box<Ast>, Box<Ast>),
    /// Zero-or-more repetition.
    Star(Box<Ast>),
}

/// Infix operator definition for Pratt parsing.
pub struct InfixOp {
    /// Left binding power.
    pub left_bp: u8,
    /// Right binding power.
    pub right_bp: u8,
    /// Build function to create AST node.
    pub build: fn(Ast, Ast) -> Ast,
}

/// Postfix operator definition for Pratt parsing.
pub struct PostfixOp {
    /// Binding power (on the left of the operator token).
    pub bp: u8,
    /// Build function to create AST node.
    pub build: fn(Ast) -> Ast,
}

/// Prefix operator definition for Pratt parsing.
pub struct PrefixOp {
    /// Binding power (on the right of the operator token).
    pub bp: u8,
    /// Build function to create AST node.
    pub build: fn(Ast) -> Ast,
}

impl OpToken {
    /// Returns the infix operator definition if this token represents an infix operator.
    /// Returns `None` otherwise.
    pub const fn infix(&self) -> Option<InfixOp> {
        match self {
            Self::Plus => Some(InfixOp {
                left_bp: 1,
                right_bp: 2,
                build: |l, r| Ast::Alt(Box::new(l), Box::new(r)),
            }),
            Self::Dot => Some(InfixOp {
                left_bp: 3,
                right_bp: 4,
                build: |l, r| Ast::Concat(Box::new(l), Box::new(r)),
            }),
            Self::Star => None,
        }
    }

    /// Returns the postfix operator definition if this token represents a postfix operator.
    /// Returns `None` otherwise.
    pub const fn postfix(&self) -> Option<PostfixOp> {
        match self {
            Self::Star => Some(PostfixOp {
                bp: 5,
                build: |operand| Ast::Star(Box::new(operand)),
            }),
            _ => None,
        }
    }

    /// Returns the prefix operator definition if this token represents a prefix operator.
    /// Returns `None` otherwise.
    pub const fn prefix(&self) -> Option<PrefixOp> {
        // NOTE: No prefix operators defined yet. We can add support for those if we want.
        None
    }
}

impl Ast {
    /// Builds an AST from the input regular expression string.
    /// Returns a [`BuildError`] if lexing or parsing fails.
    pub fn build(input: &str) -> Result<Ast, BuildError> {
        let mut lexer = Lexer::new(input)?;
        if let Token::Eof = lexer.peek().0 {
            // Empty input, interpreted as epsilon AST
            return Ok(Ast::Epsilon);
        }
        let ast = Ast::parse(&mut lexer, 0, false)?;
        Ok(ast)
    }

    /// Parses an expression from the lexer using Pratt parsing with the given minimum binding power.
    ///
    /// # Parameters
    ///
    /// * `lexer` - The lexer to read tokens from
    /// * `min_bp` - Minimum binding power. Operators with lower binding power than this will not be consumed,
    ///   causing the parser to return the current left-hand side. This is used to impose operator precedence.
    /// * `open_paren` - Whether we're currently inside a parenthesized sub-expression.
    ///
    /// Returns a [`ParseError`] if parsing fails.
    fn parse(lexer: &mut Lexer, min_bp: u8, open_paren: bool) -> Result<Ast, ParseError> {
        // Phase 1: parse primary
        //
        // Read the next token and convert it into the initial `lhs`.
        // This covers:
        //  - literal atoms
        //  - prefix operators
        //  - left parenthesis (recursively parse sub-expression)
        //
        // We capture the token index `idx` (char index) for error reporting.
        let (token, idx) = lexer.advance();
        let mut lhs = match token {
            Token::Literal(c) => Ast::Atom(c),
            Token::Op(op_token) => {
                if let Some(prefix_op) = op_token.prefix() {
                    // Found prefix operator, parse right-hand side
                    let rhs = Ast::parse(lexer, prefix_op.bp, open_paren)?;
                    (prefix_op.build)(rhs)
                } else {
                    return Err(ParseError {
                        at: idx,
                        kind: ParseErrorKind::UnexpectedPrefixOperator(op_token),
                    });
                }
            }
            Token::LParen => {
                // Parse sub-expression
                let sub_expr = Ast::parse(lexer, 0, true)?;

                // Expect closing parenthesis
                let (token, idx) = lexer.advance();
                match token {
                    Token::RParen => sub_expr,
                    other => {
                        return Err(ParseError {
                            at: idx,
                            kind: ParseErrorKind::MismatchedLeftParen { other },
                        });
                    }
                }
            }
            Token::RParen => {
                if open_paren {
                    // Found empty parentheses '()'
                    return Err(ParseError {
                        at: idx,
                        kind: ParseErrorKind::EmptyParentheses,
                    });
                }
                return Err(ParseError {
                    at: idx,
                    kind: ParseErrorKind::RightParenWithoutLeft,
                });
            }
            Token::Eof => {
                return Err(ParseError {
                    at: idx,
                    kind: ParseErrorKind::UnexpectedEof,
                });
            }
        };

        // Phase 2: Pratt loop - consume postfix/infix operators as needed
        //
        // Repeatedly inspect the next token and decide whether to:
        //  - treat it as implicit concatenation (when next is literal or '(')
        //  - consume an explicit operator (OpToken)
        //  - or stop and return `lhs` because the next token doesn't bind strongly enough.
        //
        // The distinction between explicit vs implicit: implicit concatenation
        // does not advance the lexer before we build the AST (we synthesize OpToken::Dot),
        // while explicit operators require consuming the token with `advance()`.
        loop {
            let (token, idx) = lexer.peek();
            let (op_token, is_explicit) = match token {
                Token::Literal(_) | Token::LParen => {
                    // Implicit concatenation
                    (OpToken::Dot, false)
                }
                Token::Op(op_token) => {
                    if op_token.prefix().is_some() {
                        // Next token is a prefix operator, interpret as implicit concatenation
                        (OpToken::Dot, false)
                    } else {
                        // Explicit operator
                        (op_token, true)
                    }
                }
                Token::RParen => {
                    if open_paren {
                        // Reached the end of a parenthesized expression
                        break;
                    } else {
                        return Err(ParseError {
                            at: idx,
                            kind: ParseErrorKind::RightParenWithoutLeft,
                        });
                    }
                }
                Token::Eof => break,
            };

            // NOTE: Check for postfix operator first, then infix operator (to handle cases like a*+b)
            // Postfix operators naturally bind tighter than infix operators
            if let Some(postfix_op) = op_token.postfix() {
                if postfix_op.bp < min_bp {
                    // Postfix operator does not bind strong enough, lhs is complete
                    break;
                }

                // Consume the postfix operator if it was explicit
                if is_explicit {
                    lexer.advance();
                }

                lhs = (postfix_op.build)(lhs);
                continue;
            } else if let Some(infix_op) = op_token.infix() {
                if infix_op.left_bp < min_bp {
                    // Infix operator does not bind strong enough, lhs is complete
                    break;
                }

                // Consume the infix operator if it was explicit
                if is_explicit {
                    lexer.advance();
                }

                let rhs = Ast::parse(lexer, infix_op.right_bp, open_paren)?;
                lhs = (infix_op.build)(lhs, rhs);
                continue;
            }

            // NOTE: This is where we find out that the operator is neither postfix nor infix,
            // thus should not be included in the AST. We break out of the loop, and let
            // the caller handle the lhs.
            // For example, consider the '~' operator as a prefix operator only.
            // The expression "a~b" would parse 'a' as lhs, then find '~' which is neither postfix nor infix,
            // and break out of the loop, leaving 'a' to be handled by the caller and ignoring '~b'.
            break;
        }

        Ok(lhs)
    }
}

impl Display for Ast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Print the AST in unambiguous S-expression format
        match self {
            Ast::Epsilon => write!(f, "ε"),
            Ast::Atom(c) => write!(f, "{c}"),
            Ast::Concat(lhs, rhs) => write!(f, "(. {lhs} {rhs})"),
            Ast::Alt(lhs, rhs) => write!(f, "(+ {lhs} {rhs})"),
            Ast::Star(inner) => write!(f, "(* {inner})"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_input() {
        let ast = Ast::build("").unwrap();
        assert_eq!(ast.to_string(), "ε");
    }

    #[test]
    fn test_single_literal() {
        let ast = Ast::build("a").unwrap();
        assert_eq!(ast.to_string(), "a");
    }

    #[test]
    fn test_single_parentheses() {
        let ast = Ast::build("(a)").unwrap();
        assert_eq!(ast.to_string(), "a");
    }

    #[test]
    fn test_nested_parentheses() {
        let ast = Ast::build("(((a)))").unwrap();
        assert_eq!(ast.to_string(), "a");
    }

    #[test]
    fn test_missing_parentheses() {
        let result = Ast::build("((a)");
        assert_eq!(
            result,
            Err(BuildError::Parse(ParseError {
                kind: ParseErrorKind::MismatchedLeftParen { other: Token::Eof },
                at: 4,
            }))
        );
        let result = Ast::build("((a)b");
        assert_eq!(
            result,
            Err(BuildError::Parse(ParseError {
                kind: ParseErrorKind::MismatchedLeftParen { other: Token::Eof },
                at: 5,
            }))
        );
        let result = Ast::build("a)))");
        assert_eq!(
            result,
            Err(BuildError::Parse(ParseError {
                kind: ParseErrorKind::RightParenWithoutLeft,
                at: 1,
            }))
        );
        let result = Ast::build("((a)())");
        assert_eq!(
            result,
            Err(BuildError::Parse(ParseError {
                kind: ParseErrorKind::EmptyParentheses,
                at: 5,
            }))
        );
        let result = Ast::build("()");
        assert_eq!(
            result,
            Err(BuildError::Parse(ParseError {
                kind: ParseErrorKind::EmptyParentheses,
                at: 1,
            }))
        );
        let result = Ast::build("(())");
        assert_eq!(
            result,
            Err(BuildError::Parse(ParseError {
                kind: ParseErrorKind::EmptyParentheses,
                at: 2,
            }))
        );
        let result = Ast::build("a.b*()");
        assert_eq!(
            result,
            Err(BuildError::Parse(ParseError {
                kind: ParseErrorKind::EmptyParentheses,
                at: 5,
            }))
        );
    }

    #[test]
    fn test_single_concatenation() {
        let ast = Ast::build("ab").unwrap();
        assert_eq!(ast.to_string(), "(. a b)");
    }

    #[test]
    fn test_multiple_concatenation() {
        let ast = Ast::build("abc").unwrap();
        assert_eq!(ast.to_string(), "(. (. a b) c)");
    }

    #[test]
    fn test_mixed_dot_and_literal() {
        let ast = Ast::build("a.bc").unwrap();
        assert_eq!(ast.to_string(), "(. (. a b) c)");
    }

    #[test]
    fn test_single_alternation() {
        let ast = Ast::build("a+b").unwrap();
        assert_eq!(ast.to_string(), "(+ a b)");
    }

    #[test]
    fn test_multiple_alternation() {
        let ast = Ast::build("a+b+c").unwrap();
        assert_eq!(ast.to_string(), "(+ (+ a b) c)");
    }

    #[test]
    fn test_mixed_infix_operators() {
        let ast = Ast::build("a+bc+d").unwrap();
        assert_eq!(ast.to_string(), "(+ (+ a (. b c)) d)");
    }

    #[test]
    fn test_parentheses_with_infix_operators() {
        let ast = Ast::build("(a+b)c").unwrap();
        assert_eq!(ast.to_string(), "(. (+ a b) c)");
    }

    #[test]
    fn test_single_kleene_star() {
        let ast = Ast::build("a*").unwrap();
        assert_eq!(ast.to_string(), "(* a)");
    }

    #[test]
    fn test_multiple_kleene_stars() {
        let ast = Ast::build("a**").unwrap();
        assert_eq!(ast.to_string(), "(* (* a))");
    }

    #[test]
    fn test_mixed_postfix_and_infix() {
        let ast = Ast::build("a*b+c*").unwrap();
        assert_eq!(ast.to_string(), "(+ (. (* a) b) (* c))");
        let ast = Ast::build("a+b*cd").unwrap();
        assert_eq!(ast.to_string(), "(+ a (. (. (* b) c) d))");
        let ast = Ast::build("a+b*c.d+e").unwrap();
        assert_eq!(ast.to_string(), "(+ (+ a (. (. (* b) c) d)) e)");
    }

    #[test]
    fn test_parentheses_with_postfix_and_infix() {
        let ast = Ast::build("(a+b)*c").unwrap();
        assert_eq!(ast.to_string(), "(. (* (+ a b)) c)");
        let ast = Ast::build("a*(b+c)*+d").unwrap();
        assert_eq!(ast.to_string(), "(+ (. (* a) (* (+ b c))) d)");
        let ast = Ast::build("a+(b.c*)*").unwrap();
        assert_eq!(ast.to_string(), "(+ a (* (. b (* c))))");
    }

    #[test]
    fn test_wrong_prefix_operator() {
        let result = Ast::build("+a");
        assert_eq!(
            result,
            Err(BuildError::Parse(ParseError {
                kind: ParseErrorKind::UnexpectedPrefixOperator(OpToken::Plus),
                at: 0,
            }))
        );
        let result = Ast::build("a*+.d");
        assert_eq!(
            result,
            Err(BuildError::Parse(ParseError {
                kind: ParseErrorKind::UnexpectedPrefixOperator(OpToken::Dot),
                at: 3,
            }))
        );
        let result = Ast::build("a(bc)*+*d");
        assert_eq!(
            result,
            Err(BuildError::Parse(ParseError {
                kind: ParseErrorKind::UnexpectedPrefixOperator(OpToken::Star),
                at: 7,
            }))
        );
    }

    #[test]
    fn test_unexpected_eof() {
        let result = Ast::build("a+");
        assert_eq!(
            result,
            Err(BuildError::Parse(ParseError {
                kind: ParseErrorKind::UnexpectedEof,
                at: 2,
            }))
        );
        let result = Ast::build("a*+");
        assert_eq!(
            result,
            Err(BuildError::Parse(ParseError {
                kind: ParseErrorKind::UnexpectedEof,
                at: 3,
            }))
        );
        let result = Ast::build("a.b*(");
        assert_eq!(
            result,
            Err(BuildError::Parse(ParseError {
                kind: ParseErrorKind::UnexpectedEof,
                at: 5,
            }))
        );
        let result = Ast::build("(ab");
        assert_eq!(
            result,
            Err(BuildError::Parse(ParseError {
                kind: ParseErrorKind::MismatchedLeftParen { other: Token::Eof },
                at: 3,
            }))
        );
    }
}
