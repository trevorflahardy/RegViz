use std::collections::HashMap;

use iced::{Point, Rectangle};
use regviz_core::core::automaton::{BoxId, StateId};

use super::{Graph, GraphBox, bbox::PositionedBox, edge::PositionedEdge, node::PositionedNode};

/// Complete layout ready to be rendered by the canvas.
#[derive(Debug, Clone)]
pub struct GraphLayout {
    /// All positioned bounding boxes.
    pub boxes: Vec<PositionedBox>,
    /// All positioned nodes.
    pub nodes: Vec<PositionedNode>,
    /// All positioned edges.
    pub edges: Vec<PositionedEdge>,
    /// Logical bounds of the layout prior to zooming.
    pub bounds: Rectangle,
}

const NODE_SPACING_X: f32 = 160.0;
const LEVEL_SPACING_Y: f32 = 140.0;
const NODE_RADIUS: f32 = 32.0;
const BOX_PADDING_X: f32 = 60.0;
const BOX_PADDING_Y: f32 = 70.0;

/// Computes a deterministic layout for the provided graph.
#[must_use]
pub fn layout_graph<G: Graph>(graph: &G) -> GraphLayout {
    let nodes = graph.nodes();
    let edges = graph.edges();
    let boxes = graph.boxes();

    let box_depths = compute_box_depths(&boxes);

    let mut positioned_nodes = Vec::with_capacity(nodes.len());
    let mut state_positions = HashMap::new();
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for (index, node) in nodes.iter().enumerate() {
        let depth = node
            .box_id
            .and_then(|id| box_depths.get(&id).copied())
            .unwrap_or(0);
        let position = Point::new(
            index as f32 * NODE_SPACING_X,
            depth as f32 * LEVEL_SPACING_Y,
        );
        positioned_nodes.push(PositionedNode::new(node.clone(), position, NODE_RADIUS));
        state_positions.insert(node.id, position);

        min_x = min_x.min(position.x - NODE_RADIUS);
        max_x = max_x.max(position.x + NODE_RADIUS);
        min_y = min_y.min(position.y - NODE_RADIUS);
        max_y = max_y.max(position.y + NODE_RADIUS);
    }

    let positioned_edges = edges
        .iter()
        .filter_map(|edge| {
            let from = state_positions.get(&edge.from)?;
            let to = state_positions.get(&edge.to)?;
            Some(PositionedEdge::new(edge.clone(), *from, *to))
        })
        .collect::<Vec<_>>();

    let positioned_boxes = layout_boxes(&boxes, &state_positions);
    for bbox in &positioned_boxes {
        min_x = min_x.min(bbox.rect.x);
        min_y = min_y.min(bbox.rect.y);
        max_x = max_x.max(bbox.rect.x + bbox.rect.width);
        max_y = max_y.max(bbox.rect.y + bbox.rect.height);
    }

    if !positioned_nodes.is_empty() {
        min_x -= BOX_PADDING_X * 0.25;
        min_y -= BOX_PADDING_Y * 0.25;
        max_x += BOX_PADDING_X * 0.25;
        max_y += BOX_PADDING_Y * 0.25;
    }

    if !min_x.is_finite() {
        min_x = 0.0;
        max_x = 1.0;
        min_y = 0.0;
        max_y = 1.0;
    }

    GraphLayout {
        boxes: positioned_boxes,
        nodes: positioned_nodes,
        edges: positioned_edges,
        bounds: Rectangle {
            x: min_x,
            y: min_y,
            width: (max_x - min_x).max(1.0),
            height: (max_y - min_y).max(1.0),
        },
    }
}

fn compute_box_depths(boxes: &[GraphBox]) -> HashMap<BoxId, usize> {
    let mut memo = HashMap::new();
    let map: HashMap<_, _> = boxes.iter().map(|b| (b.id, b)).collect();

    for bbox in boxes {
        depth_for_box(bbox.id, &map, &mut memo);
    }
    memo
}

fn depth_for_box<'a>(
    id: BoxId,
    boxes: &HashMap<BoxId, &'a GraphBox>,
    memo: &mut HashMap<BoxId, usize>,
) -> usize {
    if let Some(depth) = memo.get(&id) {
        return *depth;
    }
    let depth = boxes
        .get(&id)
        .and_then(|bbox| bbox.parent)
        .map(|parent| depth_for_box(parent, boxes, memo) + 1)
        .unwrap_or(0);
    memo.insert(id, depth);
    depth
}

fn layout_boxes(
    boxes: &[GraphBox],
    state_positions: &HashMap<StateId, Point>,
) -> Vec<PositionedBox> {
    let map: HashMap<BoxId, GraphBox> = boxes.iter().map(|b| (b.id, b.clone())).collect();
    let mut children: HashMap<BoxId, Vec<BoxId>> = HashMap::new();
    for bbox in boxes {
        if let Some(parent) = bbox.parent {
            children.entry(parent).or_default().push(bbox.id);
        }
    }
    let mut extents: HashMap<BoxId, Rectangle> = HashMap::new();

    for id in map.keys().cloned().collect::<Vec<_>>() {
        compute_extent(id, &map, &children, state_positions, &mut extents);
    }

    map.into_iter()
        .filter_map(|(id, data)| extents.get(&id).map(|rect| PositionedBox::new(data, *rect)))
        .collect()
}

fn compute_extent(
    id: BoxId,
    boxes: &HashMap<BoxId, GraphBox>,
    children: &HashMap<BoxId, Vec<BoxId>>,
    state_positions: &HashMap<StateId, Point>,
    cache: &mut HashMap<BoxId, Rectangle>,
) -> Option<Rectangle> {
    if let Some(rect) = cache.get(&id) {
        return Some(*rect);
    }

    let bbox = boxes.get(&id)?;
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for state in &bbox.states {
        if let Some(pos) = state_positions.get(state) {
            min_x = min_x.min(pos.x - NODE_RADIUS);
            max_x = max_x.max(pos.x + NODE_RADIUS);
            min_y = min_y.min(pos.y - NODE_RADIUS);
            max_y = max_y.max(pos.y + NODE_RADIUS);
        }
    }

    if let Some(children_ids) = children.get(&id) {
        for child_id in children_ids {
            if let Some(child_rect) =
                compute_extent(*child_id, boxes, children, state_positions, cache)
            {
                min_x = min_x.min(child_rect.x);
                min_y = min_y.min(child_rect.y);
                max_x = max_x.max(child_rect.x + child_rect.width);
                max_y = max_y.max(child_rect.y + child_rect.height);
            }
        }
    }

    if !min_x.is_finite() {
        return None;
    }

    let rect = Rectangle {
        x: min_x - BOX_PADDING_X,
        y: min_y - BOX_PADDING_Y,
        width: (max_x - min_x) + BOX_PADDING_X * 2.0,
        height: (max_y - min_y) + BOX_PADDING_Y * 2.0,
    };
    cache.insert(id, rect);
    Some(rect)
}
