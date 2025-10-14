use std::collections::HashSet;

use crate::core::ast::Ast;

pub type StateId = u32;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EdgeLabel {
    Eps,
    Sym(char),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Edge {
    pub from: StateId,
    pub to: StateId,
    pub label: EdgeLabel,
}

#[derive(Debug, Clone)]
pub struct Transition {
    pub to: StateId,
    pub label: EdgeLabel,
}

#[derive(Debug, Clone)]
pub struct Nfa {
    pub states: Vec<StateId>,
    pub start: StateId,
    pub accepts: Vec<StateId>,
    pub edges: Vec<Edge>,
    pub adjacency: Vec<Vec<Transition>>,
}

impl Nfa {
    pub fn transitions(&self, state: StateId) -> &[Transition] {
        &self.adjacency[state as usize]
    }

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
            Ast::Concat(lhs, rhs) => {
                let left = self.build(*lhs);
                let right = self.build(*rhs);
                for accept in &left.accepts {
                    self.add_edge(*accept, right.start, EdgeLabel::Eps);
                }
                Fragment {
                    start: left.start,
                    accepts: right.accepts,
                }
            }
            Ast::Alt(lhs, rhs) => {
                let left = self.build(*lhs);
                let right = self.build(*rhs);
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
            Ast::Star(inner) => {
                let frag = self.build(*inner);
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
            Ast::Plus(inner) => {
                let frag = self.build(*inner);
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
            Ast::Opt(inner) => {
                let frag = self.build(*inner);
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
