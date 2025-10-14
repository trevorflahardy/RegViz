mod app;
mod core;
mod errors;
mod examples;
mod viz;

use crate::app::{Model, update_fn, view_fn};

fn main() -> iced::Result {
    iced::application("Regex Automata Visualizer", update_fn, view_fn)
        .theme(|_| iced::Theme::Dark)
        .run_with(|| (Model::default(), iced::Task::none()))
}
