use iced::{
    Alignment, Element, Length,
    widget::{Canvas, button, column, container, row, slider, text, text_input},
};
use regviz_core::core::{BuildArtifacts, automaton::BoxKind};

use crate::graph::layout::{NfaLayoutStrategy, TreeLayoutStrategy};
use crate::graph::{AstGraph, BoxVisibility, GraphCanvas, Highlights, VisualDfa, VisualNfa};

use super::constants::{MAX_ZOOM_FACTOR, MIN_ZOOM_FACTOR};
use super::message::{Message, ViewMode};
use super::simulation::SimulationTarget;
use super::state::App;

impl App {
    /// Renders the entire application UI.
    ///
    /// The view is organized into several sections:
    /// 1. Input field for entering regex
    /// 2. Status/error display
    /// 3. View mode toggle (AST vs NFA)
    /// 4. Mode-specific controls (box toggles for NFA, zoom for both)
    /// 5. Visualization canvas
    pub fn view(&self) -> Element<'_, Message> {
        let mut col = column![].spacing(10);

        // Input section
        col = col.push(self.render_input_section());

        // Only show visualizations if we have valid artifacts
        if let Some(artifacts) = &self.build_artifacts {
            col = col.push(self.render_view_mode_toggle());
            col = col.push(self.render_mode_specific_controls(artifacts));
            col = col.push(self.render_visualization(artifacts));
        }

        col.into()
    }

    /// Renders the regex input field and status text.
    fn render_input_section(&self) -> Element<'_, Message> {
        let input_field = text_input(
            "Enter a regular expression (e.g., a+b, (a+b)*c)",
            &self.input,
        )
        .on_input(Message::InputChanged)
        .padding(8)
        .size(16);

        let status = self.render_status_text();

        column![input_field, status].spacing(8).into()
    }

    /// Renders status or error information.
    fn render_status_text(&self) -> Element<'_, Message> {
        match &self.error {
            Some(err) => text(format!("x  {}", err)).size(14).into(),
            None => match &self.build_artifacts {
                Some(artifacts) => text(format!(
                    "✓ Parsed successfully | {} states | Alphabet: {:?}",
                    artifacts.nfa.states.len(),
                    artifacts.alphabet
                ))
                .size(14)
                .into(),
                None => text("Enter a regular expression to visualize")
                    .size(14)
                    .into(),
            },
        }
    }

    /// Renders toggle buttons to switch between AST and NFA views.
    fn render_view_mode_toggle(&self) -> Element<'_, Message> {
        let ast_button = self.create_view_mode_button(ViewMode::Ast);
        let nfa_button = self.create_view_mode_button(ViewMode::Nfa);

        row![text("View:").size(16), ast_button, nfa_button,]
            .spacing(12)
            .align_y(Alignment::Center)
            .into()
    }

    /// Creates a button for switching to a specific view mode.
    fn create_view_mode_button(&self, mode: ViewMode) -> Element<'_, Message> {
        let label = mode.label();

        button(text(label).size(16))
            .padding([6, 16])
            .on_press(Message::ViewModeChanged(mode))
            .into()
    }

    /// Renders controls that are specific to the current view mode.
    fn render_mode_specific_controls(&self, artifacts: &BuildArtifacts) -> Element<'_, Message> {
        let mut controls = column![].spacing(10);

        if self.view_mode == ViewMode::Nfa {
            controls = controls.push(self.render_simulation_controls(artifacts));

            if self.simulation.target == SimulationTarget::Nfa {
                controls = controls.push(self.render_box_toggles());
            }
        }

        controls = controls.push(self.render_zoom_controls());

        controls.into()
    }

    /// Renders controls for stepping through the simulation input.
    fn render_simulation_controls(&self, artifacts: &BuildArtifacts) -> Element<'_, Message> {
        let target_toggle = row![
            text("Simulate:").size(14),
            self.create_simulation_target_button(SimulationTarget::Nfa),
            self.create_simulation_target_button(SimulationTarget::Dfa),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let input_field = text_input("Enter an input string (e.g., abab)", &self.simulation.input)
            .on_input(Message::SimulationInputChanged)
            .padding(8)
            .size(16);

        let mut prev_button = button(text("Prev").size(14)).padding([4, 12]);
        if self.simulation.can_step_backward() {
            prev_button = prev_button.on_press(Message::SimulationStepBackward);
        }

        let mut next_button = button(text("Next").size(14)).padding([4, 12]);
        if self.simulation.can_step_forward() {
            next_button = next_button.on_press(Message::SimulationStepForward);
        }

        let reset_button = button(text("Reset").size(14))
            .padding([4, 12])
            .on_press(Message::SimulationReset);

        let step_label = self
            .simulation_step_label()
            .unwrap_or_else(|| "Step –".to_string());

        let controls_row = row![
            prev_button,
            reset_button,
            next_button,
            text(step_label).size(14),
        ]
        .spacing(10)
        .align_y(Alignment::Center);

        let mut section = column![target_toggle, input_field, controls_row].spacing(6);

        if let Some(summary) = self.simulation_summary_line(artifacts) {
            section = section.push(text(summary).size(14));
        }

        if let Some(states) = self.simulation_active_states_line() {
            section = section.push(text(states).size(14));
        }

        section.into()
    }

    /// Creates a toggle button for selecting the simulation target automaton.
    fn create_simulation_target_button(&self, target: SimulationTarget) -> Element<'_, Message> {
        let label = match target {
            SimulationTarget::Nfa => "NFA",
            SimulationTarget::Dfa => "DFA",
        };
        let text_label = if self.simulation.target == target {
            format!("{} ✓", label)
        } else {
            label.to_string()
        };

        button(text(text_label).size(14))
            .padding([4, 12])
            .on_press(Message::SimulationTargetChanged(target))
            .into()
    }

    /// Renders buttons for toggling bounding box visibility (NFA only).
    fn render_box_toggles(&self) -> Element<'_, Message> {
        let toggles = row![
            self.create_box_toggle_button(BoxKind::Literal, "Literal"),
            self.create_box_toggle_button(BoxKind::Concat, "Concat"),
            self.create_box_toggle_button(BoxKind::Alternation, "Alt"),
            self.create_box_toggle_button(BoxKind::KleeneStar, "Star"),
            self.create_box_toggle_button(BoxKind::KleenePlus, "Plus"),
            self.create_box_toggle_button(BoxKind::Optional, "Optional"),
        ]
        .spacing(8);

        column![text("Bounding Boxes:").size(14), toggles,]
            .spacing(4)
            .into()
    }

    /// Creates a toggle button for a specific bounding box type.
    fn create_box_toggle_button(&self, kind: BoxKind, label: &'static str) -> Element<'_, Message> {
        let is_visible = self.box_visibility.is_visible(kind);
        let button_text = format!("{}: {}", label, if is_visible { "✓" } else { "✗" });

        button(text(button_text).size(14))
            .padding([4, 10])
            .on_press(Message::ToggleBox(kind))
            .into()
    }

    /// Renders zoom controls with slider and percentage display.
    fn render_zoom_controls(&self) -> Element<'_, Message> {
        let zoom_percentage = (self.zoom_factor * 100.0).round() as i32;
        let zoom_display = text(format!("Zoom: {}%", zoom_percentage)).size(14);

        let zoom_slider = slider(
            MIN_ZOOM_FACTOR..=MAX_ZOOM_FACTOR,
            self.zoom_factor,
            Message::ZoomChanged,
        )
        .step(0.05)
        .width(Length::Fixed(200.0));

        row![zoom_display, zoom_slider]
            .spacing(12)
            .align_y(Alignment::Center)
            .into()
    }

    /// Renders the active visualization (AST or NFA).
    fn render_visualization(&self, artifacts: &BuildArtifacts) -> Element<'_, Message> {
        let canvas = match self.view_mode {
            ViewMode::Ast => self.render_ast_canvas(artifacts),
            ViewMode::Nfa => self.render_automaton_canvas(artifacts),
        };

        let title_text = match self.view_mode {
            ViewMode::Ast => "Parse Tree Visualization",
            ViewMode::Nfa => match self.simulation.target {
                SimulationTarget::Nfa => "NFA Simulation",
                SimulationTarget::Dfa => "DFA Simulation",
            },
        };

        let title = text(title_text).size(18);
        let content = column![title, canvas].spacing(12).height(Length::Fill);

        container(content).padding(20).height(Length::Fill).into()
    }

    /// Creates an AST visualization canvas.
    fn render_ast_canvas(
        &self,
        artifacts: &regviz_core::core::BuildArtifacts,
    ) -> Element<'_, Message> {
        let ast_graph = AstGraph::new(artifacts.ast.clone());
        let canvas: GraphCanvas<AstGraph, TreeLayoutStrategy> = GraphCanvas::new(
            ast_graph,
            BoxVisibility::default(),
            self.zoom_factor,
            TreeLayoutStrategy,
        );

        Canvas::new(canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Creates an automaton visualization canvas for the active simulation target.
    fn render_automaton_canvas(&self, artifacts: &BuildArtifacts) -> Element<'_, Message> {
        match self.simulation.target {
            SimulationTarget::Nfa => {
                let highlights: Highlights =
                    self.simulation.current_highlights().unwrap_or_default();
                let graph = VisualNfa::new(artifacts.nfa.clone(), highlights);
                let canvas: GraphCanvas<VisualNfa, NfaLayoutStrategy> = GraphCanvas::new(
                    graph,
                    self.box_visibility.clone(),
                    self.zoom_factor,
                    NfaLayoutStrategy,
                );

                Canvas::new(canvas)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
            SimulationTarget::Dfa => {
                let Some(dfa) = artifacts.dfa.clone() else {
                    return text("Determinized DFA is not available").into();
                };
                let alphabet = artifacts.alphabet.clone();
                let highlights: Highlights =
                    self.simulation.current_highlights().unwrap_or_default();
                let graph = VisualDfa::new(dfa, alphabet, highlights);
                let canvas: GraphCanvas<VisualDfa, NfaLayoutStrategy> = GraphCanvas::new(
                    graph,
                    BoxVisibility::default(),
                    self.zoom_factor,
                    NfaLayoutStrategy,
                );

                Canvas::new(canvas)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .into()
            }
        }
    }

    fn simulation_step_label(&self) -> Option<String> {
        let step = self.simulation.current_step()?;
        let total = self.simulation.step_count()?;
        let max_index = total.saturating_sub(1);
        Some(format!("Step {} / {}", step.index, max_index))
    }

    fn simulation_summary_line(&self, artifacts: &BuildArtifacts) -> Option<String> {
        let step = self.simulation.current_step()?;
        let total = self.simulation.step_count()?;
        let max_index = total.saturating_sub(1);
        let consumed = match step.consumed {
            Some(ch) => format!("Consumed: '{}'", ch),
            None => "Consumed: –".to_string(),
        };
        let accepting = match self.simulation_accepting(artifacts) {
            Some(true) => "Accepting: Yes",
            Some(false) => "Accepting: No",
            None => "Accepting: –",
        };

        Some(format!(
            "Step {} / {} • {} • {}",
            step.index, max_index, consumed, accepting
        ))
    }

    fn simulation_active_states_line(&self) -> Option<String> {
        let step = self.simulation.current_step()?;
        let mut states: Vec<_> = step.active_states.iter().copied().collect();
        states.sort_unstable();

        let states_text = if states.is_empty() {
            "∅".to_string()
        } else {
            states
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        };

        let target_label = match self.simulation.target {
            SimulationTarget::Nfa => "NFA",
            SimulationTarget::Dfa => "DFA",
        };

        Some(format!("Active {} states: {}", target_label, states_text))
    }

    fn simulation_accepting(&self, artifacts: &BuildArtifacts) -> Option<bool> {
        let step = self.simulation.current_step()?;

        match self.simulation.target {
            SimulationTarget::Nfa => Some(
                step.active_states
                    .iter()
                    .any(|state| artifacts.nfa.accepts.contains(state)),
            ),
            SimulationTarget::Dfa => {
                let dfa = artifacts.dfa.as_ref()?;
                Some(
                    step.active_states
                        .iter()
                        .any(|state| dfa.accepts.contains(state)),
                )
            }
        }
    }
}
