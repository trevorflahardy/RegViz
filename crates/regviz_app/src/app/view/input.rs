use iced::{
    Length,
    widget::{column, text, text_input},
};

use crate::app::{
    message::{InputMessage, Message},
    state::App,
    theme::{ElementType, TextClass, TextInputClass, TextSize},
};

/// Renders the regex input field and status text.
pub fn render(app: &App) -> ElementType<'_> {
    let label = text("Regular Expression")
        .size(TextSize::H3)
        .class(TextClass::Primary);

    let input_field = text_input("e.g., (a|b)*c", &app.input)
        .class(if app.error.is_some() {
            TextInputClass::Invalid
        } else {
            TextInputClass::Default
        })
        .on_input(|value| Message::Input(InputMessage::Changed(value)))
        .padding([12, 16])
        .size(TextSize::Body)
        .width(Length::Fill);

    let status = status_text(app);

    column![label, input_field, status].spacing(6).into()
}

fn status_text(app: &App) -> ElementType<'_> {
    match &app.error {
        Some(err) => text(format!("Error: {err}"))
            .size(TextSize::Small)
            .class(TextClass::Error)
            .into(),
        None => match &app.build_artifacts {
            Some(artifacts) => text(format!(
                "Parsed successfully | {} states | Alphabet: {:?}",
                artifacts.nfa.states.len(),
                artifacts.alphabet
            ))
            .size(TextSize::Small)
            .class(TextClass::Success)
            .into(),
            None => text("Enter a regular expression to visualize")
                .size(TextSize::Small)
                .class(TextClass::Secondary)
                .into(),
        },
    }
}
