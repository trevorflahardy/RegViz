use iced::Font;

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

/// Font used for all text rendered on the graph canvas.
///
/// Web builds do not have access to system fonts, so we explicitly request the
/// bundled Fira Sans family provided by the `iced` crate.
pub const CANVAS_FONT: Font = Font::with_name("Fira Sans");

pub use ast::AstGraph;
pub use bbox::GraphBox;
pub use canvas::GraphCanvas;
pub use dfa::VisualDfa;
pub use draw::{DrawContext, Drawable};
pub use edge::GraphEdge;
pub use highlight::{EdgeHighlight, Highlights, StateHighlight};
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
