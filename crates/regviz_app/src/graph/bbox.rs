use iced::{Color, Point, Rectangle, border::Radius};
use iced_graphics::geometry::{Frame, Path, Renderer as GeometryRenderer, Stroke, Text};
use regviz_core::core::automaton::{self, BoxId, BoxKind, StateId};

use crate::app::theme::{AppTheme, TextSize};

use super::{DrawContext, Drawable, color_for_box};

/// Metadata describing a bounding box that groups multiple states together.
#[derive(Debug, Clone)]
pub struct GraphBox {
    /// Unique identifier of the box.
    pub id: BoxId,
    /// The semantic kind of the box (concat, alternation, literal, ...).
    pub kind: BoxKind,
    /// Optional identifier of the parent box.
    pub parent: Option<BoxId>,
    /// The states that were created while this box was active.
    pub states: Vec<StateId>,
}

impl GraphBox {
    /// Human readable label derived from the [`BoxKind`].
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self.kind {
            BoxKind::Literal => "Literal",
            BoxKind::Concat => "Concat",
            BoxKind::Alternation => "Alternation",
            BoxKind::KleeneStar => "Star",
            BoxKind::KleenePlus => "Plus",
            BoxKind::Optional => "Optional",
        }
    }
}

/// Renderable bounding box with geometry information.
#[derive(Debug, Clone)]
pub struct PositionedBox {
    /// Logical box metadata.
    pub data: GraphBox,
    /// Screen-space rectangle enclosing the box.
    pub rect: Rectangle,
    /// Fill color used to render the box.
    pub color: Color,
    /// Anchor where the label should be drawn.
    pub label_position: Point,
}

impl Drawable for PositionedBox {
    fn draw<R: GeometryRenderer>(&self, frame: &mut Frame<R>, ctx: &DrawContext, theme: &AppTheme) {
        let top_left = ctx.transform_point(Point::new(self.rect.x, self.rect.y));
        let bottom_right = ctx.transform_point(Point::new(
            self.rect.x + self.rect.width,
            self.rect.y + self.rect.height,
        ));

        let rect = Path::rounded_rectangle(
            top_left,
            iced::Size::new(
                (bottom_right.x - top_left.x).abs(),
                (bottom_right.y - top_left.y).abs(),
            ),
            Radius::from(12.0),
        );

        frame.fill(&rect, self.color);
        frame.stroke(
            &rect,
            Stroke::default()
                .with_width(1.0)
                .with_color(theme.text_secondary()),
        );

        let label_pos = ctx.transform_point(self.label_position);
        frame.fill_text(Text {
            content: self.data.label().to_string(),
            position: Point::new(label_pos.x, label_pos.y),
            color: theme.text_primary(),
            size: TextSize::Small.into(),
            ..Text::default()
        });
    }
}

impl From<automaton::BoundingBox> for GraphBox {
    fn from(value: automaton::BoundingBox) -> Self {
        Self {
            id: value.id,
            kind: value.kind,
            parent: value.parent,
            states: value.states,
        }
    }
}

impl PositionedBox {
    /// Creates a [`PositionedBox`] from a [`GraphBox`] and the computed rectangle.
    #[must_use]
    pub fn new(data: GraphBox, rect: Rectangle) -> Self {
        let color = color_for_box(data.id);
        let label_position = Point::new(rect.x + 8.0, rect.y + 18.0);
        Self {
            data,
            rect,
            color,
            label_position,
        }
    }
}
