use iced::Point;
use regviz_core::core::automaton::BoxKind;

/// Messages emitted by view and canvas controls.
#[derive(Debug, Clone)]
pub enum ViewMessage {
    /// User toggled visibility of a specific bounding box type (NFA only).
    ToggleBox(BoxKind),
    /// User adjusted the zoom slider.
    ZoomChanged(f32),
    /// User scrolled mouse wheel to zoom (positive = zoom in, negative = zoom out).
    Zoom(f32),
    /// Combined selection for bottom-right controls (NFA / DFA / AST).
    SelectRightPaneMode(RightPaneMode),
    /// User started panning the canvas.
    StartPan(Point),
    /// User is panning the canvas.
    Pan(Point),
    /// User stopped panning the canvas.
    EndPan,
    /// User clicked reset view button to center and restore default zoom.
    ResetView,
}

/// Available visualization modes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewMode {
    /// Show the Abstract Syntax Tree.
    Ast,
    /// Show the Non-deterministic Finite Automaton.
    Nfa,
}

impl ViewMode {}

/// Bottom-right tri-toggle options (unifies AST view and NFA/DFA targets).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RightPaneMode {
    Ast,
    Nfa,
    Dfa,
}
