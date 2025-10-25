use iced::{
    widget::{column, text, text_input},
    Element,
};
use regviz_core::core::BuildArtifacts;
use regviz_core::core::{lexer, nfa, parser};

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
    fn view(&self) -> Element<Message> {
        column![
            text_input("Type something here...", &self.input).on_input(Message::InputChanged),
            text(format!("Input: {}", self.input)),
            match &self.error {
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
            },
        ]
        .spacing(10)
        .into()
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
        match lexer::lex(&self.input) {
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
