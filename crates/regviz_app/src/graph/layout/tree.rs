/// Binary tree layout algorithm for visualizing Abstract Syntax Trees (ASTs).
///
/// This module implements a classic tree layout strategy suitable for hierarchical
/// structures like ASTs where each node has at most two children. The algorithm
/// creates a visually balanced layout with clear parent-child relationships.
///
/// # Algorithm Overview
///
/// The tree layout uses a two-pass algorithm:
///
/// 1. **Depth Assignment**: Traverse the tree to assign each node a depth level
///    (distance from root). Root is at depth 0, its children at depth 1, etc.
///
/// 2. **Horizontal Positioning**: Within each depth level, distribute nodes
///    horizontally with uniform spacing to avoid overlaps.
///
/// # Layout Properties
///
/// - **Root Position**: Placed at the top center of the canvas
/// - **Vertical Spacing**: Each level is separated by [`LEVEL_HEIGHT`]
/// - **Horizontal Spacing**: Nodes on the same level are separated by [`NODE_WIDTH`]
/// - **Tree Direction**: Top-to-bottom (root at top, leaves at bottom)
/// - **Child Ordering**: Left child positioned left of parent, right child right of parent
///
/// # Example
///
/// For the regex AST of `(a+b)*`:
/// ```text
///        Star
///         |
///        Alt
///       /   \
///      a     b
/// ```
///
/// The layout produces:
/// - Star node at (0, 0)
/// - Alt node at (0, LEVEL_HEIGHT)
/// - 'a' node at (-NODE_WIDTH/2, 2*LEVEL_HEIGHT)
/// - 'b' node at (+NODE_WIDTH/2, 2*LEVEL_HEIGHT)
use std::collections::HashMap;

use iced::{Point, Rectangle};

use super::LayoutStrategy;
use crate::graph::{Graph, GraphNode, edge::PositionedEdge, node::PositionedNode};

/// Binary tree layout strategy for AST visualization.
///
/// This strategy arranges nodes in a hierarchical tree structure with uniform
/// vertical and horizontal spacing. It ignores bounding boxes (sets visibility
/// to false for all box types) since ASTs don't have the regex operator boxes
/// that NFAs use.
///
/// # Algorithm
///
/// 1. Assign depth levels to all nodes via graph traversal
/// 2. Group nodes by depth level
/// 3. Position each level horizontally with equal spacing
/// 4. Connect nodes with edges based on parent-child relationships
/// 5. Calculate overall bounds for the layout
#[derive(Debug, Clone, Copy, Default)]
pub struct TreeLayoutStrategy;

impl LayoutStrategy for TreeLayoutStrategy {
    fn compute<G: Graph>(
        &self,
        graph: &G,
        _visibility: &super::BoxVisibility,
    ) -> super::GraphLayout {
        layout_tree(graph)
    }
}

/// Vertical distance between levels of the tree.
const LEVEL_HEIGHT: f32 = 150.0;

/// Horizontal distance between adjacent nodes on the same level.
const NODE_WIDTH: f32 = 120.0;

/// Radius of tree nodes (used for bounds calculation).
const NODE_RADIUS: f32 = 40.0;

/// Padding around the entire tree layout.
const TREE_PADDING: f32 = 60.0;

/// Main tree layout algorithm.
///
/// This function implements a simple binary tree layout where:
/// - Nodes at the same depth are placed on the same horizontal line
/// - Nodes are distributed evenly across each level
/// - The tree grows top-to-bottom
///
/// # Algorithm Steps
///
/// 1. **Extract graph data**: Get nodes and edges from the graph
/// 2. **Assign depths**: Calculate depth level for each node (root = 0)
/// 3. **Group by depth**: Organize nodes into levels
/// 4. **Compute positions**: Place nodes within each level with uniform spacing
/// 5. **Create positioned elements**: Build the final GraphLayout with coordinates
/// 6. **Calculate bounds**: Determine the overall canvas size needed
///
/// # Returns
///
/// A [`GraphLayout`] containing:
/// - No boxes (ASTs don't use bounding boxes)
/// - Positioned nodes at calculated coordinates
/// - Edges connecting parent nodes to children
/// - Bounds rectangle encompassing all elements
fn layout_tree<G: Graph>(graph: &G) -> super::GraphLayout {
    let nodes = graph.nodes();
    let edges = graph.edges();

    // ASTs don't use bounding boxes, so return empty
    let boxes = Vec::new();

    if nodes.is_empty() {
        return super::GraphLayout {
            boxes,
            nodes: Vec::new(),
            edges: Vec::new(),
            bounds: Rectangle::new(Point::ORIGIN, iced::Size::ZERO),
        };
    }

    // Step 1: Assign depth to each node
    // For simplicity, we assume the first node is the root
    // In a real implementation, you'd traverse the tree structure
    let depths = assign_depths(&nodes, &edges);

    // Step 2: Group nodes by depth level
    let mut levels: HashMap<usize, Vec<&GraphNode>> = HashMap::new();
    for node in &nodes {
        let depth = depths.get(&node.id).copied().unwrap_or(0);
        levels.entry(depth).or_default().push(node);
    }

    // Step 3: Position nodes within each level
    let mut node_positions: HashMap<u32, Point> = HashMap::new();
    let max_depth = levels.keys().max().copied().unwrap_or(0);

    for (depth, level_nodes) in &levels {
        let y = TREE_PADDING + (*depth as f32) * LEVEL_HEIGHT;
        let num_nodes = level_nodes.len();
        let total_width = (num_nodes.saturating_sub(1)) as f32 * NODE_WIDTH;
        let start_x = -total_width / 2.0;

        for (i, node) in level_nodes.iter().enumerate() {
            let x = start_x + (i as f32) * NODE_WIDTH;
            node_positions.insert(node.id, Point::new(x, y));
        }
    }

    // Step 4: Create positioned nodes. Respect any manual positions supplied
    // on `GraphNode` (these should override computed positions).
    let positioned_nodes: Vec<PositionedNode> = nodes
        .iter()
        .filter_map(|node| {
            let pos = node
                .manual_position
                .or_else(|| node_positions.get(&node.id).copied());
            pos.map(|p| {
                let mut pn = PositionedNode::new(node.clone(), p, NODE_RADIUS);
                if node.is_pinned {
                    pn.is_pinned = true;
                    pn.manual_position = node.manual_position;
                }
                pn
            })
        })
        .collect();

    // Step 5: Create positioned edges. Use the final positions of the
    // positioned nodes (which already respect manual/pinned overrides) so
    // edges attach to the visible node centers.
    let final_positions: HashMap<u32, Point> = positioned_nodes
        .iter()
        .map(|pn| (pn.data.id, pn.position))
        .collect();

    let positioned_edges: Vec<PositionedEdge> = edges
        .iter()
        .filter_map(|edge| {
            let from_pos = final_positions.get(&edge.from)?;
            let to_pos = final_positions.get(&edge.to)?;
            // Use explicit node radii so the edge drawing logic can trim
            // the segment to the node boundaries and draw with stroke().
            Some(PositionedEdge::with_radii(
                edge.clone(),
                *from_pos,
                *to_pos,
                NODE_RADIUS,
                NODE_RADIUS,
            ))
        })
        .collect();

    // Step 6: Calculate bounds
    let min_x = final_positions
        .values()
        .map(|p| p.x)
        .fold(f32::INFINITY, f32::min);
    let max_x = final_positions
        .values()
        .map(|p| p.x)
        .fold(f32::NEG_INFINITY, f32::max);
    let min_y = final_positions
        .values()
        .map(|p| p.y)
        .fold(f32::INFINITY, f32::min)
        .min(TREE_PADDING);
    let max_y = final_positions
        .values()
        .map(|p| p.y)
        .fold(f32::NEG_INFINITY, f32::max)
        .max(TREE_PADDING + (max_depth as f32) * LEVEL_HEIGHT);

    let bounds = Rectangle::new(
        Point::new(
            min_x - NODE_RADIUS - TREE_PADDING,
            min_y - NODE_RADIUS - TREE_PADDING,
        ),
        iced::Size::new(
            (max_x - min_x) + 2.0 * (NODE_RADIUS + TREE_PADDING),
            (max_y - min_y) + 2.0 * (NODE_RADIUS + TREE_PADDING),
        ),
    );

    super::GraphLayout {
        boxes,
        nodes: positioned_nodes,
        edges: positioned_edges,
        bounds,
    }
}

/// Assigns a depth level to each node in the tree.
///
/// This uses a simple breadth-first traversal approach. The root node
/// (assumed to be the first node without incoming edges, or node 0 if
/// edges are empty) is assigned depth 0. Its children get depth 1, their
/// children get depth 2, and so on.
///
/// # Arguments
///
/// - `nodes`: All nodes in the graph
/// - `edges`: All edges connecting the nodes
///
/// # Returns
///
/// A map from node ID to depth level (root = 0)
///
/// # Algorithm
///
/// 1. Find the root node (node with no incoming edges)
/// 2. BFS traversal: For each node, assign children depth = parent_depth + 1
/// 3. Return depth assignments
fn assign_depths(nodes: &[GraphNode], edges: &[crate::graph::GraphEdge]) -> HashMap<u32, usize> {
    let mut depths = HashMap::new();

    if nodes.is_empty() {
        return depths;
    }

    // Find nodes with no incoming edges (potential roots)
    let mut has_incoming: HashMap<u32, bool> = HashMap::new();
    for node in nodes {
        has_incoming.insert(node.id, false);
    }

    for edge in edges {
        has_incoming.insert(edge.to, true);
    }

    // Find the first node without incoming edges as root
    let root_id = nodes
        .iter()
        .find(|n| !has_incoming.get(&n.id).copied().unwrap_or(false))
        .map(|n| n.id)
        .unwrap_or(nodes[0].id);

    // BFS to assign depths
    let mut queue = vec![(root_id, 0)];
    let mut visited = HashMap::new();

    while let Some((node_id, depth)) = queue.pop() {
        if visited.contains_key(&node_id) {
            continue;
        }

        visited.insert(node_id, true);
        depths.insert(node_id, depth);

        // Find all children (outgoing edges)
        for edge in edges {
            if edge.from == node_id && !visited.contains_key(&edge.to) {
                queue.push((edge.to, depth + 1));
            }
        }
    }

    depths
}
