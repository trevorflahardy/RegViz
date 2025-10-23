use std::collections::HashSet;

use crate::core::ast::Ast;

/// Identifier type for NFA states.
pub type StateId = u32;

/// Labels describing the kind of transition.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeLabel {
    /// Epsilon transition that consumes no input.
    Eps,
    /// Consumes a specific symbol.
    Sym(char),
}

/// A flattened representation of a transition, useful for visualization.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Edge {
    /// Origin state.
    pub from: StateId,
    /// Destination state.
    pub to: StateId,
    /// Transition label.
    pub label: EdgeLabel,
}

/// Transition stored in adjacency lists.
#[derive(Debug, Clone)]
pub struct Transition {
    /// Destination state.
    pub to: StateId,
    /// Transition label.
    pub label: EdgeLabel,
}

/// Represents a Thompson-constructed nondeterministic finite automaton.
#[derive(Debug, Clone)]
pub struct Nfa {
    /// All known states.
    pub states: Vec<StateId>,
    /// Start state.
    pub start: StateId,
    /// Accepting states.
    pub accepts: Vec<StateId>,
    /// Flattened edge list.
    pub edges: Vec<Edge>,
    /// Adjacency lists for efficient traversal.
    pub adjacency: Vec<Vec<Transition>>,
}

impl Nfa {
    ///
    /// # Arguments
    ///
    /// - `state` (`StateId`) - The state whose transitions are to be retrieved.
    ///
    /// # Returns
    ///
    /// - `&[Transition]` - The outgoing transitions from the specified state.
    pub fn transitions(&self, state: StateId) -> &[Transition] {
        &self.adjacency[state as usize]
    }

    /// Computes the alphabet used in this NFA, sorted by character.
    ///
    /// # Returns
    ///
    /// - `Vec<char>` - The sorted alphabet recognized by this automaton.
    pub fn alphabet(&self) -> Vec<char> {
        let mut chars: HashSet<char> = HashSet::new();
        for row in &self.adjacency {
            for tr in row {
                if let EdgeLabel::Sym(c) = tr.label {
                    chars.insert(c);
                }
            }
        }
        let mut chars: Vec<char> = chars.into_iter().collect();
        chars.sort_unstable();
        chars
    }
}

/// Builds an [`Nfa`] using Thompson's construction algorithm.
///
/// # Arguments
///
/// - `ast` (`&Ast`) - The abstract syntax tree representing the regular expression. Will be cloned.
///
/// # Returns
///
/// - `Nfa` - The constructed nondeterministic finite automaton.
pub fn build_nfa(ast: &Ast) -> Nfa {
    let mut builder = Builder::default();
    let fragment = builder.build(ast.clone());
    builder.finalize(fragment.start, fragment.accepts)
}

#[derive(Default)]
struct Builder {
    adjacency: Vec<Vec<Transition>>,
}

#[derive(Debug, Clone)]
struct Fragment {
    start: StateId,
    accepts: Vec<StateId>,
}

impl Builder {
    fn new_state(&mut self) -> StateId {
        let id = self.adjacency.len() as StateId;
        self.adjacency.push(Vec::new());
        id
    }

    fn add_edge(&mut self, from: StateId, to: StateId, label: EdgeLabel) {
        self.adjacency[from as usize].push(Transition { to, label });
    }

    fn build(&mut self, ast: Ast) -> Fragment {
        match ast {
            Ast::Char(c) => self.build_char(c),
            Ast::Concat(lhs, rhs) => self.build_concat(*lhs, *rhs),
            Ast::Alt(lhs, rhs) => self.build_alternation(*lhs, *rhs),
            Ast::Star(inner) => self.build_star(*inner),
            Ast::Plus(inner) => self.build_plus(*inner),
            Ast::Opt(inner) => self.build_optional(*inner),
        }
    }

    fn build_char(&mut self, ch: char) -> Fragment {
        let start = self.new_state();
        let accept = self.new_state();
        self.add_edge(start, accept, EdgeLabel::Sym(ch));
        Fragment {
            start,
            accepts: vec![accept],
        }
    }

    fn build_concat(&mut self, lhs: Ast, rhs: Ast) -> Fragment {
        let left = self.build(lhs);
        let right = self.build(rhs);
        for accept in &left.accepts {
            self.add_edge(*accept, right.start, EdgeLabel::Eps);
        }
        Fragment {
            start: left.start,
            accepts: right.accepts,
        }
    }

    fn build_alternation(&mut self, lhs: Ast, rhs: Ast) -> Fragment {
        let left = self.build(lhs);
        let right = self.build(rhs);
        let start = self.new_state();
        let accept = self.new_state();
        self.add_edge(start, left.start, EdgeLabel::Eps);
        self.add_edge(start, right.start, EdgeLabel::Eps);
        for state in left.accepts.iter().chain(right.accepts.iter()) {
            self.add_edge(*state, accept, EdgeLabel::Eps);
        }
        Fragment {
            start,
            accepts: vec![accept],
        }
    }

    fn build_star(&mut self, inner: Ast) -> Fragment {
        let frag = self.build(inner);
        let start = self.new_state();
        let accept = self.new_state();
        self.add_edge(start, frag.start, EdgeLabel::Eps);
        self.add_edge(start, accept, EdgeLabel::Eps);
        for state in frag.accepts {
            self.add_edge(state, frag.start, EdgeLabel::Eps);
            self.add_edge(state, accept, EdgeLabel::Eps);
        }
        Fragment {
            start,
            accepts: vec![accept],
        }
    }

    fn build_plus(&mut self, inner: Ast) -> Fragment {
        let frag = self.build(inner);
        let start = self.new_state();
        let accept = self.new_state();
        self.add_edge(start, frag.start, EdgeLabel::Eps);
        for state in &frag.accepts {
            self.add_edge(*state, frag.start, EdgeLabel::Eps);
            self.add_edge(*state, accept, EdgeLabel::Eps);
        }
        Fragment {
            start,
            accepts: vec![accept],
        }
    }

    fn build_optional(&mut self, inner: Ast) -> Fragment {
        let frag = self.build(inner);
        let start = self.new_state();
        let accept = self.new_state();
        self.add_edge(start, frag.start, EdgeLabel::Eps);
        self.add_edge(start, accept, EdgeLabel::Eps);
        for state in frag.accepts {
            self.add_edge(state, accept, EdgeLabel::Eps);
        }
        Fragment {
            start,
            accepts: vec![accept],
        }
    }

    fn finalize(self, start: StateId, accepts: Vec<StateId>) -> Nfa {
        let accepts = unique_sorted(accepts);
        let states: Vec<StateId> = (0..self.adjacency.len()).map(|i| i as StateId).collect();
        let mut edges = Vec::new();
        for (from, row) in self.adjacency.iter().enumerate() {
            for tr in row {
                edges.push(Edge {
                    from: from as StateId,
                    to: tr.to,
                    label: tr.label,
                });
            }
        }
        Nfa {
            states,
            start,
            accepts,
            edges,
            adjacency: self.adjacency,
        }
    }
}

fn unique_sorted(mut ids: Vec<StateId>) -> Vec<StateId> {
    ids.sort_unstable();
    ids.dedup();
    ids
}
