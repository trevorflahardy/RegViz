use crate::core::ast::Ast;
use std::collections::HashSet;

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

impl Into<String> for EdgeLabel {
    fn into(self) -> String {
        match self {
            EdgeLabel::Eps => "ε".to_string(),
            EdgeLabel::Sym(c) => c.to_string(),
        }
    }
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

/// The internal builder struct for converting an AST to an NFA.
/// Holds adjacency lists and provides methods for constructing NFA fragments.
#[derive(Default)]
struct Builder {
    /// A list of adjacency lists for each state, where each list contains outgoing transitions.
    adjacency: Vec<Vec<Transition>>,
}

/// Represents a fragment of an NFA with a start state and accepting states.
/// Created when building NFA components from AST nodes.
#[derive(Debug, Clone)]
struct Fragment {
    /// The ID of the start state.
    start: StateId,
    /// A list of accepting state IDs.
    accepts: Vec<StateId>,
}

impl Builder {
    /// Adds a new, empty state to this NFA with no outgoing or incoming transitions.
    ///
    /// # Returns
    ///
    /// - `StateId` - The identifier of the newly created state.
    fn new_state(&mut self) -> StateId {
        let id = self.adjacency.len() as StateId;
        self.adjacency.push(Vec::new());
        id
    }

    /// Adds an edge on the graph from->to with the given label.
    ///
    /// # Arguments
    ///
    /// - `from` (`StateId`) - The source state of the edge.
    /// - `to` (`StateId`) - The destination state of the edge.
    /// - `label` (`EdgeLabel`) - The label on the edge.
    ///
    /// # Returns
    /// None
    fn add_edge(&mut self, from: StateId, to: StateId, label: EdgeLabel) -> () {
        self.adjacency[from as usize].push(Transition { to, label });
    }

    /// A general blanket to build from a given AST input.
    ///
    /// # Arguments
    ///
    /// - `ast` (`Ast`) - The abstract syntax tree node to build from.
    ///
    /// # Returns
    ///
    /// - `Fragment` - The NFA fragment constructed from the AST node.
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

    /// Builds a character AST symbol within the NFA. Creates a start and accept state,
    /// connecting the two with an edge labeled with the character.
    ///
    /// # Arguments
    ///
    /// - `ch` (`char`) - The character to build the NFA fragment for.
    ///
    /// # Returns
    ///
    /// - `Fragment` - The NFA fragment representing the character.
    fn build_char(&mut self, ch: char) -> Fragment {
        let start = self.new_state();
        let accept = self.new_state();

        self.add_edge(start, accept, EdgeLabel::Sym(ch));

        Fragment {
            start,
            accepts: vec![accept],
        }
    }

    /// Builds both the left hand side (lhs) and right hand side (rhs), connecting the accept states from the
    /// left hand side to the input of the right hand side.
    ///
    /// # Arguments
    ///
    /// - `lhs` (`Ast`) - The left hand side AST node.
    /// - `rhs` (`Ast`) - The right hand side AST node.
    ///
    /// # Returns
    ///
    /// - `Fragment` - The NFA fragment representing the concatenation.
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

    /// Converts the alteration (or OR) '|' AST into its NFA fragment representation.
    ///
    /// # Arguments
    ///
    /// - `lhs` (`Ast`) - The left hand side AST node.
    /// - `rhs` (`Ast`) - The right hand side AST node.
    ///
    /// # Returns
    ///
    /// - `Fragment` - The NFA fragment representing the alternation.
    fn build_alternation(&mut self, lhs: Ast, rhs: Ast) -> Fragment {
        let left = self.build(lhs);
        let right = self.build(rhs);

        // New start and accept states sit between the OR operation.
        let start = self.new_state();
        let accept = self.new_state();

        // We can walk as an edge from the new start to either side.
        self.add_edge(start, left.start, EdgeLabel::Eps);
        self.add_edge(start, right.start, EdgeLabel::Eps);

        // And for each accept state on either side, we can walk to the new accept.
        for state in left.accepts.iter().chain(right.accepts.iter()) {
            self.add_edge(*state, accept, EdgeLabel::Eps);
        }

        Fragment {
            start,
            accepts: vec![accept],
        }
    }

    /// Builds the klnee-star operation from the given inner AST/
    ///
    /// # Arguments
    ///
    /// - `inner` (`Ast`) - The inner AST node to apply the kleene star operation on.
    ///
    /// # Returns
    ///
    /// - `Fragment` - The NFA fragment representing the kleene star operation.
    fn build_star(&mut self, inner: Ast) -> Fragment {
        // Build our NFA fragment.
        let frag = self.build(inner);

        // Create two fresh states: start for the started fragment, and a new accept for the entire star fragment.
        let start = self.new_state();
        let accept = self.new_state();

        // Add EPS transitions from the new start to:
        // - the inner fragment's start (to begin one or more iterations)
        // - directly to the new accept (to allow zero iterations)
        self.add_edge(start, frag.start, EdgeLabel::Eps);
        self.add_edge(start, accept, EdgeLabel::Eps);

        // For each accept state of the inner fragment, add EPS transitions to:
        // - Back to the inner fragment's start (to allow repeated iterations)
        // - The new accept state (to allow exiting the star)
        for state in frag.accepts {
            self.add_edge(state, frag.start, EdgeLabel::Eps);
            self.add_edge(state, accept, EdgeLabel::Eps);
        }

        Fragment {
            start,
            accepts: vec![accept],
        }
    }

    /// Applies the plus operand (1 or more occurrences) to the inner AST node.
    ///
    /// # Arguments
    ///
    /// - `inner` (`Ast`) - The inner AST node to apply the plus operation on.
    ///
    /// # Returns
    ///
    /// - `Fragment` - The NFA fragment representing the plus operation.
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

    /// Builds an optional (zero or one occurrence) fragment from the given inner AST.
    ///
    /// # Arguments
    ///
    /// - `inner` (`Ast`) - The inner AST node to apply the optional operation on.
    ///
    /// # Returns
    ///
    /// - `Fragment` - The NFA fragment representing the optional operation.
    fn build_optional(&mut self, inner: Ast) -> Fragment {
        let frag = self.build(inner);
        let start = self.new_state();
        let accept = self.new_state();

        // Simply walk around the inner fragment or go through it.
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

    /// Finalizes the NFA construction, producing the complete NFA structure.
    /// Creates the list of states and edges from the adjacency lists,
    /// and ensures the accepting states are unique and sorted.
    ///
    /// # Arguments
    /// - `start` (`StateId`) - The start state of the NFA.
    /// - `accepts` (`Vec<StateId>`) - The list of accepting states
    ///
    /// # Returns
    ///
    /// - `Nfa` - The finalized NFA structure.
    fn finalize(self, start: StateId, accepts: Vec<StateId>) -> Nfa {
        let accepts = unique_sorted(accepts);
        let states: Vec<StateId> = (0..self.adjacency.len()).map(|i| i as StateId).collect();
        let mut edges = Vec::new();

        for (from, row) in self.adjacency.iter().enumerate() {
            // Iterate over each transition in the adjacency list row.
            for tr in row {
                // And push this as a concrete edge on the edge list.
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
