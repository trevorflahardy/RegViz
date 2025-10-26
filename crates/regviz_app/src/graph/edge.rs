use iced::{
    Point, Vector,
    widget::canvas::{Frame, Path, Stroke, Text},
};
use iced_graphics::geometry::Renderer;
use regviz_core::core::automaton::StateId;

use super::{DrawContext, Drawable};

/// Distance between the edge segment and its label in logical units.
const LABEL_DISTANCE: f32 = 18.0;
/// Length of each arrow head side.
const ARROW_HEAD_LENGTH: f32 = 10.0;
/// Half-width of the arrow head at its base.
const ARROW_HEAD_HALF_WIDTH: f32 = ARROW_HEAD_LENGTH * 0.5;

/// Renderable description of a transition between two states.
#[derive(Debug, Clone)]
pub struct GraphEdge {
    /// Origin node identifier.
    pub from: StateId,
    /// Destination node identifier.
    pub to: StateId,
    /// Label displayed next to the edge.
    pub label: String,
}

impl GraphEdge {
    /// Creates a new [`GraphEdge`].
    #[must_use]
    pub fn new(from: StateId, to: StateId, label: String) -> Self {
        Self { from, to, label }
    }
}

/// [`GraphEdge`] enriched with layout information.
#[derive(Debug, Clone)]
pub struct PositionedEdge {
    /// Edge metadata.
    pub data: GraphEdge,
    /// Start position.
    pub from: Point,
    /// End position.
    pub to: Point,
    /// Suggested position for the label.
    pub label_position: Point,
    /// Radius of the source node (used to adjust edge start point).
    pub from_radius: f32,
    /// Radius of the destination node (used to adjust edge end point).
    pub to_radius: f32,
}

impl PositionedEdge {
    /// Creates a new positioned edge from metadata and coordinates, keeping the label legible
    /// by offsetting it away from the rendered segment.
    ///
    /// Note: The `from` and `to` points represent node centers. The actual edge will be
    /// drawn from the edge of the source node to the edge of the destination node.
    #[must_use]
    pub fn new(data: GraphEdge, from: Point, to: Point) -> Self {
        let label_position = compute_label_anchor(from, to);
        Self {
            data,
            from,
            to,
            label_position,
            from_radius: 32.0, // Default node radius
            to_radius: 32.0,   // Default node radius
        }
    }

    /// Creates a new positioned edge with explicit node radii.
    ///
    /// This is useful when nodes have different radii (e.g., in different visualization modes).
    /// ! NOTE: This function is currently unused but kept for potential future use.
    #[must_use]
    #[allow(dead_code)]
    pub fn with_radii(
        data: GraphEdge,
        from: Point,
        to: Point,
        from_radius: f32,
        to_radius: f32,
    ) -> Self {
        let label_position = compute_label_anchor(from, to);
        Self {
            data,
            from,
            to,
            label_position,
            from_radius,
            to_radius,
        }
    }
}

impl Drawable for PositionedEdge {
    /// Draws a directed edge from one state to another with an arrow head and label.
    ///
    /// This function performs several tasks:
    /// 1. Adjusts edge endpoints to stop at node boundaries (not centers)
    /// 2. Draws the main line segment connecting the two states
    /// 3. Draws an arrow head at the destination to show direction
    /// 4. Renders the transition label near the midpoint of the edge
    ///
    /// The edge is shortened on both ends so it doesn't overlap with the node circles.
    fn draw<R: Renderer>(&self, frame: &mut Frame<R>, ctx: &DrawContext) {
        // Transform logical coordinates to screen coordinates based on zoom/pan
        let from_center = ctx.transform_point(self.from);
        let to_center = ctx.transform_point(self.to);

        // Calculate the direction vector from source to destination
        let direction = Vector::new(to_center.x - from_center.x, to_center.y - from_center.y);
        let length = (direction.x * direction.x + direction.y * direction.y).sqrt();

        // If nodes are at the same position, don't draw anything
        if length <= f32::EPSILON {
            return;
        }

        // Normalize direction to unit vector
        let unit = Vector::new(direction.x / length, direction.y / length);

        // Scale node radii by zoom factor
        let from_radius = self.from_radius * ctx.zoom;
        let to_radius = self.to_radius * ctx.zoom;

        // Adjust the start point: move from node center outward by the node radius
        let from = Point::new(
            from_center.x + unit.x * from_radius,
            from_center.y + unit.y * from_radius,
        );

        // Adjust the end point: move from node center inward by the node radius
        // This ensures the arrow head sits right at the edge of the destination node
        let to = Point::new(
            to_center.x - unit.x * to_radius,
            to_center.y - unit.y * to_radius,
        );

        // Draw the main line connecting the two states (now shortened to node boundaries)
        let line = Path::line(from, to);
        frame.stroke(&line, Stroke::default().with_width(1.3));

        // Calculate the perpendicular vector (rotated 90° counterclockwise) for arrow wings
        let normal = perpendicular(unit);

        // Build the arrow head triangle at the destination point (edge of destination node)
        let tip = to;

        // Left wing: move back along the edge direction, then offset perpendicular
        let left = Point::new(
            tip.x - unit.x * ARROW_HEAD_LENGTH + normal.x * ARROW_HEAD_HALF_WIDTH,
            tip.y - unit.y * ARROW_HEAD_LENGTH + normal.y * ARROW_HEAD_HALF_WIDTH,
        );

        // Right wing: move back along the edge direction, then offset opposite perpendicular
        let right = Point::new(
            tip.x - unit.x * ARROW_HEAD_LENGTH - normal.x * ARROW_HEAD_HALF_WIDTH,
            tip.y - unit.y * ARROW_HEAD_LENGTH - normal.y * ARROW_HEAD_HALF_WIDTH,
        );

        // Create a filled triangle for the arrow head
        let arrow_head = Path::new(|builder| {
            builder.move_to(tip);
            builder.line_to(left);
            builder.line_to(right);
            builder.close();
        });
        frame.fill(&arrow_head, iced::Color::WHITE);
        frame.stroke(&arrow_head, Stroke::default().with_width(1.0));

        // Draw the transition label (e.g., 'a', 'b', 'ε') near the edge
        if !self.data.label.is_empty() {
            let label_pos = ctx.transform_point(self.label_position);
            frame.fill_text(Text {
                content: self.data.label.clone(),
                position: label_pos,
                color: iced::Color::from_rgb(0.1, 0.1, 0.1),
                ..Text::default()
            });
        }
    }
}

/// Calculates where to place the label text for an edge connecting two points.
///
/// The label is positioned at the midpoint of the edge, offset perpendicular to the
/// edge direction. The offset is always placed "above" the edge (negative y direction)
/// to keep labels readable and consistent.
///
/// # Algorithm
/// 1. Find the midpoint between the two endpoints
/// 2. Calculate the perpendicular (normal) vector to the edge direction
/// 3. Flip the normal if it points downward (so labels always appear above)
/// 4. Offset the midpoint by LABEL_DISTANCE in the normal direction
fn compute_label_anchor(from: Point, to: Point) -> Point {
    // Calculate the midpoint of the edge
    let mid = Point::new((from.x + to.x) * 0.5, (from.y + to.y) * 0.5);

    // Calculate the vector from 'from' to 'to'
    let direction = Vector::new(to.x - from.x, to.y - from.y);
    let length = (direction.x * direction.x + direction.y * direction.y).sqrt();

    // If the two points are basically the same, just place label above
    if length <= f32::EPSILON {
        return Point::new(mid.x, mid.y - LABEL_DISTANCE);
    }

    // Get the perpendicular (normal) vector - rotated 90° from the edge direction
    let mut normal = perpendicular(Vector::new(direction.x / length, direction.y / length));

    // Flip the normal if it points downward (positive y) so labels always appear above
    if normal.y > 0.0 {
        normal = Vector::new(-normal.x, -normal.y);
    }

    // Offset the midpoint by the normal vector scaled by LABEL_DISTANCE
    Point::new(
        mid.x + normal.x * LABEL_DISTANCE,
        mid.y + normal.y * LABEL_DISTANCE,
    )
}

/// Returns a vector perpendicular (at 90°) to the input vector.
///
/// This is a 2D rotation by 90° counterclockwise:
/// - If input is (x, y), output is (-y, x)
/// - Example: (1, 0) becomes (0, 1), which is 90° counterclockwise
/// - Example: (0, 1) becomes (-1, 0), which is 90° counterclockwise
///
/// This is useful for calculating normals to edges (for arrow heads and label placement).
fn perpendicular(vector: Vector) -> Vector {
    Vector::new(-vector.y, vector.x)
}
