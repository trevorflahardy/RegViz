use iced::widget::pane_grid::{self, Axis};
use iced::{Point, Vector};
use regviz_core::core::BuildArtifacts;

use super::constants::DEFAULT_ZOOM_FACTOR;
use super::message::ViewMode;
use super::simulation::SimulationState;
use crate::app::theme::AppTheme;
use crate::graph::BoxVisibility;

const PANEL_SPLIT_RATIO: f32 = 0.35;

/// Identifiers for content in each pane of the `PaneGrid`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneContent {
    Controls,
    Visualization,
}

/// Main application state.
pub struct App {
    /// Current regex input from the user.
    pub input: String,

    /// Error message from lexing or parsing, if any.
    pub error: Option<String>,

    /// Successfully built AST, NFA, and alphabet, if available.
    pub build_artifacts: Option<BuildArtifacts>,

    /// Controls which bounding boxes are visible in NFA view.
    pub box_visibility: BoxVisibility,

    /// Current zoom level for visualizations (1.0 = fit to screen).
    pub zoom_factor: f32,

    /// Currently active visualization mode.
    pub view_mode: ViewMode,

    /// Interactive simulation state for stepping through input strings.
    pub simulation: SimulationState,

    /// Validation error for the simulation input, if any.
    pub simulation_error: Option<String>,

    /// Pane grid state for left (controls) and right (visualization) panes.
    pub panes: pane_grid::State<PaneContent>,

    pub(crate) theme: AppTheme,

    /// Pan offset for dragging the canvas.
    pub pan_offset: Vector,

    /// Whether the canvas is currently being dragged.
    pub dragging: bool,

    /// Last cursor position during drag operation.
    pub last_cursor_position: Option<Point>,
}

impl Default for App {
    fn default() -> Self {
        // Initialize two-pane layout: left controls | right visualization
        let (mut panes, left) = pane_grid::State::new(PaneContent::Controls);
        let (_pane, split) = panes
            .split(Axis::Vertical, left, PaneContent::Visualization)
            .expect("split pane should succeed");

        panes.resize(split, PANEL_SPLIT_RATIO);

        Self {
            input: String::new(),
            error: None,
            build_artifacts: None,
            box_visibility: BoxVisibility::minimized(),
            zoom_factor: DEFAULT_ZOOM_FACTOR,
            view_mode: ViewMode::Nfa, // Default to NFA view
            simulation: SimulationState::default(),
            simulation_error: None,
            panes,
            theme: AppTheme::Dark,
            pan_offset: Vector::ZERO,
            dragging: false,
            last_cursor_position: None,
        }
    }
}
