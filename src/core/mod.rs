pub mod ast;
pub mod dfa;
pub mod lexer;
pub mod min;
pub mod nfa;
pub mod parser;
pub mod sim;
pub mod tokens;

use self::ast::Ast;
use self::dfa::Dfa;
use self::nfa::Nfa;

#[derive(Debug, Clone)]
pub struct BuildArtifacts {
    pub ast: Ast,
    pub nfa: Nfa,
    pub alphabet: Vec<char>,
    pub dfa: Option<Dfa>,
    pub min_dfa: Option<Dfa>,
}

impl BuildArtifacts {
    pub fn new(ast: Ast, nfa: Nfa, alphabet: Vec<char>) -> Self {
        Self {
            ast,
            nfa,
            alphabet,
            dfa: None,
            min_dfa: None,
        }
    }
}
