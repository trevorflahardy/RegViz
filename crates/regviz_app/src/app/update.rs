use std::collections::BTreeSet;

use super::constants::{MAX_ZOOM_FACTOR, MIN_ZOOM_FACTOR, ZOOM_STEP};
use super::message::{
    InputMessage, Message, PaneGridMessage, RightPaneMode, SimulationMessage, ViewMessage, ViewMode,
};
use super::simulation::{SimulationTarget, build_dfa_trace, build_nfa_trace};
use super::state::App;
use iced::{Point, Task, Vector};
use regviz_core::core::{dfa, min};

impl App {
    /// Handles incoming messages and updates application state accordingly.
    ///
    /// This is the main state transition function. It processes user actions
    /// and updates the app state in response.
    pub fn update(&mut self, message: Message) -> Task<Message> {
        match message {
            Message::Input(input_msg) => match input_msg {
                InputMessage::Changed(value) => {
                    self.handle_input_changed(value);
                    ().into()
                }
            },
            Message::Simulation(sim_msg) => match sim_msg {
                SimulationMessage::InputChanged(value) => {
                    self.handle_simulation_input_changed(value);
                    ().into()
                }
                SimulationMessage::StepForward => {
                    self.handle_simulation_step_forward();
                    ().into()
                }
                SimulationMessage::StepBackward => {
                    self.handle_simulation_step_backward();
                    ().into()
                }
                SimulationMessage::Reset => {
                    self.handle_simulation_reset();
                    ().into()
                } // Target switching handled via ViewMessage::SelectRightPaneMode
            },
            Message::View(view_msg) => match view_msg {
                ViewMessage::ToggleBox(kind) => {
                    self.handle_toggle_box(kind);
                    ().into()
                }
                ViewMessage::ZoomChanged(value) => {
                    self.handle_zoom_changed(value);
                    ().into()
                }
                ViewMessage::Zoom(delta) => {
                    self.handle_zoom(delta);
                    ().into()
                }
                ViewMessage::SelectRightPaneMode(mode) => {
                    self.handle_right_pane_mode(mode);
                    ().into()
                }
                ViewMessage::StartPan(position) => {
                    self.handle_start_pan(position);
                    ().into()
                }
                ViewMessage::NodeDragStart(id, position) => {
                    // Start dragging immediately and persist the initial
                    // manual position so click-and-drag works without a
                    // second click.
                    self.node_dragging = Some(id);
                    self.last_node_cursor_position = Some(position);
                    self.selected_node = Some(id);

                    // Mark the node as pinned by inserting initial position
                    // into the appropriate per-view pinned map.
                    if self.view_mode == ViewMode::Ast {
                        self.pinned_positions_ast.insert(id, position);
                    } else {
                        match self.simulation.target {
                            SimulationTarget::Nfa => {
                                self.pinned_positions_nfa.insert(id, position);
                            }
                            SimulationTarget::Dfa => {
                                self.pinned_positions_dfa.insert(id, position);
                            }
                            SimulationTarget::MinDfa => {
                                self.pinned_positions_min_dfa.insert(id, position);
                            }
                        }
                    }

                    ().into()
                }
                ViewMessage::NodeDrag(id, position) => {
                    if self.node_dragging == Some(id) {
                        // Update the appropriate pinned map depending on the
                        // current view (AST vs automaton) and simulation target.
                        if self.view_mode == ViewMode::Ast {
                            self.pinned_positions_ast.insert(id, position);
                        } else {
                            match self.simulation.target {
                                SimulationTarget::Nfa => {
                                    self.pinned_positions_nfa.insert(id, position);
                                }
                                SimulationTarget::Dfa => {
                                    self.pinned_positions_dfa.insert(id, position);
                                }
                                SimulationTarget::MinDfa => {
                                    self.pinned_positions_min_dfa.insert(id, position);
                                }
                            }
                        }
                        self.last_node_cursor_position = Some(position);
                    }
                    ().into()
                }
                ViewMessage::NodeDragEnd(id, position) => {
                    if self.node_dragging == Some(id) {
                        if self.view_mode == ViewMode::Ast {
                            self.pinned_positions_ast.insert(id, position);
                        } else {
                            match self.simulation.target {
                                SimulationTarget::Nfa => {
                                    self.pinned_positions_nfa.insert(id, position);
                                }
                                SimulationTarget::Dfa => {
                                    self.pinned_positions_dfa.insert(id, position);
                                }
                                SimulationTarget::MinDfa => {
                                    self.pinned_positions_min_dfa.insert(id, position);
                                }
                            }
                        }
                        self.node_dragging = None;
                        self.last_node_cursor_position = None;
                    }
                    ().into()
                }
                ViewMessage::Pan(position) => {
                    self.handle_pan(position);
                    ().into()
                }
                ViewMessage::EndPan => {
                    self.handle_end_pan();
                    ().into()
                }
                ViewMessage::ResetView => {
                    self.handle_reset_view();
                    ().into()
                }
            },
            Message::PaneGrid(event) => match event {
                PaneGridMessage::Resized(event) => {
                    self.panes.resize(event.split, event.ratio);
                    ().into()
                }
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

    /// Handles mouse wheel scroll zoom (delta > 0 = zoom in, delta < 0 = zoom out).
    fn handle_zoom(&mut self, delta: f32) {
        // Each scroll "tick" adjusts zoom by 10%
        let zoom_change = delta * ZOOM_STEP;
        let new_zoom = self.zoom_factor + zoom_change;
        self.zoom_factor = new_zoom.clamp(MIN_ZOOM_FACTOR, MAX_ZOOM_FACTOR);
    }

    /// Handles the combined bottom-right selection buttons.
    fn handle_right_pane_mode(&mut self, mode: RightPaneMode) {
        match mode {
            RightPaneMode::Ast => {
                self.view_mode = ViewMode::Ast;
            }
            RightPaneMode::Nfa => {
                self.view_mode = ViewMode::Nfa;
                self.handle_simulation_target_changed(SimulationTarget::Nfa);
            }
            RightPaneMode::Dfa => {
                self.view_mode = ViewMode::Nfa;
                self.handle_simulation_target_changed(SimulationTarget::Dfa);
            }
            RightPaneMode::MinDfa => {
                self.view_mode = ViewMode::Nfa;
                self.handle_simulation_target_changed(SimulationTarget::MinDfa);
            }
        }
    }

    /// Updates the simulation input string and rebuilds the trace.
    fn handle_simulation_input_changed(&mut self, input: String) {
        if self.build_artifacts.is_none() {
            return;
        }

        self.simulation.input = input;
        self.simulation.reset_cursor();
        self.refresh_simulation_trace();
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
        self.refresh_simulation_trace();
    }

    /// Switches between NFA and DFA simulation modes.
    fn handle_simulation_target_changed(&mut self, target: SimulationTarget) {
        if self.simulation.target == target {
            return;
        }

        self.simulation.target = target;
        self.simulation.reset_cursor();
        self.refresh_simulation_trace();
    }

    /// Validates the simulation input and rebuilds the trace when valid.
    pub(crate) fn refresh_simulation_trace(&mut self) {
        if let Some(error) = self.validate_simulation_input() {
            self.simulation_error = Some(error);
            self.simulation.clear_trace();
            return;
        }

        self.simulation_error = None;
        self.rebuild_simulation_trace();
    }

    /// Returns an error if the simulation input uses symbols outside the regex alphabet.
    fn validate_simulation_input(&self) -> Option<String> {
        let Some(artifacts) = &self.build_artifacts else {
            return None;
        };

        if self.simulation.input.is_empty() {
            return None;
        }

        let alphabet: BTreeSet<char> = artifacts.alphabet.iter().copied().collect();
        let invalid: BTreeSet<char> = self
            .simulation
            .input
            .chars()
            .filter(|symbol| !alphabet.contains(symbol))
            .collect();

        if invalid.is_empty() {
            None
        } else {
            let symbols = invalid
                .iter()
                .map(|symbol| format!("'{symbol}'"))
                .collect::<Vec<_>>()
                .join(", ");

            Some(format!(
                "Input contains symbol(s) outside the regex alphabet: {symbols}"
            ))
        }
    }

    /// Recomputes the simulation trace for the current target automaton.
    pub(crate) fn rebuild_simulation_trace(&mut self) {
        if self.simulation_error.is_some() {
            return;
        }

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
                // Ensure the determinized DFA exists
                let taken_dfa = match artifacts.dfa.take() {
                    Some(dfa_ref) => dfa_ref,
                    None => dfa::determinize(&artifacts.nfa),
                };

                let trace = build_dfa_trace(&taken_dfa, &artifacts.alphabet, input);
                // Store back the taken DFA
                artifacts.dfa = Some(taken_dfa);
                self.simulation.set_trace(Some(trace));
            }
            SimulationTarget::MinDfa => {
                // Ensure the minimized DFA exists. This may require determinization first.

                let (taken_min_dfa, taken_dfa) =
                    match (artifacts.min_dfa.take(), artifacts.dfa.take()) {
                        (Some(min_dfa), Some(dfa)) => (min_dfa, dfa),
                        (Some(_), None) => {
                            // dfa is missing, compute from nfa
                            let dfa = dfa::determinize(&artifacts.nfa);
                            // compute min_dfa from dfa to ensure consistency
                            let min_dfa = min::minimize(&dfa);
                            (min_dfa, dfa)
                        }
                        (None, Some(dfa)) => {
                            // min_dfa is missing, compute from dfa
                            let min_dfa = min::minimize(&dfa);
                            (min_dfa, dfa)
                        }
                        (None, None) => {
                            // both missing, compute dfa from nfa, then min_dfa
                            let dfa = dfa::determinize(&artifacts.nfa);
                            let min_dfa = min::minimize(&dfa);
                            (min_dfa, dfa)
                        }
                    };
                let trace = build_dfa_trace(&taken_min_dfa, &artifacts.alphabet, input);
                // Store back the taken DFAs
                artifacts.dfa = Some(taken_dfa);
                artifacts.min_dfa = Some(taken_min_dfa);
                self.simulation.set_trace(Some(trace));
            }
        }
    }

    /// Starts a pan operation at the given cursor position.
    fn handle_start_pan(&mut self, position: Point) {
        self.dragging = true;
        self.last_cursor_position = Some(position);
    }

    /// Updates the pan offset based on cursor movement.
    fn handle_pan(&mut self, position: Point) {
        if self.dragging
            && let Some(last_pos) = self.last_cursor_position
        {
            let delta = Vector::new(position.x - last_pos.x, position.y - last_pos.y);
            self.pan_offset = self.pan_offset + delta;
            self.last_cursor_position = Some(position);
        }
    }

    /// Ends the current pan operation.
    fn handle_end_pan(&mut self) {
        self.dragging = false;
        self.last_cursor_position = None;
    }

    /// Resets the view to center with default zoom.
    fn handle_reset_view(&mut self) {
        self.pan_offset = Vector::ZERO;
        self.zoom_factor = super::constants::DEFAULT_ZOOM_FACTOR;
        self.dragging = false;
        self.last_cursor_position = None;
        // Clear any user-pinned/manual node positions so the layout
        // reverts to its automatically computed positions.
        self.pinned_positions_ast.clear();
        self.pinned_positions_nfa.clear();
        self.pinned_positions_dfa.clear();
        self.pinned_positions_min_dfa.clear();
        // Clear any node-drag/selection state.
        self.node_dragging = None;
        self.last_node_cursor_position = None;
        self.selected_node = None;
    }
}
