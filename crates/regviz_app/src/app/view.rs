use iced::{
    Alignment, Element, Length,
    widget::{Canvas, button, column, container, row, slider, text, text_input},
};
use regviz_core::core::{automaton::BoxKind, nfa};

use crate::graph::layout::{NfaLayoutStrategy, TreeLayoutStrategy};
use crate::graph::{AstGraph, BoxVisibility, GraphCanvas};

use super::constants::{MAX_ZOOM_FACTOR, MIN_ZOOM_FACTOR};
use super::message::{Message, ViewMode};
use super::state::App;

impl App {
    /// Renders the entire application UI.
    ///
    /// The view is organized into several sections:
    /// 1. Input field for entering regex
    /// 2. Status/error display
    /// 3. View mode toggle (AST vs NFA)
    /// 4. Mode-specific controls (box toggles for NFA, zoom for both)
    /// 5. Visualization canvas
    pub fn view(&self) -> Element<'_, Message> {
        let mut col = column![].spacing(10);

        // Input section
        col = col.push(self.render_input_section());

        // Only show visualizations if we have valid artifacts
        if self.build_artifacts.is_some() {
            col = col.push(self.render_view_mode_toggle());
            col = col.push(self.render_mode_specific_controls());
            col = col.push(self.render_visualization());
        }

        col.into()
    }

    /// Renders the regex input field and status text.
    fn render_input_section(&self) -> Element<'_, Message> {
        let input_field = text_input(
            "Enter a regular expression (e.g., a+b, (a+b)*c)",
            &self.input,
        )
        .on_input(Message::InputChanged)
        .padding(8)
        .size(16);

        let status = self.render_status_text();

        column![input_field, status].spacing(8).into()
    }

    /// Renders status or error information.
    fn render_status_text(&self) -> Element<'_, Message> {
        match &self.error {
            Some(err) => text(format!("x  {}", err)).size(14).into(),
            None => match &self.build_artifacts {
                Some(artifacts) => text(format!(
                    "✓ Parsed successfully | {} states | Alphabet: {:?}",
                    artifacts.nfa.states.len(),
                    artifacts.alphabet
                ))
                .size(14)
                .into(),
                None => text("Enter a regular expression to visualize")
                    .size(14)
                    .into(),
            },
        }
    }

    /// Renders toggle buttons to switch between AST and NFA views.
    fn render_view_mode_toggle(&self) -> Element<'_, Message> {
        let ast_button = self.create_view_mode_button(ViewMode::Ast);
        let nfa_button = self.create_view_mode_button(ViewMode::Nfa);

        row![text("View:").size(16), ast_button, nfa_button,]
            .spacing(12)
            .align_y(Alignment::Center)
            .into()
    }

    /// Creates a button for switching to a specific view mode.
    fn create_view_mode_button(&self, mode: ViewMode) -> Element<'_, Message> {
        let label = mode.label();

        button(text(label).size(16))
            .padding([6, 16])
            .on_press(Message::ViewModeChanged(mode))
            .into()
    }

    /// Renders controls that are specific to the current view mode.
    fn render_mode_specific_controls(&self) -> Element<'_, Message> {
        let mut controls = column![].spacing(8);

        // Show box visibility toggles only in NFA mode
        if self.view_mode == ViewMode::Nfa {
            controls = controls.push(self.render_box_toggles());
        }

        // Zoom controls are available in both modes
        controls = controls.push(self.render_zoom_controls());

        controls.into()
    }

    /// Renders buttons for toggling bounding box visibility (NFA only).
    fn render_box_toggles(&self) -> Element<'_, Message> {
        let toggles = row![
            self.create_box_toggle_button(BoxKind::Literal, "Literal"),
            self.create_box_toggle_button(BoxKind::Concat, "Concat"),
            self.create_box_toggle_button(BoxKind::Alternation, "Alt"),
            self.create_box_toggle_button(BoxKind::KleeneStar, "Star"),
            self.create_box_toggle_button(BoxKind::KleenePlus, "Plus"),
            self.create_box_toggle_button(BoxKind::Optional, "Optional"),
        ]
        .spacing(8);

        column![text("Bounding Boxes:").size(14), toggles,]
            .spacing(4)
            .into()
    }

    /// Creates a toggle button for a specific bounding box type.
    fn create_box_toggle_button(&self, kind: BoxKind, label: &'static str) -> Element<'_, Message> {
        let is_visible = self.box_visibility.is_visible(kind);
        let button_text = format!("{}: {}", label, if is_visible { "✓" } else { "✗" });

        button(text(button_text).size(14))
            .padding([4, 10])
            .on_press(Message::ToggleBox(kind))
            .into()
    }

    /// Renders zoom controls with slider and percentage display.
    fn render_zoom_controls(&self) -> Element<'_, Message> {
        let zoom_percentage = (self.zoom_factor * 100.0).round() as i32;
        let zoom_display = text(format!("Zoom: {}%", zoom_percentage)).size(14);

        let zoom_slider = slider(
            MIN_ZOOM_FACTOR..=MAX_ZOOM_FACTOR,
            self.zoom_factor,
            Message::ZoomChanged,
        )
        .step(0.05)
        .width(Length::Fixed(200.0));

        row![zoom_display, zoom_slider]
            .spacing(12)
            .align_y(Alignment::Center)
            .into()
    }

    /// Renders the active visualization (AST or NFA).
    fn render_visualization(&self) -> Element<'_, Message> {
        let Some(artifacts) = &self.build_artifacts else {
            return text("No visualization available").into();
        };

        let canvas = match self.view_mode {
            ViewMode::Ast => self.render_ast_canvas(artifacts),
            ViewMode::Nfa => self.render_nfa_canvas(artifacts),
        };

        let title = text(match self.view_mode {
            ViewMode::Ast => "Parse Tree Visualization",
            ViewMode::Nfa => "NFA Visualization",
        })
        .size(18);

        let content = column![title, canvas].spacing(12).height(Length::Fill);

        container(content).padding(20).height(Length::Fill).into()
    }

    /// Creates an AST visualization canvas.
    fn render_ast_canvas(
        &self,
        artifacts: &regviz_core::core::BuildArtifacts,
    ) -> Element<'_, Message> {
        let ast_graph = AstGraph::new(artifacts.ast.clone());
        let canvas: GraphCanvas<AstGraph, TreeLayoutStrategy> = GraphCanvas::new(
            ast_graph,
            BoxVisibility::default(),
            self.zoom_factor,
            TreeLayoutStrategy,
        );

        Canvas::new(canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }

    /// Creates an NFA visualization canvas.
    fn render_nfa_canvas(
        &self,
        artifacts: &regviz_core::core::BuildArtifacts,
    ) -> Element<'_, Message> {
        let canvas: GraphCanvas<nfa::Nfa, NfaLayoutStrategy> = GraphCanvas::new(
            artifacts.nfa.clone(),
            self.box_visibility.clone(),
            self.zoom_factor,
            NfaLayoutStrategy,
        );

        Canvas::new(canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    }
}
