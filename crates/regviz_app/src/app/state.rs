use iced::widget::pane_grid::{self, Axis};
use regviz_core::core::BuildArtifacts;

use super::constants::DEFAULT_ZOOM_FACTOR;
use super::message::ViewMode;
use super::simulation::SimulationState;
use crate::graph::BoxVisibility;

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
}

impl Default for App {
    fn default() -> Self {
        // Initialize two-pane layout: left controls | right visualization
        let (mut panes, left) = pane_grid::State::new(PaneContent::Controls);
        let (_pane, split) = panes
            .split(Axis::Vertical, left, PaneContent::Visualization)
            .expect("split pane should succeed");

        panes.resize(split, 0.3);

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
        }
    }
}

/// Identifiers for content in each pane of the `PaneGrid`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneContent {
    Controls,
    Visualization,
}
