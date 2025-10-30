use crate::app::simulation::SimulationTarget;

/// Messages emitted by the simulation control panel.
#[derive(Debug, Clone)]
pub enum SimulationMessage {
    /// User modified the simulation input string.
    InputChanged(String),
    /// Advance to the next simulation step (if available).
    StepForward,
    /// Move back to the previous simulation step (if available).
    StepBackward,
    /// Reset the simulation to the initial step.
    Reset,
    /// User selected a different automaton to simulate (NFA or DFA).
    TargetChanged(SimulationTarget),
}
