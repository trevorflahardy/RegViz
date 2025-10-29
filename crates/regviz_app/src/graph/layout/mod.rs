pub mod nfa;
/// Layout algorithms for graph visualization.
///
/// This module provides a pluggable strategy pattern for laying out different types
/// of graphs (NFAs, DFAs, ASTs) with appropriate algorithms for each.
///
/// # Architecture
///
/// The layout system uses the Strategy pattern:
/// - **[`LayoutStrategy`]**: Trait defining the interface for layout algorithms
/// - **[`NfaLayoutStrategy`]**: Hierarchical layout respecting regex bounding boxes
/// - **[`TreeLayoutStrategy`]**: Binary tree layout for AST visualization
///
/// The layout strategies.
pub mod tree;

pub use nfa::NfaLayoutStrategy;
pub use tree::TreeLayoutStrategy;

use iced::Rectangle;
use regviz_core::core::automaton::BoxKind;

use super::{Graph, bbox::PositionedBox, edge::PositionedEdge, node::PositionedNode};

/// Strategy pattern interface for graph layout algorithms.
///
/// Different graph types (NFA, DFA, AST) require different layout approaches:
/// - NFAs benefit from hierarchical layouts that respect regex operator structure
/// - ASTs are naturally visualized as binary trees
/// - DFAs can use force-directed or circular layouts
///
/// By abstracting the layout algorithm behind this trait, we can:
/// - Keep [`GraphCanvas`](super::GraphCanvas) generic and reusable
/// - Easily add new layout strategies without changing rendering code
/// - Allow users to choose visualization style per graph type
pub trait LayoutStrategy {
    /// Computes positions for all nodes, edges, and bounding boxes in the graph.
    ///
    /// # Arguments
    /// - `graph`: The graph to layout (implements [`Graph`] trait)
    /// - `visibility`: Controls which bounding boxes should be rendered (ignored by strategies that don't use boxes)
    ///
    /// # Returns
    /// A [`GraphLayout`] containing positioned nodes, edges, boxes, and overall bounds.
    fn compute<G: Graph>(&self, graph: &G, visibility: &BoxVisibility) -> GraphLayout;
}

/// Controls which bounding boxes are rendered on the canvas.
///
/// Bounding boxes visually group states that belong to the same regex operator.
/// Users can toggle visibility of different operator types to simplify or emphasize
/// parts of the visualization.
#[derive(Debug, Clone)]
pub struct BoxVisibility {
    literal: bool,
    concat: bool,
    alternation: bool,
    kleene_star: bool,
    kleene_plus: bool,
    optional: bool,
}

impl Default for BoxVisibility {
    fn default() -> Self {
        Self {
            literal: true,
            concat: true,
            alternation: true,
            kleene_star: true,
            kleene_plus: true,
            optional: true,
        }
    }
}

impl BoxVisibility {
    /// Returns whether a bounding box of the provided [`BoxKind`] should be shown.
    #[must_use]
    pub fn is_visible(&self, kind: BoxKind) -> bool {
        match kind {
            BoxKind::Literal => self.literal,
            BoxKind::Concat => self.concat,
            BoxKind::Alternation => self.alternation,
            BoxKind::KleeneStar => self.kleene_star,
            BoxKind::KleenePlus => self.kleene_plus,
            BoxKind::Optional => self.optional,
        }
    }

    /// Flips the visibility of the provided [`BoxKind`].
    pub fn toggle(&mut self, kind: BoxKind) {
        match kind {
            BoxKind::Literal => self.literal = !self.literal,
            BoxKind::Concat => self.concat = !self.concat,
            BoxKind::Alternation => self.alternation = !self.alternation,
            BoxKind::KleeneStar => self.kleene_star = !self.kleene_star,
            BoxKind::KleenePlus => self.kleene_plus = !self.kleene_plus,
            BoxKind::Optional => self.optional = !self.optional,
        }
    }
}

/// Complete layout ready to be rendered by the canvas.
///
/// Contains all visual elements (nodes, edges, boxes) with their final screen positions
/// and the overall bounding rectangle needed to size the canvas appropriately.
#[derive(Debug, Clone)]
pub struct GraphLayout {
    /// All positioned bounding boxes (drawn first, behind nodes/edges).
    pub boxes: Vec<PositionedBox>,
    /// All positioned nodes (automaton states).
    pub nodes: Vec<PositionedNode>,
    /// All positioned edges (transitions between states).
    pub edges: Vec<PositionedEdge>,
    /// Logical bounds of the layout prior to zooming/panning.
    pub bounds: Rectangle,
}
