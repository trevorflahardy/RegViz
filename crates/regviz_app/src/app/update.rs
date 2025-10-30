use super::constants::{MAX_ZOOM_FACTOR, MIN_ZOOM_FACTOR};
use super::message::{InputMessage, Message, SimulationMessage, ViewMessage, ViewMode};
use super::simulation::{SimulationTarget, build_dfa_trace, build_nfa_trace};
use super::state::App;
use regviz_core::core::dfa;

impl App {
    /// Handles incoming messages and updates application state accordingly.
    ///
    /// This is the main state transition function. It processes user actions
    /// and updates the app state in response.
    pub fn update(&mut self, message: Message) {
        match message {
            Message::Input(input_msg) => match input_msg {
                InputMessage::Changed(value) => self.handle_input_changed(value),
            },
            Message::Simulation(sim_msg) => match sim_msg {
                SimulationMessage::InputChanged(value) => {
                    self.handle_simulation_input_changed(value)
                }
                SimulationMessage::StepForward => self.handle_simulation_step_forward(),
                SimulationMessage::StepBackward => self.handle_simulation_step_backward(),
                SimulationMessage::Reset => self.handle_simulation_reset(),
                SimulationMessage::TargetChanged(target) => {
                    self.handle_simulation_target_changed(target)
                }
            },
            Message::View(view_msg) => match view_msg {
                ViewMessage::ToggleBox(kind) => self.handle_toggle_box(kind),
                ViewMessage::ZoomChanged(value) => self.handle_zoom_changed(value),
                ViewMessage::ViewModeChanged(mode) => self.handle_view_mode_changed(mode),
            },
        }
    }

    /// Updates the input text and re-parses the regex.
    fn handle_input_changed(&mut self, input: String) {
        self.input = input;
        self.lex_and_parse();
    }

    /// Toggles visibility of a specific bounding box type in the NFA view.
    fn handle_toggle_box(&mut self, kind: regviz_core::core::automaton::BoxKind) {
        self.box_visibility.toggle(kind);
    }

    /// Updates the zoom factor, clamping it to valid range.
    fn handle_zoom_changed(&mut self, value: f32) {
        self.zoom_factor = value.clamp(MIN_ZOOM_FACTOR, MAX_ZOOM_FACTOR);
    }

    /// Switches between AST and NFA visualization modes.
    fn handle_view_mode_changed(&mut self, mode: ViewMode) {
        self.view_mode = mode;
    }

    /// Updates the simulation input string and rebuilds the trace.
    fn handle_simulation_input_changed(&mut self, input: String) {
        self.simulation.input = input;
        self.simulation.reset_cursor();
        self.rebuild_simulation_trace();
    }

    /// Steps the simulation forward when possible.
    fn handle_simulation_step_forward(&mut self) {
        self.simulation.step_forward();
    }

    /// Steps the simulation backward when possible.
    fn handle_simulation_step_backward(&mut self) {
        self.simulation.step_backward();
    }

    /// Resets the simulation to the initial state.
    fn handle_simulation_reset(&mut self) {
        self.simulation.reset_cursor();
    }

    /// Switches between NFA and DFA simulation modes.
    fn handle_simulation_target_changed(&mut self, target: SimulationTarget) {
        if self.simulation.target == target {
            return;
        }

        self.simulation.target = target;
        self.simulation.reset_cursor();
        self.rebuild_simulation_trace();
    }

    /// Recomputes the simulation trace for the current target automaton.
    pub(crate) fn rebuild_simulation_trace(&mut self) {
        let Some(artifacts) = self.build_artifacts.as_mut() else {
            self.simulation.clear_trace();
            return;
        };

        let input = self.simulation.input.as_str();

        match self.simulation.target {
            SimulationTarget::Nfa => {
                let trace = build_nfa_trace(&artifacts.nfa, input);
                self.simulation.set_trace(Some(trace));
            }
            SimulationTarget::Dfa => {
                if artifacts.dfa.is_none() {
                    let (dfa, alphabet) = dfa::determinize(&artifacts.nfa);
                    artifacts.dfa = Some(dfa);
                    artifacts.dfa_alphabet = Some(alphabet);
                }

                let Some(dfa) = artifacts.dfa.as_ref() else {
                    self.simulation.clear_trace();
                    return;
                };
                let Some(alphabet) = artifacts.dfa_alphabet.as_ref() else {
                    self.simulation.clear_trace();
                    return;
                };

                let trace = build_dfa_trace(dfa, alphabet, input);
                self.simulation.set_trace(Some(trace));
            }
        }
    }
}
