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
    fn draw<R: Renderer>(&self, frame: &mut Frame<R>, ctx: &DrawContext) {
        let from = ctx.transform_point(self.from);
        let to = ctx.transform_point(self.to);
        let line = Path::line(from, to);

        frame.stroke(&line, Stroke::default().with_width(1.3));

        let unit = normalise_vector(Vector::new(to.x - from.x, to.y - from.y));
        let normal = perpendicular(unit);

        let tip = to;
        let left = Point::new(
            tip.x - unit.x * ARROW_HEAD_LENGTH + normal.x * ARROW_HEAD_HALF_WIDTH,
            tip.y - unit.y * ARROW_HEAD_LENGTH + normal.y * ARROW_HEAD_HALF_WIDTH,
        );
        let right = Point::new(
            tip.x - unit.x * ARROW_HEAD_LENGTH - normal.x * ARROW_HEAD_HALF_WIDTH,
            tip.y - unit.y * ARROW_HEAD_LENGTH - normal.y * ARROW_HEAD_HALF_WIDTH,
        );

        let arrow_head = Path::new(|builder| {
            builder.move_to(tip);
            builder.line_to(left);
            builder.line_to(right);
            builder.close();
        });
        frame.fill(&arrow_head, iced::Color::WHITE);
        frame.stroke(&arrow_head, Stroke::default().with_width(1.0));

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

fn compute_label_anchor(from: Point, to: Point) -> Point {
    let mid = Point::new((from.x + to.x) * 0.5, (from.y + to.y) * 0.5);
    let direction = Vector::new(to.x - from.x, to.y - from.y);
    let length = (direction.x * direction.x + direction.y * direction.y).sqrt();

    if length <= f32::EPSILON {
        return Point::new(mid.x, mid.y - LABEL_DISTANCE);
    }

    let mut normal = perpendicular(Vector::new(direction.x / length, direction.y / length));
    if normal.y > 0.0 {
        normal = Vector::new(-normal.x, -normal.y);
    }

    Point::new(
        mid.x + normal.x * LABEL_DISTANCE,
        mid.y + normal.y * LABEL_DISTANCE,
    )
}

fn normalise_vector(vector: Vector) -> Vector {
    let length = (vector.x * vector.x + vector.y * vector.y).sqrt();
    if length <= f32::EPSILON {
        return Vector::new(1.0, 0.0);
    }
    Vector::new(vector.x / length, vector.y / length)
}

fn perpendicular(vector: Vector) -> Vector {
    Vector::new(-vector.y, vector.x)
}
