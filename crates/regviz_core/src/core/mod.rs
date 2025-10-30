pub mod automaton;
pub mod dfa;
pub mod lexer;
pub mod min;
pub mod nfa;
pub mod parser;
pub mod sim;

use self::dfa::Dfa;
use self::nfa::Nfa;
use self::parser::Ast;

/// Aggregates the intermediate products generated while building automata
/// from a regular expression.
#[derive(Debug, Clone)]
pub struct BuildArtifacts {
    /// The parsed regular-expression abstract syntax tree.
    pub ast: Ast,
    /// The Thompson-constructed nondeterministic automaton.
    pub nfa: Nfa,
    /// The alphabet recognized by the NFA (and derived DFAs).
    pub alphabet: Vec<char>,
    /// A lazily computed DFA generated via subset construction.
    pub dfa: Option<Dfa>,
    /// A lazily computed minimal DFA.
    pub min_dfa: Option<Dfa>,
    /// Alphabet corresponding to the DFA transition table when available.
    pub dfa_alphabet: Option<Vec<char>>,
}

impl BuildArtifacts {
    /// Creates a new container for build artifacts.
    #[must_use]
    pub fn new(ast: Ast, nfa: Nfa, alphabet: Vec<char>) -> Self {
        Self {
            ast,
            nfa,
            alphabet,
            dfa: None,
            min_dfa: None,
            dfa_alphabet: None,
        }
    }
}
