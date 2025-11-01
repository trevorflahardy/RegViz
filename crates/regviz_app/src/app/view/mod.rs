mod controls;
mod input;
mod simulation;
mod visualization;

use iced::{
    Alignment, Length,
    widget::{column, container, pane_grid, text},
};

use regviz_core::core::BuildArtifacts;

use crate::app::theme::{ContainerClass, ElementType, TextSize};

use super::message::{Message, PaneGridMessage, ViewMode};
use super::simulation::SimulationTarget;
use super::state::{App, PaneContent};

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

    controls_column = controls_column.push(controls::zoom(app));

    container(controls_column).align_x(Alignment::Start).into()
}

fn left_controls(app: &App) -> ElementType<'_> {
    let mut col = column![
        column![
            text!("Regular Expression Visualizer").size(TextSize::H1),
            text!("Build and visualize finite automata from regular expressions.")
                .size(TextSize::Body),
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
