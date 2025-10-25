use iced::widget::canvas::{self, Frame, Program};
use iced::{Rectangle, Size, Vector, mouse};
use iced_graphics::geometry::Renderer;

use super::{DrawContext, Drawable, Graph, GraphLayout, layout_graph};

/// Interactive canvas responsible for rendering graphs with zoom support.
#[derive(Debug)]
pub struct GraphCanvas<G: Graph> {
    graph: G,
}

impl<G: Graph> GraphCanvas<G> {
    /// Creates a new canvas for the provided graph implementation.
    #[must_use]
    pub fn new(graph: G) -> Self {
        Self { graph }
    }
}

/// Persistent state associated with the [`GraphCanvas`].
#[derive(Debug, Clone)]
pub struct GraphCanvasState {
    zoom: Option<f32>,
}

impl Default for GraphCanvasState {
    fn default() -> Self {
        Self { zoom: None }
    }
}

impl<G, Message, R> Program<Message, iced::Theme, R> for GraphCanvas<G>
where
    G: Graph,
    R: Renderer,
{
    type State = GraphCanvasState;

    fn draw(
        &self,
        state: &Self::State,
        renderer: &R,
        _theme: &iced::Theme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<R>> {
        let layout = layout_graph(&self.graph);
        let fit_zoom = fit_zoom(bounds.size(), &layout);
        let zoom = state.zoom.unwrap_or(fit_zoom);

        let translation = center_translation(bounds.size(), &layout, zoom);
        let ctx = DrawContext { zoom, translation };

        let mut frame = Frame::new(renderer, bounds.size());

        for bbox in &layout.boxes {
            bbox.draw(&mut frame, &ctx);
        }
        for edge in &layout.edges {
            edge.draw(&mut frame, &ctx);
        }
        for node in &layout.nodes {
            node.draw(&mut frame, &ctx);
        }

        vec![frame.into_geometry()]
    }

    fn update(
        &self,
        state: &mut Self::State,
        event: canvas::Event,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> (iced::event::Status, Option<Message>) {
        if let canvas::Event::Mouse(mouse::Event::WheelScrolled { delta }) = event {
            let layout = layout_graph(&self.graph);
            let fit_zoom = fit_zoom(bounds.size(), &layout);
            let current_zoom = state.zoom.unwrap_or(fit_zoom);
            let scroll = match delta {
                mouse::ScrollDelta::Lines { y, .. } => y,
                mouse::ScrollDelta::Pixels { y, .. } => y / 120.0,
            };
            let factor = (1.0 + scroll * 0.1).clamp(0.5, 1.5);
            let new_zoom = (current_zoom * factor).clamp(0.1, 8.0);
            state.zoom = Some(new_zoom);
            return (iced::event::Status::Captured, None);
        }

        (iced::event::Status::Ignored, None)
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
