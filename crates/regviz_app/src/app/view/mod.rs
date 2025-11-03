mod controls;
mod input;
mod simulation;
mod visualization;

use iced::{
    Alignment, Length,
    widget::{button, column, container, pane_grid, row, text},
};

use regviz_core::core::BuildArtifacts;

use crate::app::{
    message::InputMessage,
    theme::{ContainerClass, ElementType, TextClass, TextSize},
};

use super::message::{Message, PaneGridMessage, ViewMode};
use super::simulation::SimulationTarget;
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

fn render_mode_specific<'a>(app: &'a App, artifacts: &'a BuildArtifacts) -> ElementType<'a> {
    let mut controls_column = column![].spacing(10);

    if app.view_mode == ViewMode::Nfa {
        controls_column = controls_column.push(simulation::panel(app, artifacts));

        if app.simulation.target == SimulationTarget::Nfa {
            controls_column = controls_column.push(controls::bounding_boxes(app));
        }
    }

    container(controls_column).align_x(Alignment::Start).into()
}

fn left_controls(app: &App) -> ElementType<'_> {
    let mut col = column![
        column![
            text!("Regular Expression Visualizer").size(TextSize::H1),
            text!("Build and visualize finite automata from regular expressions.")
                .size(TextSize::Body)
                .class(TextClass::Secondary),
        ],
        text(
            "\
Alphanumeric characters (a-z, A-Z, 0-9) and the following special characters are supported:
1. '\\e': epsilon
2. '(', ')': for grouping
3. '+': alternation
4. '*': kleene star
5. '.': concatenation
"
        )
        .font(iced::Font::with_name("JetBrains Mono"))
        .size(TextSize::Body)
        .class(TextClass::Secondary),
        column![
            text("Examples:")
                .size(TextSize::H3)
                .class(TextClass::Primary),
            row(INPUT_EXAMPLES.iter().map(|&example| {
                button(example)
                    .on_press(Message::Input(InputMessage::Changed(example.to_string())))
                    .into()
            }))
            .spacing(8)
            .wrap(),
        ],
        input::render(app)
    ]
    .spacing(10);

    if let Some(artifacts) = &app.build_artifacts {
        col = col.push(render_mode_specific(app, artifacts));
    }

    container(col)
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
