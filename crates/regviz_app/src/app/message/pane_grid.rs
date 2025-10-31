use iced::widget::pane_grid;

/// Messages related to PaneGrid interactions.
#[derive(Debug, Clone)]
pub enum PaneGridMessage {
    Resized(pane_grid::ResizeEvent),
}
