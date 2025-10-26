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

/// Horizontal distance between consecutive nodes on the same level.
const NODE_SPACING_X: f32 = 200.0;
/// Vertical distance between levels of the automaton.
const LEVEL_SPACING_Y: f32 = 200.0;
/// Radius used when drawing states.
const NODE_RADIUS: f32 = 32.0;
/// Horizontal padding applied to bounding boxes.
const BOX_PADDING_X: f32 = 30.0;
/// Vertical padding applied to bounding boxes.
const BOX_PADDING_Y: f32 = 50.0;
/// Gap inserted between fragments when laying out concatenations.
const INLINE_GAP_X: f32 = NODE_SPACING_X * INLINE_GAP_RATIO;
/// Distance inserted between stacked alternation branches.
const BRANCH_GAP_Y: f32 = LEVEL_SPACING_Y * BRANCH_GAP_RATIO;
/// Multiplier converting node spacing into inline gaps.
const INLINE_GAP_RATIO: f32 = 0.7;
/// Multiplier converting level spacing into branch gaps.
const BRANCH_GAP_RATIO: f32 = 0.9;
/// Additional padding applied around the entire layout once nodes are placed.
const LAYOUT_PADDING_RATIO: f32 = 0.25;
/// Ratio used when normalising child layouts to keep content away from the border horizontally.
const NORMALISE_PADDING_X_RATIO: f32 = 0.2;
/// Ratio used when normalising child layouts to keep content away from the border vertically.
const NORMALISE_PADDING_Y_RATIO: f32 = 0.3;
/// Minimum content width retained during normalisation relative to [`NODE_SPACING_X`].
const MIN_CONTENT_WIDTH_RATIO: f32 = 0.3;
/// Minimum content height retained during normalisation relative to [`LEVEL_SPACING_Y`].
const MIN_CONTENT_HEIGHT_RATIO: f32 = 0.3;

/// Represents the hierarchy of bounding boxes produced by the backend.
#[derive(Debug, Clone)]
struct BoxHierarchy {
    map: HashMap<BoxId, GraphBox>,
    children: HashMap<BoxId, Vec<BoxId>>,
    parents: HashMap<BoxId, Option<BoxId>>,
    roots: Vec<BoxId>,
}

impl BoxHierarchy {
    fn new(boxes: &[GraphBox]) -> Self {
        let mut map: HashMap<BoxId, GraphBox> = HashMap::with_capacity(boxes.len());
        let mut children: HashMap<BoxId, Vec<BoxId>> = HashMap::new();
        let mut parents: HashMap<BoxId, Option<BoxId>> = HashMap::with_capacity(boxes.len());

        for bbox in boxes {
            if let Some(parent) = bbox.parent {
                children.entry(parent).or_default().push(bbox.id);
            }
            parents.insert(bbox.id, bbox.parent);
            map.insert(bbox.id, bbox.clone());
        }

        for ids in children.values_mut() {
            ids.sort_unstable();
        }

        let mut roots = map
            .values()
            .filter(|bbox| bbox.parent.is_none())
            .map(|bbox| bbox.id)
            .collect::<Vec<_>>();
        roots.sort_unstable();

        Self {
            map,
            children,
            parents,
            roots,
        }
    }
}

/// Tracks the minimum rectangle that encloses all rendered primitives.
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

/// Computes a deterministic layout for the provided graph.
#[must_use]
pub fn layout_graph<G: Graph>(graph: &G, visibility: &BoxVisibility) -> GraphLayout {
    let nodes = graph.nodes();
    let edges = graph.edges();
    let boxes = graph.boxes();

    let hierarchy = BoxHierarchy::new(&boxes);
    let mut state_positions: HashMap<StateId, Point> = HashMap::new();

    let vertical_offset = assign_box_positions(&hierarchy, &mut state_positions);
    assign_fallback_positions(
        &nodes,
        &mut state_positions,
        vertical_offset + LEVEL_SPACING_Y,
    );

    let mut bounds = BoundsTracker::new();
    let mut positioned_nodes = Vec::with_capacity(nodes.len());

    for node in nodes {
        if let Some(position) = state_positions.get(&node.id).copied() {
            bounds.include_circle(position, NODE_RADIUS);
            positioned_nodes.push(PositionedNode::new(node, position, NODE_RADIUS));
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

    // Compute rectangles for every box that should be visible.
    let positioned_boxes = layout_boxes(&hierarchy, &state_positions, visibility);
    for bbox in &positioned_boxes {
        bounds.include_rect(bbox.rect);
    }

    if !positioned_nodes.is_empty() {
        bounds.pad(
            BOX_PADDING_X * LAYOUT_PADDING_RATIO,
            BOX_PADDING_Y * LAYOUT_PADDING_RATIO,
        );
    }

    GraphLayout {
        boxes: positioned_boxes,
        nodes: positioned_nodes,
        edges: positioned_edges,
        bounds: bounds.finish(),
    }
}

/// Recursively computes positions for the states contained within the provided bounding box.
fn compute_box_layout(
    bbox: &GraphBox,
    boxes: &HashMap<BoxId, GraphBox>,
    children: &HashMap<BoxId, Vec<BoxId>>,
) -> BoxLayoutResult {
    // Recursively evaluate each child first so we can stitch them together below.
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

    // Normalising ensures the fragment uses a consistent local coordinate system
    // so parents can place it using its reported width/height without extra math.
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
        // The literal fragment has two states that sit on the same horizontal line.
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

    // All children are stacked horizontally along a common baseline.
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

    // Each branch becomes a vertically stacked child, separated by a branch gap.
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
    // Offset the child so that we can draw the operator decorations around it.
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
///
/// Child fragments frequently report positions that dip into negative coordinates during
/// intermediate calculations. Normalisation slides every point so that the minimum x/y move
/// to zero (with configurable padding) and records the resulting width/height, making it easy
/// for parent layouts to compose fragments using only the returned metrics.
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

    // Normalisation converts the fragment so that its minimum x/y become `0` with
    // a little padding. This keeps every fragment in the positive quadrant and
    // prevents parents from having to apply ad-hoc offsets when composing.
    let horizontal_padding = NODE_SPACING_X * NORMALISE_PADDING_X_RATIO;
    let vertical_padding = LEVEL_SPACING_Y * NORMALISE_PADDING_Y_RATIO;
    let content_width = (max_x - min_x).max(NODE_SPACING_X * MIN_CONTENT_WIDTH_RATIO);
    let content_height = (max_y - min_y).max(LEVEL_SPACING_Y * MIN_CONTENT_HEIGHT_RATIO);
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

/// Builds rectangles for every bounding box that should be displayed.
fn layout_boxes(
    hierarchy: &BoxHierarchy,
    state_positions: &HashMap<StateId, Point>,
    visibility: &BoxVisibility,
) -> Vec<PositionedBox> {
    let mut extents: HashMap<BoxId, Rectangle> = HashMap::new();

    for id in hierarchy.map.keys().copied().collect::<Vec<_>>() {
        compute_extent(id, hierarchy, state_positions, &mut extents);
    }

    let mut depth_cache: HashMap<BoxId, usize> = HashMap::new();
    let mut positioned = Vec::new();
    for (&id, data) in &hierarchy.map {
        if !visibility.is_visible(data.kind) {
            continue;
        }
        if let Some(rect) = extents.get(&id) {
            let depth = compute_depth(id, &hierarchy.parents, &mut depth_cache);
            positioned.push((depth, id, PositionedBox::new(data.clone(), *rect)));
        }
    }

    positioned.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));
    positioned.into_iter().map(|(_, _, pb)| pb).collect()
}

/// Computes the rectangle that encloses a bounding box and all of its visible children.
fn compute_extent(
    id: BoxId,
    hierarchy: &BoxHierarchy,
    state_positions: &HashMap<StateId, Point>,
    cache: &mut HashMap<BoxId, Rectangle>,
) -> Option<Rectangle> {
    if let Some(rect) = cache.get(&id) {
        return Some(*rect);
    }

    let bbox = hierarchy.map.get(&id)?;
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

    if let Some(children_ids) = hierarchy.children.get(&id) {
        for child_id in children_ids {
            if let Some(child_rect) = compute_extent(*child_id, hierarchy, state_positions, cache) {
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

/// Calculates how deep in the box hierarchy a particular box resides.
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

fn assign_box_positions(
    hierarchy: &BoxHierarchy,
    state_positions: &mut HashMap<StateId, Point>,
) -> f32 {
    let mut vertical_offset = 0.0;

    for root_id in &hierarchy.roots {
        if let Some(layout) = hierarchy
            .map
            .get(root_id)
            .map(|bbox| compute_box_layout(bbox, &hierarchy.map, &hierarchy.children))
        {
            for (state, pos) in layout.positions {
                let absolute = Point::new(pos.x, pos.y + vertical_offset);
                state_positions.insert(state, absolute);
            }

            vertical_offset += layout.height + LEVEL_SPACING_Y;
        }
    }

    vertical_offset
}

fn assign_fallback_positions(
    nodes: &[super::GraphNode],
    state_positions: &mut HashMap<StateId, Point>,
    baseline_y: f32,
) {
    let mut index = 0usize;
    for node in nodes {
        if state_positions.contains_key(&node.id) {
            continue;
        }

        let position = Point::new(index as f32 * NODE_SPACING_X, baseline_y);
        state_positions.insert(node.id, position);
        index += 1;
    }
}
