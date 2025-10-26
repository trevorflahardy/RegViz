use iced::alignment::{Horizontal, Vertical};
use iced::widget::canvas::{Frame, Path, Stroke, Text};
use iced::{Point, Vector};
use iced_graphics::geometry::Renderer;
use regviz_core::core::automaton::{BoxId, StateId};

use super::{DrawContext, Drawable};

/// Visual representation of a state in the rendered graph.
#[derive(Debug, Clone)]
pub struct GraphNode {
    /// Identifier of the state being rendered.
    pub id: StateId,
    /// Human readable label.
    pub label: String,
    /// Whether this node is the start state.
    pub is_start: bool,
    /// Whether this node is an accepting state.
    pub is_accept: bool,
    /// Bounding box identifier that owns this node, if any.
    #[allow(dead_code)]
    pub box_id: Option<BoxId>,
}

impl GraphNode {
    /// Creates a new [`GraphNode`] with sensible defaults.
    #[must_use]
    pub fn new(
        id: StateId,
        label: String,
        is_start: bool,
        is_accept: bool,
        box_id: Option<BoxId>,
    ) -> Self {
        Self {
            id,
            label,
            is_start,
            is_accept,
            box_id,
        }
    }
}

/// [`GraphNode`] accompanied by layout information.
#[derive(Debug, Clone)]
pub struct PositionedNode {
    /// Node metadata.
    pub data: GraphNode,
    /// Logical position before transforms.
    pub position: Point,
    /// Node radius.
    pub radius: f32,
}

impl PositionedNode {
    /// Builds a positioned node from metadata and coordinates.
    #[must_use]
    pub fn new(data: GraphNode, position: Point, radius: f32) -> Self {
        Self {
            data,
            position,
            radius,
        }
    }
}

impl Drawable for PositionedNode {
    fn draw<R: Renderer>(&self, frame: &mut Frame<R>, ctx: &DrawContext) {
        let center = ctx.transform_point(self.position);
        let radius = self.radius * ctx.zoom;
        let circle = Path::circle(center, radius);

        frame.fill(&circle, iced::Color::WHITE);
        frame.stroke(&circle, Stroke::default().with_width(1.5));

        if self.data.is_accept {
            // Accepting states have an inner circle
            let inner = Path::circle(center, radius - 4.0);
            frame.stroke(&inner, Stroke::default().with_width(1.2));
        }

        if self.data.is_start {
            // Start nodes have an arrow pointing to them as a start indicator
            let arrow_start = Point::new(center.x - radius * 1.7, center.y);
            let arrow_end = Point::new(center.x - radius, center.y);
            let arrow = Path::line(arrow_start, arrow_end);
            frame.stroke(&arrow, Stroke::default().with_width(1.3));

            // Arrow head
            let arrow_head = Path::new(|builder| {
                let offset = Vector::new(6.0, 4.0);
                builder.move_to(arrow_end);
                builder.line_to(Point::new(arrow_end.x - offset.x, arrow_end.y - offset.y));
                builder.line_to(Point::new(arrow_end.x - offset.x, arrow_end.y + offset.y));
                builder.close();
            });
            frame.fill(&arrow_head, iced::Color::WHITE);
            frame.stroke(&arrow_head, Stroke::default().with_width(1.0));
        }

        if !self.data.label.is_empty() {
            frame.fill_text(Text {
                content: self.data.label.clone(),
                position: center,
                color: iced::Color::from_rgb(0.1, 0.1, 0.1),
                horizontal_alignment: Horizontal::Center,
                vertical_alignment: Vertical::Center,
                ..Text::default()
            });
        }
    }
}
