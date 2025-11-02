use iced::{
    Color, Point, Vector,
    alignment::{Horizontal, Vertical},
    widget::canvas::{Frame, Path, Stroke, Text},
};
use iced_graphics::geometry::Renderer;
use regviz_core::core::automaton::StateId;

use crate::app::theme::AppTheme;

use super::{CANVAS_FONT, DrawContext, Drawable};

/// Distance between the edge segment and its label in logical units.
/// With centered text alignment, this needs to be smaller than before.
const LABEL_DISTANCE: f32 = 13.0;
/// Length of each arrow head side.
const ARROW_HEAD_LENGTH: f32 = 10.0;
/// Half-width of the arrow head at its base.
const ARROW_HEAD_HALF_WIDTH: f32 = ARROW_HEAD_LENGTH * 0.5;

const INACTIVE_EDGE_STROKE_WIDTH: f32 = 1.3;
const ACTIVE_EDGE_STROKE_WIDTH: f32 = 2.4;
const ACTIVE_ARROW_ALPHA: f32 = 0.35;

/// Edge curvature style for different types of transitions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EdgeCurve {
    /// Straight line between nodes.
    Straight,
    /// Curved downward.
    CurveDown,
    /// Curved upward (for star closure loop-back: inner_accept → inner_start).
    CurveUp,
}

/// Renderable description of a transition between two states.
#[derive(Debug, Clone)]
pub struct GraphEdge {
    /// Origin node identifier.
    pub from: StateId,
    /// Destination node identifier.
    pub to: StateId,
    /// Label displayed next to the edge.
    pub label: String,
    /// Curvature style for this edge.
    pub curve: EdgeCurve,
    /// Whether this edge was traversed in the current simulation step.
    pub is_active: bool,
}

impl GraphEdge {
    /// Creates a new [`GraphEdge`] with a straight line.
    #[must_use]
    pub fn new(from: StateId, to: StateId, label: String) -> Self {
        Self {
            from,
            to,
            label,
            curve: EdgeCurve::Straight,
            is_active: false,
        }
    }

    /// Creates a new [`GraphEdge`] with a specified curve style.
    #[must_use]
    pub fn with_curve(from: StateId, to: StateId, label: String, curve: EdgeCurve) -> Self {
        Self {
            from,
            to,
            label,
            curve,
            is_active: false,
        }
    }

    /// Marks the edge as active (or inactive) for the current simulation step.
    #[must_use]
    pub fn with_active(mut self, is_active: bool) -> Self {
        self.is_active = is_active;
        self
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
    /// Radius of the source node (used to adjust edge start point).
    pub from_radius: f32,
    /// Radius of the destination node (used to adjust edge end point).
    pub to_radius: f32,
}

impl PositionedEdge {
    /// Creates a new positioned edge from metadata and coordinates, keeping the label legible
    /// by offsetting it away from the rendered segment.
    ///
    /// Note: The `from` and `to` points represent node centers. The actual edge will be
    /// drawn from the edge of the source node to the edge of the destination node.
    #[must_use]
    pub fn new(data: GraphEdge, from: Point, to: Point) -> Self {
        let label_position = compute_label_anchor(from, to);
        Self {
            data,
            from,
            to,
            label_position,
            from_radius: 32.0, // Default node radius
            to_radius: 32.0,   // Default node radius
        }
    }

    /// Creates a new positioned edge with explicit node radii.
    ///
    /// This is useful when nodes have different radii (e.g., in different visualization modes).
    ///
    /// # Note
    ///
    /// This function is currently unused but kept for potential future use.
    #[must_use]
    #[allow(dead_code)]
    pub fn with_radii(
        data: GraphEdge,
        from: Point,
        to: Point,
        from_radius: f32,
        to_radius: f32,
    ) -> Self {
        let label_position = compute_label_anchor(from, to);
        Self {
            data,
            from,
            to,
            label_position,
            from_radius,
            to_radius,
        }
    }
}

impl Drawable for PositionedEdge {
    /// Draws a directed edge from one state to another with an arrow head and label.
    ///
    /// This function handles both straight and curved edges:
    /// - Straight edges: Direct line between two nodes
    /// - Curved edges: Quadratic Bezier curve for star closure epsilon transitions
    ///
    /// The function performs these tasks:
    /// 1. Transforms coordinates from logical to screen space
    /// 2. Determines if edge should be curved based on edge type
    /// 3. Adjusts endpoints to stop at node boundaries
    /// 4. Draws the edge path (line or curve)
    /// 5. Draws an arrow head at the destination
    /// 6. Renders the transition label
    fn draw<R: Renderer>(&self, frame: &mut Frame<R>, ctx: &DrawContext, theme: &AppTheme) {
        // Transform logical coordinates to screen coordinates based on zoom/pan
        let from_center = ctx.transform_point(self.from);
        let to_center = ctx.transform_point(self.to);

        // Calculate the direction vector from source to destination
        let direction = Vector::new(to_center.x - from_center.x, to_center.y - from_center.y);
        let length = (direction.x * direction.x + direction.y * direction.y).sqrt();

        // If nodes are at the same position, don't draw anything
        if length <= f32::EPSILON {
            return;
        }

        // Normalize direction to unit vector
        let unit = Vector::new(direction.x / length, direction.y / length);

        // Scale node radii by zoom factor
        let from_radius = self.from_radius * ctx.zoom;
        let to_radius = self.to_radius * ctx.zoom;

        let stroke_color = if self.data.is_active {
            theme.graph_edge_active()
        } else {
            theme.graph_edge_default()
        };

        let stroke_width = if self.data.is_active {
            ACTIVE_EDGE_STROKE_WIDTH
        } else {
            INACTIVE_EDGE_STROKE_WIDTH
        };

        match self.data.curve {
            EdgeCurve::Straight => {
                self.draw_straight_edge(
                    frame,
                    from_center,
                    to_center,
                    unit,
                    from_radius,
                    to_radius,
                    ctx,
                    stroke_color,
                    stroke_width,
                );
            }
            EdgeCurve::CurveDown => {
                self.draw_curved_edge(
                    frame,
                    from_center,
                    to_center,
                    from_radius,
                    to_radius,
                    true,
                    ctx,
                    stroke_color,
                    stroke_width,
                );
            }
            EdgeCurve::CurveUp => {
                self.draw_curved_edge(
                    frame,
                    from_center,
                    to_center,
                    from_radius,
                    to_radius,
                    false,
                    ctx,
                    stroke_color,
                    stroke_width,
                );
            }
        }
    }
}

impl PositionedEdge {
    /// Draws a straight edge between two nodes.
    ///
    /// # Arguments
    /// - `frame`: Canvas frame to draw on
    /// - `from_center`: Center of source node (screen coordinates)
    /// - `to_center`: Center of destination node (screen coordinates)
    /// - `unit`: Unit direction vector from source to destination
    /// - `from_radius`: Radius of source node (scaled)
    /// - `to_radius`: Radius of destination node (scaled)
    /// - `ctx`: Drawing context with zoom/pan information
    #[allow(clippy::too_many_arguments)]
    fn draw_straight_edge<R: Renderer>(
        &self,
        frame: &mut Frame<R>,
        from_center: Point,
        to_center: Point,
        unit: Vector,
        from_radius: f32,
        to_radius: f32,
        ctx: &DrawContext,
        stroke_color: Color,
        stroke_width: f32,
    ) {
        // Adjust the start point: move from node center outward by the node radius
        let from = Point::new(
            from_center.x + unit.x * from_radius,
            from_center.y + unit.y * from_radius,
        );

        // Adjust the end point: move from node center inward by the node radius
        let to = Point::new(
            to_center.x - unit.x * to_radius,
            to_center.y - unit.y * to_radius,
        );

        // Draw the main line connecting the two states
        let line = Path::line(from, to);
        frame.stroke(
            &line,
            Stroke::default()
                .with_width(stroke_width)
                .with_color(stroke_color),
        );

        // Draw arrow head at destination
        self.draw_arrow_head(frame, to, unit, stroke_color);

        // Draw label
        self.draw_label(frame, ctx, stroke_color);
    }

    /// Draws a curved edge using a quadratic Bezier curve.
    ///
    /// For star closures, we need curved arrows:
    /// - `curve_down = true`: Curve wraps below (start → inner_start bypass)
    /// - `curve_down = false`: Curve wraps above (inner_accept → inner_start loop)
    ///
    /// # Arguments
    /// - `frame`: Canvas frame to draw on
    /// - `from_center`: Center of source node (screen coordinates)
    /// - `to_center`: Center of destination node (screen coordinates)
    /// - `from_radius`: Radius of source node (scaled)
    /// - `to_radius`: Radius of destination node (scaled)
    /// - `curve_down`: If true, curve bends downward; if false, upward
    /// - `ctx`: Drawing context with zoom/pan information
    #[allow(clippy::too_many_arguments)]
    fn draw_curved_edge<R: Renderer>(
        &self,
        frame: &mut Frame<R>,
        from_center: Point,
        to_center: Point,
        from_radius: f32,
        to_radius: f32,
        curve_down: bool,
        ctx: &DrawContext,
        stroke_color: Color,
        stroke_width: f32,
    ) {
        // Calculate the perpendicular offset for the control point
        let direction = Vector::new(to_center.x - from_center.x, to_center.y - from_center.y);
        let length = (direction.x * direction.x + direction.y * direction.y).sqrt();

        if length <= f32::EPSILON {
            return;
        }

        let unit = Vector::new(direction.x / length, direction.y / length);

        // Get perpendicular vector (rotates 90° counterclockwise)
        // For horizontal edges (left to right), this gives an upward normal
        let normal = perpendicular(unit);

        // Control point offset: position it perpendicular to the line between nodes
        // Reduce the control offset to prevent curves from extending too far outside bounding boxes
        // Use a smaller factor to keep curves more contained
        let max_curve_height = length * 0.25; // Limit curve height to 25% of distance between nodes
        let base_control_offset = if curve_down {
            -max_curve_height // Negative = downward
        } else {
            max_curve_height // Positive = upward
        };

        // Midpoint between nodes
        let mid = Point::new(
            (from_center.x + to_center.x) * 0.5,
            (from_center.y + to_center.y) * 0.5,
        );

        // Control point offset perpendicular to the edge
        let control = Point::new(
            mid.x + normal.x * base_control_offset,
            mid.y + normal.y * base_control_offset,
        );

        // Find where the curve intersects the node boundaries
        // Use the tangent at t=0 for the start point and t=1 for the end point
        let start_tangent = quadratic_bezier_tangent(from_center, control, to_center, 0.0);
        let start_tangent_len =
            (start_tangent.x * start_tangent.x + start_tangent.y * start_tangent.y).sqrt();
        let start_tangent_unit = if start_tangent_len > f32::EPSILON {
            Vector::new(
                start_tangent.x / start_tangent_len,
                start_tangent.y / start_tangent_len,
            )
        } else {
            unit
        };

        let end_tangent = quadratic_bezier_tangent(from_center, control, to_center, 1.0);
        let end_tangent_len =
            (end_tangent.x * end_tangent.x + end_tangent.y * end_tangent.y).sqrt();
        let end_tangent_unit = if end_tangent_len > f32::EPSILON {
            Vector::new(
                end_tangent.x / end_tangent_len,
                end_tangent.y / end_tangent_len,
            )
        } else {
            Vector::new(-unit.x, -unit.y)
        };

        // Start point on the edge of the source node
        let start = Point::new(
            from_center.x + start_tangent_unit.x * from_radius,
            from_center.y + start_tangent_unit.y * from_radius,
        );

        // End point on the edge of the destination node (where arrow tip will be)
        let end = Point::new(
            to_center.x - end_tangent_unit.x * to_radius,
            to_center.y - end_tangent_unit.y * to_radius,
        );

        // Draw the quadratic Bezier curve
        let curve_path = Path::new(|builder| {
            builder.move_to(start);
            builder.quadratic_curve_to(control, end);
        });
        frame.stroke(
            &curve_path,
            Stroke::default()
                .with_width(stroke_width)
                .with_color(stroke_color),
        );

        // Draw arrow head at the end point with the correct tangent direction
        self.draw_arrow_head(frame, end, end_tangent_unit, stroke_color);

        // Draw label on the curve - calculate position based on curve midpoint
        self.draw_curved_label(
            frame,
            from_center,
            control,
            to_center,
            curve_down,
            ctx,
            stroke_color,
        );
    }

    /// Draws an arrow head at the specified point, oriented in the given direction.
    ///
    /// # Arguments
    /// - `frame`: Canvas frame to draw on
    /// - `tip`: Point where the arrow tip should be
    /// - `direction`: Unit vector indicating the direction the arrow points
    fn draw_arrow_head<R: Renderer>(
        &self,
        frame: &mut Frame<R>,
        tip: Point,
        direction: Vector,
        color: Color,
    ) {
        let normal = perpendicular(direction);

        // Left wing: move back along the direction, then offset perpendicular
        let left = Point::new(
            tip.x - direction.x * ARROW_HEAD_LENGTH + normal.x * ARROW_HEAD_HALF_WIDTH,
            tip.y - direction.y * ARROW_HEAD_LENGTH + normal.y * ARROW_HEAD_HALF_WIDTH,
        );

        // Right wing: move back along the direction, then offset opposite perpendicular
        let right = Point::new(
            tip.x - direction.x * ARROW_HEAD_LENGTH - normal.x * ARROW_HEAD_HALF_WIDTH,
            tip.y - direction.y * ARROW_HEAD_LENGTH - normal.y * ARROW_HEAD_HALF_WIDTH,
        );

        // Create a filled triangle for the arrow head
        let arrow_head = Path::new(|builder| {
            builder.move_to(tip);
            builder.line_to(left);
            builder.line_to(right);
            builder.close();
        });
        frame.fill(
            &arrow_head,
            Color::from_rgba(color.r, color.g, color.b, ACTIVE_ARROW_ALPHA),
        );
        frame.stroke(
            &arrow_head,
            Stroke::default().with_width(1.0).with_color(color),
        );
    }

    /// Draws the edge label at the pre-calculated label position.
    ///
    /// # Arguments
    /// - `frame`: Canvas frame to draw on
    /// - `ctx`: Drawing context with zoom/pan information
    fn draw_label<R: Renderer>(&self, frame: &mut Frame<R>, ctx: &DrawContext, color: Color) {
        if self.data.label.is_empty() {
            return;
        }

        let label_pos = ctx.transform_point(self.label_position);
        frame.fill_text(Text {
            content: self.data.label.clone(),
            position: label_pos,
            color,
            font: CANVAS_FONT,
            align_x: Horizontal::Center.into(),
            align_y: Vertical::Center,
            ..Text::default()
        });
    }

    /// Draws the label for a curved edge at the curve's midpoint.
    ///
    /// The label is positioned above the curve for upward curves and below for downward curves.
    ///
    /// # Arguments
    /// - `frame`: Canvas frame to draw on
    /// - `p0`: Start point of the curve (screen coordinates)
    /// - `p1`: Control point of the curve (screen coordinates)
    /// - `p2`: End point of the curve (screen coordinates)
    /// - `curve_down`: Whether the curve bends downward
    /// - `ctx`: Drawing context with zoom/pan information (used for zoom scaling)
    #[allow(clippy::too_many_arguments)]
    fn draw_curved_label<R: Renderer>(
        &self,
        frame: &mut Frame<R>,
        p0: Point,
        p1: Point,
        p2: Point,
        curve_down: bool,
        ctx: &DrawContext,
        color: Color,
    ) {
        if self.data.label.is_empty() {
            return;
        }

        // Calculate the point on the curve at t=0.5 (midpoint)
        // All points are already in screen coordinates
        let mid_point = quadratic_bezier_point(p0, p1, p2, 0.5);

        // Get the tangent at the midpoint to determine the perpendicular direction
        let tangent = quadratic_bezier_tangent(p0, p1, p2, 0.5);
        let tangent_len = (tangent.x * tangent.x + tangent.y * tangent.y).sqrt();

        if tangent_len <= f32::EPSILON {
            // Fallback to regular label positioning if tangent is degenerate
            self.draw_label(frame, ctx, color);
            return;
        }

        let tangent_unit = Vector::new(tangent.x / tangent_len, tangent.y / tangent_len);
        let normal = perpendicular(tangent_unit);

        // Position label on the outer (convex) side of the curve
        let scaled_label_distance = LABEL_DISTANCE * ctx.zoom;
        let label_offset = if curve_down {
            -scaled_label_distance // Control point is down, label goes up (negative y)
        } else {
            scaled_label_distance // Control point is up, label goes down (positive y)
        };

        // Apply the offset in the normal direction
        // No need to transform - we're already in screen coordinates
        let label_position = Point::new(
            mid_point.x + normal.x * label_offset,
            mid_point.y + normal.y * label_offset,
        );

        frame.fill_text(Text {
            content: self.data.label.clone(),
            position: label_position,
            color,
            font: CANVAS_FONT,
            align_x: Horizontal::Center.into(),
            align_y: Vertical::Center,
            ..Text::default()
        });
    }
}

/// Calculates where to place the label text for an edge connecting two points.
///
/// The label is positioned at the midpoint of the edge, offset perpendicular to the
/// edge direction. The offset is always placed "above" the edge (negative y direction)
/// to keep labels readable and consistent.
///
/// # Algorithm
/// 1. Find the midpoint between the two endpoints
/// 2. Calculate the perpendicular (normal) vector to the edge direction
/// 3. Flip the normal if it points downward (so labels always appear above)
/// 4. Offset the midpoint by LABEL_DISTANCE in the normal direction
fn compute_label_anchor(from: Point, to: Point) -> Point {
    // Calculate the midpoint of the edge
    let mid = Point::new((from.x + to.x) * 0.5, (from.y + to.y) * 0.5);

    // Calculate the vector from 'from' to 'to'
    let direction = Vector::new(to.x - from.x, to.y - from.y);
    let length = (direction.x * direction.x + direction.y * direction.y).sqrt();

    // If the two points are basically the same, just place label above
    if length <= f32::EPSILON {
        return Point::new(mid.x, mid.y - LABEL_DISTANCE);
    }

    // Get the perpendicular (normal) vector - rotated 90° from the edge direction
    let mut normal = perpendicular(Vector::new(direction.x / length, direction.y / length));

    // Flip the normal if it points downward (positive y) so labels always appear above
    if normal.y > 0.0 {
        normal = Vector::new(-normal.x, -normal.y);
    }

    // Offset the midpoint by the normal vector scaled by LABEL_DISTANCE
    Point::new(
        mid.x + normal.x * LABEL_DISTANCE,
        mid.y + normal.y * LABEL_DISTANCE,
    )
}

/// Returns a vector perpendicular (at 90°) to the input vector.
///
/// This is a 2D rotation by 90° counterclockwise:
/// - If input is (x, y), output is (-y, x)
/// - Example: (1, 0) becomes (0, 1), which is 90° counterclockwise
/// - Example: (0, 1) becomes (-1, 0), which is 90° counterclockwise
///
/// This is useful for calculating normals to edges (for arrow heads and label placement).
fn perpendicular(vector: Vector) -> Vector {
    Vector::new(-vector.y, vector.x)
}

/// Computes the tangent vector of a quadratic Bezier curve at parameter t.
///
/// The tangent represents the direction of the curve at a given point,
/// which is useful for orienting arrow heads and labels.
///
/// The tangent is the derivative: B'(t) = 2(1-t)(p1-p0) + 2t(p2-p1)
///
/// # Arguments
/// - `p0`: Start point of the curve
/// - `p1`: Control point
/// - `p2`: End point of the curve
/// - `t`: Parameter in range [0, 1]
///
/// # Returns
/// The tangent vector at parameter t (not normalized)
fn quadratic_bezier_tangent(p0: Point, p1: Point, p2: Point, t: f32) -> Vector {
    let mt = 1.0 - t;

    Vector::new(
        2.0 * mt * (p1.x - p0.x) + 2.0 * t * (p2.x - p1.x),
        2.0 * mt * (p1.y - p0.y) + 2.0 * t * (p2.y - p1.y),
    )
}

/// Computes a point on a quadratic Bezier curve at parameter t.
///
/// Uses the quadratic Bezier formula: B(t) = (1-t)²p0 + 2(1-t)t*p1 + t²p2
///
/// # Arguments
/// - `p0`: Start point of the curve
/// - `p1`: Control point
/// - `p2`: End point of the curve
/// - `t`: Parameter in range [0, 1]
///
/// # Returns
/// The point on the curve at parameter t
fn quadratic_bezier_point(p0: Point, p1: Point, p2: Point, t: f32) -> Point {
    let mt = 1.0 - t;
    let mt2 = mt * mt;
    let t2 = t * t;

    Point::new(
        mt2 * p0.x + 2.0 * mt * t * p1.x + t2 * p2.x,
        mt2 * p0.y + 2.0 * mt * t * p1.y + t2 * p2.y,
    )
}
