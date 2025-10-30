mod input;
mod simulation;
mod view_controls;

pub use input::InputMessage;
pub use simulation::SimulationMessage;
pub use view_controls::{ViewMessage, ViewMode};

/// Aggregated application messages routed through the update loop.
#[derive(Debug, Clone)]
pub enum Message {
    /// Regex input field events.
    Input(InputMessage),
    /// Simulation control events.
    Simulation(SimulationMessage),
    /// Canvas/view configuration events.
    View(ViewMessage),
}
