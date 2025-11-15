use iced::widget::canvas::{self, Frame, Program};
use iced::{Point, Rectangle, Size, Vector, mouse};
use iced_graphics::geometry::Renderer;
use regviz_core::core::automaton::StateId;

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
    pub pan_offset: Vector,
    /// Track if currently panning
    pub panning: bool,
}

/// Mutable runtime state for the canvas program.
#[derive(Debug, Clone, Default)]
pub struct CanvasState {
    /// Currently dragged node id + position, if any.
    node_dragging: Option<(StateId, Point)>,
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
            panning: false,
        }
    }
}

impl<G, S, R> Program<Message, AppTheme, R> for GraphCanvas<G, S>
where
    G: Graph,
    S: LayoutStrategy,
    R: Renderer + iced_graphics::geometry::Renderer,
{
    type State = CanvasState;

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
        state: &mut Self::State,
        event: &canvas::Event,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> Option<canvas::Action<Message>> {
        // We'll need the computed layout and transform to translate cursor
        // screen coordinates into layout coordinates for hit testing.
        let layout = self.strategy.compute(&self.graph, &self.visibility);
        let fit = fit_zoom(bounds.size(), &layout);
        let zoom = fit * self.zoom_factor;
        let translation = center_translation(bounds.size(), &layout, zoom) + self.pan_offset;

        if let canvas::Event::Mouse(mouse_event) = event {
            match mouse_event {
                // Left mouse press: either start a node drag (if clicked a node)
                // or start panning the canvas.
                mouse::Event::ButtonPressed(mouse::Button::Left) => {
                    if let Some(screen_pos) = cursor.position_in(bounds) {
                        // Convert to layout coordinates (inverse transform)
                        let logical = Point::new(
                            (screen_pos.x - translation.x) / zoom,
                            (screen_pos.y - translation.y) / zoom,
                        );

                        // Hit-test nodes by radius in layout coordinates.
                        if let Some(hit) = layout.nodes.iter().find(|n| {
                            let dx = n.position.x - logical.x;
                            let dy = n.position.y - logical.y;
                            (dx * dx + dy * dy) <= (n.radius * n.radius)
                        }) {
                            // Start node-drag locally so subsequent cursor
                            // moves will immediately emit NodeDrag messages
                            // without waiting for the app->view roundtrip.
                            state.node_dragging = Some((hit.data.id, logical));

                            // Tell the app about the initial drag
                            return Some(canvas::Action::publish(Message::View(
                                ViewMessage::NodeDrag(hit.data.id, logical),
                            )));
                        }

                        // No node hit — start panning instead. This message will tell the app
                        // to set canvas' panning state to true.
                        return Some(canvas::Action::publish(Message::View(
                            ViewMessage::StartPan(screen_pos),
                        )));
                    }
                }

                // Cursor movement: if a node drag is active, publish NodeDrag;
                // otherwise publish Pan if we're currently panning.
                mouse::Event::CursorMoved { .. } => {
                    if let Some((node_id, _)) = state.node_dragging
                        && let Some(screen_pos) = cursor.position_in(bounds)
                    {
                        let logical = Point::new(
                            (screen_pos.x - translation.x) / zoom,
                            (screen_pos.y - translation.y) / zoom,
                        );

                        // Update last known position
                        state.node_dragging = Some((node_id, logical));

                        return Some(canvas::Action::publish(Message::View(
                            ViewMessage::NodeDrag(node_id, logical),
                        )));
                    }

                    if self.panning
                        && let Some(position) = cursor.position_in(bounds)
                    {
                        return Some(canvas::Action::publish(Message::View(ViewMessage::Pan(
                            position,
                        ))));
                    }
                }

                // Mouse release: end node drag if active, otherwise end pan.
                mouse::Event::ButtonReleased(mouse::Button::Left) => {
                    if let Some((node_id, position)) = state.node_dragging {
                        // Clear local drag state
                        state.node_dragging = None;

                        let final_position = if let Some(screen_pos) = cursor.position_in(bounds) {
                            Point::new(
                                (screen_pos.x - translation.x) / zoom,
                                (screen_pos.y - translation.y) / zoom,
                            )
                        } else {
                            // Use last known if cursor is outside bounds
                            position
                        };

                        // Notify app about final position
                        return Some(canvas::Action::publish(Message::View(
                            ViewMessage::NodeDrag(node_id, final_position),
                        )));
                    }

                    if self.panning {
                        // Notify app that panning ended. The app will update its canvas' panning state to false.
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
            }
        }

        None
    }

    fn mouse_interaction(
        &self,
        state: &Self::State,
        bounds: Rectangle,
        cursor: mouse::Cursor,
    ) -> mouse::Interaction {
        if self.panning || state.node_dragging.is_some() {
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
