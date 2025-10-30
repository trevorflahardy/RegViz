use iced::alignment::{Horizontal, Vertical};
use iced::widget::canvas::{Frame, Path, Stroke, Text};
use iced::{Color, Point};
use iced_graphics::geometry::Renderer;
use regviz_core::core::automaton::{BoxId, StateId};

use super::{DrawContext, Drawable, StateHighlight};

/// Width of the gap between the outer and inner circle for accepting states.
const ACCEPT_RING_GAP: f32 = 4.0;
/// Distance between the centre of a start node and the tail of its arrow as a multiple of the radius.
const START_ARROW_DISTANCE_FACTOR: f32 = 1.7;
/// Distance between the centre of a start node and the start of the arrow head along the shaft.
const START_ARROW_HEAD_OFFSET: f32 = 1.0;
/// Length of the arrow head segment.
const START_ARROW_HEAD_LENGTH: f32 = 6.0;
/// Half-height of the arrow head used to form the triangle.
const START_ARROW_HEAD_HALF_HEIGHT: f32 = 4.0;
/// Stroke width applied to node outlines.
const NODE_OUTLINE_WIDTH: f32 = 1.5;
/// Stroke width applied to auxiliary shapes such as the accepting ring and start arrow.
const AUXILIARY_STROKE_WIDTH: f32 = 1.2;
/// Stroke width used for the start arrow shaft.
const START_ARROW_STROKE_WIDTH: f32 = 1.3;

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
    /// Optional highlight applied during simulation.
    pub highlight: Option<StateHighlight>,
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
            highlight: None,
        }
    }

    /// Applies a highlight style for the current simulation step.
    #[must_use]
    pub fn with_highlight(mut self, highlight: Option<StateHighlight>) -> Self {
        self.highlight = highlight;
        self
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
        let highlight = self.data.highlight;
        let fill_color = highlight_fill_color(highlight);
        let outline_color = highlight_outline_color(highlight);

        frame.fill(&circle, fill_color);
        frame.stroke(
            &circle,
            Stroke::default()
                .with_width(NODE_OUTLINE_WIDTH)
                .with_color(outline_color),
        );

        if self.data.is_accept {
            draw_accepting_ring(frame, center, radius);
        }

        if self.data.is_start {
            draw_start_arrow(frame, center, radius, outline_color);
        }

        if !self.data.label.is_empty() {
            frame.fill_text(Text {
                content: self.data.label.clone(),
                position: center,
                color: label_color(highlight),
                horizontal_alignment: Horizontal::Center,
                vertical_alignment: Vertical::Center,
                ..Text::default()
            });
        }
    }
}

fn draw_accepting_ring<R: Renderer>(frame: &mut Frame<R>, center: Point, radius: f32) {
    let inner = Path::circle(center, radius - ACCEPT_RING_GAP);
    frame.stroke(&inner, Stroke::default().with_width(AUXILIARY_STROKE_WIDTH));
}

fn draw_start_arrow<R: Renderer>(
    frame: &mut Frame<R>,
    center: Point,
    radius: f32,
    outline_color: Color,
) {
    let arrow_tail = Point::new(center.x - radius * START_ARROW_DISTANCE_FACTOR, center.y);
    let arrow_tip = Point::new(center.x - radius * START_ARROW_HEAD_OFFSET, center.y);
    let arrow = Path::line(arrow_tail, arrow_tip);
    frame.stroke(
        &arrow,
        Stroke::default()
            .with_width(START_ARROW_STROKE_WIDTH)
            .with_color(outline_color),
    );

    let head_base = Point::new(arrow_tip.x - START_ARROW_HEAD_LENGTH, arrow_tip.y);
    let head = Path::new(|builder| {
        builder.move_to(arrow_tip);
        builder.line_to(Point::new(
            head_base.x,
            head_base.y - START_ARROW_HEAD_HALF_HEIGHT,
        ));
        builder.line_to(Point::new(
            head_base.x,
            head_base.y + START_ARROW_HEAD_HALF_HEIGHT,
        ));
        builder.close();
    });
    frame.fill(&head, Color::WHITE);
    frame.stroke(
        &head,
        Stroke::default()
            .with_width(AUXILIARY_STROKE_WIDTH)
            .with_color(outline_color),
    );
}

fn active_node_fill() -> Color {
    Color::from_rgb8(129, 199, 132)
}

fn active_node_outline() -> Color {
    Color::from_rgb8(56, 142, 60)
}

fn default_node_outline() -> Color {
    Color::from_rgb(0.15, 0.15, 0.15)
}

fn rejected_node_fill() -> Color {
    Color::from_rgb8(239, 154, 154)
}

fn rejected_node_outline() -> Color {
    Color::from_rgb8(198, 40, 40)
}

fn highlight_fill_color(highlight: Option<StateHighlight>) -> Color {
    match highlight {
        Some(StateHighlight::Active) => active_node_fill(),
        Some(StateHighlight::Rejected) => rejected_node_fill(),
        None => Color::WHITE,
    }
}

fn highlight_outline_color(highlight: Option<StateHighlight>) -> Color {
    match highlight {
        Some(StateHighlight::Active) => active_node_outline(),
        Some(StateHighlight::Rejected) => rejected_node_outline(),
        None => default_node_outline(),
    }
}

fn label_color(highlight: Option<StateHighlight>) -> Color {
    match highlight {
        Some(StateHighlight::Active) => Color::from_rgb8(15, 63, 21),
        Some(StateHighlight::Rejected) => Color::from_rgb8(97, 13, 20),
        None => Color::from_rgb(0.1, 0.1, 0.1),
    }
}
