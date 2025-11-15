use iced::{
    Alignment, Element, Length,
    alignment::{Horizontal, Vertical},
    widget::{Canvas, button, column, container, row, text, themer},
};

use crate::app::{
    message::{Message, RightPaneMode, ViewMessage, ViewMode},
    theme::{ButtonClass, ElementType, TextClass, TextSize},
};
use crate::app::{simulation::SimulationTarget, theme::AppTheme};
use crate::graph::layout::{DfaLayoutStrategy, NfaLayoutStrategy, TreeLayoutStrategy};
use crate::graph::{AstGraph, BoxVisibility, GraphCanvas, Highlights, VisualDfa, VisualNfa};
use crate::{
    app::state::App,
    graph::{Graph, layout::LayoutStrategy},
};

use super::controls;

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
            SimulationTarget::MinDfa => "Minimized DFA Simulation",
        },
    };

    let title = text(title_text)
        .size(TextSize::H2)
        .class(TextClass::Primary);
    let bottom = bottom_controls(app);

    let content = column![title, canvas, bottom]
        .spacing(12)
        .height(Length::Fill)
        .align_x(Alignment::Start);

    container(content).padding(20).height(Length::Fill).into()
}

/// Applies pan and drag state from the app to the given canvas.
fn apply_pan_state<'a, G, S>(app: &App, canvas: &mut GraphCanvas<G, S>)
where
    G: Graph + 'a,
    S: LayoutStrategy + 'a,
{
    // Apply pan state from app
    canvas.set_pan_offset(app.pan_offset);
    if app.last_cursor_position.is_some() {
        canvas.start_drag();
    } else {
        canvas.end_drag();
    }
}

fn render_ast_canvas<'a>(
    app: &App,
    artifacts: &'a regviz_core::core::BuildArtifacts,
) -> ElementType<'a> {
    let ast_graph = AstGraph::new(artifacts.ast.clone(), app.pinned_positions_ast.clone());
    let mut canvas: GraphCanvas<AstGraph, TreeLayoutStrategy> = GraphCanvas::new(
        ast_graph,
        BoxVisibility::default(),
        app.zoom_factor,
        TreeLayoutStrategy,
    );

    apply_pan_state(app, &mut canvas);

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
        .size(TextSize::Body)
        .class(TextClass::Secondary)
        .align_y(Vertical::Top)
        .align_x(Horizontal::Center);

    let bottom = bottom_controls(app);

    let content = column![hint, bottom]
        .spacing(12)
        .height(Length::Fill)
        .align_x(Alignment::Start);

    container(content).padding(20).height(Length::Fill).into()
}

fn bottom_controls(app: &App) -> ElementType<'_> {
    let is_ast = app.view_mode == ViewMode::Ast;
    let is_nfa = app.view_mode == ViewMode::Nfa && app.simulation.target == SimulationTarget::Nfa;
    let is_dfa = app.view_mode == ViewMode::Nfa && app.simulation.target == SimulationTarget::Dfa;
    let is_min_dfa =
        app.view_mode == ViewMode::Nfa && app.simulation.target == SimulationTarget::MinDfa;

    let selector = selector_buttons(is_nfa, is_dfa, is_min_dfa, is_ast);
    let selector_elem: Element<'_, Message, AppTheme> = selector.into();
    let zoom_controls = controls::zoom(app);

    let row = row![selector_elem, zoom_controls]
        .spacing(16)
        .align_y(Alignment::Center)
        .width(Length::Fill);

    container(row)
        .align_x(Horizontal::Center)
        .align_y(Vertical::Bottom)
        .width(Length::Fill)
        .into()
}

fn selector_buttons<'a>(
    is_nfa: bool,
    is_dfa: bool,
    is_min_dfa: bool,
    is_ast: bool,
) -> iced::widget::Row<'a, Message, AppTheme> {
    row![
        tri_button("NFA", is_nfa, RightPaneMode::Nfa),
        tri_button("DFA", is_dfa, RightPaneMode::Dfa),
        tri_button("Min DFA", is_min_dfa, RightPaneMode::MinDfa),
        tri_button("AST", is_ast, RightPaneMode::Ast),
    ]
    .spacing(12)
    .align_y(Alignment::Center)
}

fn tri_button(label: &str, active: bool, mode: RightPaneMode) -> ElementType<'_> {
    let label_text = text(label).size(TextSize::Body).class(if active {
        TextClass::Primary
    } else {
        TextClass::Secondary
    });

    button(label_text)
        .class(if active {
            ButtonClass::Primary
        } else {
            ButtonClass::Secondary
        })
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
            let graph = VisualNfa::new(
                artifacts.nfa.clone(),
                highlights,
                app.pinned_positions_nfa.clone(),
            );
            let mut canvas: GraphCanvas<VisualNfa, NfaLayoutStrategy> = GraphCanvas::new(
                graph,
                app.box_visibility.clone(),
                app.zoom_factor,
                NfaLayoutStrategy,
            );
            apply_pan_state(app, &mut canvas);

            Canvas::new(canvas)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
        SimulationTarget::Dfa => {
            // Prefer the determinized DFA, fall back to minimized if only that exists.
            let maybe_dfa = artifacts.dfa.clone().or_else(|| artifacts.min_dfa.clone());

            let Some(dfa) = maybe_dfa else {
                return text("Determinized DFA is not available")
                    .size(TextSize::Body)
                    .class(TextClass::Warning)
                    .into();
            };

            let highlights: Highlights = app.simulation.current_highlights().unwrap_or_default();
            let graph = VisualDfa::new(
                dfa,
                artifacts.alphabet.clone(),
                highlights,
                app.pinned_positions_dfa.clone(),
            );
            let mut canvas: GraphCanvas<VisualDfa, DfaLayoutStrategy> = GraphCanvas::new(
                graph,
                BoxVisibility::default(),
                app.zoom_factor,
                DfaLayoutStrategy,
            );

            apply_pan_state(app, &mut canvas);

            Canvas::new(canvas)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
        SimulationTarget::MinDfa => {
            // Prefer the minimized DFA, fall back to determinized if only that exists.
            let maybe_dfa = artifacts.min_dfa.clone().or_else(|| artifacts.dfa.clone());

            let Some(dfa) = maybe_dfa else {
                return text("Minimized DFA is not available")
                    .size(TextSize::Body)
                    .class(TextClass::Warning)
                    .into();
            };

            let highlights: Highlights = app.simulation.current_highlights().unwrap_or_default();
            let graph = VisualDfa::new(
                dfa,
                artifacts.alphabet.clone(),
                highlights,
                app.pinned_positions_min_dfa.clone(),
            );
            let mut canvas: GraphCanvas<VisualDfa, DfaLayoutStrategy> = GraphCanvas::new(
                graph,
                BoxVisibility::default(),
                app.zoom_factor,
                DfaLayoutStrategy,
            );

            apply_pan_state(app, &mut canvas);

            Canvas::new(canvas)
                .width(Length::Fill)
                .height(Length::Fill)
                .into()
        }
    }
}
