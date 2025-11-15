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
    let canvas = match app.view_mode() {
        ViewMode::Ast => render_ast_canvas(app, artifacts),
        ViewMode::Nfa | ViewMode::Dfa | ViewMode::MinDfa => render_automaton_canvas(app, artifacts),
    };

    let title_text = match app.view_mode() {
        ViewMode::Ast => "Parse Tree Visualization",
        ViewMode::Nfa => "NFA Simulation",
        ViewMode::Dfa => "DFA Simulation",
        ViewMode::MinDfa => "Minimized DFA Simulation",
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
    canvas.set_pan_offset(app.view_data().pan_offset);
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
    let ast_graph = AstGraph::new(
        artifacts.ast.clone(),
        app.view_data().pinned_node_positions.clone(),
    );
    let mut canvas: GraphCanvas<AstGraph, TreeLayoutStrategy> = GraphCanvas::new(
        ast_graph,
        BoxVisibility::default(),
        app.view_data().zoom_factor,
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
    let selector = selector_buttons(app);
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

fn selector_buttons<'a>(app: &App) -> iced::widget::Row<'a, Message, AppTheme> {
    let curr_view_mode = app.view_mode();
    let is_nfa = curr_view_mode == ViewMode::Nfa;
    let is_dfa = curr_view_mode == ViewMode::Dfa;
    let is_min_dfa = curr_view_mode == ViewMode::MinDfa;
    let is_ast = curr_view_mode == ViewMode::Ast;
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
    app: &'a App,
    artifacts: &'a regviz_core::core::BuildArtifacts,
) -> ElementType<'a> {
    let pinned_node_positions = &app.view_data().pinned_node_positions;
    let zoom_factor = app.view_data().zoom_factor;

    match app.simulation.target {
        SimulationTarget::Nfa => {
            let highlights: Highlights = app.simulation.current_highlights().unwrap_or_default();
            let graph = VisualNfa::new(&artifacts.nfa, highlights, pinned_node_positions);
            let mut canvas: GraphCanvas<VisualNfa, NfaLayoutStrategy> = GraphCanvas::new(
                graph,
                app.box_visibility.clone(),
                zoom_factor,
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
            let maybe_dfa = artifacts.dfa.as_ref().or(artifacts.min_dfa.as_ref());

            let Some(dfa) = maybe_dfa else {
                return text("Determinized DFA is not available")
                    .size(TextSize::Body)
                    .class(TextClass::Warning)
                    .into();
            };

            let highlights: Highlights = app.simulation.current_highlights().unwrap_or_default();
            let graph = VisualDfa::new(dfa, &artifacts.alphabet, highlights, pinned_node_positions);
            let mut canvas: GraphCanvas<VisualDfa, DfaLayoutStrategy> = GraphCanvas::new(
                graph,
                BoxVisibility::default(),
                zoom_factor,
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
            let maybe_dfa = artifacts.min_dfa.as_ref().or(artifacts.dfa.as_ref());

            let Some(dfa) = maybe_dfa else {
                return text("Minimized DFA is not available")
                    .size(TextSize::Body)
                    .class(TextClass::Warning)
                    .into();
            };

            let highlights: Highlights = app.simulation.current_highlights().unwrap_or_default();
            let graph = VisualDfa::new(dfa, &artifacts.alphabet, highlights, pinned_node_positions);
            let mut canvas: GraphCanvas<VisualDfa, DfaLayoutStrategy> = GraphCanvas::new(
                graph,
                BoxVisibility::default(),
                zoom_factor,
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
