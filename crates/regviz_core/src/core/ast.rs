use std::fmt::Display;

/// An abstract syntax tree for a regular expression.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ast {
    /// A literal character.
    Char(char),
    /// Concatenation of two expressions.
    Concat(Box<Ast>, Box<Ast>),
    /// Alternation between two expressions.
    Alt(Box<Ast>, Box<Ast>),
    /// Zero-or-more repetition.
    Star(Box<Ast>),
    /// One-or-more repetition.
    Plus(Box<Ast>),
    /// Optional expression.
    Opt(Box<Ast>),
}

impl Ast {
    /// Creates a concatenation node.
    #[must_use]
    pub fn concat(lhs: Ast, rhs: Ast) -> Ast {
        Ast::Concat(Box::new(lhs), Box::new(rhs))
    }

    /// Creates an alternation node.
    #[must_use]
    pub fn alt(lhs: Ast, rhs: Ast) -> Ast {
        Ast::Alt(Box::new(lhs), Box::new(rhs))
    }

    /// Creates a Kleene star node.
    #[must_use]
    pub fn star(inner: Ast) -> Ast {
        Ast::Star(Box::new(inner))
    }

    /// Creates a Kleene plus node.
    #[must_use]
    pub fn plus(inner: Ast) -> Ast {
        Ast::Plus(Box::new(inner))
    }

    /// Creates an optional node.
    #[must_use]
    pub fn opt(inner: Ast) -> Ast {
        Ast::Opt(Box::new(inner))
    }
}

impl Display for Ast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn fmt_with_indent(
            ast: &Ast,
            f: &mut std::fmt::Formatter<'_>,
            indent: usize,
        ) -> std::fmt::Result {
            let pad = "| ".repeat(indent);
            match ast {
                Ast::Char(c) => writeln!(f, "{}Char({})", pad, c),
                Ast::Concat(lhs, rhs) => {
                    writeln!(f, "{}Concat", pad)?;
                    fmt_with_indent(lhs, f, indent + 1)?;
                    fmt_with_indent(rhs, f, indent + 1)
                }
                Ast::Alt(lhs, rhs) => {
                    writeln!(f, "{}Alt", pad)?;
                    fmt_with_indent(lhs, f, indent + 1)?;
                    fmt_with_indent(rhs, f, indent + 1)
                }
                Ast::Star(inner) => {
                    writeln!(f, "{}Star", pad)?;
                    fmt_with_indent(inner, f, indent + 1)
                }
                Ast::Plus(inner) => {
                    writeln!(f, "{}Plus", pad)?;
                    fmt_with_indent(inner, f, indent + 1)
                }
                Ast::Opt(inner) => {
                    writeln!(f, "{}Opt", pad)?;
                    fmt_with_indent(inner, f, indent + 1)
                }
            }
        }
        writeln!(f, "\n")?;
        fmt_with_indent(self, f, 0)
    }
}
