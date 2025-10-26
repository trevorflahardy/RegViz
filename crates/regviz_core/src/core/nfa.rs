use crate::core::Ast;
use crate::core::automaton::{
    BoundingBox, BoxId, BoxKind, Edge, EdgeLabel, State, StateId, Transition,
};
use std::collections::HashSet;

/// Represents a Thompson-constructed nondeterministic finite automaton.
#[derive(Debug, Clone)]
pub struct Nfa {
    /// All known states.
    pub states: Vec<State>,
    /// Start state.
    pub start: StateId,
    /// Accepting states.
    pub accepts: Vec<StateId>,
    /// Flattened edge list.
    pub edges: Vec<Edge>,
    /// Adjacency lists for efficient traversal.
    pub adjacency: Vec<Vec<Transition>>,
    /// Bounding boxes describing the AST nesting this NFA was built from.
    pub boxes: Vec<BoundingBox>,
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
    /// Metadata for each state allocated so far.
    states: Vec<State>,
    /// Bounding boxes describing AST structure during construction.
    boxes: Vec<BoundingBox>,
    /// Stack tracking the current bounding box hierarchy.
    box_stack: Vec<BoxId>,
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
    fn with_box<F>(&mut self, kind: BoxKind, f: F) -> Fragment
    where
        F: FnOnce(&mut Self) -> Fragment,
    {
        self.begin_box(kind);
        let fragment = f(self);
        self.end_box();
        fragment
    }

    fn begin_box(&mut self, kind: BoxKind) {
        let id = self.boxes.len() as BoxId;
        let parent = self.box_stack.last().copied();
        self.boxes.push(BoundingBox {
            id,
            kind,
            parent,
            states: Vec::new(),
        });
        self.box_stack.push(id);
    }

    fn end_box(&mut self) {
        self.box_stack.pop();
    }

    /// Adds a new, empty state to this NFA with no outgoing or incoming transitions.
    ///
    /// # Returns
    ///
    /// - `StateId` - The identifier of the newly created state.
    fn new_state(&mut self) -> StateId {
        let id = self.adjacency.len() as StateId;
        self.adjacency.push(Vec::new());
        let box_id = self.box_stack.last().copied();
        if let Some(current) = box_id {
            if let Some(bbox) = self.boxes.get_mut(current as usize) {
                bbox.states.push(id);
            }
        }
        self.states.push(State { id, box_id });
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
    fn add_edge(&mut self, from: StateId, to: StateId, label: EdgeLabel) {
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
            Ast::Epsilon => self.build_char('ε'),
            Ast::Atom(c) => self.build_char(c),
            Ast::Concat(lhs, rhs) => self.build_concat(*lhs, *rhs),
            Ast::Alt(lhs, rhs) => self.build_alternation(*lhs, *rhs),
            Ast::Star(inner) => self.build_star(*inner),
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
        self.with_box(BoxKind::Literal, move |builder| {
            let start = builder.new_state();
            let accept = builder.new_state();

            builder.add_edge(start, accept, EdgeLabel::Sym(ch));

            Fragment {
                start,
                accepts: vec![accept],
            }
        })
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
        self.with_box(BoxKind::Concat, move |builder| {
            let left = builder.build(lhs);
            let right = builder.build(rhs);

            let Fragment {
                start: left_start,
                accepts: left_accepts,
            } = left;
            let Fragment {
                start: right_start,
                accepts: right_accepts,
            } = right;

            for accept in &left_accepts {
                builder.add_edge(*accept, right_start, EdgeLabel::Eps);
            }

            Fragment {
                start: left_start,
                accepts: right_accepts,
            }
        })
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
        self.with_box(BoxKind::Alternation, move |builder| {
            let left = builder.build(lhs);
            let right = builder.build(rhs);

            let Fragment {
                start: left_start,
                accepts: left_accepts,
            } = left;
            let Fragment {
                start: right_start,
                accepts: right_accepts,
            } = right;

            let start = builder.new_state();
            let accept = builder.new_state();

            builder.add_edge(start, left_start, EdgeLabel::Eps);
            builder.add_edge(start, right_start, EdgeLabel::Eps);

            for state in left_accepts.iter().chain(right_accepts.iter()) {
                builder.add_edge(*state, accept, EdgeLabel::Eps);
            }

            Fragment {
                start,
                accepts: vec![accept],
            }
        })
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
        self.with_box(BoxKind::KleeneStar, move |builder| {
            let frag = builder.build(inner);
            let Fragment {
                start: inner_start,
                accepts: inner_accepts,
            } = frag;

            let start = builder.new_state();
            let accept = builder.new_state();

            builder.add_edge(start, inner_start, EdgeLabel::Eps);
            builder.add_edge(start, accept, EdgeLabel::Eps);

            for state in inner_accepts {
                builder.add_edge(state, inner_start, EdgeLabel::Eps);
                builder.add_edge(state, accept, EdgeLabel::Eps);
            }

            Fragment {
                start,
                accepts: vec![accept],
            }
        })
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
    fn finalize(mut self, start: StateId, accepts: Vec<StateId>) -> Nfa {
        let accepts = unique_sorted(accepts);
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
            states: std::mem::take(&mut self.states),
            start,
            accepts,
            edges,
            adjacency: self.adjacency,
            boxes: self.boxes,
        }
    }
}

fn unique_sorted(mut ids: Vec<StateId>) -> Vec<StateId> {
    ids.sort_unstable();
    ids.dedup();
    ids
}
