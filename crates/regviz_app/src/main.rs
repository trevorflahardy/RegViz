mod graph;

use iced::{
    Element,
    widget::{Canvas, column, container, text, text_input},
};
use regviz_core::core::{BuildArtifacts, lexer, nfa, parser};

use graph::GraphCanvas;

#[derive(Default)]
struct App {
    input: String,
    error: Option<String>,
    build_artifacts: Option<BuildArtifacts>,
}

#[derive(Debug, Clone)]
enum Message {
    InputChanged(String),
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
            let graph_canvas: GraphCanvas<nfa::Nfa> =
                GraphCanvas::new(artifacts.nfa.clone().into());

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
        }
    }

    fn lex_and_parse(&mut self) {
        if self.input.is_empty() {
            self.error = None;
            self.build_artifacts = None;
            return;
        }
        match lexer::lex(self.input.trim()) {
            Ok(tokens) => match parser::parse(&tokens) {
                Ok(ast) => {
                    let nfa = nfa::build_nfa(&ast);
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
            },
            Err(e) => {
                self.error = Some(format!("Lex error: {}", e));
                self.build_artifacts = None;
            }
        }
    }
}

fn main() -> iced::Result {
    iced::run("RegViz", App::update, App::view)
}
