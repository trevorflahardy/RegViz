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
    /// Manual per-node positions for the AST view. Keys are numeric node ids
    /// assigned when converting the AST to a graph.
    pub pinned_positions_ast: HashMap<u32, iced::Point>,
    /// Manual per-node positions for the NFA visualization (by state id).
    pub pinned_positions_nfa: HashMap<StateId, iced::Point>,
    /// Manual per-node positions for the DFA visualization (by state id).
    pub pinned_positions_dfa: HashMap<StateId, iced::Point>,
    /// Manual per-node positions for the Minimized DFA visualization (by state id).
    pub pinned_positions_min_dfa: HashMap<StateId, iced::Point>,
    /// If the user is currently dragging a node, this holds its StateId.
    pub node_dragging: Option<StateId>,
    /// Last cursor position during node drag (layout coordinates).
    pub last_node_cursor_position: Option<Point>,
    /// Selected node (single-click selection), if any.
    pub selected_node: Option<StateId>,
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
            pinned_positions_ast: HashMap::new(),
            pinned_positions_nfa: HashMap::new(),
            pinned_positions_dfa: HashMap::new(),
            pinned_positions_min_dfa: HashMap::new(),
            node_dragging: None,
            last_node_cursor_position: None,
            selected_node: None,
        }
    }
}
