use iced::{
    Alignment, Length,
    widget::{Space, button, column, row, text, text_input},
};

use crate::app::simulation::SimulationTarget;
use crate::app::state::App;
use crate::app::{
    message::{Message, SimulationMessage},
    theme::{ButtonClass, ElementType, TextClass, TextInputClass, TextSize},
};

use super::controls;

/// Renders controls for stepping through the simulation input.
pub fn panel(app: &App) -> ElementType<'_> {
    let ready = app.build_artifacts.is_some();
    let (status_label, status_class) = simulation_status(app);

    let header = row![
        text("Simulation")
            .size(TextSize::H3)
            .class(TextClass::Primary),
        Space::new().width(Length::Fill),
        text(status_label).size(TextSize::Small).class(status_class),
    ]
    .align_y(Alignment::Center);

    let disabled = !ready || app.simulation_error.is_some();
    let controls_section = simulation_controls_section(app, disabled);

    column![header, controls_section].spacing(12).into()
}

fn simulation_status(app: &App) -> (String, TextClass) {
    if app.build_artifacts.is_none() {
        ("Regex required".to_string(), TextClass::Secondary)
    } else if app.simulation_error.is_some() {
        ("Input error".to_string(), TextClass::Error)
    } else {
        ("Ready".to_string(), TextClass::Success)
    }
}

/// Renders the test string input field positioned near the regex input.
pub fn test_string_input(app: &App) -> ElementType<'_> {
    let enabled = app.build_artifacts.is_some();
    let placeholder = if enabled {
        "Enter a string to validate"
    } else {
        "Provide a regex to enable simulation"
    };

    let helper = if let Some(error) = &app.simulation_error {
        text(error).size(TextSize::Small).class(TextClass::Error)
    } else if !enabled {
        text("Build a valid regular expression to unlock the simulation.")
            .size(TextSize::Small)
            .class(TextClass::Secondary)
    } else {
        text("Simulate against the currently selected automaton.")
            .size(TextSize::Small)
            .class(TextClass::Secondary)
    };

    let bounding_boxes = controls::bounding_boxes(app);

    column![
        text("Test String")
            .size(TextSize::H3)
            .class(TextClass::Primary),
        text_input(placeholder, &app.simulation.input)
            .class(if app.simulation_error.is_some() {
                TextInputClass::Invalid
            } else {
                TextInputClass::Default
            })
            .on_input_maybe(if enabled {
                Some(|value| Message::Simulation(SimulationMessage::InputChanged(value)))
            } else {
                None
            })
            .padding([12, 16])
            .size(TextSize::Body)
            .width(Length::Fill),
        helper,
        bounding_boxes
    ]
    .spacing(6)
    .into()
}

fn simulation_controls_section(app: &App, disabled: bool) -> ElementType<'_> {
    let mut content = column![step_controls(app, disabled)].spacing(6);

    for message in summary_messages(app) {
        content = content.push(
            text(message.text)
                .size(TextSize::Small)
                .class(message.class),
        );
    }

    content.into()
}

fn step_controls(app: &App, disabled: bool) -> ElementType<'_> {
    let prev_active = !disabled && app.simulation.can_step_backward();
    let mut prev_button = button(text("Previous").size(TextSize::Body))
        .class(if prev_active {
            ButtonClass::Primary
        } else {
            ButtonClass::Secondary
        })
        .padding([10, 16])
        .width(Length::Fill);

    if prev_active {
        prev_button = prev_button.on_press(Message::Simulation(SimulationMessage::StepBackward));
    }

    let reset_active = !disabled;
    let mut reset_button = button(text("Reset").size(TextSize::Body))
        .class(if reset_active {
            ButtonClass::Primary
        } else {
            ButtonClass::Secondary
        })
        .padding([10, 16])
        .width(Length::Fill);

    if reset_active {
        reset_button = reset_button.on_press(Message::Simulation(SimulationMessage::Reset));
    }

    let next_active = !disabled && app.simulation.can_step_forward();
    let mut next_button = button(text("Next").size(TextSize::Body))
        .class(if next_active {
            ButtonClass::Primary
        } else {
            ButtonClass::Secondary
        })
        .padding([10, 16])
        .width(Length::Fill);

    if next_active {
        next_button = next_button.on_press(Message::Simulation(SimulationMessage::StepForward));
    }

    row![prev_button, reset_button, next_button]
        .spacing(12)
        .align_y(Alignment::Center)
        .width(Length::Fill)
        .into()
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
        format!(
            "{{{}}}",
            states
                .iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(", ")
        )
    };

    let target_label = match app.simulation.target {
        SimulationTarget::Nfa => "NFA",
        SimulationTarget::Dfa => "DFA",
        SimulationTarget::MinDfa => "Min DFA",
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

struct SummaryMessage {
    text: String,
    class: TextClass,
}

fn summary_messages(app: &App) -> Vec<SummaryMessage> {
    let mut messages = Vec::new();

    if let Some(summary) = summary_line(app) {
        messages.push(SummaryMessage {
            text: summary,
            class: TextClass::Secondary,
        });
    }

    if let Some(states) = active_states_line(app) {
        messages.push(SummaryMessage {
            text: states,
            class: TextClass::Secondary,
        });
    }

    if app.simulation.is_current_rejection() {
        messages.push(SummaryMessage {
            text: "Input string is not accepted.".to_string(),
            class: TextClass::Error,
        });
    } else if acceptance_hint(app) {
        messages.push(SummaryMessage {
            text: "Input string is accepted.".to_string(),
            class: TextClass::Success,
        });
    }

    messages
}
