mod controls;
mod input;
mod mode_toggle;
mod simulation;
mod visualization;

use iced::{
    Alignment, Element,
    widget::{column, container},
};

use regviz_core::core::BuildArtifacts;

use super::message::{Message, ViewMode};
use super::simulation::SimulationTarget;
use super::state::App;

impl App {
    /// Renders the entire application UI.
    pub fn view(&self) -> Element<'_, Message> {
        let mut content = column![input::render(self)].spacing(10);

        if let Some(artifacts) = &self.build_artifacts {
            content = content
                .push(mode_toggle::render(self))
                .push(render_mode_specific(self, artifacts))
                .push(visualization::render(self, artifacts));
        }

        content.into()
    }
}

fn render_mode_specific<'a>(app: &'a App, artifacts: &'a BuildArtifacts) -> Element<'a, Message> {
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
