use std::collections::HashSet;

use regviz_core::core::automaton::{EdgeLabel, StateId};

/// Key identifying a transition in an automaton.
///
/// This is used to track which edges should be highlighted when stepping through
/// a simulation. Two edges are considered identical if they originate from the
/// same state, terminate at the same state, and carry the same label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdgeHighlight {
    /// Origin state identifier.
    pub from: StateId,
    /// Destination state identifier.
    pub to: StateId,
    /// Transition label (ε or a concrete symbol).
    pub label: EdgeLabel,
}

impl EdgeHighlight {
    /// Creates a new keyed highlight reference for an edge.
    #[must_use]
    pub fn new(from: StateId, to: StateId, label: EdgeLabel) -> Self {
        Self { from, to, label }
    }
}

/// Collection of states and transitions that should be emphasised in the UI.
#[derive(Debug, Clone, Default)]
pub struct Highlights {
    /// Set of states that are currently active.
    pub states: HashSet<StateId>,
    /// Set of edges that were traversed in the current simulation step.
    pub edges: HashSet<EdgeHighlight>,
}

impl Highlights {
    /// Creates a highlights set from explicit state and edge collections.
    #[must_use]
    pub fn from_sets(states: HashSet<StateId>, edges: HashSet<EdgeHighlight>) -> Self {
        Self { states, edges }
    }

    /// Returns whether a state is part of the active frontier.
    #[must_use]
    pub fn is_state_active(&self, state: StateId) -> bool {
        self.states.contains(&state)
    }

    /// Returns whether a given transition should be emphasised.
    #[must_use]
    pub fn is_edge_active(&self, from: StateId, to: StateId, label: EdgeLabel) -> bool {
        self.edges.contains(&EdgeHighlight::new(from, to, label))
    }
}
