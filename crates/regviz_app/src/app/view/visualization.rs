use iced::{
    Element, Length,
    widget::{Canvas, column, container, text},
};

use crate::app::message::{Message, ViewMode};
use crate::app::simulation::SimulationTarget;
use crate::app::state::App;
use crate::graph::layout::{NfaLayoutStrategy, TreeLayoutStrategy};
use crate::graph::{AstGraph, BoxVisibility, GraphCanvas, Highlights, VisualDfa, VisualNfa};

/// Renders the active visualization (AST or automaton).
pub fn render<'a>(
    app: &'a App,
    artifacts: &'a regviz_core::core::BuildArtifacts,
) -> Element<'a, Message> {
    let canvas = match app.view_mode {
        ViewMode::Ast => render_ast_canvas(app, artifacts),
        ViewMode::Nfa => render_automaton_canvas(app, artifacts),
    };

    let title_text = match app.view_mode {
        ViewMode::Ast => "Parse Tree Visualization",
        ViewMode::Nfa => match app.simulation.target {
            SimulationTarget::Nfa => "NFA Simulation",
            SimulationTarget::Dfa => "DFA Simulation",
        },
    };

    let title = text(title_text).size(18);
    let content = column![title, canvas].spacing(12).height(Length::Fill);

    container(content).padding(20).height(Length::Fill).into()
}

fn render_ast_canvas<'a>(
    app: &'a App,
    artifacts: &'a regviz_core::core::BuildArtifacts,
) -> Element<'a, Message> {
    let ast_graph = AstGraph::new(artifacts.ast.clone());
    let canvas: GraphCanvas<AstGraph, TreeLayoutStrategy> = GraphCanvas::new(
        ast_graph,
        BoxVisibility::default(),
        app.zoom_factor,
        TreeLayoutStrategy,
    );

    Canvas::new(canvas)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn render_automaton_canvas<'a>(
    app: &'a App,
    artifacts: &'a regviz_core::core::BuildArtifacts,
) -> Element<'a, Message> {
    match app.simulation.target {
        SimulationTarget::Nfa => {
            let highlights: Highlights = app.simulation.current_highlights().unwrap_or_default();
            let graph = VisualNfa::new(artifacts.nfa.clone(), highlights);
            let canvas: GraphCanvas<VisualNfa, NfaLayoutStrategy> = GraphCanvas::new(
                graph,
                app.box_visibility.clone(),
                app.zoom_factor,
                NfaLayoutStrategy,
            );

            Canvas::new(canvas)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
        SimulationTarget::Dfa => {
            let Some(dfa) = artifacts.dfa.clone() else {
                return text("Determinized DFA is not available").into();
            };
            let Some(alphabet) = artifacts.dfa_alphabet.clone() else {
                return text("DFA alphabet is not available").into();
            };
            let highlights: Highlights = app.simulation.current_highlights().unwrap_or_default();
            let graph = VisualDfa::new(dfa, alphabet, highlights);
            let canvas: GraphCanvas<VisualDfa, NfaLayoutStrategy> = GraphCanvas::new(
                graph,
                BoxVisibility::default(),
                app.zoom_factor,
                NfaLayoutStrategy,
            );

            Canvas::new(canvas)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    }
}
