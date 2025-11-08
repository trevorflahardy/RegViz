mod controls;
mod input;
mod simulation;
mod visualization;

use iced::{
    Alignment, Length,
    widget::{Space, button, column, container, pane_grid, row, scrollable, text},
};

use crate::app::{
    message::InputMessage,
    theme::{ButtonClass, ContainerClass, ElementType, TextClass, TextSize},
};

use super::message::{Message, PaneGridMessage};
use super::state::{App, PaneContent};

const INPUT_EXAMPLES: &[&str] = &["a+b", "\\e", "(a+b)*c", "ab+cd?", "a(bc)*d+e?"];

impl App {
    /// Renders the entire application UI.
    pub fn view(&self) -> ElementType<'_> {
        let grid = pane_grid::PaneGrid::new(&self.panes, |_, pane, _| match pane {
            PaneContent::Controls => pane_grid::Content::new(left_controls(self)),
            PaneContent::Visualization => pane_grid::Content::new(right_visual(self)),
        })
        .on_resize(8, |event| {
            Message::PaneGrid(PaneGridMessage::Resized(event))
        })
        .width(Length::Fill)
        .height(Length::Fill);

        container(grid)
            .class(ContainerClass::RoundedLarge)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}

fn information_block() -> ElementType<'static> {
    column![
        text(
            "
Alphanumeric characters (a-z, A-Z, 0-9) and the following special characters are supported:
1. '\\e': epsilon
2. '(', ')': for grouping
3. '+': alternation
4. '*': kleene star
5. '.': concatenation
"
        )
        .size(TextSize::Small)
        .class(TextClass::Secondary),
    ]
    .spacing(8)
    .into()
}

fn left_controls(app: &App) -> ElementType<'_> {
    let examples_row = row(INPUT_EXAMPLES.iter().map(|&example| {
        button(text(example).size(TextSize::Small))
            .class(ButtonClass::Secondary)
            .on_press(Message::Input(InputMessage::Changed(example.to_string())))
            .into()
    }))
    .spacing(6)
    .wrap();

    let content = column![
        column![
            text!("Regular Expression Visualizer").size(TextSize::H1),
            text!("Build and visualize finite automata from regular expressions.")
                .size(TextSize::Body)
                .class(TextClass::Secondary),
        ]
        .spacing(4),
        information_block(),
        input::render(app),
        examples_row,
        simulation::test_string_input(app),
        Space::new().height(Length::Fill),
        simulation::panel(app),
    ]
    .spacing(16);

    let scrollable_content = scrollable(content).width(Length::Fill).height(Length::Fill);

    container(scrollable_content)
        .padding(15)
        .align_x(Alignment::Start)
        .width(Length::Fill)
        .height(Length::Fill)
        .class(ContainerClass::FilledWith(app.theme.bg_mid()))
        .into()
}

fn right_visual(app: &App) -> ElementType<'_> {
    if let Some(artifacts) = &app.build_artifacts {
        visualization::render(app, artifacts)
    } else {
        visualization::render_empty(app)
    }
}
