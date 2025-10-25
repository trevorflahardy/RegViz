use iced::widget::canvas::Frame;
use iced::{Point, Vector};
use iced_graphics::geometry::Renderer;

/// Shared drawing context containing the active transform parameters.
#[derive(Debug, Clone, Copy)]
pub struct DrawContext {
    /// Zoom factor applied to graph elements.
    pub zoom: f32,
    /// Translation applied after zooming.
    pub translation: Vector,
}

impl DrawContext {
    /// Transforms a logical point into screen space using the active zoom and translation.
    #[must_use]
    pub fn transform_point(&self, point: Point) -> Point {
        Point::new(
            point.x * self.zoom + self.translation.x,
            point.y * self.zoom + self.translation.y,
        )
    }
}

/// Trait implemented by renderable items on the canvas.
pub trait Drawable {
    /// Draws the element to the frame using the provided [`DrawContext`].
    fn draw<R: Renderer>(&self, frame: &mut Frame<R>, ctx: &DrawContext);
}
