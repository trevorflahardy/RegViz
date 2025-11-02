use iced::widget::canvas::{self, Frame, Program};
use iced::{Rectangle, Size, Vector, mouse};
use iced_graphics::geometry::Renderer;

use super::layout::LayoutStrategy;
use super::{BoxVisibility, DrawContext, Drawable, Graph, GraphLayout};
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
        }
    }
}

impl<G, S, Message, R> Program<Message, AppTheme, R> for GraphCanvas<G, S>
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
        _theme: &AppTheme,
        bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Vec<canvas::Geometry<R>> {
        // Use the configured layout strategy
        let layout = self.strategy.compute(&self.graph, &self.visibility);
        let fit_zoom = fit_zoom(bounds.size(), &layout);
        let zoom = fit_zoom * self.zoom_factor;

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
        _state: &mut Self::State,
        _event: &iced::Event,
        _bounds: Rectangle,
        _cursor: mouse::Cursor,
    ) -> Option<iced::widget::Action<Message>> {
        None
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
