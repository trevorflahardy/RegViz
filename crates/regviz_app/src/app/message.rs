use regviz_core::core::automaton::BoxKind;

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
