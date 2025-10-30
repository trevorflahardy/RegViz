mod ast;
mod bbox;
mod canvas;
mod dfa;
mod draw;
mod edge;
mod highlight;
pub mod layout;
mod nfa;
mod node;
mod style;

pub use ast::AstGraph;
pub use bbox::GraphBox;
pub use canvas::GraphCanvas;
pub use dfa::VisualDfa;
pub use draw::{DrawContext, Drawable};
pub use edge::GraphEdge;
pub use highlight::{EdgeHighlight, Highlights};
pub use layout::{BoxVisibility, GraphLayout};
pub use nfa::VisualNfa;
pub use node::GraphNode;
pub use style::color_for_box;

pub trait Graph {
    /// Returns all renderable nodes for the graph.
    fn nodes(&self) -> Vec<GraphNode>;

    /// Returns all edges between the nodes.
    fn edges(&self) -> Vec<GraphEdge>;

    /// Returns bounding boxes that should be rendered behind the nodes.
    fn boxes(&self) -> Vec<GraphBox>;
}
