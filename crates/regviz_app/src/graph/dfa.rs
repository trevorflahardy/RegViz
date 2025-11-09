use std::collections::HashMap;

use regviz_core::core::automaton::{EdgeLabel, StateId};
use regviz_core::core::dfa::Dfa;

use super::edge::EdgeCurve;
use super::{Graph, GraphBox, GraphEdge, GraphNode, Highlights};

/// Visual wrapper around a DFA with highlight metadata for simulation playback.
#[derive(Debug, Clone)]
pub struct VisualDfa {
    dfa: Dfa,
    alphabet: Vec<char>,
    highlights: Highlights,
    pinned_positions: HashMap<StateId, iced::Point>,
}

impl VisualDfa {
    /// Creates a new highlighted DFA graph.
    #[must_use]
    pub fn new(
        dfa: Dfa,
        alphabet: Vec<char>,
        highlights: Highlights,
        pinned_positions: HashMap<StateId, iced::Point>,
    ) -> Self {
        Self {
            dfa,
            alphabet,
            highlights,
            pinned_positions,
        }
    }
}

impl Graph for VisualDfa {
    fn nodes(&self) -> Vec<GraphNode> {
        build_nodes(&self.dfa, &self.highlights, &self.pinned_positions)
    }

    fn edges(&self) -> Vec<GraphEdge> {
        build_edges(&self.dfa, &self.alphabet, &self.highlights)
    }

    fn boxes(&self) -> Vec<GraphBox> {
        Vec::new()
    }
}

fn build_nodes(
    dfa: &Dfa,
    highlights: &Highlights,
    pinned: &HashMap<StateId, iced::Point>,
) -> Vec<GraphNode> {
    dfa.states
        .iter()
        .map(|state_id| {
            let highlight = highlights.state_style(*state_id);
            let mut node = GraphNode::new(
                *state_id,
                state_id.to_string(),
                dfa.start == *state_id,
                dfa.accepts.contains(state_id),
                None,
            )
            .with_highlight(highlight);

            if let Some(pos) = pinned.get(state_id) {
                node.manual_position = Some(*pos);
                node.is_pinned = true;
            }

            node
        })
        .collect()
}

fn build_edges(dfa: &Dfa, alphabet: &[char], highlights: &Highlights) -> Vec<GraphEdge> {
    // Group transitions between the same pair of states so multiple labels are
    // rendered as a single comma-separated label. Also collect activity state.
    let mut map: HashMap<(StateId, StateId), (Vec<char>, bool)> = HashMap::new();
    for (state_idx, state_id) in dfa.states.iter().enumerate() {
        for (symbol_idx, symbol) in alphabet.iter().enumerate() {
            let next = dfa.trans[state_idx][symbol_idx];
            let edge_label = EdgeLabel::Sym(*symbol);
            let is_active = highlights.is_edge_active(*state_id, next, edge_label);
            let key = (*state_id, next);
            let entry = map.entry(key).or_insert_with(|| (Vec::new(), false));
            entry.0.push(*symbol);
            entry.1 = entry.1 || is_active;
        }
    }

    // Build edges from grouped labels
    let edges: Vec<GraphEdge> = map
        .iter()
        .map(|((from, to), (syms, is_active))| {
            // Create a sorted, deduplicated, comma-separated label
            let unique_syms: Vec<char> = {
                let mut s = syms.clone();
                s.sort_unstable();
                s.dedup();
                s
            };
            let label = unique_syms
                .iter()
                .map(|c| format!("'{}'", c))
                .collect::<Vec<_>>()
                .join(", ");

            // Consider edge curves based on from/to states
            let curve = if from == to {
                EdgeCurve::Loop
            } else if map.contains_key(&(*to, *from)) {
                EdgeCurve::CurveDown
            } else {
                EdgeCurve::Straight
            };
            GraphEdge::with_curve(*from, *to, label, curve).with_active(*is_active)
        })
        .collect();

    edges
}
