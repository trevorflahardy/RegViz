use iced::{
    Alignment, Element,
    widget::{button, column, row, text, text_input},
};

use regviz_core::core::BuildArtifacts;

use crate::app::message::{Message, SimulationMessage};
use crate::app::simulation::SimulationTarget;
use crate::app::state::App;

/// Renders controls for stepping through the simulation input.
pub fn panel<'a>(app: &'a App, _artifacts: &'a BuildArtifacts) -> Element<'a, Message> {
    let target_toggle = target_buttons(app);

    let input_field = text_input("Enter an input string (e.g., abab)", &app.simulation.input)
        .on_input(|value| Message::Simulation(SimulationMessage::InputChanged(value)))
        .padding(8)
        .size(16);

    let controls_row = step_controls(app);

    let mut section = column![target_toggle, input_field, controls_row].spacing(6);

    if let Some(summary) = summary_line(app) {
        section = section.push(text(summary).size(14));
    }

    if let Some(states) = active_states_line(app) {
        section = section.push(text(states).size(14));
    }

    if app.simulation.is_current_rejection() {
        section = section.push(text("Input string is not accepted.").size(14));
    } else if acceptance_hint(app) {
        section = section.push(text("Input string is accepted.").size(14));
    }

    section.into()
}

fn target_buttons(app: &App) -> Element<'_, Message> {
    let nfa = toggle_button(
        "NFA",
        app.simulation.target == SimulationTarget::Nfa,
        SimulationTarget::Nfa,
    );
    let dfa = toggle_button(
        "DFA",
        app.simulation.target == SimulationTarget::Dfa,
        SimulationTarget::Dfa,
    );

    row![text("Simulate:").size(14), nfa, dfa,]
        .spacing(10)
        .align_y(Alignment::Center)
        .into()
}

fn toggle_button(
    label: &str,
    is_active: bool,
    target: SimulationTarget,
) -> Element<'static, Message> {
    let text_label = if is_active {
        format!("{} ✓", label)
    } else {
        label.to_string()
    };

    button(text(text_label).size(14))
        .padding([4, 12])
        .on_press(Message::Simulation(SimulationMessage::TargetChanged(
            target,
        )))
        .into()
}

fn step_controls(app: &App) -> Element<'_, Message> {
    let mut prev_button = button(text("Prev").size(14)).padding([4, 12]);
    if app.simulation.can_step_backward() {
        prev_button = prev_button.on_press(Message::Simulation(SimulationMessage::StepBackward));
    }

    let reset_button = button(text("Reset").size(14))
        .padding([4, 12])
        .on_press(Message::Simulation(SimulationMessage::Reset));

    let mut next_button = button(text("Next").size(14)).padding([4, 12]);
    if app.simulation.can_step_forward() {
        next_button = next_button.on_press(Message::Simulation(SimulationMessage::StepForward));
    }

    let step_label = step_label(app).unwrap_or_else(|| "Step –".to_string());

    row![
        prev_button,
        reset_button,
        next_button,
        text(step_label).size(14),
    ]
    .spacing(10)
    .align_y(Alignment::Center)
    .into()
}

fn step_label(app: &App) -> Option<String> {
    let step = app.simulation.current_step()?;
    let total = app.simulation.step_count()?;
    let max_index = total.saturating_sub(1);
    Some(format!("Step {} / {}", step.index, max_index))
}

fn summary_line(app: &App) -> Option<String> {
    let step = app.simulation.current_step()?;
    let total = app.simulation.step_count()?;
    let max_index = total.saturating_sub(1);
    let consumed = match step.consumed {
        Some(ch) => format!("Consumed: '{}'", ch),
        None => "Consumed: –".to_string(),
    };
    let accepting = if step.accepted {
        "Accepting: Yes"
    } else {
        "Accepting: No"
    };

    Some(format!(
        "Step {} / {} • {} • {}",
        step.index, max_index, consumed, accepting
    ))
}

fn active_states_line(app: &App) -> Option<String> {
    let step = app.simulation.current_step()?;
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

    let target_label = match app.simulation.target {
        SimulationTarget::Nfa => "NFA",
        SimulationTarget::Dfa => "DFA",
    };

    Some(format!("Active {} states: {}", target_label, states_text))
}

fn acceptance_hint(app: &App) -> bool {
    let Some(step) = app.simulation.current_step() else {
        return false;
    };
    let Some(total_steps) = app.simulation.step_count() else {
        return false;
    };

    if app.simulation.is_current_rejection() {
        return false;
    }

    app.simulation.cursor + 1 == total_steps && step.accepted
}
