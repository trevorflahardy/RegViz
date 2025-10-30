use regviz_core::core::automaton::BoxKind;

/// Messages emitted by view and canvas controls.
#[derive(Debug, Clone)]
pub enum ViewMessage {
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
