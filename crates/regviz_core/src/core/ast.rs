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
