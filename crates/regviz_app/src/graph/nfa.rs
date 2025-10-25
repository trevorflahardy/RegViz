use regviz_core::core::automaton::EdgeLabel;
use regviz_core::core::nfa::Nfa;

use super::{Graph, GraphBox, GraphEdge, GraphNode};

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
        let mut edges = Vec::new();
        for state in &self.states {
            let transitions = self.transitions(state.id);
            for transition in transitions {
                let label: String = match transition.label {
                    EdgeLabel::Eps => "ε".to_string(),
                    EdgeLabel::Sym(ch) => ch.to_string(),
                };
                edges.push(GraphEdge::new(state.id, transition.to, label));
            }
        }
        edges
    }

    fn boxes(&self) -> Vec<GraphBox> {
        self.boxes.clone().into_iter().map(Into::into).collect()
    }
}
