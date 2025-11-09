use std::collections::{BTreeMap, HashMap, VecDeque};

use iced::{Point, Rectangle};
use regviz_core::core::automaton::StateId;

use super::{BoxVisibility, GraphLayout, LayoutStrategy};
use crate::graph::{Graph, GraphEdge, GraphNode, edge::PositionedEdge, node::PositionedNode};

/// Horizontal distance between consecutive BFS layers.
const LAYER_SPACING_X: f32 = 240.0;
/// Vertical distance between nodes within a layer.
const NODE_SPACING_Y: f32 = 150.0;
/// Radius of each rendered DFA state.
const NODE_RADIUS: f32 = 32.0;
/// Horizontal padding added to the final layout bounds.
const LAYOUT_PADDING_X: f32 = 80.0;
/// Vertical padding added to the final layout bounds.
const LAYOUT_PADDING_Y: f32 = 90.0;

/// Layered layout strategy specialised for DFA graphs.
///
/// DFAs do not carry bounding-box metadata like NFAs, so we arrange states by
/// performing a breadth-first search (BFS) from the start state and placing
/// each level in a vertical column. Unreachable components fall back to their
/// own BFS traversal and are appended to the right-hand side so that the canvas
/// remains deterministic and readable.
#[derive(Debug, Clone, Copy, Default)]
pub struct DfaLayoutStrategy;

impl LayoutStrategy for DfaLayoutStrategy {
    fn compute<G: Graph>(&self, graph: &G, _visibility: &BoxVisibility) -> GraphLayout {
        layout_graph(graph)
    }
}

fn layout_graph<G: Graph>(graph: &G) -> GraphLayout {
    let nodes = graph.nodes();
    let edges = graph.edges();

    let adjacency = build_adjacency(&edges);
    let mut state_positions = compute_positions(&nodes, &adjacency);

    // If any nodes carry manual positions (pinned by the user), merge those
    // positions into the computed map so edges and positioned nodes use the
    // manual coordinates instead of the automatically computed ones.
    for node in &nodes {
        if let Some(manual) = node.manual_position {
            state_positions.insert(node.id, manual);
        }
    }

    let mut bounds = BoundsTracker::new();
    let mut positioned_nodes = Vec::with_capacity(nodes.len());

    for node in nodes {
        if let Some(position) = state_positions.get(&node.id).copied() {
            bounds.include_circle(position, NODE_RADIUS);
            let mut pnode = PositionedNode::new(node, position, NODE_RADIUS);
            if pnode.data.is_pinned {
                pnode.is_pinned = true;
                pnode.manual_position = pnode.data.manual_position;
            }
            positioned_nodes.push(pnode);
        }
    }

    let positioned_edges = edges
        .into_iter()
        .filter_map(|edge| {
            let from = state_positions.get(&edge.from)?;
            let to = state_positions.get(&edge.to)?;
            Some(PositionedEdge::new(edge, *from, *to))
        })
        .collect::<Vec<_>>();

    if !positioned_nodes.is_empty() {
        bounds.pad(LAYOUT_PADDING_X, LAYOUT_PADDING_Y);
    }

    GraphLayout {
        boxes: Vec::new(),
        nodes: positioned_nodes,
        edges: positioned_edges,
        bounds: bounds.finish(),
    }
}

fn build_adjacency(edges: &[GraphEdge]) -> HashMap<StateId, Vec<StateId>> {
    let mut adjacency: HashMap<StateId, Vec<StateId>> = HashMap::new();

    for edge in edges {
        adjacency.entry(edge.from).or_default().push(edge.to);
    }

    for neighbours in adjacency.values_mut() {
        neighbours.sort_unstable();
        neighbours.dedup();
    }

    adjacency
}

fn compute_positions(
    nodes: &[GraphNode],
    adjacency: &HashMap<StateId, Vec<StateId>>,
) -> HashMap<StateId, Point> {
    let levels = assign_levels(nodes, adjacency);

    let mut layers: BTreeMap<usize, Vec<StateId>> = BTreeMap::new();
    for node in nodes {
        let level = levels.get(&node.id).copied().unwrap_or(0);
        layers.entry(level).or_default().push(node.id);
    }

    for ids in layers.values_mut() {
        ids.sort_unstable();
    }

    let mut unique_levels: Vec<usize> = layers.keys().copied().collect();
    unique_levels.sort_unstable();
    let level_index: HashMap<usize, usize> = unique_levels
        .iter()
        .enumerate()
        .map(|(idx, level)| (*level, idx))
        .collect();

    let mut positions = HashMap::new();

    for (level, ids) in layers {
        let column = level_index[&level] as f32;
        let x = column * LAYER_SPACING_X;
        let count = ids.len();
        let base_y = if count <= 1 {
            0.0
        } else {
            -((count as f32 - 1.0) * NODE_SPACING_Y * 0.5)
        };

        for (index, state_id) in ids.into_iter().enumerate() {
            let y = base_y + index as f32 * NODE_SPACING_Y;
            positions.insert(state_id, Point::new(x, y));
        }
    }

    normalise_positions(&mut positions);
    positions
}

fn assign_levels(
    nodes: &[GraphNode],
    adjacency: &HashMap<StateId, Vec<StateId>>,
) -> HashMap<StateId, usize> {
    let mut levels: HashMap<StateId, usize> = HashMap::new();

    if let Some(start) = nodes.iter().find(|node| node.is_start).map(|node| node.id) {
        bfs_assign(start, 0, adjacency, &mut levels);
    }

    let mut remaining = nodes
        .iter()
        .map(|node| node.id)
        .filter(|id| !levels.contains_key(id))
        .collect::<Vec<_>>();
    remaining.sort_unstable();

    for state_id in remaining {
        let offset = levels
            .values()
            .copied()
            .max()
            .map(|max| max + 1)
            .unwrap_or(0);
        bfs_assign(state_id, offset, adjacency, &mut levels);
    }

    levels
}

fn bfs_assign(
    start: StateId,
    offset: usize,
    adjacency: &HashMap<StateId, Vec<StateId>>,
    levels: &mut HashMap<StateId, usize>,
) {
    if levels.contains_key(&start) {
        return;
    }

    let mut queue = VecDeque::new();
    levels.insert(start, offset);
    queue.push_back(start);

    while let Some(current) = queue.pop_front() {
        let current_level = levels[&current];
        if let Some(neighbours) = adjacency.get(&current) {
            for &next in neighbours {
                if levels.contains_key(&next) {
                    continue;
                }
                levels.insert(next, current_level + 1);
                queue.push_back(next);
            }
        }
    }
}

fn normalise_positions(positions: &mut HashMap<StateId, Point>) {
    if positions.is_empty() {
        return;
    }

    let min_x = positions
        .values()
        .map(|point| point.x)
        .fold(f32::INFINITY, f32::min);
    let min_y = positions
        .values()
        .map(|point| point.y)
        .fold(f32::INFINITY, f32::min);

    let offset_x = if min_x.is_finite() && min_x < 0.0 {
        -min_x
    } else {
        0.0
    };
    let offset_y = if min_y.is_finite() && min_y < 0.0 {
        -min_y
    } else {
        0.0
    };

    if offset_x != 0.0 || offset_y != 0.0 {
        for point in positions.values_mut() {
            point.x += offset_x;
            point.y += offset_y;
        }
    }
}

#[derive(Debug, Clone)]
struct BoundsTracker {
    min_x: f32,
    min_y: f32,
    max_x: f32,
    max_y: f32,
    has_content: bool,
}

impl BoundsTracker {
    fn new() -> Self {
        Self {
            min_x: f32::INFINITY,
            min_y: f32::INFINITY,
            max_x: f32::NEG_INFINITY,
            max_y: f32::NEG_INFINITY,
            has_content: false,
        }
    }

    fn include_circle(&mut self, center: Point, radius: f32) {
        self.include_rect(Rectangle {
            x: center.x - radius,
            y: center.y - radius,
            width: radius * 2.0,
            height: radius * 2.0,
        });
    }

    fn include_rect(&mut self, rect: Rectangle) {
        self.min_x = self.min_x.min(rect.x);
        self.min_y = self.min_y.min(rect.y);
        self.max_x = self.max_x.max(rect.x + rect.width);
        self.max_y = self.max_y.max(rect.y + rect.height);
        self.has_content = true;
    }

    fn pad(&mut self, horizontal: f32, vertical: f32) {
        if self.has_content {
            self.min_x -= horizontal;
            self.max_x += horizontal;
            self.min_y -= vertical;
            self.max_y += vertical;
        }
    }

    fn finish(self) -> Rectangle {
        if !self.has_content {
            return Rectangle {
                x: 0.0,
                y: 0.0,
                width: 1.0,
                height: 1.0,
            };
        }

        Rectangle {
            x: self.min_x,
            y: self.min_y,
            width: (self.max_x - self.min_x).max(1.0),
            height: (self.max_y - self.min_y).max(1.0),
        }
    }
}
