use regviz_core::core::automaton::BoxKind;

use super::simulation::SimulationTarget;

/// Messages that can be sent to update the application state.
#[derive(Debug, Clone)]
pub enum Message {
    /// User changed the regex input text.
    InputChanged(String),

    /// User toggled visibility of a specific bounding box type (NFA only).
    ToggleBox(BoxKind),

    /// User adjusted the zoom slider.
    ZoomChanged(f32),

    /// User switched between visualization screens.
    ViewModeChanged(ViewMode),

    /// User modified the simulation input string.
    SimulationInputChanged(String),

    /// Advance to the next simulation step (if available).
    SimulationStepForward,

    /// Move back to the previous simulation step (if available).
    SimulationStepBackward,

    /// Reset the simulation to the initial step.
    SimulationReset,

    /// User selected a different automaton to simulate (NFA or DFA).
    SimulationTargetChanged(SimulationTarget),
}

/// Available visualization modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    /// Show the Abstract Syntax Tree.
    Ast,

    /// Show the Non-deterministic Finite Automaton.
    Nfa,
}

impl ViewMode {
    /// Returns a human-readable label for this view mode.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Ast => "Parse Tree",
            Self::Nfa => "NFA",
        }
    }
}
