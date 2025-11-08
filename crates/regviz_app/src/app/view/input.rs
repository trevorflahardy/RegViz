use iced::{
    Length,
    widget::{column, container, scrollable, text, text_input},
};

use iced::widget::row;

use crate::app::{
    message::{InputMessage, Message},
    state::App,
    theme::{ContainerClass, ElementType, TextClass, TextInputClass, TextSize},
};
use regviz_core::errors::BuildError;

/// Renders the regex input field and status text.
pub fn render(app: &App) -> ElementType<'_> {
    let label = text("Regular Expression")
        .size(TextSize::H3)
        .class(TextClass::Primary);

    let input_field = text_input("e.g., (a+b)c", &app.input)
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
        Some(err) => error_box(&app.input, err),
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

/// Displays an error with highlighted character at the error position.
fn error_box<'a>(input: &'a str, err: &'a BuildError) -> ElementType<'a> {
    let error_char_index = match err {
        BuildError::Lex(lex_err) => lex_err.at,
        BuildError::Parse(parse_err) => parse_err.at,
    };

    // Iterate through characters with their index
    let char_count = input.chars().count();
    let spans = input
        .chars()
        .enumerate()
        .map(|(idx, ch)| {
            if idx == error_char_index {
                // Error character (white text, red background)
                container(
                    text(ch.to_string())
                        .size(TextSize::Small)
                        .class(TextClass::Primary),
                )
                .class(ContainerClass::FilledWith(
                    iced::Color::from_rgba(1.0, 0.2, 0.2, 0.8), // Red background
                ))
                .into()
            } else {
                // Normal character
                text(ch.to_string())
                    .size(TextSize::Small)
                    .class(TextClass::Primary)
                    .into()
            }
        })
        .chain((error_char_index == char_count).then(|| {
            container(text("⏎").size(TextSize::Small).class(TextClass::Primary))
                .class(ContainerClass::FilledWith(
                    crate::app::theme::colors::RED_500,
                ))
                .into()
        }));

    let error_display = column![
        row(spans).spacing(0), // Display the highlighted input
        text(format!("Error: {err}"))
            .size(TextSize::Small)
            .class(TextClass::Error),
    ]
    .spacing(4);

    let scrollable_error = scrollable(error_display)
        .direction(scrollable::Direction::Horizontal(Default::default()))
        .width(Length::Fill);

    container(scrollable_error)
        .padding(8)
        .class(ContainerClass::FilledWith(iced::Color::from_rgba(
            0.6, 0.2, 0.2, 0.2,
        )))
        .width(Length::Fill)
        .into()
}
