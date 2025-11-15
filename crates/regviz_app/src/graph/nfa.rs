use iced::Point;
use regviz_core::core::automaton::{BoxKind, EdgeLabel, StateId};
use regviz_core::core::nfa::Nfa;
use std::collections::HashMap;

use super::{Graph, GraphBox, GraphEdge, GraphNode, Highlights, edge::EdgeCurve};

impl Graph for Nfa {
    fn nodes(&self) -> Vec<GraphNode> {
        let empty = Highlights::default();
        let pinned: HashMap<StateId, Point> = HashMap::new();
        build_nodes(self, &empty, &pinned)
    }

    fn edges(&self) -> Vec<GraphEdge> {
        let empty = Highlights::default();
        build_edges(self, &empty)
    }

    fn boxes(&self) -> Vec<GraphBox> {
        self.boxes.iter().cloned().map(Into::into).collect()
    }
}

/// Visual wrapper that augments an NFA with highlight metadata for rendering.
#[derive(Debug, Clone)]
pub struct VisualNfa<'a> {
    nfa: &'a Nfa,
    highlights: Highlights,
    /// User-supplied manual positions for states (layout coordinates).
    pinned_positions: &'a HashMap<StateId, Point>,
}

impl<'a> VisualNfa<'a> {
    /// Creates a new highlighted NFA ready for visualization. `pinned_positions`
    /// allows the application to persist user-moved node coordinates.
    #[must_use]
    pub fn new(
        nfa: &'a Nfa,
        highlights: Highlights,
        pinned_positions: &'a HashMap<StateId, Point>,
    ) -> Self {
        Self {
            nfa,
            highlights,
            pinned_positions,
        }
    }
}

impl<'a> Graph for VisualNfa<'a> {
    fn nodes(&self) -> Vec<GraphNode> {
        build_nodes(self.nfa, &self.highlights, self.pinned_positions)
    }

    fn edges(&self) -> Vec<GraphEdge> {
        build_edges(self.nfa, &self.highlights)
    }

    fn boxes(&self) -> Vec<GraphBox> {
        self.nfa.boxes.iter().cloned().map(Into::into).collect()
    }
}

fn build_nodes(
    nfa: &Nfa,
    highlights: &Highlights,
    pinned: &HashMap<StateId, Point>,
) -> Vec<GraphNode> {
    nfa.states
        .iter()
        .map(|state| {
            let highlight = highlights.state_style(state.id);
            let mut node = GraphNode::new(
                state.id,
                state.id.to_string(),
                nfa.start == state.id,
                nfa.accepts.contains(&state.id),
                state.box_id,
            )
            .with_highlight(highlight);

            if let Some(pos) = pinned.get(&state.id) {
                node.manual_position = Some(*pos);
                node.is_pinned = true;
            }

            node
        })
        .collect()
}

fn build_edges(nfa: &Nfa, highlights: &Highlights) -> Vec<GraphEdge> {
    // Build a map of box_id -> box for easy lookup
    let box_map: HashMap<_, _> = nfa.boxes.iter().map(|b| (b.id, b)).collect();

    let mut edges = Vec::new();
    for state in &nfa.states {
        let transitions = nfa.transitions(state.id);
        for transition in transitions {
            let label = transition.label;
            let label_text: String = match label {
                EdgeLabel::Eps => "ε".to_string(),
                EdgeLabel::Sym(ch) => format!("'{ch}'"),
            };

            // Determine if this edge should be curved based on star closure patterns
            let curve = determine_edge_curve(
                state.id,
                transition.to,
                &transition.label,
                state.box_id,
                &box_map,
                nfa,
            );

            let is_active = highlights.is_edge_active(state.id, transition.to, label);
            edges.push(
                GraphEdge::with_curve(state.id, transition.to, label_text, curve)
                    .with_active(is_active),
            );
        }
    }
    edges
}

/// Determines the curvature style for an edge based on its role in the NFA structure.
///
/// For star closures, Thompson's construction creates:
/// ```text
///        ┌────────ε (bypass, curve down)─────┐
///        ↓                                    ↓
/// START ──ε→ inner_start ──'a'→ inner_accept ──ε→ ACCEPT
///   (straight)                   ↑
///                 └───ε (loop up)┘
/// ```
///
/// The star box contains [START, ACCEPT].
/// The inner fragment is in a child box.
///
/// We curve two edges:
/// 1. START → ACCEPT: Curve down (bypass, wraps below inner fragment)
/// 2. inner_accept → inner_start: Curve up (loop-back, wraps above inner fragment)
///
/// All other edges (entry, exit, literal) are straight.
///
/// # Arguments
/// - `from`: Source state ID
/// - `to`: Destination state ID
/// - `label`: Edge label (epsilon or symbol)
/// - `_from_box_id`: The box ID containing the source state (unused)
/// - `box_map`: Map of box IDs to box metadata
/// - `nfa`: The NFA being processed
///
/// # Returns
/// The appropriate `EdgeCurve` for this edge
fn determine_edge_curve(
    from: u32,
    to: u32,
    label: &EdgeLabel,
    _from_box_id: Option<u32>,
    box_map: &HashMap<u32, &regviz_core::core::automaton::BoundingBox>,
    nfa: &Nfa,
) -> EdgeCurve {
    // Only epsilon transitions can be curved
    if *label != EdgeLabel::Eps {
        return EdgeCurve::Straight;
    }

    // Check all star/plus/optional boxes to see if this edge matches a curved pattern
    for bbox in box_map.values() {
        // Only apply curves to unary operators
        if !matches!(
            bbox.kind,
            BoxKind::KleeneStar | BoxKind::KleenePlus | BoxKind::Optional
        ) {
            continue;
        }

        // Star box should have exactly 2 states: [start, accept]
        if bbox.states.len() != 2 {
            continue;
        }

        let star_start = bbox.states[0];
        let star_accept = bbox.states[1];

        // Pattern 1: star_start → star_accept (bypass, curve down)
        if from == star_start && to == star_accept {
            return EdgeCurve::CurveDown;
        }

        // Pattern 2: inner_accept → inner_start (loop-back, curve up)
        // Find inner_start: it's the epsilon target of star_start that's NOT star_accept
        let inner_start = nfa
            .transitions(star_start)
            .iter()
            .find(|t| t.label == EdgeLabel::Eps && t.to != star_accept)
            .map(|t| t.to);

        if let Some(inner_start_id) = inner_start {
            // Check if this edge goes TO inner_start
            if to == inner_start_id {
                // If FROM is star_start, this is the entry edge - keep it straight
                if from == star_start {
                    continue; // Don't curve the entry edge
                }

                // Check if FROM is an inner_accept (has epsilon edge to star_accept)
                let from_is_inner_accept = nfa
                    .transitions(from)
                    .iter()
                    .any(|t| t.to == star_accept && t.label == EdgeLabel::Eps);

                if from_is_inner_accept {
                    // This is the loop-back edge - curve it upward
                    return EdgeCurve::CurveUp;
                }
            }
        }
    }

    // All other edges (entry, exit, literal) are straight
    EdgeCurve::Straight
}
