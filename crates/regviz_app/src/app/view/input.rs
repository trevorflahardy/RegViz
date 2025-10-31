use iced::{
    Element,
    widget::{column, text, text_input},
};

use crate::app::message::{InputMessage, Message};
use crate::app::state::App;

/// Renders the regex input field and status text.
pub fn render(app: &App) -> Element<'_, Message> {
    let input_field = text_input(
        "Enter a regular expression (e.g., a+b, (a+b)*c)",
        &app.input,
    )
    .on_input(|value| Message::Input(InputMessage::Changed(value)))
    .padding(8)
    .size(16);

    let status = status_text(app);

    column![input_field, status].spacing(8).into()
}

fn status_text(app: &App) -> Element<'_, Message> {
    match &app.error {
        Some(err) => text(format!("x  {err}")).size(14).into(),
        None => match &app.build_artifacts {
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
