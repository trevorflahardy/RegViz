/// Hierarchical layout algorithm for visualizing finite automata.
///
/// This module implements a specialized layout algorithm that arranges automaton states
/// based on the structure of the original regex. Unlike generic graph layout algorithms,
/// this approach leverages the hierarchical bounding boxes produced by the regex compiler
/// to create visually meaningful layouts.
///
/// # High-Level Algorithm
///
/// 1. **Hierarchy Construction**: Build a tree of bounding boxes representing regex operators
/// 2. **Recursive Layout**: Layout each operator bottom-up using operator-specific rules:
///    - Literals: Two states side-by-side
///    - Concatenation: Children arranged horizontally in sequence
///    - Alternation: Branches stacked vertically with shared entry/exit
///    - Unary operators (*, +, ?): Child centered with operator states on sides
/// 3. **Normalization**: Convert each fragment to positive coordinates with padding
/// 4. **Composition**: Stack root boxes vertically, merge all state positions
/// 5. **Rendering**: Compute visual rectangles for boxes, create positioned nodes/edges
///
/// # Example
///
/// For the regex `(a+b)*c`:
/// - Alternation box contains 'a' and 'b' branches stacked vertically
/// - Kleene star box wraps the alternation with entry/exit states
/// - Concatenation places the star result and 'c' horizontally
///
/// # Key Data Structures
///
/// - `BoxHierarchy`: Parent-child relationships between bounding boxes
/// - `BoxLayoutResult`: Intermediate layout with local coordinates and entry/exit points
/// - [`GraphLayout`](super::GraphLayout): Final positioned elements ready for rendering
use std::collections::HashMap;

use iced::{Point, Rectangle};
use regviz_core::core::automaton::{BoxId, BoxKind, StateId};

use super::{BoxVisibility, GraphLayout, LayoutStrategy};
use crate::graph::{
    Graph, GraphBox, GraphNode, bbox::PositionedBox, edge::PositionedEdge, node::PositionedNode,
};

/// NFA-specific hierarchical layout strategy.
///
/// This strategy is designed for NFAs that have bounding box metadata representing
/// the regex operator structure. It creates layouts that visually reflect the
/// hierarchical nature of regular expressions.
///
/// # When to Use
///
/// Use this strategy when:
/// - The graph represents an NFA compiled from a regex
/// - Bounding boxes are available (via [`Graph::boxes()`])
/// - You want the layout to reflect regex operator grouping
///
/// # Algorithm
///
/// The layout is computed bottom-up:
/// 1. Process each bounding box recursively (children first)
/// 2. Apply operator-specific positioning rules
/// 3. Normalize coordinates to positive quadrant with padding
/// 4. Compose fragments into final layout
#[derive(Debug, Clone, Copy, Default)]
pub struct NfaLayoutStrategy;

impl LayoutStrategy for NfaLayoutStrategy {
    fn compute<G: Graph>(
        &self,
        graph: &G,
        visibility: &super::BoxVisibility,
    ) -> super::GraphLayout {
        layout_graph(graph, visibility)
    }
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
const NORMALIZE_PADDING_X_RATIO: f32 = 0.2;
/// Ratio used when normalising child layouts to keep content away from the border vertically.
const NORMALIZE_PADDING_Y_RATIO: f32 = 0.3;
/// Minimum content width retained during normalisation relative to [`NODE_SPACING_X`].
const MIN_CONTENT_WIDTH_RATIO: f32 = 0.3;
/// Minimum content height retained during normalisation relative to [`LEVEL_SPACING_Y`].
const MIN_CONTENT_HEIGHT_RATIO: f32 = 0.3;

/// Represents the hierarchy of bounding boxes produced by the backend.
///
/// Bounding boxes form a tree structure that mirrors the regex's syntax tree.
/// For example, in `(a+b)*`, the alternation box is a child of the star box.
///
/// This structure enables:
/// - Bottom-up layout (layout children before parents)
/// - Determining rendering order (deeper boxes drawn first)
/// - Computing visual extents (box includes all its children)
#[derive(Debug, Clone)]
struct BoxHierarchy {
    /// All bounding boxes indexed by their ID
    map: HashMap<BoxId, GraphBox>,
    /// For each box, the IDs of its immediate children (sorted)
    children: HashMap<BoxId, Vec<BoxId>>,
    /// For each box, the ID of its parent (if any)
    parents: HashMap<BoxId, Option<BoxId>>,
    /// IDs of all root boxes (boxes with no parent), sorted for determinism
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
///
/// This is the main entry point for the layout algorithm. It coordinates several steps:
///
/// # Algorithm Overview
/// 1. **Build hierarchy**: Organize bounding boxes into a parent-child tree structure
/// 2. **Position states**: Recursively layout states within each bounding box based on regex operators
/// 3. **Fallback positioning**: Place any orphaned states that aren't in bounding boxes
/// 4. **Create positioned elements**: Convert logical positions to renderable nodes and edges
/// 5. **Compute bounding boxes**: Calculate rectangles for visible bounding boxes
/// 6. **Track bounds**: Determine the overall canvas size needed for the layout
///
/// # Example Flow
/// For the regex `a+b`, this function will:
/// - Create an alternation box containing both 'a' and 'b' branches
/// - Stack the branches vertically with proper spacing
/// - Position entry/exit states for the alternation
/// - Calculate the overall bounds for rendering
#[must_use]
fn layout_graph<G: Graph>(graph: &G, visibility: &super::BoxVisibility) -> super::GraphLayout {
    let nodes = graph.nodes();
    let edges = graph.edges();
    let boxes = graph.boxes();

    // Step 1: Build a tree structure of all bounding boxes (who contains whom)
    let hierarchy = BoxHierarchy::new(&boxes);
    let mut state_positions: HashMap<StateId, Point> = HashMap::new();

    // Step 2: Recursively position all states within their bounding boxes
    // This handles literals, concatenations, alternations, and unary operators
    let vertical_offset = assign_box_positions(&hierarchy, &mut state_positions);

    // Step 3: Position any states that aren't part of a bounding box
    // These get placed in a simple horizontal line below the main layout
    assign_fallback_positions(
        &nodes,
        &mut state_positions,
        vertical_offset + LEVEL_SPACING_Y,
    );

    // Step 4: Convert state positions into renderable nodes with visual properties
    let mut bounds = BoundsTracker::new();
    let mut positioned_nodes = Vec::with_capacity(nodes.len());

    for node in nodes {
        if let Some(position) = state_positions.get(&node.id).copied() {
            bounds.include_circle(position, NODE_RADIUS);
            positioned_nodes.push(PositionedNode::new(node, position, NODE_RADIUS));
        }
    }

    // Step 5: Create positioned edges connecting the states
    // Only create edges if both endpoints have valid positions
    let positioned_edges = edges
        .into_iter()
        .filter_map(|edge| {
            let from = state_positions.get(&edge.from)?;
            let to = state_positions.get(&edge.to)?;
            Some(PositionedEdge::new(edge, *from, *to))
        })
        .collect::<Vec<_>>();

    // Step 6: Compute rectangles for every bounding box that should be visible
    let positioned_boxes = layout_boxes(&hierarchy, &state_positions, visibility);
    for bbox in &positioned_boxes {
        bounds.include_rect(bbox.rect);
    }

    // Step 7: Add padding around the entire layout for aesthetics
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
///
/// This is the core of the hierarchical layout algorithm. Each bounding box represents a
/// regex operator (literal, concatenation, alternation, etc.) and needs its child states
/// positioned according to the operator's semantics.
///
/// # Algorithm
/// 1. **Recurse on children**: Layout all child bounding boxes first (bottom-up approach)
/// 2. **Apply operator-specific layout**: Use the appropriate layout strategy based on the box type:
///    - Literal: Two states side-by-side (start → accept)
///    - Concat: Children placed horizontally in sequence
///    - Alternation: Children stacked vertically with entry/exit on the sides
///    - Unary operators (*, +, ?): Child centered with operator states on sides
/// 3. **Normalize**: Convert all coordinates to a consistent positive coordinate system
///
/// # Returns
/// A `BoxLayoutResult` containing:
/// - Width/height of the entire fragment
/// - Entry/exit points for connecting to parent layouts
/// - Absolute positions for all states within the fragment
fn compute_box_layout(
    bbox: &GraphBox,
    boxes: &HashMap<BoxId, GraphBox>,
    children: &HashMap<BoxId, Vec<BoxId>>,
) -> BoxLayoutResult {
    // Recursively evaluate each child first so we can stitch them together below.
    // This bottom-up approach ensures we know the size of each child before positioning it.
    let child_layouts = children
        .get(&bbox.id)
        .into_iter()
        .flat_map(|ids| {
            ids.iter()
                .filter_map(|child_id| boxes.get(child_id))
                .map(|child| compute_box_layout(child, boxes, children))
        })
        .collect::<Vec<_>>();

    // Apply the layout strategy appropriate for this operator
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
    // This converts all coordinates so min(x,y) = 0 with some padding.
    normalize_layout(&mut layout);
    layout
}

/// Intermediate layout information for a single bounding box fragment.
///
/// This represents a "fragment" of the layout - a self-contained piece that can be
/// composed with other fragments to build the complete layout. Each fragment has:
/// - Local coordinate system (positions are relative to fragment origin)
/// - Known dimensions (width x height)
/// - Entry and exit points for connecting to other fragments
///
/// # Composition
/// Parent layouts use entry/exit points to align and connect child fragments:
/// - Concatenation: Connect exit of fragment[i] to entry of fragment[i+1]
/// - Alternation: All children's entries connect to parent's entry, exits to parent's exit
#[derive(Debug, Clone)]
struct BoxLayoutResult {
    /// Total width of this fragment in logical units
    width: f32,
    /// Total height of this fragment in logical units
    height: f32,
    /// Point where transitions enter this fragment (left side typically)
    entry: Point,
    /// Point where transitions exit this fragment (right side typically)
    exit: Point,
    /// Positions of all states within this fragment (in fragment's local coordinates)
    positions: HashMap<StateId, Point>,
}

/// Arranges the two states that form a literal fragment.
///
/// A literal like 'a' is represented by two states:
/// - Start state (entry point)
/// - Accept state (exit point)
/// - Connected by a single edge labeled 'a'
///
/// # Visual Layout
/// ```text
/// ●──a──●
/// start accept
/// ```
///
/// Both states are placed on the same horizontal line, separated by NODE_SPACING_X.
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
///
/// Concatenation (e.g., `ab`) means "match a, then match b". The layout places
/// child fragments horizontally in sequence, aligned along a common baseline.
///
/// # Visual Layout
/// For `abc`, this creates:
/// ```text
///        baseline ─────────────────────
///                 ●─a─●  ●─b─●  ●─c─●
///                 (a)    (b)    (c)
/// ```
///
/// # Algorithm
/// 1. **Find baseline**: Use the maximum entry.y of all children (keeps things aligned)
/// 2. **Place children left-to-right**: Each child is offset horizontally with INLINE_GAP_X spacing
/// 3. **Align to baseline**: Offset each child vertically so its entry point sits on the baseline
/// 4. **Track dimensions**: Record the first child's entry and last child's exit as fragment endpoints
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
    // The baseline is chosen as the maximum entry.y across all children to ensure alignment.
    let mut baseline = child_layouts
        .iter()
        .map(|child| child.entry.y)
        .fold(0.0, f32::max);
    if baseline.abs() < f32::EPSILON {
        baseline = LEVEL_SPACING_Y * 0.5;
    }

    let mut positions = HashMap::new();
    let mut cursor_x = 0.0; // Current horizontal position for placing next child
    let mut max_bottom = baseline; // Track the lowest point to calculate total height
    let mut entry = Point::new(0.0, baseline);
    let mut exit = Point::new(0.0, baseline);
    let count = child_layouts.len();

    for (index, child) in child_layouts.into_iter().enumerate() {
        // Offset this child: horizontally by cursor_x, vertically to align entry with baseline
        let offset = Point::new(cursor_x, baseline - child.entry.y);
        merge_positions(&mut positions, child.positions, offset);

        let child_entry = Point::new(child.entry.x + offset.x, child.entry.y + offset.y);
        let child_exit = Point::new(child.exit.x + offset.x, child.exit.y + offset.y);

        // The first child's entry becomes the fragment's entry
        if index == 0 {
            entry = child_entry;
        }
        // The last child's exit becomes the fragment's exit
        if index + 1 == count {
            exit = child_exit;
        }

        // Move cursor to the right for the next child
        cursor_x += child.width;
        if index + 1 < count {
            cursor_x += INLINE_GAP_X; // Add gap between children
        }

        // Track the maximum height needed
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
///
/// Alternation (e.g., `a+b`) means "match a OR match b". The layout stacks each alternative
/// vertically, with a shared entry state on the left and exit state on the right.
///
/// # Visual Layout
/// For `a+b|c`, this creates:
/// ```text
///       ┌─────●─a─●─────┐
///   ●───┼─────●─b─●─────┼───●
/// entry │     ●─c─●     │  exit
///       └───────────────┘
/// ```
///
/// # Algorithm
/// 1. **Stack branches vertically**: Each child fragment placed below the previous with BRANCH_GAP_Y spacing
/// 2. **Track maximum width**: Need to know widest branch for positioning exit state
/// 3. **Center entry/exit**: Place entry/exit states at vertical midpoint, horizontally on the sides
/// 4. **Add operator states**: The first/last states in bbox.states become the entry/exit nodes
fn layout_alternation_box(bbox: &GraphBox, child_layouts: Vec<BoxLayoutResult>) -> BoxLayoutResult {
    let mut positions = HashMap::new();
    let mut current_y = 0.0; // Current vertical position for placing next branch
    let mut max_width: f32 = 0.0; // Widest branch determines exit position
    let count = child_layouts.len();

    // Each branch becomes a vertically stacked child, separated by a branch gap.
    for (index, child) in child_layouts.into_iter().enumerate() {
        // Offset horizontally to leave room for entry state, vertically to stack branches
        let offset = Point::new(NODE_SPACING_X, current_y);
        max_width = max_width.max(child.width);
        merge_positions(&mut positions, child.positions, offset);

        // Move down for next branch
        current_y += child.height;
        if index + 1 < count {
            current_y += BRANCH_GAP_Y; // Add vertical gap between branches
        }
    }

    let total_height = current_y.max(LEVEL_SPACING_Y);
    let entry_y = total_height * 0.5; // Center entry/exit vertically

    // Entry state on the left, exit state on the right
    let entry = Point::new(0.0, entry_y);
    let exit = Point::new(NODE_SPACING_X + max_width + NODE_SPACING_X, entry_y);

    // Add the alternation's entry and exit states
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
///
/// Unary operators (*, +, ?) apply to a single child fragment and add entry/exit states
/// to implement the operator semantics:
/// - `a*` (Kleene star): Match 'a' zero or more times
/// - `a+` (Kleene plus): Match 'a' one or more times
/// - `a?` (Optional): Match 'a' zero or one time
///
/// # Visual Layout
/// For `a*`, this creates:
/// ```text
///       ┌───────────────────┐
///       │    ε (bypass)     │
///   ●───┼──────────────────►├───●
/// entry │  ●───a───●        │  exit
///       │  └───ε───┘ (loop)│
///       └───────────────────┘
/// ```
///
/// The entry and exit states are positioned with vertical offset to create
/// visual separation for the epsilon transitions (bypass and loop-back).
///
/// # Arguments
/// - `bbox`: The bounding box metadata for this operator
/// - `child_layouts`: Layouts of child fragments (should be exactly 1 for unary ops)
///
/// # Returns
/// A layout with the child centered and operator states positioned to show control flow
fn layout_unary_box(bbox: &GraphBox, mut child_layouts: Vec<BoxLayoutResult>) -> BoxLayoutResult {
    let mut positions = HashMap::new();
    let child = child_layouts.pop();

    let (child_width, child_height, _child_entry, _child_exit, child_positions) =
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

    // Position child fragment in the center, with vertical padding for epsilon arcs
    let vertical_padding = LEVEL_SPACING_Y * 0.4; // Space for curved epsilon transitions
    let offset = Point::new(NODE_SPACING_X, vertical_padding);
    merge_positions(&mut positions, child_positions, offset);

    // Entry state on the left, centered vertically in the available space
    let total_height = child_height + vertical_padding * 2.0;
    let entry_y = total_height * 0.5;
    let entry = Point::new(0.0, entry_y);

    // Exit state on the right, aligned with entry
    let exit = Point::new(offset.x + child_width + NODE_SPACING_X, entry_y);

    // Add the operator's entry and exit states
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

/// Offsets the provided child positions by the given amount and merges into the parent map.
///
/// When composing child layouts into a parent layout, we need to translate the child's
/// local coordinates into the parent's coordinate system. This function does that translation
/// and adds the results to the parent's position map.
///
/// # Example
/// If a child has a state at (10, 20) and we apply an offset of (100, 50),
/// the state will be placed at (110, 70) in the parent's coordinate system.
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
///
/// # Why This Is Needed
///
/// Different layout strategies produce coordinates in different ranges:
/// - Concatenation might start at x=0
/// - Alternation might have branches at negative y values (centered around y=0)
/// - After recursive composition, coordinates can be arbitrary
///
/// Normalization ensures that every fragment reports its content in a consistent way:
/// - min(x) = padding (content starts just inside the left edge)
/// - min(y) = padding (content starts just inside the top edge)
/// - width/height accurately reflect the content + padding
///
/// # Algorithm
///
/// 1. **Find bounds**: Scan all positions, entry, and exit to find min/max x/y
/// 2. **Calculate shift**: Determine how much to translate to move min to padding
/// 3. **Apply shift**: Add shift to all positions, entry, and exit points
/// 4. **Update dimensions**: Set width/height based on content size + padding
fn normalize_layout(layout: &mut BoxLayoutResult) {
    // Step 1: Find the current bounding box of all content
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    // Check all state positions
    for pos in layout.positions.values() {
        min_x = min_x.min(pos.x);
        max_x = max_x.max(pos.x);
        min_y = min_y.min(pos.y);
        max_y = max_y.max(pos.y);
    }

    // Also include entry/exit points in the bounds
    min_x = min_x.min(layout.entry.x).min(layout.exit.x);
    max_x = max_x.max(layout.entry.x).max(layout.exit.x);
    min_y = min_y.min(layout.entry.y).min(layout.exit.y);
    max_y = max_y.max(layout.entry.y).max(layout.exit.y);

    // Step 2: Handle empty fragments (no valid positions)
    if !min_x.is_finite() || !min_y.is_finite() {
        layout.width = NODE_SPACING_X;
        layout.height = LEVEL_SPACING_Y;
        layout.entry = Point::new(0.0, LEVEL_SPACING_Y * 0.5);
        layout.exit = Point::new(NODE_SPACING_X, LEVEL_SPACING_Y * 0.5);
        layout.positions.clear();
        return;
    }

    // Step 3: Calculate padding and content dimensions
    // Normalisation converts the fragment so that its minimum x/y become `0` with
    // a little padding. This keeps every fragment in the positive quadrant and
    // prevents parents from having to apply ad-hoc offsets when composing.
    let horizontal_padding = NODE_SPACING_X * NORMALIZE_PADDING_X_RATIO;
    let vertical_padding = LEVEL_SPACING_Y * NORMALIZE_PADDING_Y_RATIO;
    let content_width = (max_x - min_x).max(NODE_SPACING_X * MIN_CONTENT_WIDTH_RATIO);
    let content_height = (max_y - min_y).max(LEVEL_SPACING_Y * MIN_CONTENT_HEIGHT_RATIO);

    // Calculate how much to shift to move min to the padding offset
    let shift_x = -min_x + horizontal_padding;
    let shift_y = -min_y + vertical_padding;

    // Step 4: Apply the shift to all coordinates
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

    // Step 5: Record final dimensions (content + padding on both sides)
    layout.width = content_width + horizontal_padding * 2.0;
    layout.height = content_height + vertical_padding * 2.0;
}

/// Builds rectangles for every bounding box that should be displayed.
///
/// This function determines which boxes to render and in what order:
/// 1. Compute extents (rectangles) for all boxes
/// 2. Filter to only visible boxes based on user preferences
/// 3. Sort by depth (deeper boxes first) for proper layering
///
/// # Rendering Order
/// Boxes are drawn back-to-front (deepest first) so that:
/// - Child boxes appear inside parent boxes
/// - Nested structures are visually clear
/// - Overlapping boxes render correctly
fn layout_boxes(
    hierarchy: &BoxHierarchy,
    state_positions: &HashMap<StateId, Point>,
    visibility: &BoxVisibility,
) -> Vec<PositionedBox> {
    // Step 1: Compute rectangles for all boxes (with caching)
    let mut extents: HashMap<BoxId, Rectangle> = HashMap::new();
    for id in hierarchy.map.keys().copied().collect::<Vec<_>>() {
        compute_extent(id, hierarchy, state_positions, &mut extents);
    }

    // Step 2: Filter to visible boxes and annotate with depth
    let mut depth_cache: HashMap<BoxId, usize> = HashMap::new();
    let mut positioned = Vec::new();

    for (&id, data) in &hierarchy.map {
        // Skip boxes the user has hidden
        if !visibility.is_visible(data.kind) {
            continue;
        }

        if let Some(rect) = extents.get(&id) {
            let depth = compute_depth(id, &hierarchy.parents, &mut depth_cache);
            positioned.push((depth, id, PositionedBox::new(data.clone(), *rect)));
        }
    }

    // Step 3: Sort by depth (deeper first), then by ID for determinism
    positioned.sort_by(|a, b| a.0.cmp(&b.0).then_with(|| a.1.cmp(&b.1)));

    // Return just the positioned boxes (drop depth and id used for sorting)
    positioned.into_iter().map(|(_, _, pb)| pb).collect()
}

/// Computes the rectangle that encloses a bounding box and all of its visible children.
///
/// This function recursively calculates the visual extent of a bounding box by:
/// 1. Finding the positions of all states directly in this box
/// 2. Recursively computing extents of all child boxes
/// 3. Expanding to include all descendants
/// 4. Adding padding around the edges
///
/// The result is cached to avoid redundant computation when boxes are reused.
///
/// # Returns
/// A `Rectangle` that completely encloses the box and all its contents with padding,
/// or `None` if the box has no positioned states or children.
fn compute_extent(
    id: BoxId,
    hierarchy: &BoxHierarchy,
    state_positions: &HashMap<StateId, Point>,
    cache: &mut HashMap<BoxId, Rectangle>,
) -> Option<Rectangle> {
    // Check cache first - avoid recomputing the same box multiple times
    if let Some(rect) = cache.get(&id) {
        return Some(*rect);
    }

    let bbox = hierarchy.map.get(&id)?;

    // Start with infinite/impossible bounds - will be tightened by actual positions
    let mut min_x = f32::INFINITY;
    let mut max_x = f32::NEG_INFINITY;
    let mut min_y = f32::INFINITY;
    let mut max_y = f32::NEG_INFINITY;

    // Expand bounds to include all states directly in this box
    // Each state is a circle, so we include the radius around its center
    for state in &bbox.states {
        if let Some(pos) = state_positions.get(state) {
            min_x = min_x.min(pos.x - NODE_RADIUS);
            max_x = max_x.max(pos.x + NODE_RADIUS);
            min_y = min_y.min(pos.y - NODE_RADIUS);
            max_y = max_y.max(pos.y + NODE_RADIUS);
        }
    }

    // Recursively expand bounds to include all child boxes
    if let Some(children_ids) = hierarchy.children.get(&id) {
        for child_id in children_ids {
            if let Some(child_rect) = compute_extent(*child_id, hierarchy, state_positions, cache) {
                // Include the entire child rectangle
                min_x = min_x.min(child_rect.x);
                min_y = min_y.min(child_rect.y);
                max_x = max_x.max(child_rect.x + child_rect.width);
                max_y = max_y.max(child_rect.y + child_rect.height);
            }
        }
    }

    // If no valid positions found, this box is empty
    if !min_x.is_finite() {
        return None;
    }

    // Create the final rectangle with padding around the content
    let rect = Rectangle {
        x: min_x - BOX_PADDING_X,
        y: min_y - BOX_PADDING_Y,
        width: (max_x - min_x) + BOX_PADDING_X * 2.0,
        height: (max_y - min_y) + BOX_PADDING_Y * 2.0,
    };

    // Cache the result for future lookups
    cache.insert(id, rect);
    Some(rect)
}

/// Calculates how deep in the box hierarchy a particular box resides.
///
/// Depth is measured from the root:
/// - Root boxes (no parent) have depth 0
/// - Direct children of roots have depth 1
/// - And so on...
///
/// This is used to determine rendering order - deeper boxes should be drawn
/// first so they appear behind shallower boxes.
///
/// # Example
/// For regex `(a+b)*`:
/// - Alternation box has depth 0 (root)
/// - Kleene star box has depth 1 (child of root)
/// - Literal boxes for 'a' and 'b' have depth 2 (children of alternation)
fn compute_depth(
    id: BoxId,
    parents: &HashMap<BoxId, Option<BoxId>>,
    cache: &mut HashMap<BoxId, usize>,
) -> usize {
    // Check cache to avoid recomputing
    if let Some(depth) = cache.get(&id) {
        return *depth;
    }

    // Recursively calculate: parent's depth + 1, or 0 if no parent
    let depth = match parents.get(&id).and_then(|p| *p) {
        Some(parent) => compute_depth(parent, parents, cache) + 1,
        None => 0, // Root box
    };

    cache.insert(id, depth);
    depth
}

/// Positions all states within their bounding boxes by processing the hierarchy top-down.
///
/// This function iterates through all root-level bounding boxes and recursively
/// lays them out vertically. Each root box gets its own vertical slice of the canvas,
/// stacked one below the other.
///
/// # Algorithm
/// 1. For each root bounding box:
///    - Recursively compute its layout (positions all states within)
///    - Add those positions to the global state_positions map
///    - Offset the positions by vertical_offset to stack roots
///    - Increment vertical_offset for the next root
///
/// # Returns
/// The total vertical height used by all root boxes, which becomes the baseline
/// for positioning any orphaned states not in bounding boxes.
fn assign_box_positions(
    hierarchy: &BoxHierarchy,
    state_positions: &mut HashMap<StateId, Point>,
) -> f32 {
    let mut vertical_offset = 0.0;

    // Process each root box (boxes with no parent)
    for root_id in &hierarchy.roots {
        if let Some(layout) = hierarchy
            .map
            .get(root_id)
            .map(|bbox| compute_box_layout(bbox, &hierarchy.map, &hierarchy.children))
        {
            // Add all state positions from this box's layout, offset vertically
            for (state, pos) in layout.positions {
                let absolute = Point::new(pos.x, pos.y + vertical_offset);
                state_positions.insert(state, absolute);
            }

            // Move down for the next root box
            vertical_offset += layout.height + LEVEL_SPACING_Y;
        }
    }

    vertical_offset
}

/// Places any states not contained in bounding boxes in a simple horizontal line.
///
/// Some states may not belong to any bounding box (orphaned states). This function
/// gives them a fallback position so they're still visible, arranged in a horizontal
/// line at the specified vertical baseline.
///
/// # Arguments
/// - `nodes`: All nodes in the graph
/// - `state_positions`: Existing positions (will only add missing ones)
/// - `baseline_y`: Vertical position for the fallback line
fn assign_fallback_positions(
    nodes: &[GraphNode],
    state_positions: &mut HashMap<StateId, Point>,
    baseline_y: f32,
) {
    let mut index = 0usize;

    for node in nodes {
        // Skip nodes that already have positions from bounding box layout
        if state_positions.contains_key(&node.id) {
            continue;
        }

        // Place orphaned states in a horizontal line, evenly spaced
        let position = Point::new(index as f32 * NODE_SPACING_X, baseline_y);
        state_positions.insert(node.id, position);
        index += 1;
    }
}
