mod input;
mod pane_grid;
mod simulation;
mod view_controls;

pub use input::InputMessage;
pub use pane_grid::PaneGridMessage;
pub use simulation::SimulationMessage;
pub use view_controls::{RightPaneMode, ViewMessage, ViewMode};

/// Aggregated application messages routed through the update loop.
#[derive(Debug, Clone)]
pub enum Message {
    /// Regex input field events.
    Input(InputMessage),
    /// Simulation control events.
    Simulation(SimulationMessage),
    /// Canvas/view configuration events.
    View(ViewMessage),
    /// PaneGrid drag/resize events
    PaneGrid(PaneGridMessage),
}
