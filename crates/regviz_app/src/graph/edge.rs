use iced::{
    Point, Vector,
    widget::canvas::{Frame, Path, Stroke, Text},
};
use iced_graphics::geometry::Renderer;
use regviz_core::core::automaton::StateId;

use super::{DrawContext, Drawable};

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
        let mid = Point::new((from.x + to.x) * 0.5, (from.y + to.y) * 0.5);
        let direction = Vector::new(to.x - from.x, to.y - from.y);
        let length = (direction.x * direction.x + direction.y * direction.y).sqrt();
        let label_position = if length > f32::EPSILON {
            let mut normal = Vector::new(-direction.y / length, direction.x / length);
            if normal.y > 0.0 {
                normal = Vector::new(-normal.x, -normal.y);
            }
            let offset = 18.0;
            Point::new(mid.x + normal.x * offset, mid.y + normal.y * offset)
        } else {
            Point::new(mid.x, mid.y - 18.0)
        };
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

        let direction = Vector::new(to.x - from.x, to.y - from.y);
        let length = (direction.x * direction.x + direction.y * direction.y)
            .sqrt()
            .max(1.0);
        let unit = Vector::new(direction.x / length, direction.y / length);
        let head_len = 10.0;
        let normal = Vector::new(-unit.y, unit.x);

        let tip = to;
        let left = Point::new(
            tip.x - unit.x * head_len + normal.x * (head_len * 0.5),
            tip.y - unit.y * head_len + normal.y * (head_len * 0.5),
        );
        let right = Point::new(
            tip.x - unit.x * head_len - normal.x * (head_len * 0.5),
            tip.y - unit.y * head_len - normal.y * (head_len * 0.5),
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
