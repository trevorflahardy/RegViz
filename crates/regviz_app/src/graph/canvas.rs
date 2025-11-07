use iced::widget::canvas::{self, Frame, Program};
use iced::{Point, Rectangle, Size, Vector, mouse};
use iced_graphics::geometry::Renderer;

use super::layout::LayoutStrategy;
use super::{BoxVisibility, DrawContext, Drawable, Graph, GraphLayout};
use crate::app::message::{Message, ViewMessage};
use crate::app::theme::AppTheme;

/// Interactive canvas responsible for rendering graphs with zoom support.
///
/// The canvas is generic over both the graph type and the layout strategy,
/// allowing different visualization approaches for different graph types.
#[derive(Debug)]
pub struct GraphCanvas<G: Graph, S: LayoutStrategy> {
    graph: G,
    visibility: BoxVisibility,
    zoom_factor: f32,
    strategy: S,
    /// Pan offset for dragging the canvas
    pan_offset: Vector,
    /// Track if currently dragging
    dragging: bool,
    /// Last cursor position during drag
    last_cursor_position: Option<Point>,
}

impl<G: Graph, S: LayoutStrategy> GraphCanvas<G, S> {
    /// Creates a new canvas for the provided graph implementation with a specific layout strategy.
    ///
    /// # Arguments
    ///
    /// - `graph`: The graph to render
    /// - `visibility`: Controls which bounding boxes are visible
    /// - `zoom_factor`: Initial zoom level (1.0 = fit to screen)
    /// - `strategy`: The layout algorithm to use for positioning nodes
    #[must_use]
    pub fn new(graph: G, visibility: BoxVisibility, zoom_factor: f32, strategy: S) -> Self {
        Self {
            graph,
            visibility,
            zoom_factor,
            strategy,
            pan_offset: Vector::ZERO,
            dragging: false,
            last_cursor_position: None,
        }
    }

    /// Sets the pan offset for this canvas.
    pub fn set_pan_offset(&mut self, offset: Vector) {
        self.pan_offset = offset;
    }

    /// Starts a drag operation at the given cursor position.
    pub fn start_drag(&mut self, position: Point) {
        self.dragging = true;
        self.last_cursor_position = Some(position);
    }
}

impl<G, S, R> Program<Message, AppTheme, R> for GraphCanvas<G, S>
where
    G: Graph,
    S: LayoutStrategy,
    R: Renderer + iced_graphics::geometry::Renderer,
{
    type State = ();

    fn draw(
        &self,
        _state: &Self::State,
        renderer: &R,
        theme: &AppTheme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<R>> {
        // Use the configured layout strategy
        let layout = self.strategy.compute(&self.graph, &self.visibility);
        let fit_zoom = fit_zoom(bounds.size(), &layout);
        let zoom = fit_zoom * self.zoom_factor;

        let translation = center_translation(bounds.size(), &layout, zoom);
        // Apply pan offset to translation
        let translation = translation + self.pan_offset;
        let ctx = DrawContext { zoom, translation };

        let mut frame = Frame::new(renderer, bounds.size());

        for bbox in &layout.boxes {
            bbox.draw(&mut frame, &ctx, theme);
        }
        for edge in &layout.edges {
            edge.draw(&mut frame, &ctx, theme);
        }
        for node in &layout.nodes {
            node.draw(&mut frame, &ctx, theme);
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        _state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        if let canvas::Event::Mouse(mouse_event) = event { match mouse_event {
            // Start dragging on left mouse button press
            mouse::Event::ButtonPressed(mouse::Button::Left) => {
                if let Some(position) = cursor.position_in(bounds) {
                    return Some(canvas::Action::publish(Message::View(
                        ViewMessage::StartPan(position),
                    )));
                }
            }
            // Pan while dragging
            mouse::Event::CursorMoved { .. } => {
                if self.dragging {
                    if let Some(position) = cursor.position_in(bounds) {
                        return Some(canvas::Action::publish(Message::View(ViewMessage::Pan(
                            position,
                        ))));
                    }
                }
            }
            // End dragging on mouse release
            mouse::Event::ButtonReleased(mouse::Button::Left) => {
                if self.dragging {
                    return Some(canvas::Action::publish(Message::View(ViewMessage::EndPan)));
                }
            }
            // Handle scroll wheel for zooming
            mouse::Event::WheelScrolled { delta } => {
                if cursor.is_over(bounds) {
                    let zoom_delta = match delta {
                        // Positive delta for scrolling up (zoom in)
                        mouse::ScrollDelta::Lines { y, .. } => *y,
                        mouse::ScrollDelta::Pixels { y, .. } => y / 50.0, // Scale pixel deltas
                    };
                    return Some(canvas::Action::publish(Message::View(ViewMessage::Zoom(
                        zoom_delta,
                    ))));
                }
            }
            _ => {}
        } }

        None
    }

    fn mouse_interaction(
        &self,
        _state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if self.dragging {
            mouse::Interaction::Grabbing
        } else if cursor.is_over(bounds) {
            mouse::Interaction::Grab
        } else {
            mouse::Interaction::default()
        }
    }
}

fn fit_zoom(size: Size, layout: &GraphLayout) -> f32 {
    if layout.bounds.width <= 0.0 || layout.bounds.height <= 0.0 {
        return 1.0;
    }
    let zoom_x = size.width / layout.bounds.width;
    let zoom_y = size.height / layout.bounds.height;
    zoom_x.min(zoom_y).max(0.01)
}

fn center_translation(size: Size, layout: &GraphLayout, zoom: f32) -> Vector {
    let center_x = layout.bounds.x + layout.bounds.width / 2.0;
    let center_y = layout.bounds.y + layout.bounds.height / 2.0;

    Vector::new(
        size.width / 2.0 - center_x * zoom,
        size.height / 2.0 - center_y * zoom,
    )
}
