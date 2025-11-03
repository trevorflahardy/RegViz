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
    /// Builds an [`Nfa`] using Thompson's construction algorithm.
    ///
    /// # Arguments
    ///
    /// - `ast` (`&Ast`) - The abstract syntax tree representing the regular expression. Will be cloned.
    ///
    /// # Returns
    ///
    /// - `Nfa` - The constructed nondeterministic finite automaton.
    pub fn build(ast: &Ast) -> Nfa {
        let mut builder = Builder::default();
        let fragment = builder.build(ast);
        builder.finalize(fragment)
    }

    /// Retrieves the outgoing transitions from a given state.
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

/// Represents a fragment of an NFA with a start state and an accepting state.
/// Created when building NFA components from AST nodes.
#[derive(Debug, Clone)]
struct Fragment {
    /// The ID of the start state.
    start: StateId,
    /// The accepting state ID.
    /// There's always exactly one accept state per fragment, according to Thompson's construction.
    accept: StateId,
}

impl Builder {
    /// A helper method to create a bounding box around a fragment-building operation.
    /// # Arguments
    /// - `kind` (`BoxKind`) - The kind of bounding box to create.
    /// - `f` (`F`) - A closure that takes a mutable reference to the builder and returns a Fragment.
    /// # Returns
    /// - `Fragment` - The fragment produced by the closure within the bounding box.
    fn with_box<F>(&mut self, kind: BoxKind, f: F) -> Fragment
    where
        F: FnOnce(&mut Self) -> Fragment,
    {
        self.begin_box(kind);
        let fragment = f(self);
        self.end_box();
        fragment
    }

    /// Begins a new bounding box of the specified kind, pushing it onto the box stack.
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

    /// Ends the current bounding box, popping it from the box stack.
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
        if let Some(current) = box_id
            && let Some(bbox) = self.boxes.get_mut(current as usize)
        {
            bbox.states.push(id);
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
    fn build(&mut self, ast: &Ast) -> Fragment {
        match ast {
            Ast::Epsilon => self.build_epsilon(),
            Ast::Atom(c) => self.build_char(*c),
            Ast::Concat(lhs, rhs) => self.build_concat(lhs, rhs),
            Ast::Alt(lhs, rhs) => self.build_alternation(lhs, rhs),
            Ast::Star(inner) => self.build_star(inner),
            Ast::Opt(inner) => self.build_optional(inner),
        }
    }

    /// Builds an epsilon AST symbol within the NFA. Creates a single state that is both the start and accept state.
    ///
    /// # Returns
    ///
    /// - `Fragment` - The NFA fragment representing the epsilon transition.
    fn build_epsilon(&mut self) -> Fragment {
        self.with_box(BoxKind::Literal, move |builder| {
            let state = builder.new_state();
            Fragment {
                start: state,
                accept: state,
            }
        })
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

            Fragment { start, accept }
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
    fn build_concat(&mut self, lhs: &Ast, rhs: &Ast) -> Fragment {
        self.with_box(BoxKind::Concat, move |builder| {
            let left = builder.build(lhs);
            let right = builder.build(rhs);

            let Fragment {
                start: left_start,
                accept: left_accept,
            } = left;
            let Fragment {
                start: right_start,
                accept: right_accept,
            } = right;

            builder.add_edge(left_accept, right_start, EdgeLabel::Eps);

            Fragment {
                start: left_start,
                accept: right_accept,
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
    fn build_alternation(&mut self, lhs: &Ast, rhs: &Ast) -> Fragment {
        self.with_box(BoxKind::Alternation, move |builder| {
            let left = builder.build(lhs);
            let right = builder.build(rhs);

            let Fragment {
                start: left_start,
                accept: left_accept,
            } = left;
            let Fragment {
                start: right_start,
                accept: right_accept,
            } = right;

            let start = builder.new_state();
            let accept = builder.new_state();

            builder.add_edge(start, left_start, EdgeLabel::Eps);
            builder.add_edge(start, right_start, EdgeLabel::Eps);

            for state in &[left_accept, right_accept] {
                builder.add_edge(*state, accept, EdgeLabel::Eps);
            }

            Fragment { start, accept }
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
    fn build_star(&mut self, inner: &Ast) -> Fragment {
        self.with_box(BoxKind::KleeneStar, move |builder| {
            let frag = builder.build(inner);
            let Fragment {
                start: inner_start,
                accept: inner_accept,
            } = frag;

            let start = builder.new_state();
            let accept = builder.new_state();

            builder.add_edge(start, inner_start, EdgeLabel::Eps);
            builder.add_edge(start, accept, EdgeLabel::Eps);

            builder.add_edge(inner_accept, inner_start, EdgeLabel::Eps);
            builder.add_edge(inner_accept, accept, EdgeLabel::Eps);

            Fragment { start, accept }
        })
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
    fn build_optional(&mut self, inner: &Ast) -> Fragment {
        self.with_box(BoxKind::Optional, move |builder| {
            let frag = builder.build(inner);
            let Fragment {
                start: inner_start,
                accept: inner_accept,
            } = frag;
            let start = builder.new_state();
            let accept = builder.new_state();

            builder.add_edge(start, inner_start, EdgeLabel::Eps);
            builder.add_edge(start, accept, EdgeLabel::Eps);

            builder.add_edge(inner_accept, accept, EdgeLabel::Eps);

            Fragment { start, accept }
        })
    }

    /// Finalizes the NFA construction, producing the complete NFA structure.
    /// Creates the list of states and edges from the adjacency lists,
    /// and ensures the accepting states are unique and sorted.
    ///
    /// # Arguments
    /// - `fragment` ([`Fragment`]) - The final fragment representing the entire NFA.
    ///
    /// # Returns
    ///
    /// - [`Nfa`] - The finalized NFA structure.
    fn finalize(mut self, fragment: Fragment) -> Nfa {
        let mut edges = Vec::new();

        for (from, row) in self.adjacency.iter_mut().enumerate() {
            // Sort the adjacency list row by destination state for consistency.
            row.sort_by_key(|tr| tr.to);
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
            states: self.states,
            start: fragment.start,
            accepts: vec![fragment.accept],
            edges,
            adjacency: self.adjacency,
            boxes: self.boxes,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_epsilon() {
        let ast = Ast::build("").unwrap();
        let nfa = Nfa::build(&ast);
        // State creation order:
        // Epsilon Fragment:
        // Start 0, Accept 0, +0 edges
        assert_eq!(nfa.states.len(), 1);
        assert_eq!(nfa.start, 0);
        assert_eq!(nfa.accepts, vec![0]);
        assert_eq!(nfa.edges.len(), 0);
        assert!(nfa.adjacency[0].is_empty());
    }

    #[test]
    fn test_build_char() {
        let ast = Ast::Atom('a');
        let nfa = Nfa::build(&ast);
        // State creation order:
        // Literal Fragment 'a':
        // Start 0, Accept 1, +1 edge
        assert_eq!(nfa.states.len(), 2);
        assert_eq!(nfa.start, 0);
        assert_eq!(nfa.accepts, vec![1]);
        assert_eq!(nfa.edges.len(), 1);
        assert_eq!(
            nfa.adjacency,
            vec![
                // 0 -> 'a' -> 1
                vec![Transition {
                    to: 1,
                    label: EdgeLabel::Sym('a'),
                }],
                // 1 -> no edges
                vec![]
            ]
        );
    }

    #[test]
    fn test_build_concat() {
        let ast = Ast::build("ab").unwrap();
        let nfa = Nfa::build(&ast);
        // State creation order:
        // Literal Fragment 'a':
        // Start 0, Accept 1, +1 edge
        // Literal Fragment 'b':
        // Start 2, Accept 3, +1 edge
        // Concat Fragment:
        // Start 0, Accept 3, +1 edge
        assert_eq!(nfa.states.len(), 4);
        assert_eq!(nfa.start, 0);
        assert_eq!(nfa.accepts, vec![3]);
        assert_eq!(nfa.edges.len(), 3);
        assert_eq!(
            nfa.adjacency,
            vec![
                // 0 -> 'a' -> 1
                vec![Transition {
                    to: 1,
                    label: EdgeLabel::Sym('a'),
                }],
                // 1 -> eps -> 2
                vec![Transition {
                    to: 2,
                    label: EdgeLabel::Eps,
                }],
                // 2 -> 'b' -> 3
                vec![Transition {
                    to: 3,
                    label: EdgeLabel::Sym('b'),
                }],
                // 3 -> no edges
                vec![]
            ]
        )
    }

    #[test]
    fn test_build_alternation() {
        let ast = Ast::build("a+b").unwrap();
        let nfa = Nfa::build(&ast);
        // State creation order:
        // Literal Fragment 'a':
        // Start 0, Accept 1, +1 edge
        // Literal Fragment 'b':
        // Start 2, Accept 3, +1 edge
        // Alternation Fragment:
        // Start 4, Accept 5, +4 edges
        assert_eq!(nfa.states.len(), 6);
        assert_eq!(nfa.start, 4);
        assert_eq!(nfa.accepts, vec![5]);
        assert_eq!(nfa.edges.len(), 6);
        assert_eq!(
            nfa.adjacency,
            vec![
                // 0 -> 'a' -> 1
                vec![Transition {
                    to: 1,
                    label: EdgeLabel::Sym('a'),
                }],
                // 1 -> eps -> 5
                vec![Transition {
                    to: 5,
                    label: EdgeLabel::Eps,
                }],
                // 2 -> 'b' -> 3
                vec![Transition {
                    to: 3,
                    label: EdgeLabel::Sym('b'),
                }],
                // 3 -> eps -> 5
                vec![Transition {
                    to: 5,
                    label: EdgeLabel::Eps,
                }],
                // 4 -> eps -> 0
                //   -> eps -> 2
                vec![
                    Transition {
                        to: 0,
                        label: EdgeLabel::Eps,
                    },
                    Transition {
                        to: 2,
                        label: EdgeLabel::Eps,
                    },
                ],
                // 5 -> no edges
                vec![],
            ]
        );
    }

    #[test]
    fn test_build_star() {
        let ast = Ast::build("a*").unwrap();
        let nfa = Nfa::build(&ast);
        // State creation order:
        // Literal Fragment 'a':
        // Start 0, Accept 1, +1 edge
        // Kleene Star Fragment:
        // Start 2, Accept 3, +4 edges
        assert_eq!(nfa.states.len(), 4);
        assert_eq!(nfa.start, 2);
        assert_eq!(nfa.accepts, vec![3]);
        assert_eq!(nfa.edges.len(), 5);
        assert_eq!(
            nfa.adjacency,
            vec![
                // 0 -> 'a' -> 1
                vec![Transition {
                    to: 1,
                    label: EdgeLabel::Sym('a'),
                }],
                // 1 -> eps -> 0
                //   -> eps -> 3
                vec![
                    Transition {
                        to: 0,
                        label: EdgeLabel::Eps,
                    },
                    Transition {
                        to: 3,
                        label: EdgeLabel::Eps,
                    },
                ],
                // 2 -> eps -> 0
                //   -> eps -> 3
                vec![
                    Transition {
                        to: 0,
                        label: EdgeLabel::Eps,
                    },
                    Transition {
                        to: 3,
                        label: EdgeLabel::Eps,
                    },
                ],
                // 3 -> no edges
                vec![],
            ]
        );
    }

    #[test]
    fn test_build_complex() {
        let ast = Ast::build("(ab+c)*").unwrap();
        let nfa = Nfa::build(&ast);
        // State creation order:
        // Literal Fragment 'a':
        // Start 0, Accept 1, +1 edge
        // Literal Fragment 'b':
        // Start 2, Accept 3, +1 edge
        // Concat Fragment 'ab':
        // Start 0, Accept 3, +1 edge
        // Literal Fragment 'c':
        // Start 4, Accept 5, +1 edge
        // Alternation Fragment 'ab+c':
        // Start 6, Accept 7, +4 edges
        // Kleene Star Fragment '(ab+c)*':
        // Start 8, Accept 9, +4 edges
        assert_eq!(nfa.states.len(), 10);
        assert_eq!(nfa.start, 8);
        assert_eq!(nfa.accepts, vec![9]);
        assert_eq!(nfa.edges.len(), 12);
        assert_eq!(
            nfa.adjacency,
            vec![
                // 0 -> 'a' -> 1
                vec![Transition {
                    to: 1,
                    label: EdgeLabel::Sym('a'),
                }],
                // 1 -> eps -> 2
                vec![Transition {
                    to: 2,
                    label: EdgeLabel::Eps,
                }],
                // 2 -> 'b' -> 3
                vec![Transition {
                    to: 3,
                    label: EdgeLabel::Sym('b'),
                }],
                // 3 -> eps -> 7
                vec![Transition {
                    to: 7,
                    label: EdgeLabel::Eps,
                }],
                // 4 -> 'c' -> 5
                vec![Transition {
                    to: 5,
                    label: EdgeLabel::Sym('c'),
                }],
                // 5 -> eps -> 7
                vec![Transition {
                    to: 7,
                    label: EdgeLabel::Eps,
                }],
                // 6 -> eps -> 0
                //   -> eps -> 4
                vec![
                    Transition {
                        to: 0,
                        label: EdgeLabel::Eps,
                    },
                    Transition {
                        to: 4,
                        label: EdgeLabel::Eps,
                    },
                ],
                // 7 -> eps -> 6
                //   -> eps -> 9
                vec![
                    Transition {
                        to: 6,
                        label: EdgeLabel::Eps,
                    },
                    Transition {
                        to: 9,
                        label: EdgeLabel::Eps,
                    },
                ],
                // 8 -> eps -> 6
                //   -> eps -> 9
                vec![
                    Transition {
                        to: 6,
                        label: EdgeLabel::Eps,
                    },
                    Transition {
                        to: 9,
                        label: EdgeLabel::Eps,
                    },
                ],
                // 9 -> no edges
                vec![],
            ]
        );
    }

    #[test]
    fn test_build_optional_literal() {
        let ast = Ast::build("a?").unwrap();
        let nfa = Nfa::build(&ast);
        // State creation order:
        // Literal 'a': start 0 accept 1 (+1 edge)
        // Optional wrapper: start 2 accept 3 (+3 edges)
        assert_eq!(nfa.states.len(), 4);
        assert_eq!(nfa.start, 2);
        assert_eq!(nfa.accepts, vec![3]);
        // edges: 0->1 (Sym 'a'), 1->3 (Eps), 2->0 (Eps), 2->3 (Eps) => total 4 edges
        assert_eq!(nfa.edges.len(), 4);
        assert_eq!(
            nfa.adjacency,
            vec![
                // 0 -> 'a' -> 1
                vec![Transition {
                    to: 1,
                    label: EdgeLabel::Sym('a'),
                }],
                // 1 -> eps -> 3
                vec![Transition {
                    to: 3,
                    label: EdgeLabel::Eps,
                }],
                // 2 -> eps -> 0
                //   -> eps -> 3
                vec![
                    Transition {
                        to: 0,
                        label: EdgeLabel::Eps,
                    },
                    Transition {
                        to: 3,
                        label: EdgeLabel::Eps,
                    },
                ],
                // 3 -> no edges
                vec![],
            ]
        );
    }

    #[test]
    fn test_build_optional_epsilon() {
        // "\\e?" = optional epsilon
        let ast = Ast::build("\\e?").unwrap();
        let nfa = Nfa::build(&ast);
        // Epsilon fragment: state 0 (start & accept)
        // Optional wrapper: start 1 accept 2
        // Edges: 0->2 (Eps), 1->0 (Eps), 1->2 (Eps) => 3 edges
        assert_eq!(nfa.states.len(), 3);
        assert_eq!(nfa.start, 1);
        assert_eq!(nfa.accepts, vec![2]);
        assert_eq!(nfa.edges.len(), 3);
        assert_eq!(
            nfa.adjacency,
            vec![
                // 0 -> eps -> 2
                vec![Transition {
                    to: 2,
                    label: EdgeLabel::Eps,
                }],
                // 1 -> eps -> 0
                //   -> eps -> 2
                vec![
                    Transition {
                        to: 0,
                        label: EdgeLabel::Eps,
                    },
                    Transition {
                        to: 2,
                        label: EdgeLabel::Eps,
                    },
                ],
                // 2 -> no edges
                vec![],
            ]
        );
    }
}
