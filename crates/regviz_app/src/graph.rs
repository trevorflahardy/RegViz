use iced::widget::canvas::Program;
use iced::widget::canvas::{self, Frame, Path, Stroke, Text as CText};
use iced::{Color, Point, Rectangle, Vector, mouse};
use iced_graphics::geometry::Renderer;
use regviz_core::core::nfa::Nfa;
use std::collections::HashMap;

pub type NodeId = usize;
pub type EdgeId = usize;

pub const NODE_DEFAULT_SIZE_X: f32 = 25.0;
pub const NODE_DEFAULT_SIZE_Y: f32 = 25.0;

const WHITE: Color = Color::from_rgb(1.0, 1.0, 1.0);

/// Represents a renderable GraphNode with visual properties.
#[derive(Debug, Clone)]
pub struct GraphNode {
    pub id: NodeId,
    pub label: String,
    pub pos: Point,
    pub size: Vector,
    pub is_start: bool,
    pub is_accept: bool,
}

impl<R: Renderer> GraphNode {
    /// Creates a new GraphNode with default size and position at origin.
    #[must_use]
    pub fn new(id: NodeId, label: String, is_start: bool, is_accept: bool) -> Self {
        Self {
            id,
            label,
            pos: Point::ORIGIN,
            size: Vector::new(NODE_DEFAULT_SIZE_X, NODE_DEFAULT_SIZE_Y),
            is_start,
            is_accept,
        }
    }

    /// Creates the node geometry
    fn draw(&self, at: Point, frame: &mut Frame<R>) -> () {
        let radius = self.size.x.max(14.0);

        // If this node is a start, draw an incoming arrow
        if self.is_start {
            // TODO: Draw arrow logic here
        }

        let circle = Path::circle(at, radius);
        frame.fill(&circle, WHITE);

        // If this node is an accept state, draw a double circle
        if self.is_accept {
            let inner = Path::circle(at, radius - 4.0);
            frame.stroke(&inner, Stroke::default().with_width(1.2));
        }

        // Draw the label, if any
        if !self.label.is_empty() {
            frame.fill_text(CText {
                content: self.label.clone(),
                position: Point::new(at.x - radius * 0.6, at.y + 4.0),
                ..CText::default()
            });
        }
    }
}

/// Represents a renderable Graph Edge between two nodes with a label.
#[derive(Debug, Clone)]
pub struct GraphEdge {
    pub id: EdgeId,
    pub from: NodeId,
    pub to: NodeId,
    pub label: String,
}

/// A common Graph trait for graph structures.
/// A graph contains a set of nodes and arrows (edges) connecting them,
pub trait Graph {
    /// Gets all nodes in the graph
    fn nodes(&self) -> Vec<GraphNode>;

    /// Gets all edges in the graph
    fn edges(&self) -> Vec<GraphEdge>;

    /// Get outgoing edges from a node
    fn outgoing_edges(&self, node_id: NodeId) -> Vec<GraphEdge>;

    /// Get incoming edges to a specific node
    fn incoming_edges(&self, node_id: NodeId) -> Vec<GraphEdge>;

    /// Get a specific node by its ID
    fn get_node(&self, node_id: NodeId) -> Option<GraphNode>;

    /// Get the neighbors of a node
    fn neighbors(&self, node_id: NodeId) -> Vec<GraphNode> {
        self.outgoing_edges(node_id)
            .iter()
            .filter_map(|edge| self.get_node(edge.to))
            .collect()
    }
}

impl Graph for Nfa {
    fn nodes(&self) -> Vec<GraphNode> {
        self.states
            .iter()
            .map(|state_id| {
                GraphNode::new(
                    *state_id as NodeId,
                    format!("{}", state_id),
                    self.start_state == *state_id,
                    self.accept_states.contains(state_id),
                )
            })
            .collect()
    }

    fn edges(&self) -> Vec<GraphEdge> {
        // For each state in our NFA, get its transitions and create GraphEdges
        let mut edges = Vec::new();
        for state_id in &self.states {
            let transitions = self.transitions(*state_id);

            transitions.iter().for_each(|transition| {
                let label: String = transition.label.clone().into();
                edges.push(GraphEdge {
                    id: edges.len(),
                    from: *state_id as NodeId,
                    to: transition.to as NodeId,
                    label,
                });
            })
        }

        edges
    }

    fn outgoing_edges(&self, node_id: NodeId) -> Vec<GraphEdge> {
        let transitions = self.transitions(node_id as u32);
        transitions
            .iter()
            .enumerate()
            .map(|(idx, transition)| {
                let label: String = transition.label.clone().into();
                GraphEdge {
                    id: idx,
                    from: node_id,
                    to: transition.to as NodeId,
                    label,
                }
            })
            .collect()
    }

    fn incoming_edges(&self, node_id: NodeId) -> Vec<GraphEdge> {
        let mut incoming = Vec::new();
        for state_id in &self.states {
            let transitions = self.transitions(*state_id);
            for (idx, transition) in transitions.iter().enumerate() {
                if transition.to as NodeId == node_id {
                    let label: String = transition.label.clone().into();
                    incoming.push(GraphEdge {
                        id: idx,
                        from: *state_id as NodeId,
                        to: node_id,
                        label,
                    });
                }
            }
        }
        incoming
    }

    fn get_node(&self, node_id: NodeId) -> Option<GraphNode> {
        self.nodes().into_iter().find(|node| node.id == node_id)
    }
}

// ----- minimalist canvas -----
pub struct GraphCanvas<G: Graph> {
    graph: G,
}

impl<G: Graph> GraphCanvas<G> {
    pub fn new(graph: G) -> Self {
        Self { graph }
    }
}

impl<G, Message, R> Program<Message, iced::Theme, R> for GraphCanvas<G>
where
    G: Graph,
    R: Renderer,
{
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &R,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<R>> {
        let mut f = Frame::new(renderer, bounds.size());

        let nodes = self.graph.nodes();
        let edges = self.graph.edges();

        // compute positions: use node.pos unless it is origin; otherwise lay out in a row
        let spacing = 500.0_f32;
        let center_y = bounds.height / 2.0;
        let n = nodes.len() as f32;
        let row_start_x = (bounds.width / 2.0) - spacing * (n - 1.0).max(0.0) / 2.0;

        let mut pos: HashMap<NodeId, Point> = HashMap::new();
        for (i, node) in nodes.iter().enumerate() {
            let p = if node.pos == Point::ORIGIN {
                Point::new(row_start_x + i as f32 * spacing, center_y)
            } else {
                node.pos
            };
            pos.insert(node.id, p);
        }

        // draw edges with arrowheads and labels
        for e in &edges {
            if let (Some(a), Some(b)) = (pos.get(&e.from), pos.get(&e.to)) {
                let line = Path::line(*a, *b);
                f.stroke(&line, Stroke::default().with_width(1.5));

                // arrowhead
                let dir = iced::Vector::new(b.x - a.x, b.y - a.y);
                let len = (dir.x * dir.x + dir.y * dir.y).sqrt().max(1.0);
                let ux = dir.x / len;
                let uy = dir.y / len;
                let head = 10.0;
                let tip = *b;
                let left = Point::new(
                    tip.x - ux * head - uy * (head * 0.5),
                    tip.y - uy * head + ux * (head * 0.5),
                );
                let right = Point::new(
                    tip.x - ux * head + uy * (head * 0.5),
                    tip.y - uy * head - ux * (head * 0.5),
                );
                let tri = Path::new(|b| {
                    b.move_to(tip);
                    b.line_to(left);
                    b.line_to(right);
                    b.close();
                });
                f.fill(&tri, WHITE);

                // edge label at midpoint
                let mid = Point::new((a.x + b.x) * 0.5, (a.y + b.y) * 0.5 - 6.0);
                if !e.label.is_empty() {
                    f.fill_text(CText {
                        content: e.label.clone(),
                        position: mid,
                        ..CText::default()
                    });
                }
            }
        }

        // draw nodes
        for node in &nodes {
            let p = pos[&node.id];
            node.draw(p, &mut f);
        }

        vec![f.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        _event: iced::widget::canvas::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> (iced::event::Status, Option<Message>) {
        (iced::event::Status::Ignored, None)
    }
}
