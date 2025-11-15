use iced::widget::pane_grid::{self, Axis};
use iced::{Point, Vector};
use regviz_core::core::automaton::StateId;
use regviz_core::{core::BuildArtifacts, errors::BuildError};
use std::collections::HashMap;

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

/// Per-viewmode data for panning, zooming, and pinned node positions.
pub struct ViewData {
    /// Pan offset for dragging the canvas.
    pub pan_offset: Vector,
    /// Current zoom level for visualizations (1.0 = fit to screen).
    pub zoom_factor: f32,
    /// Manual per-node positions for the view
    pub pinned_node_positions: HashMap<StateId, iced::Point>,
}

impl Default for ViewData {
    fn default() -> Self {
        Self {
            pan_offset: Vector::ZERO,
            zoom_factor: DEFAULT_ZOOM_FACTOR,
            pinned_node_positions: HashMap::new(),
        }
    }
}

pub struct ViewState {
    /// Currently active visualization mode.
    pub mode: ViewMode,
    /// Per-viewmode data.
    data: [ViewData; 4], // One for each ViewMode
}

impl Default for ViewState {
    fn default() -> Self {
        Self {
            mode: ViewMode::Nfa,
            data: [
                ViewData::default(),
                ViewData::default(),
                ViewData::default(),
                ViewData::default(),
            ],
        }
    }
}

impl ViewState {
    /// Gets the index in the data array corresponding to the current view mode.
    fn index(&self) -> usize {
        match self.mode {
            ViewMode::Ast => 0,
            ViewMode::Nfa => 1,
            ViewMode::Dfa => 2,
            ViewMode::MinDfa => 3,
        }
    }

    /// Gets a mutable reference to the current view's data.
    pub fn data_mut(&mut self) -> &mut ViewData {
        &mut self.data[self.index()]
    }

    /// Gets an immutable reference to the current view's data.
    pub fn data(&self) -> &ViewData {
        &self.data[self.index()]
    }
}

/// Main application state.
pub struct App {
    /// Current regex input from the user.
    pub input: String,

    /// Error from lexing or parsing, if any.
    pub error: Option<BuildError>,

    /// Successfully built AST, NFA, and alphabet, if available.
    pub build_artifacts: Option<BuildArtifacts>,

    /// Controls which bounding boxes are visible in NFA view.
    pub box_visibility: BoxVisibility,

    /// Interactive simulation state for stepping through input strings.
    pub simulation: SimulationState,

    /// Validation error for the simulation input, if any.
    pub simulation_error: Option<String>,

    /// Pane grid state for left (controls) and right (visualization) panes.
    pub panes: pane_grid::State<PaneContent>,

    pub(crate) theme: AppTheme,

    pub view_state: ViewState,

    /// Last cursor position during panning operation.
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
            simulation: SimulationState::default(),
            simulation_error: None,
            panes,
            theme: AppTheme::Dark,
            view_state: ViewState::default(),
            last_cursor_position: None,
        }
    }
}

impl App {
    /// Gets an immutable reference to the current view's data.
    pub fn view_data(&self) -> &ViewData {
        self.view_state.data()
    }

    /// Gets a mutable reference to the current view's data.
    pub fn view_data_mut(&mut self) -> &mut ViewData {
        self.view_state.data_mut()
    }

    /// Returns the current view mode.
    pub fn view_mode(&self) -> ViewMode {
        self.view_state.mode
    }

    /// Sets the current view mode.
    pub fn set_view_mode(&mut self, view_mode: ViewMode) {
        self.view_state.mode = view_mode;
    }
}
