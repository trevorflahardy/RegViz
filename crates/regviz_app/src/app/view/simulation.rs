use iced::{
    Alignment, Element,
    widget::{button, column, row, text, text_input},
};

use regviz_core::core::BuildArtifacts;

use crate::app::state::App;
use crate::app::{
    message::{Message, SimulationMessage},
    theme::ElementType,
};
use crate::app::{simulation::SimulationTarget, theme::AppTheme};

/// Renders controls for stepping through the simulation input.
pub fn panel<'a>(app: &'a App, artifacts: &'a BuildArtifacts) -> ElementType<'a> {
    panel_column(app, artifacts).into()
}

fn panel_column<'a>(
    app: &'a App,
    _artifacts: &'a BuildArtifacts,
) -> iced::widget::Column<'a, Message, AppTheme> {
    let input_field = text_input("Enter an input string (e.g., abab)", &app.simulation.input)
        .on_input(|value| Message::Simulation(SimulationMessage::InputChanged(value)))
        .padding(8)
        .size(16);

    let controls_row = step_controls(app, app.simulation_error.is_some());
    let mut section = column![input_field].spacing(6);
    let PanelMessages {
        validation,
        summary,
    } = panel_messages(app);

    if let Some(error) = validation {
        section = section.push(text(error).size(14));
    }

    section = section.push(controls_row);

    for message in summary {
        section = section.push(text(message).size(14));
    }

    section
}

// Target toggle buttons were moved to the right pane tri-toggle.

fn step_controls(app: &App, disabled: bool) -> ElementType<'_> {
    let mut prev_button = button(text("Prev").size(14)).padding([4, 12]);
    if !disabled && app.simulation.can_step_backward() {
        prev_button = prev_button.on_press(Message::Simulation(SimulationMessage::StepBackward));
    }

    let mut reset_button = button(text("Reset").size(14)).padding([4, 12]);
    if !disabled {
        reset_button = reset_button.on_press(Message::Simulation(SimulationMessage::Reset));
    }

    let mut next_button = button(text("Next").size(14)).padding([4, 12]);
    if !disabled && app.simulation.can_step_forward() {
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
        Some(ch) => format!("Consumed: '{ch}'"),
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

    Some(format!("Active {target_label} states: {states_text}"))
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

struct PanelMessages {
    validation: Option<String>,
    summary: Vec<String>,
}

fn panel_messages(app: &App) -> PanelMessages {
    let validation = app.simulation_error.clone();
    let summary = summary_messages(app);
    PanelMessages {
        validation,
        summary,
    }
}

fn summary_messages(app: &App) -> Vec<String> {
    let mut messages = Vec::new();

    if let Some(summary) = summary_line(app) {
        messages.push(summary);
    }

    if let Some(states) = active_states_line(app) {
        messages.push(states);
    }

    if app.simulation.is_current_rejection() {
        messages.push("Input string is not accepted.".to_string());
    } else if acceptance_hint(app) {
        messages.push("Input string is accepted.".to_string());
    }

    messages
}
