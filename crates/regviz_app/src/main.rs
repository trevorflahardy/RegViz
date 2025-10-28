/// RegViz - Regular Expression Visualizer
///
/// This application provides interactive visualizations of regular expressions,
/// allowing users to see both the parse tree (AST) and the non-deterministic
/// finite automaton (NFA) representation.
///
/// # Features
///
/// - **Parse Tree**: Hierarchical tree layout showing operator precedence
/// - **NFA**: State machine visualization with configurable bounding boxes
/// - **Interactive**: Zoom in/out, toggle elements, switch between views
/// - **Real-time**: Immediate feedback as you type regex patterns
mod app;
mod graph;

use iced::{
    Element,
    widget::{Canvas, button, column, container, row, text, text_input},
};
use regviz_core::core::automaton::BoxKind;
use regviz_core::core::{BuildArtifacts, nfa, parser};

use graph::{BoxVisibility, GraphCanvas};

#[derive(Default)]
struct App {
    input: String,
    error: Option<String>,
    build_artifacts: Option<BuildArtifacts>,
    box_visibility: BoxVisibility,
}

#[derive(Debug, Clone)]
enum Message {
    InputChanged(String),
    ToggleBox(BoxKind),
}

impl App {
    fn view(&self) -> Element<'_, Message> {
        let status_text = match &self.error {
            Some(err) => text(format!("Error: {}", err)),
            None => match &self.build_artifacts {
                Some(artifacts) => text(format!(
                    "AST: {:?}\nNFA states: {}\nAlphabet: {:?}",
                    artifacts.ast,
                    artifacts.nfa.states.len(),
                    artifacts.alphabet
                )),
                None => text("No build artifacts available."),
            },
        };

        let mut col = column![
            text_input("Type something here...", &self.input).on_input(Message::InputChanged),
            text(format!("Input: {}", self.input)),
            status_text,
        ]
        .spacing(10);

        // Conditionally add the canvas
        if let Some(artifacts) = &self.build_artifacts {
            let toggles = row![
                self.box_toggle_button(BoxKind::Literal, "Literal"),
                self.box_toggle_button(BoxKind::Concat, "Concat"),
                self.box_toggle_button(BoxKind::Alternation, "Alternation"),
                self.box_toggle_button(BoxKind::KleeneStar, "Star"),
                self.box_toggle_button(BoxKind::KleenePlus, "Plus"),
                self.box_toggle_button(BoxKind::Optional, "Optional"),
            ]
            .spacing(8);

            col = col.push(toggles);

            let graph_canvas: GraphCanvas<nfa::Nfa> =
                GraphCanvas::new(artifacts.nfa.clone(), self.box_visibility.clone());

            // Canvas that takes up max width and height of the column
            let canvas = Canvas::new(graph_canvas)
                .width(iced::Length::Fill)
                .height(iced::Length::Fill);

            // Wrap in container with padding
            let canvas_with_padding = container(canvas).padding(20); // Add 20 pixels of padding on all sides

            col = col.push(canvas_with_padding);
        }

        col.into()
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::InputChanged(input) => {
                self.input = input;
                self.lex_and_parse();
            }
            Message::ToggleBox(kind) => {
                self.box_visibility.toggle(kind);
            }
        }
    }

    fn lex_and_parse(&mut self) {
        if self.input.is_empty() {
            self.error = None;
            self.build_artifacts = None;
            return;
        }
        match parser::Ast::build(&self.input) {
            Ok(ast) => {
                let nfa = nfa::build_nfa(ast.clone());
                let alphabet = nfa.alphabet();
                self.build_artifacts = Some(BuildArtifacts {
                    ast,
                    nfa,
                    alphabet,
                    dfa: None,
                    min_dfa: None,
                });
                self.error = None;
            }
            Err(e) => {
                self.error = Some(format!("Parse error: {}", e));
                self.build_artifacts = None;
            }
        }
    }
}

impl App {
    fn box_toggle_button<'a>(
        &'a self,
        kind: BoxKind,
        label: &'static str,
    ) -> iced::Element<'a, Message> {
        let active = self.box_visibility.is_visible(kind);
        let text_label = format!("{}: {}", label, if active { "On" } else { "Off" });
        button(text(text_label).size(16))
            .padding([4, 12])
            .on_press(Message::ToggleBox(kind))
            .into()
    }
}

/// Application entry point.
///
/// Initializes tracing (in debug mode) and starts the Iced event loop
/// with the RegViz application.
fn main() -> iced::Result {
    #[cfg(debug_assertions)]
    init_tracing();

    iced::run(
        "RegViz - Regular Expression Visualizer",
        App::update,
        App::view,
    )
}
