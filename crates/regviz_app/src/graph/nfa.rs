use regviz_core::core::automaton::{BoxKind, EdgeLabel};
use regviz_core::core::nfa::Nfa;
use std::collections::HashMap;

use super::{Graph, GraphBox, GraphEdge, GraphNode, edge::EdgeCurve};

impl Graph for Nfa {
    fn nodes(&self) -> Vec<GraphNode> {
        self.states
            .iter()
            .map(|state| {
                GraphNode::new(
                    state.id,
                    state.id.to_string(),
                    self.start == state.id,
                    self.accepts.contains(&state.id),
                    state.box_id,
                )
            })
            .collect()
    }

    fn edges(&self) -> Vec<GraphEdge> {
        // Build a map of box_id -> box for easy lookup
        let box_map: HashMap<_, _> = self.boxes.iter().map(|b| (b.id, b)).collect();

        let mut edges = Vec::new();
        for state in &self.states {
            let transitions = self.transitions(state.id);
            for transition in transitions {
                let label: String = match transition.label {
                    EdgeLabel::Eps => "ε".to_string(),
                    EdgeLabel::Sym(ch) => ch.to_string(),
                };

                // Determine if this edge should be curved based on star closure patterns
                let curve = determine_edge_curve(
                    state.id,
                    transition.to,
                    &transition.label,
                    state.box_id,
                    &box_map,
                    self,
                );

                edges.push(GraphEdge::with_curve(state.id, transition.to, label, curve));
            }
        }
        edges
    }

    fn boxes(&self) -> Vec<GraphBox> {
        self.boxes.clone().into_iter().map(Into::into).collect()
    }
}

/// Determines the curvature style for an edge based on its role in the NFA structure.
///
/// For star closures (and plus/optional), we identify special epsilon transitions:
/// - Entry → Inner Start: Should curve downward (wraps around bottom)
/// - Inner Accept → Inner Start: Should curve upward (wraps around top)
/// - All other edges: Straight
///
/// # Algorithm
/// 1. Check if the edge is an epsilon transition in a star/plus/optional box
/// 2. Identify the entry and exit states of the box
/// 3. Determine if this is a "bypass" (entry → inner) or "loop" (inner → inner) edge
/// 4. Return appropriate curve direction
///
/// # Arguments
/// - `from`: Source state ID
/// - `to`: Destination state ID
/// - `label`: Edge label (epsilon or symbol)
/// - `from_box_id`: The box ID containing the source state
/// - `box_map`: Map of box IDs to box metadata
/// - `nfa`: The NFA being processed
///
/// # Returns
/// The appropriate `EdgeCurve` for this edge
fn determine_edge_curve(
    from: u32,
    to: u32,
    label: &EdgeLabel,
    from_box_id: Option<u32>,
    box_map: &HashMap<u32, &regviz_core::core::automaton::BoundingBox>,
    nfa: &Nfa,
) -> EdgeCurve {
    // Only epsilon transitions in star/plus/optional boxes can be curved
    if *label != EdgeLabel::Eps {
        return EdgeCurve::Straight;
    }

    let Some(box_id) = from_box_id else {
        return EdgeCurve::Straight;
    };

    let Some(bbox) = box_map.get(&box_id) else {
        return EdgeCurve::Straight;
    };

    // Only apply curves to unary operators (star, plus, optional)
    if !matches!(
        bbox.kind,
        BoxKind::KleeneStar | BoxKind::KleenePlus | BoxKind::Optional
    ) {
        return EdgeCurve::Straight;
    }

    // For star/plus/optional, the structure is:
    // states[0] = entry, states[last] = exit, middle states = inner fragment
    if bbox.states.len() < 2 {
        return EdgeCurve::Straight;
    }

    let entry = bbox.states[0];
    let exit = *bbox.states.last().unwrap();

    // Get the inner fragment's first state (the target of the entry → inner transition)
    // This is tricky - we need to look at the children boxes or inner states
    // For now, we'll use a heuristic: the inner_start is any state that's not entry/exit
    let inner_states: Vec<_> = bbox
        .states
        .iter()
        .filter(|&&s| s != entry && s != exit)
        .copied()
        .collect();

    // Pattern 1: Entry → Inner Start (should curve down)
    // This is the edge that bypasses zero iterations
    if from == entry && inner_states.contains(&to) {
        return EdgeCurve::CurveDown;
    }

    // Pattern 2: Inner state → Inner Start (loop back, should curve up)
    // This is the edge that repeats the inner fragment
    if inner_states.contains(&from) && inner_states.contains(&to) {
        // Additional check: this should be going backwards (from an accept state)
        // We can check if 'from' is in the inner_accepts by seeing if it has an edge to exit
        let has_edge_to_exit = nfa
            .transitions(from)
            .iter()
            .any(|t| t.to == exit && t.label == EdgeLabel::Eps);

        if has_edge_to_exit {
            return EdgeCurve::CurveUp;
        }
    }

    EdgeCurve::Straight
}
