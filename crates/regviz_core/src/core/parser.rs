use std::fmt::Display;

use crate::{
    core::lexer::Lexer,
    errors::{BuildError, ParseError},
};

/// An abstract syntax tree for a regular expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ast {
    /// A literal character.
    Atom(char),
    /// Concatenation of two expressions.
    Concat(Box<Ast>, Box<Ast>),
    /// Alternation between two expressions.
    Alt(Box<Ast>, Box<Ast>),
    /// Zero-or-more repetition.
    Star(Box<Ast>),
}

impl Ast {
    pub fn build(input: &str) -> Result<Ast, BuildError> {
        let mut lexer = Lexer::new(input)?;
        let ast = Ast::parse(&mut lexer)?;
        Ok(ast)
    }

    pub fn parse(_lexer: &mut Lexer) -> Result<Ast, ParseError> {
        unimplemented!()
    }
}

impl Display for Ast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // In-fix notation with parentheses for clarity
        match self {
            Ast::Atom(c) => write!(f, "{}", c),
            Ast::Concat(lhs, rhs) => write!(f, "(. {} {})", lhs, rhs),
            Ast::Alt(lhs, rhs) => write!(f, "(+ {} {})", lhs, rhs),
            Ast::Star(inner) => write!(f, "(* {})", inner),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_alteration() {
        let input = "a|b";
        let ast = Ast::build(input).unwrap();

        assert_eq!(
            ast,
            Ast::Alt(Box::new(Ast::Atom('a')), Box::new(Ast::Atom('b'))),
        )
    }

    #[test]
    fn test_concatenation() {
        let input = "ab";
        let ast = Ast::build(input).unwrap();
        assert_eq!(
            ast,
            Ast::Concat(Box::new(Ast::Atom('a')), Box::new(Ast::Atom('b'))),
        )
    }

    #[test]
    fn test_star() {
        let input = "a*";
        let ast = Ast::build(input).unwrap();
        assert_eq!(ast, Ast::Star(Box::new(Ast::Atom('a'))))
    }

    #[test]
    fn test_grouping() {
        let input = "(a|b)c";
        let ast = Ast::build(input).unwrap();
        assert_eq!(
            ast,
            Ast::Concat(
                Box::new(Ast::Alt(Box::new(Ast::Atom('a')), Box::new(Ast::Atom('b')))),
                Box::new(Ast::Atom('c')),
            ),
        )
    }

    #[test]
    fn test_grouping_star() {
        let input = "(a|b)*";
        let ast = Ast::build(input).unwrap();
        assert_eq!(
            ast,
            Ast::Star(Box::new(Ast::Alt(
                Box::new(Ast::Atom('a')),
                Box::new(Ast::Atom('b')),
            ))),
        )
    }

    #[test]
    fn test_nested_grouping_alteration() {
        let input = "a|(b|c)";
        let ast = Ast::build(input).unwrap();
        assert_eq!(
            ast,
            Ast::Alt(
                Box::new(Ast::Atom('a')),
                Box::new(Ast::Alt(Box::new(Ast::Atom('b')), Box::new(Ast::Atom('c')),)),
            ),
        )
    }

    #[test]
    fn test_nested_grouping_concatenation() {
        let input = "(ab)c";
        let ast = Ast::build(input).unwrap();
        assert_eq!(
            ast,
            Ast::Concat(
                Box::new(Ast::Concat(
                    Box::new(Ast::Atom('a')),
                    Box::new(Ast::Atom('b')),
                )),
                Box::new(Ast::Atom('c')),
            ),
        )
    }

    #[test]
    #[rustfmt::skip]
    fn test_complex_expression() {
        let input = "(a|b)*abb";
let ast = Ast::build(input).unwrap();
        assert_eq!(
            ast,
            Ast::Concat(
                Box::new(Ast::Concat(
                    Box::new(Ast::Concat(
                        Box::new(Ast::Star(
                            Box::new(Ast::Alt(
                                Box::new(Ast::Atom('a')),
                                Box::new(Ast::Atom('b')),
                            ))
                        )),
                        Box::new(Ast::Atom('a')),
                    )),
                    Box::new(Ast::Atom('b')),
                )),
                Box::new(Ast::Atom('b')),
            ),
        )
    }
}
