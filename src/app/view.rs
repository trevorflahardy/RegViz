use iced::widget::canvas::Canvas;
use iced::widget::{
    Column, Row, Rule, button, column, container, row, scrollable, text, text_input,
};
use iced::{Element, Length};

use crate::app::{Message, Model, TestResult, ViewTab, canvas::AutomatonCanvas};

pub fn view(model: &Model) -> Element<Message> {
    let left_panel = column![
        text("Regular Expression").size(18),
        text_input("Enter regex", &model.regex_src)
            .on_input(Message::RegexChanged)
            .padding(8)
            .size(16),
        row![
            button("Build ε-NFA").on_press(Message::BuildNfa),
            button("Build DFA").on_press(Message::BuildDfa),
            button("Minimize DFA").on_press(Message::MinimizeDfa),
        ]
        .spacing(8),
        text("Test String").size(18),
        row![
            text_input("Input", &model.test_input)
                .on_input(Message::TestInputChanged)
                .padding(8)
                .size(16),
            button("Test").on_press(Message::TestNow),
        ]
        .spacing(8),
        result_badge(model),
        examples_row(model),
    ]
    .spacing(12)
    .max_width(340.0);

    let tabs = tab_row(model.view_tab);
    let content = match model.view_tab {
        ViewTab::Automaton => automaton_view(model),
        ViewTab::Table => table_view(model),
        ViewTab::Logs => log_view(model),
    };

    let right_panel = column![tabs, content].spacing(12).width(Length::Fill);

    row![left_panel, divider(), right_panel]
        .spacing(16)
        .padding(20)
        .into()
}

fn divider() -> Element<'static, Message> {
    Rule::vertical(1.0).into()
}

fn result_badge(model: &Model) -> Element<Message> {
    let label = match model.test_result {
        Some(TestResult::Accepted) => container(text("✅ accepted")).padding(6).style(|_theme| {
            iced::widget::container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.15, 0.35, 0.15,
                ))),
                ..Default::default()
            }
        }),
        Some(TestResult::Rejected) => container(text("❌ rejected")).padding(6).style(|_theme| {
            iced::widget::container::Style {
                background: Some(iced::Background::Color(iced::Color::from_rgb(
                    0.35, 0.15, 0.15,
                ))),
                ..Default::default()
            }
        }),
        None => container(text("Awaiting test")),
    };
    label.width(Length::Shrink).into()
}

fn examples_row(model: &Model) -> Element<Message> {
    let mut row = Row::new().spacing(6).push(text("Examples:"));
    for (idx, example) in model.examples.iter().enumerate() {
        let label = if idx == model.selected_example {
            format!("{} •", example.name)
        } else {
            example.name.to_string()
        };
        row = row.push(button(text(label)).on_press(Message::SelectExample(idx)));
    }
    row.into()
}

fn tab_row(active: ViewTab) -> Element<'static, Message> {
    let mut row = Row::new().spacing(8);
    for tab in [ViewTab::Automaton, ViewTab::Table, ViewTab::Logs] {
        let label = match tab {
            ViewTab::Automaton => "Automaton",
            ViewTab::Table => "Transitions",
            ViewTab::Logs => "Logs",
        };
        let label = if tab == active {
            format!("{label} •")
        } else {
            label.to_string()
        };
        row = row.push(button(text(label)).on_press(Message::SwitchTab(tab)));
    }
    row.into()
}

fn automaton_view(model: &Model) -> Element<Message> {
    if let Some(artifacts) = model.last_build.as_ref() {
        let canvas = Canvas::new(AutomatonCanvas::new(artifacts))
            .width(Length::Fill)
            .height(Length::Fill);
        container(canvas)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x(Length::Fill)
            .center_y(Length::Fill)
            .into()
    } else {
        info_placeholder("Build an automaton to visualize.")
    }
}

fn table_view(model: &Model) -> Element<Message> {
    if let Some(artifacts) = model.last_build.as_ref() {
        let mut table = Column::new().spacing(4);
        table = table.push(text(format!("NFA states: {}", artifacts.nfa.states.len())));
        table = table.push(text(format!(
            "NFA transitions: {}",
            artifacts.nfa.edges.len()
        )));
        if let Some(ref dfa) = artifacts.dfa {
            table = table.push(text("DFA transition table:"));
            table = table.push(header_row(&artifacts.alphabet));
            for (idx, row_data) in dfa.trans.iter().enumerate() {
                let mut row = Row::new().spacing(6).push(text(format!("q{}", idx)));
                for entry in row_data {
                    let cell = entry
                        .map(|id| format!("q{}", id))
                        .unwrap_or_else(|| "∅".into());
                    row = row.push(text(cell));
                }
                table = table.push(row);
            }
        } else {
            table = table.push(text("Build the DFA to inspect transitions."));
        }
        container(scrollable(table))
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    } else {
        info_placeholder("Build an automaton to inspect its tables.")
    }
}

fn header_row(alphabet: &[char]) -> Row<'static, Message> {
    let mut row = Row::new().spacing(6).push(text("State"));
    for sym in alphabet {
        row = row.push(text(sym.to_string()));
    }
    row
}

fn log_view(model: &Model) -> Element<Message> {
    let mut list = Column::new().spacing(4);
    if model.messages.is_empty() {
        list = list.push(text("No messages yet."));
    } else {
        for message in &model.messages {
            list = list.push(text(message.clone()));
        }
    }
    column![
        row![button(text("Clear")).on_press(Message::ClearLogs)].align_y(iced::Alignment::Center),
        scrollable(list).height(Length::Fill)
    ]
    .spacing(8)
    .into()
}

fn info_placeholder(message: &str) -> Element<Message> {
    container(text(message))
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x(Length::Fill)
        .center_y(Length::Fill)
        .into()
}
