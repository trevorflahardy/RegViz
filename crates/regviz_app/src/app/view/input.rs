use iced::{
    Length,
    widget::{column, container, scrollable, text, text_input},
};
use unicode_width::UnicodeWidthStr;

use crate::app::{
    APP_FONT,
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

/// Displays an error with the input and an arrow pointing to the error position.
fn error_box<'a>(input: &'a str, err: &'a BuildError) -> ElementType<'a> {
    let error_index = match err {
        BuildError::Lex(lex_err) => lex_err.at,
        BuildError::Parse(parse_err) => parse_err.at,
    };

    // Calculate visual width up to error position using unicode-width
    let text_before_error = &input[..error_index.min(input.len())];
    let visual_width = UnicodeWidthStr::width(text_before_error);

    // Create arrow line with padding
    let arrow_padding = " ".repeat(visual_width);
    let arrow_line = format!("{arrow_padding}^");

    let error_display = column![
        text(input)
            .size(TextSize::Small)
            .font(APP_FONT)
            .class(TextClass::Primary),
        text(arrow_line)
            .size(TextSize::Small)
            .font(APP_FONT)
            .class(TextClass::Error),
        text(format!("Error: {err}"))
            .size(TextSize::Small)
            .class(TextClass::Error),
    ]
    .spacing(2);

    let scrollable_error = scrollable(error_display)
        .direction(scrollable::Direction::Horizontal(Default::default()))
        .width(Length::Fill);

    container(scrollable_error)
        .padding(8)
        .class(ContainerClass::FilledWith(
            // Use a slightly different background for the error box
            iced::Color::from_rgba(0.6, 0.2, 0.2, 0.2),
        ))
        .width(Length::Fill)
        .into()
}
