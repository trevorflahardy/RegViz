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
}

impl PositionedEdge {
    /// Creates a new positioned edge from metadata and coordinates, keeping the label legible
    /// by offsetting it away from the rendered segment.
    #[must_use]
    pub fn new(data: GraphEdge, from: Point, to: Point) -> Self {
        let label_position = compute_label_anchor(from, to);
        Self {
            data,
            from,
            to,
            label_position,
        }
    }
}

impl Drawable for PositionedEdge {
    /// Draws a directed edge from one state to another with an arrow head and label.
    ///
    /// This function performs three main tasks:
    /// 1. Draws the main line segment connecting the two states
    /// 2. Draws an arrow head at the destination to show direction
    /// 3. Renders the transition label near the midpoint of the edge
    fn draw<R: Renderer>(&self, frame: &mut Frame<R>, ctx: &DrawContext) {
        // Transform logical coordinates to screen coordinates based on zoom/pan
        let from = ctx.transform_point(self.from);
        let to = ctx.transform_point(self.to);

        // Draw the main line connecting the two states
        let line = Path::line(from, to);
        frame.stroke(&line, Stroke::default().with_width(1.3));

        // Calculate the direction of the edge (unit vector pointing from 'from' to 'to')
        let unit = normalise_vector(Vector::new(to.x - from.x, to.y - from.y));
        // Calculate the perpendicular vector (rotated 90° counterclockwise) for arrow wings
        let normal = perpendicular(unit);

        // Build the arrow head triangle at the destination point
        // The arrow points in the direction of the edge (unit vector)
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

/// Converts a vector into a unit vector (length = 1.0) pointing in the same direction.
///
/// This is essential for calculating directions without caring about distance.
/// For example, if you have a vector (300, 400), the unit vector would be (0.6, 0.8).
///
/// # Returns
/// A vector with length 1.0, or (1.0, 0.0) if the input vector has zero length.
fn normalise_vector(vector: Vector) -> Vector {
    // Calculate the length using Pythagorean theorem: √(x² + y²)
    let length = (vector.x * vector.x + vector.y * vector.y).sqrt();

    // Avoid division by zero - if vector has no length, return a default direction
    if length <= f32::EPSILON {
        return Vector::new(1.0, 0.0);
    }

    // Divide both components by length to get a unit vector
    Vector::new(vector.x / length, vector.y / length)
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
