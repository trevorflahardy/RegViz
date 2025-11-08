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
}

impl VisualDfa {
    /// Creates a new highlighted DFA graph.
    #[must_use]
    pub fn new(dfa: Dfa, alphabet: Vec<char>, highlights: Highlights) -> Self {
        Self {
            dfa,
            alphabet,
            highlights,
        }
    }
}

impl Graph for VisualDfa {
    fn nodes(&self) -> Vec<GraphNode> {
        build_nodes(&self.dfa, &self.highlights)
    }

    fn edges(&self) -> Vec<GraphEdge> {
        build_edges(&self.dfa, &self.alphabet, &self.highlights)
    }

    fn boxes(&self) -> Vec<GraphBox> {
        Vec::new()
    }
}

fn build_nodes(dfa: &Dfa, highlights: &Highlights) -> Vec<GraphNode> {
    dfa.states
        .iter()
        .map(|state_id| {
            let highlight = highlights.state_style(*state_id);
            GraphNode::new(
                *state_id,
                state_id.to_string(),
                dfa.start == *state_id,
                dfa.accepts.contains(state_id),
                None,
            )
            .with_highlight(highlight)
        })
        .collect()
}

fn build_edges(dfa: &Dfa, alphabet: &[char], highlights: &Highlights) -> Vec<GraphEdge> {
    // Group transitions between the same pair of states so multiple labels are
    // rendered as a single comma-separated label. Also collect activity state.
    let mut map: HashMap<(StateId, StateId), (Vec<char>, bool)> = HashMap::new();
    for (state_idx, state_id) in dfa.states.iter().enumerate() {
        for (symbol_idx, symbol) in alphabet.iter().enumerate() {
            let Some(next) = dfa.trans[state_idx][symbol_idx] else {
                continue;
            };
            let edge_label = EdgeLabel::Sym(*symbol);
            let is_active = highlights.is_edge_active(*state_id, next, edge_label);
            let key = (*state_id, next);
            let entry = map.entry(key).or_insert_with(|| (Vec::new(), false));
            entry.0.push(*symbol);
            entry.1 = entry.1 || is_active;
        }
    }

    // Build edges from grouped labels
    let mut edges: Vec<GraphEdge> = map
        .into_iter()
        .map(|((from, to), (syms, is_active))| {
            let mut syms = syms;
            syms.sort_unstable();
            syms.dedup();
            let label = syms
                .iter()
                .map(|c| format!("'{}'", c))
                .collect::<Vec<_>>()
                .join(", ");
            GraphEdge::new(from, to, label).with_active(is_active)
        })
        .collect();

    adjust_bidirectional_labels(&mut edges);
    edges
}

fn adjust_bidirectional_labels(edges: &mut [GraphEdge]) {
    let mut paired: HashMap<(StateId, StateId), (Vec<usize>, Vec<usize>)> = HashMap::new();

    for (idx, edge) in edges.iter().enumerate() {
        if edge.from == edge.to {
            continue;
        }
        let (a, b) = if edge.from < edge.to {
            (edge.from, edge.to)
        } else {
            (edge.to, edge.from)
        };
        let entry = paired
            .entry((a, b))
            .or_insert_with(|| (Vec::new(), Vec::new()));
        if edge.from <= edge.to {
            entry.0.push(idx);
        } else {
            entry.1.push(idx);
        }
    }

    for (_, (forward, backward)) in paired {
        if forward.is_empty() || backward.is_empty() {
            continue;
        }

        for idx in forward {
            if let Some(edge) = edges.get_mut(idx) {
                edge.curve = EdgeCurve::CurveDown;
            }
        }
        for idx in backward {
            if let Some(edge) = edges.get_mut(idx) {
                edge.curve = EdgeCurve::CurveDown;
            }
        }
    }
}
