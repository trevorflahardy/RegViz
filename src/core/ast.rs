#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ast {
    Char(char),
    Concat(Box<Ast>, Box<Ast>),
    Alt(Box<Ast>, Box<Ast>),
    Star(Box<Ast>),
    Plus(Box<Ast>),
    Opt(Box<Ast>),
}

impl Ast {
    pub fn concat(lhs: Ast, rhs: Ast) -> Ast {
        Ast::Concat(Box::new(lhs), Box::new(rhs))
    }

    pub fn alt(lhs: Ast, rhs: Ast) -> Ast {
        Ast::Alt(Box::new(lhs), Box::new(rhs))
    }

    pub fn star(inner: Ast) -> Ast {
        Ast::Star(Box::new(inner))
    }

    pub fn plus(inner: Ast) -> Ast {
        Ast::Plus(Box::new(inner))
    }

    pub fn opt(inner: Ast) -> Ast {
        Ast::Opt(Box::new(inner))
    }
}
