use iced::Task;

use crate::app::{Message, Model, TestResult};
use crate::core::{self, BuildArtifacts};
use crate::examples::presets::Sample;

const LOG_LIMIT: usize = 200;

pub fn update(model: &mut Model, message: Message) -> Task<Message> {
    match message {
        Message::RegexChanged(text) => {
            model.regex_src = text;
        }
        Message::BuildNfa => match build_artifacts(&model.regex_src) {
            Ok(art) => {
                model.last_build = Some(art);
                model.test_result = None;
                log(model, "Built ε-NFA");
            }
            Err(err) => log(model, err),
        },
        Message::BuildDfa => {
            ensure_built(model);
            if let Some(artifacts) = model.last_build.as_mut() {
                let (dfa, alphabet) = core::dfa::determinize(&artifacts.nfa);
                artifacts.dfa = Some(dfa);
                artifacts.alphabet = alphabet;
                log(model, "Built DFA via subset construction");
            }
        }
        Message::MinimizeDfa => {
            ensure_built(model);
            if let Some(artifacts) = model.last_build.as_mut() {
                if artifacts.dfa.is_none() {
                    let (dfa, alphabet) = core::dfa::determinize(&artifacts.nfa);
                    artifacts.alphabet = alphabet;
                    artifacts.dfa = Some(dfa);
                }
                if let Some(ref dfa) = artifacts.dfa {
                    let min = core::min::minimize(dfa, &artifacts.alphabet);
                    artifacts.min_dfa = Some(min);
                    log(model, "Minimized DFA using Hopcroft");
                }
            }
        }
        Message::TestInputChanged(text) => {
            model.test_input = text;
        }
        Message::TestNow => {
            ensure_built(model);
            if let Some(artifacts) = model.last_build.as_ref() {
                let accepted = core::sim::nfa_accepts(&artifacts.nfa, &model.test_input);
                model.test_result = Some(if accepted {
                    TestResult::Accepted
                } else {
                    TestResult::Rejected
                });
                log(
                    model,
                    format!(
                        "Tested input: '{}' => {}",
                        model.test_input,
                        if accepted { "accepted" } else { "rejected" }
                    ),
                );
            }
        }
        Message::SelectExample(idx) => {
            if idx < model.examples.len() {
                model.selected_example = idx;
                let example = &model.examples[idx];
                model.regex_src = example.regex.to_string();
                model.test_input = example
                    .samples
                    .first()
                    .map(|Sample { input, .. }| (*input).to_string())
                    .unwrap_or_default();
                model.test_result = None;
                log(
                    model,
                    format!("Loaded example '{}': {}", example.name, example.regex),
                );
            }
        }
        Message::SwitchTab(tab) => {
            model.view_tab = tab;
        }
        Message::ClearLogs => {
            model.messages.clear();
        }
    }
    Task::none()
}

fn ensure_built(model: &mut Model) {
    if model.last_build.is_none() {
        if let Ok(art) = build_artifacts(&model.regex_src) {
            model.last_build = Some(art);
            model.test_result = None;
            log(model, "Lazy-built ε-NFA");
        }
    }
}

fn build_artifacts(pattern: &str) -> Result<BuildArtifacts, String> {
    let tokens = core::lexer::lex(pattern).map_err(|err| format!("Lex error: {err}"))?;
    let ast = core::parser::parse(&tokens).map_err(|err| format!("Parse error: {err}"))?;
    let nfa = core::nfa::build_nfa(&ast);
    let alphabet = nfa.alphabet();
    Ok(BuildArtifacts::new(ast, nfa, alphabet))
}

fn log(model: &mut Model, msg: impl Into<String>) {
    model.messages.push(msg.into());
    if model.messages.len() > LOG_LIMIT {
        let overflow = model.messages.len() - LOG_LIMIT;
        model.messages.drain(0..overflow);
    }
}
