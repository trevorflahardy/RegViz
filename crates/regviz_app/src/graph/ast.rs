/// AST graph wrapper that converts Abstract Syntax Trees into renderable graphs.
///
/// This module provides `AstGraph`, which implements the `Graph` trait for
/// `regviz_core::core::parser::Ast`. It converts the recursive tree structure into
/// a flat representation of nodes and edges suitable for visualization.
///
/// # Node Representation
///
/// Each AST node (Char, Concat, Alt, Star, Plus, Opt) becomes a `GraphNode` with:
/// - A unique numeric ID
/// - A label describing the operator or character
/// - No special start/accept state markers (ASTs don't have those)
///
/// # Edge Representation
///
/// Parent-child relationships in the AST become directed edges:
/// - Binary operators (Concat, Alt) have two outgoing edges (left, right)
/// - Unary operators (Star, Plus, Opt) have one outgoing edge (child)
/// - Leaf nodes (Char) have no outgoing edges
///
/// # Example
///
/// For the regex `a+b`:
/// ```text
/// AST:       Graph:
///   Alt       Node 0: Alt
///   / \       ├─ Edge "left" → Node 1
///  a   b      └─ Edge "right" → Node 2
///            Node 1: Char(a)
///            Node 2: Char(b)
/// ```
use regviz_core::core::parser::Ast;

use super::{Graph, GraphBox, GraphEdge, GraphNode};

/// Wrapper around an AST that implements the `Graph` trait.
///
/// This struct lazily converts an AST into a graph representation when
/// the `Graph` trait methods are called. The conversion assigns unique
/// IDs to each AST node and creates edges for parent-child relationships.
#[derive(Debug, Clone)]
pub struct AstGraph<'a> {
    /// The abstract syntax tree to visualize.
    ast: &'a Ast,
    /// Optional pinned positions for specific AST nodes (by generated numeric id).
    pinned_positions: &'a std::collections::HashMap<u32, iced::Point>,
}

impl<'a> AstGraph<'a> {
    /// Creates a new AST graph wrapper.
    #[must_use]
    pub fn new(
        ast: &'a Ast,
        pinned_positions: &'a std::collections::HashMap<u32, iced::Point>,
    ) -> Self {
        Self {
            ast,
            pinned_positions,
        }
    }
}

impl<'a> Graph for AstGraph<'a> {
    fn nodes(&self) -> Vec<GraphNode> {
        let mut nodes = Vec::new();
        let mut next_id = 0;
        collect_nodes(self.ast, &mut nodes, &mut next_id, self.pinned_positions);
        nodes
    }

    fn edges(&self) -> Vec<GraphEdge> {
        let mut edges = Vec::new();
        let mut next_id = 0;
        collect_edges(self.ast, &mut edges, &mut next_id);
        edges
    }

    fn boxes(&self) -> Vec<GraphBox> {
        // ASTs don't have bounding boxes like NFAs do
        Vec::new()
    }
}

/// Recursively collects all nodes from the AST.
///
/// Each AST node is converted to a `GraphNode` with a unique ID and
/// descriptive label. The `next_id` counter is incremented for each
/// node to ensure uniqueness.
///
/// # Arguments
///
/// - `ast`: The current AST node being processed
/// - `nodes`: Accumulated list of graph nodes
/// - `next_id`: Counter for generating unique node IDs
fn collect_nodes(
    ast: &Ast,
    nodes: &mut Vec<GraphNode>,
    next_id: &mut u32,
    pinned: &std::collections::HashMap<u32, iced::Point>,
) {
    let id = *next_id;
    *next_id += 1;

    let label = match ast {
        Ast::Atom(c) => format!("'{c}'"),
        Ast::Concat(_, _) => "·".to_string(), // Concatenation operator
        Ast::Alt(_, _) => "+".to_string(),
        Ast::Star(_) => "*".to_string(),
        Ast::Epsilon => "ε".to_string(),
        Ast::Opt(_) => "?".to_string(),
    };

    let mut node = GraphNode {
        id,
        label,
        is_start: false,
        is_accept: false,
        box_id: None,
        highlight: None,
        is_pinned: false,
        manual_position: None,
    };

    if let Some(pos) = pinned.get(&id) {
        node.manual_position = Some(*pos);
        node.is_pinned = true;
    }

    nodes.push(node);

    // Recursively process children
    match ast {
        Ast::Atom(_) => {} // Leaf node, no children
        Ast::Epsilon => {} // Leaf node, no children
        Ast::Concat(left, right) | Ast::Alt(left, right) => {
            collect_nodes(left, nodes, next_id, pinned);
            collect_nodes(right, nodes, next_id, pinned);
        }
        Ast::Star(inner) | Ast::Opt(inner) => {
            collect_nodes(inner, nodes, next_id, pinned);
        }
    }
}

/// Recursively collects all edges from the AST.
///
/// Each parent-child relationship becomes an edge. Binary operators
/// create two edges (labeled "L" for left, "R" for right), and unary
/// operators create one edge (labeled "").
///
/// # Arguments
///
/// - `ast`: The current AST node being processed
/// - `edges`: Accumulated list of graph edges
/// - `next_id`: Counter tracking the current node ID (must match collect_nodes)
///
/// # Returns
///
/// The ID of the current node (used by parent to create edges)
fn collect_edges(ast: &Ast, edges: &mut Vec<GraphEdge>, next_id: &mut u32) -> u32 {
    let current_id = *next_id;
    *next_id += 1;

    match ast {
        Ast::Atom(_) => {} // Leaf node, no outgoing edges
        Ast::Epsilon => {} // Leaf node, no outgoing edges
        Ast::Concat(left, right) | Ast::Alt(left, right) => {
            let left_id = collect_edges(left, edges, next_id);
            let right_id = collect_edges(right, edges, next_id);

            // AST edges are always straight (they're not automaton transitions)
            edges.push(GraphEdge::new(current_id, left_id, "L".to_string()));
            edges.push(GraphEdge::new(current_id, right_id, "R".to_string()));
        }
        Ast::Star(inner) | Ast::Opt(inner) => {
            let child_id = collect_edges(inner, edges, next_id);
            // AST edges are always straight (they're not automaton transitions)
            edges.push(GraphEdge::new(current_id, child_id, String::new()));
        }
    }

    current_id
}
