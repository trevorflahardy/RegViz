use iced::{
    Element, Program, Task,
    advanced::graphics::core::window::{Id as WindowId, Settings as WindowSettings},
    widget::pane_grid::{self, Axis},
};
use regviz_core::core::BuildArtifacts;

use super::constants::DEFAULT_ZOOM_FACTOR;
use super::message::ViewMode;
use super::simulation::SimulationState;
use crate::app::message::Message;
use crate::app::theme::AppTheme;
use crate::graph::BoxVisibility;

const PANEL_SPLIT_RATIO: f32 = 0.4;

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
        }
    }
}

impl Program for App {
    type State = Self;
    type Message = Message;
    type Theme = AppTheme;
    type Renderer = iced::Renderer;
    type Executor = iced::executor::Default;

    fn name() -> &'static str {
        "RegViz - Regex Visualizer"
    }

    fn settings(&self) -> iced::Settings {
        iced::Settings::default()
    }

    fn window(&self) -> Option<WindowSettings> {
        Some(WindowSettings {
            maximized: true,
            resizable: true,
            decorations: false,
            transparent: true,
            ..Default::default()
        })
    }

    /// Responsible for initializing the application state and any startup tasks.
    /// Currently, no startup tasks are needed.
    fn boot(&self) -> (Self::State, Task<Self::Message>) {
        (Self::State::default(), Task::none())
    }

    fn update(&self, state: &mut Self::State, message: Self::Message) -> Task<Self::Message> {
        Self::State::update(state, message)
    }

    fn view<'a>(
        &self,
        state: &'a Self::State,
        _window: WindowId,
    ) -> Element<'a, Self::Message, Self::Theme, Self::Renderer> {
        Self::State::view(state)
    }
}
