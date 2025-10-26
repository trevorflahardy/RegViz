use std::collections::HashMap;

use iced::{Point, Rectangle};
use regviz_core::core::automaton::{BoxId, BoxKind, StateId};

use super::{Graph, GraphBox, bbox::PositionedBox, edge::PositionedEdge, node::PositionedNode};

/// Controls which bounding boxes are rendered on the canvas.
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

const NODE_SPACING_X: f32 = 200.0;
const LEVEL_SPACING_Y: f32 = 200.0;
const NODE_RADIUS: f32 = 32.0;
const BOX_PADDING_X: f32 = 30.0;
const BOX_PADDING_Y: f32 = 50.0;
const INLINE_GAP_X: f32 = NODE_SPACING_X * 0.7;
const BRANCH_GAP_Y: f32 = LEVEL_SPACING_Y * 0.9;

/// Computes a deterministic layout for the provided graph.
#[must_use]
pub fn layout_graph<G: Graph>(graph: &G, visibility: &BoxVisibility) -> GraphLayout {
    let nodes = graph.nodes();
    let edges = graph.edges();
    let boxes = graph.boxes();

    let mut positioned_nodes = Vec::with_capacity(nodes.len());
    let mut state_positions: HashMap<StateId, Point> = HashMap::new();
    let mut min_x = f32::INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    // Prepare quick lookups for traversing the bounding box hierarchy.
    let box_map: HashMap<_, _> = boxes.iter().map(|b| (b.id, b.clone())).collect();
    let mut children: HashMap<BoxId, Vec<BoxId>> = HashMap::new();
    for bbox in &boxes {
        if let Some(parent) = bbox.parent {
            children.entry(parent).or_default().push(bbox.id);
        }
    }
    for ids in children.values_mut() {
        ids.sort_unstable();
    }

    // Layout every root bounding box (processing children before parents) and stack them
    // vertically if necessary.
    let mut vertical_offset = 0.0;
    let mut root_ids = boxes
        .iter()
        .filter(|b| b.parent.is_none())
        .map(|b| b.id)
        .collect::<Vec<_>>();
    root_ids.sort_unstable();
    for root_id in root_ids {
        if let Some(layout) = box_map
            .get(&root_id)
            .map(|bbox| compute_box_layout(bbox, &box_map, &children))
        {
            for (state, pos) in layout.positions {
                let absolute = Point::new(pos.x, pos.y + vertical_offset);
                state_positions.insert(state, absolute);
            }
            vertical_offset += layout.height + LEVEL_SPACING_Y;
        }
    }

    // Provide a fallback arrangement for states that were not covered by a box.
    let mut fallback_index = 0usize;
    for node in &nodes {
        state_positions.entry(node.id).or_insert_with(|| {
            let position = Point::new(
                fallback_index as f32 * NODE_SPACING_X,
                vertical_offset + LEVEL_SPACING_Y,
            );
            fallback_index += 1;
            position
        });
    }

    for node in nodes {
        if let Some(position) = state_positions.get(&node.id).copied() {
            min_x = min_x.min(position.x - NODE_RADIUS);
            max_x = max_x.max(position.x + NODE_RADIUS);
            min_y = min_y.min(position.y - NODE_RADIUS);
            max_y = max_y.max(position.y + NODE_RADIUS);
            positioned_nodes.push(PositionedNode::new(node, position, NODE_RADIUS));
        }
    }

    let positioned_edges = edges
        .iter()
        .filter_map(|edge| {
            let from = state_positions.get(&edge.from)?;
            let to = state_positions.get(&edge.to)?;
            Some(PositionedEdge::new(edge.clone(), *from, *to))
        })
        .collect::<Vec<_>>();

    let positioned_boxes = layout_boxes(&boxes, &state_positions, visibility);
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

/// Recursively computes positions for the states contained within the provided bounding box.
fn compute_box_layout(
    bbox: &GraphBox,
    boxes: &HashMap<BoxId, GraphBox>,
    children: &HashMap<BoxId, Vec<BoxId>>,
) -> BoxLayoutResult {
    let child_layouts = children
        .get(&bbox.id)
        .into_iter()
        .flat_map(|ids| {
            ids.iter()
                .filter_map(|child_id| boxes.get(child_id))
                .map(|child| compute_box_layout(child, boxes, children))
        })
        .collect::<Vec<_>>();

    let mut layout = match bbox.kind {
        BoxKind::Literal => layout_literal_box(bbox),
        BoxKind::Concat => layout_concat_box(child_layouts),
        BoxKind::Alternation => layout_alternation_box(bbox, child_layouts),
        BoxKind::KleeneStar | BoxKind::KleenePlus | BoxKind::Optional => {
            layout_unary_box(bbox, child_layouts)
        }
    };

    normalize_layout(&mut layout);
    layout
}

/// Intermediate layout information for a single bounding box.
#[derive(Debug, Clone)]
struct BoxLayoutResult {
    width: f32,
    height: f32,
    entry: Point,
    exit: Point,
    positions: HashMap<StateId, Point>,
}

/// Arranges the two states that form a literal fragment.
fn layout_literal_box(bbox: &GraphBox) -> BoxLayoutResult {
    let mut positions = HashMap::new();
    if let Some((&start, rest)) = bbox.states.split_first() {
        let start_pos = Point::new(0.0, LEVEL_SPACING_Y * 0.5);
        positions.insert(start, start_pos);
        if let Some(&accept) = rest.last() {
            let accept_pos = Point::new(NODE_SPACING_X, LEVEL_SPACING_Y * 0.5);
            positions.insert(accept, accept_pos);
        }
    }

    BoxLayoutResult {
        width: NODE_SPACING_X,
        height: LEVEL_SPACING_Y,
        entry: Point::new(0.0, LEVEL_SPACING_Y * 0.5),
        exit: Point::new(NODE_SPACING_X, LEVEL_SPACING_Y * 0.5),
        positions,
    }
}

/// Places child fragments of a concatenation next to each other on a shared baseline.
fn layout_concat_box(child_layouts: Vec<BoxLayoutResult>) -> BoxLayoutResult {
    if child_layouts.is_empty() {
        return BoxLayoutResult {
            width: NODE_SPACING_X,
            height: LEVEL_SPACING_Y,
            entry: Point::new(0.0, LEVEL_SPACING_Y * 0.5),
            exit: Point::new(NODE_SPACING_X, LEVEL_SPACING_Y * 0.5),
            positions: HashMap::new(),
        };
    }

    let mut baseline = child_layouts
        .iter()
        .map(|child| child.entry.y)
        .fold(0.0, f32::max);
    if baseline.abs() < f32::EPSILON {
        baseline = LEVEL_SPACING_Y * 0.5;
    }
    let mut positions = HashMap::new();
    let mut cursor_x = 0.0;
    let mut max_bottom = baseline;
    let mut entry = Point::new(0.0, baseline);
    let mut exit = Point::new(0.0, baseline);
    let count = child_layouts.len();

    for (index, child) in child_layouts.into_iter().enumerate() {
        let offset = Point::new(cursor_x, baseline - child.entry.y);
        merge_positions(&mut positions, child.positions, offset);

        let child_entry = Point::new(child.entry.x + offset.x, child.entry.y + offset.y);
        let child_exit = Point::new(child.exit.x + offset.x, child.exit.y + offset.y);

        if index == 0 {
            entry = child_entry;
        }
        if index + 1 == count {
            exit = child_exit;
        }

        cursor_x += child.width;
        if index + 1 < count {
            cursor_x += INLINE_GAP_X;
        }
        max_bottom = max_bottom.max(offset.y + child.height);
    }

    BoxLayoutResult {
        width: cursor_x,
        height: max_bottom.max(baseline + LEVEL_SPACING_Y * 0.25),
        entry,
        exit,
        positions,
    }
}

/// Stacks the branches of an alternation vertically while keeping the entry and exit horizontal.
fn layout_alternation_box(bbox: &GraphBox, child_layouts: Vec<BoxLayoutResult>) -> BoxLayoutResult {
    let mut positions = HashMap::new();
    let mut current_y = 0.0;
    let mut max_width: f32 = 0.0;
    let count = child_layouts.len();

    for (index, child) in child_layouts.into_iter().enumerate() {
        let offset = Point::new(NODE_SPACING_X, current_y);
        max_width = max_width.max(child.width);
        merge_positions(&mut positions, child.positions, offset);

        current_y += child.height;
        if index + 1 < count {
            current_y += BRANCH_GAP_Y;
        }
    }

    let total_height = current_y.max(LEVEL_SPACING_Y);
    let entry_y = total_height * 0.5;

    let entry = Point::new(0.0, entry_y);
    let exit = Point::new(NODE_SPACING_X + max_width + NODE_SPACING_X, entry_y);

    if let Some(start) = bbox.states.first() {
        positions.insert(*start, entry);
    }
    if let Some(accept) = bbox.states.last() {
        positions.insert(*accept, exit);
    }

    BoxLayoutResult {
        width: exit.x,
        height: total_height,
        entry,
        exit,
        positions,
    }
}

/// Lays out unary operators such as star, plus and optional by surrounding their operand.
fn layout_unary_box(bbox: &GraphBox, mut child_layouts: Vec<BoxLayoutResult>) -> BoxLayoutResult {
    let mut positions = HashMap::new();
    let child = child_layouts.pop();

    let (child_width, child_height, child_entry, _child_exit, child_positions) =
        if let Some(child) = child {
            (
                child.width,
                child.height,
                child.entry,
                child.exit,
                child.positions,
            )
        } else {
            (
                NODE_SPACING_X,
                LEVEL_SPACING_Y,
                Point::new(0.0, LEVEL_SPACING_Y * 0.5),
                Point::new(NODE_SPACING_X, LEVEL_SPACING_Y * 0.5),
                HashMap::new(),
            )
        };

    let offset = Point::new(NODE_SPACING_X, 0.0);
    merge_positions(&mut positions, child_positions, offset);

    let entry = Point::new(0.0, child_entry.y + offset.y);
    let exit = Point::new(
        offset.x + child_width + NODE_SPACING_X,
        child_entry.y + offset.y,
    );

    if let Some(start) = bbox.states.first() {
        positions.insert(*start, entry);
    }
    if let Some(accept) = bbox.states.last() {
        positions.insert(*accept, exit);
    }

    BoxLayoutResult {
        width: exit.x,
        height: child_height.max(LEVEL_SPACING_Y),
        entry,
        exit,
        positions,
    }
}

/// Offsets the provided child positions by the given amount.
fn merge_positions(
    positions: &mut HashMap<StateId, Point>,
    child_positions: HashMap<StateId, Point>,
    offset: Point,
) {
    for (state, pos) in child_positions {
        positions.insert(state, Point::new(pos.x + offset.x, pos.y + offset.y));
    }
}

/// Normalises a layout so that all coordinates are expressed relative to its top-left origin.
fn normalize_layout(layout: &mut BoxLayoutResult) {
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    for pos in layout.positions.values() {
        min_x = min_x.min(pos.x);
        max_x = max_x.max(pos.x);
        min_y = min_y.min(pos.y);
        max_y = max_y.max(pos.y);
    }

    min_x = min_x.min(layout.entry.x).min(layout.exit.x);
    max_x = max_x.max(layout.entry.x).max(layout.exit.x);
    min_y = min_y.min(layout.entry.y).min(layout.exit.y);
    max_y = max_y.max(layout.entry.y).max(layout.exit.y);

    if !min_x.is_finite() || !min_y.is_finite() {
        layout.width = NODE_SPACING_X;
        layout.height = LEVEL_SPACING_Y;
        layout.entry = Point::new(0.0, LEVEL_SPACING_Y * 0.5);
        layout.exit = Point::new(NODE_SPACING_X, LEVEL_SPACING_Y * 0.5);
        layout.positions.clear();
        return;
    }

    let horizontal_padding = NODE_SPACING_X * 0.2;
    let vertical_padding = LEVEL_SPACING_Y * 0.3;
    let content_width = (max_x - min_x).max(NODE_SPACING_X * 0.3);
    let content_height = (max_y - min_y).max(LEVEL_SPACING_Y * 0.3);
    let shift_x = -min_x + horizontal_padding;
    let shift_y = -min_y + vertical_padding;

    if shift_x.abs() > f32::EPSILON || shift_y.abs() > f32::EPSILON {
        for pos in layout.positions.values_mut() {
            pos.x += shift_x;
            pos.y += shift_y;
        }
        layout.entry.x += shift_x;
        layout.entry.y += shift_y;
        layout.exit.x += shift_x;
        layout.exit.y += shift_y;
    }

    layout.width = content_width + horizontal_padding * 2.0;
    layout.height = content_height + vertical_padding * 2.0;
}

fn layout_boxes(
    boxes: &[GraphBox],
    state_positions: &HashMap<StateId, Point>,
    visibility: &BoxVisibility,
) -> Vec<PositionedBox> {
    let map: HashMap<BoxId, GraphBox> = boxes.iter().map(|b| (b.id, b.clone())).collect();
    let mut children: HashMap<BoxId, Vec<BoxId>> = HashMap::new();
    let mut parents: HashMap<BoxId, Option<BoxId>> = HashMap::new();
    for bbox in boxes {
        if let Some(parent) = bbox.parent {
            children.entry(parent).or_default().push(bbox.id);
        }
        parents.insert(bbox.id, bbox.parent);
    }
    for ids in children.values_mut() {
        ids.sort_unstable();
    }
    let mut extents: HashMap<BoxId, Rectangle> = HashMap::new();

    for id in map.keys().copied().collect::<Vec<_>>() {
        compute_extent(id, &map, &children, state_positions, &mut extents);
    }

    let mut depth_cache: HashMap<BoxId, usize> = HashMap::new();
    let mut positioned = Vec::new();
    for (id, data) in map.into_iter() {
        if !visibility.is_visible(data.kind) {
            continue;
        }
        if let Some(rect) = extents.get(&id) {
            let depth = compute_depth(id, &parents, &mut depth_cache);
            positioned.push((depth, id, PositionedBox::new(data, *rect)));
        }
    }

    positioned.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    positioned.into_iter().map(|(_, _, pb)| pb).collect()
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

fn compute_depth(
    id: BoxId,
    parents: &HashMap<BoxId, Option<BoxId>>,
    cache: &mut HashMap<BoxId, usize>,
) -> usize {
    if let Some(depth) = cache.get(&id) {
        return *depth;
    }

    let depth = match parents.get(&id).and_then(|p| *p) {
        Some(parent) => compute_depth(parent, parents, cache) + 1,
        None => 0,
    };
    cache.insert(id, depth);
    depth
}
