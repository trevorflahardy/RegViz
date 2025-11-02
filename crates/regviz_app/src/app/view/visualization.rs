use iced::{
    Alignment, Element, Length,
    alignment::{Horizontal, Vertical},
    widget::{Canvas, button, column, container, row, text, themer},
};

use crate::app::state::App;
use crate::app::{
    message::{Message, RightPaneMode, ViewMessage, ViewMode},
    theme::ElementType,
};
use crate::app::{simulation::SimulationTarget, theme::AppTheme};
use crate::graph::layout::{DfaLayoutStrategy, NfaLayoutStrategy, TreeLayoutStrategy};
use crate::graph::{AstGraph, BoxVisibility, GraphCanvas, Highlights, VisualDfa, VisualNfa};

/// Renders the active visualization (AST or automaton).
pub fn render<'a>(
    app: &'a App,
    artifacts: &'a regviz_core::core::BuildArtifacts,
) -> ElementType<'a> {
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
    let bottom = bottom_tri_toggle(app);

    let content = column![title, canvas, bottom]
        .spacing(12)
        .height(Length::Fill)
        .align_x(Alignment::Start);

    container(content).padding(20).height(Length::Fill).into()
}

fn render_ast_canvas<'a>(
    app: &App,
    artifacts: &'a regviz_core::core::BuildArtifacts,
) -> ElementType<'a> {
    let ast_graph = AstGraph::new(artifacts.ast.clone());
    let canvas: GraphCanvas<AstGraph, TreeLayoutStrategy> = GraphCanvas::new(
        ast_graph,
        BoxVisibility::default(),
        app.zoom_factor,
        TreeLayoutStrategy,
    );

    let canvas_elem: Element<'_, Message, AppTheme> = Canvas::new(canvas)
        .width(Length::Fill)
        .height(Length::Fill)
        .into();

    themer(app.theme.into(), canvas_elem).into()
}

/// Renders an empty right pane when no artifacts are available.
pub fn render_empty(app: &App) -> ElementType<'_> {
    let hint = text("Enter a regular expression to visualize")
        .height(Length::Fill)
        .align_y(Vertical::Top)
        .align_x(Horizontal::Center);

    let bottom = bottom_tri_toggle(app);

    let content = column![hint, bottom]
        .spacing(12)
        .height(Length::Fill)
        .align_x(Alignment::Start);

    container(content).padding(20).height(Length::Fill).into()
}

fn bottom_tri_toggle(app: &App) -> ElementType<'_> {
    let is_ast = app.view_mode == ViewMode::Ast;
    let is_nfa = app.view_mode == ViewMode::Nfa && app.simulation.target == SimulationTarget::Nfa;
    let is_dfa = app.view_mode == ViewMode::Nfa && app.simulation.target == SimulationTarget::Dfa;

    let ast = tri_button("AST", is_ast, RightPaneMode::Ast);
    let nfa = tri_button("NFA", is_nfa, RightPaneMode::Nfa);
    let dfa = tri_button("DFA", is_dfa, RightPaneMode::Dfa);

    let row = row![nfa, dfa, ast].spacing(12).align_y(Alignment::Center);
    container(row)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Bottom)
        .width(Length::Fill)
        .into()
}

fn tri_button(label: &str, active: bool, mode: RightPaneMode) -> ElementType<'_> {
    let mut text_label = label.to_string();
    if active {
        text_label.push_str(" ✓");
    }

    button(text(text_label).size(14))
        .padding([4, 12])
        .on_press(Message::View(ViewMessage::SelectRightPaneMode(mode)))
        .into()
}

fn render_automaton_canvas<'a>(
    app: &App,
    artifacts: &'a regviz_core::core::BuildArtifacts,
) -> ElementType<'a> {
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
            let highlights: Highlights = app.simulation.current_highlights().unwrap_or_default();
            let graph = VisualDfa::new(dfa, artifacts.alphabet.clone(), highlights);
            let canvas: GraphCanvas<VisualDfa, DfaLayoutStrategy> = GraphCanvas::new(
                graph,
                BoxVisibility::default(),
                app.zoom_factor,
                DfaLayoutStrategy,
            );

            Canvas::new(canvas)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    }
}
