use iced::{Element, Task};

pub mod canvas;
pub mod update;
pub mod view;

use crate::core::BuildArtifacts;
use crate::examples::presets::{self, Example};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ViewTab {
    Automaton,
    Table,
    Logs,
}

#[derive(Debug, Clone)]
pub enum TestResult {
    Accepted,
    Rejected,
}

#[derive(Debug, Clone)]
pub struct Model {
    pub regex_src: String,
    pub test_input: String,
    pub view_tab: ViewTab,
    pub last_build: Option<BuildArtifacts>,
    pub test_result: Option<TestResult>,
    pub messages: Vec<String>,
    pub selected_example: usize,
    pub examples: Vec<Example>,
}

impl Default for Model {
    fn default() -> Self {
        let examples = presets::presets().to_vec();
        Self {
            regex_src: String::new(),
            test_input: String::new(),
            view_tab: ViewTab::Automaton,
            last_build: None,
            test_result: None,
            messages: Vec::new(),
            selected_example: 0,
            examples,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Message {
    RegexChanged(String),
    BuildNfa,
    BuildDfa,
    MinimizeDfa,
    TestInputChanged(String),
    TestNow,
    SelectExample(usize),
    SwitchTab(ViewTab),
    ClearLogs,
}

// Create wrapper functions for the new API
pub fn update_fn(model: &mut Model, message: Message) -> Task<Message> {
    update::update(model, message)
}

pub fn view_fn(model: &Model) -> Element<Message> {
    view::view(model)
}
