use iced::alignment::{Horizontal, Vertical};
use iced::widget::canvas::{Frame, Path, Stroke, Text};
use iced::{Color, Pixels, Point};
use iced_graphics::geometry::Renderer;
use regviz_core::core::automaton::{BoxId, StateId};

use crate::app::theme::AppTheme;

use super::{CANVAS_FONT, DrawContext, Drawable, StateHighlight};

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
/// Minimum scale for the start arrow head to prevent it becoming invisible.
const START_ARROW_MIN_SCALE: f32 = 0.4;
/// Maximum scale for the start arrow head to avoid excessive growth.
const START_ARROW_MAX_SCALE: f32 = 4.0;
/// Stroke width applied to node outlines.
const NODE_OUTLINE_WIDTH: f32 = 1.5;
/// Stroke width applied to auxiliary shapes such as the accepting ring and start arrow.
const AUXILIARY_STROKE_WIDTH: f32 = 1.2;
/// Stroke width used for the start arrow shaft.
const START_ARROW_STROKE_WIDTH: f32 = 1.3;
/// Base font size for node labels before zoom is applied.
const NODE_LABEL_BASE_SIZE: f32 = 18.0;
/// Minimum font size for node labels.
const NODE_LABEL_MIN_SIZE: f32 = 10.0;
/// Maximum font size for node labels.
const NODE_LABEL_MAX_SIZE: f32 = 52.0;

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
    fn draw<R: Renderer>(&self, frame: &mut Frame<R>, ctx: &DrawContext, theme: &AppTheme) {
        let center = ctx.transform_point(self.position);
        let radius = self.radius * ctx.zoom;
        let circle = Path::circle(center, radius);
        let highlight = self.data.highlight;
        let fill_color = highlight_fill_color(highlight, theme);
        let outline_color = highlight_outline_color(highlight, theme);

        frame.fill(&circle, fill_color);
        frame.stroke(
            &circle,
            Stroke::default()
                .with_width(NODE_OUTLINE_WIDTH)
                .with_color(outline_color),
        );

        if self.data.is_accept {
            draw_accepting_ring(frame, center, radius, ctx.zoom);
        }

        if self.data.is_start {
            draw_start_arrow(frame, center, radius, outline_color, ctx.zoom);
        }

        if !self.data.label.is_empty() {
            let font_size = Pixels::from(
                (NODE_LABEL_BASE_SIZE * ctx.zoom).clamp(NODE_LABEL_MIN_SIZE, NODE_LABEL_MAX_SIZE),
            );
            frame.fill_text(Text {
                content: self.data.label.clone(),
                position: center,
                color: theme.text_primary_inverse(),
                font: CANVAS_FONT,
                align_x: Horizontal::Center.into(),
                align_y: Vertical::Center,
                size: font_size,
                ..Text::default()
            });
        }
    }
}

fn draw_accepting_ring<R: Renderer>(frame: &mut Frame<R>, center: Point, radius: f32, zoom: f32) {
    let gap = (ACCEPT_RING_GAP * zoom).clamp(1.0, radius.max(1.0));
    let inner_radius = (radius - gap).max(0.0);
    if inner_radius <= 0.0 {
        return;
    }
    let inner = Path::circle(center, inner_radius);
    frame.stroke(&inner, Stroke::default().with_width(AUXILIARY_STROKE_WIDTH));
}

fn draw_start_arrow<R: Renderer>(
    frame: &mut Frame<R>,
    center: Point,
    radius: f32,
    outline_color: Color,
    zoom: f32,
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

    let arrow_scale = zoom.clamp(START_ARROW_MIN_SCALE, START_ARROW_MAX_SCALE);
    let head_length = START_ARROW_HEAD_LENGTH * arrow_scale;
    let head_half_height = START_ARROW_HEAD_HALF_HEIGHT * arrow_scale;
    let head_base = Point::new(arrow_tip.x - head_length, arrow_tip.y);
    let head = Path::new(|builder| {
        builder.move_to(arrow_tip);
        builder.line_to(Point::new(head_base.x, head_base.y - head_half_height));
        builder.line_to(Point::new(head_base.x, head_base.y + head_half_height));
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

fn highlight_fill_color(highlight: Option<StateHighlight>, theme: &AppTheme) -> Color {
    match highlight {
        Some(StateHighlight::Active) => theme.graph_node_active(),
        Some(StateHighlight::Rejected) => theme.graph_node_rejected(),
        None => theme.graph_node_default(),
    }
}

fn highlight_outline_color(highlight: Option<StateHighlight>, theme: &AppTheme) -> Color {
    match highlight {
        Some(StateHighlight::Active) => theme.graph_node_outline_active(),
        Some(StateHighlight::Rejected) => theme.graph_node_outline_rejected(),
        None => theme.graph_node_outline_default(),
    }
}
