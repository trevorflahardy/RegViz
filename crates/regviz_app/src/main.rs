mod graph;

use iced::{
    Alignment, Element,
    widget::{self, Canvas, button, column, container, row, slider, text, text_input},
};
use regviz_core::core::automaton::BoxKind;
use regviz_core::core::{BuildArtifacts, lexer, nfa, parser};

use graph::{BoxVisibility, GraphCanvas};

struct App {
    input: String,
    error: Option<String>,
    build_artifacts: Option<BuildArtifacts>,
    box_visibility: BoxVisibility,
    zoom_factor: f32,
}

impl Default for App {
    fn default() -> Self {
        Self {
            input: String::new(),
            error: None,
            build_artifacts: None,
            box_visibility: BoxVisibility::default(),
            zoom_factor: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
enum Message {
    InputChanged(String),
    ToggleBox(BoxKind),
    ZoomChanged(f32),
}

const MIN_ZOOM_FACTOR: f32 = 0.25;
const MAX_ZOOM_FACTOR: f32 = 4.0;

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

            let zoom_display = text(format!("Zoom: {:.0}%", self.zoom_factor * 100.0));
            let zoom_slider = slider(
                MIN_ZOOM_FACTOR..=MAX_ZOOM_FACTOR,
                self.zoom_factor,
                Message::ZoomChanged,
            )
            .step(0.01);
            let zoom_controls = row![zoom_display, widget::Space::with_width(8.0), zoom_slider]
                .spacing(12)
                .align_y(Alignment::Center);

            col = col.push(zoom_controls);

            let graph_canvas: GraphCanvas<nfa::Nfa> = GraphCanvas::new(
                artifacts.nfa.clone(),
                self.box_visibility.clone(),
                self.zoom_factor,
            );

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
            Message::ZoomChanged(value) => {
                self.zoom_factor = value.clamp(MIN_ZOOM_FACTOR, MAX_ZOOM_FACTOR);
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

#[cfg(debug_assertions)]
fn init_tracing() {
    use tracing_subscriber::EnvFilter;
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .pretty()
        .try_init()
        .ok();
}

fn main() -> iced::Result {
    init_tracing();

    iced::run("RegViz", App::update, App::view)
}
