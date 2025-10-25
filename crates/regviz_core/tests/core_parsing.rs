//! Comprehensive tests for core parsing systems (lexer, parser, nfa, dfa, min, sim)

use regviz_core::core::{ast, dfa, lexer, min, nfa, parser, sim, tokens};

#[test]
fn test_lexer_basic() {
    let input = "a|b*";
    let tokens = lexer::lex(input).expect("Lexer should succeed");
    assert!(tokens.len() > 0);
    assert_eq!(tokens[0].kind, tokens::TokenKind::Char('a'));
}

#[test]
fn test_parser_basic() {
    let input = "a|b*";
    let tokens = lexer::lex(input).unwrap();
    let ast = parser::parse(&tokens).expect("Parser should succeed");
    // Check AST root type
    match ast {
        ast::Ast::Alt(_, _) => {}
        _ => panic!("Expected Alt node at root"),
    }
}

#[test]
fn test_nfa_construction() {
    let input = "ab*";
    let tokens = lexer::lex(input).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    let nfa = nfa::build_nfa(&ast);
    assert!(nfa.states.len() > 0);
    assert!(nfa.edges.len() > 0);
}

#[test]
fn test_dfa_determinize() {
    let input = "a*";
    let tokens = lexer::lex(input).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    let nfa = nfa::build_nfa(&ast);
    let (dfa, alphabet) = dfa::determinize(&nfa);
    assert!(dfa.states.len() > 0);
    assert!(alphabet.len() > 0);
}

#[test]
fn test_minimize_dfa() {
    let input = "a*|b";
    let tokens = lexer::lex(input).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    let nfa = nfa::build_nfa(&ast);
    let (dfa, alphabet) = dfa::determinize(&nfa);
    let min_dfa = min::minimize(&dfa, &alphabet);
    assert!(min_dfa.states.len() > 0);
}

#[test]
fn test_simulate_dfa_accept() {
    let input = "a*";
    let tokens = lexer::lex(input).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    let nfa = nfa::build_nfa(&ast);
    let (dfa, alphabet) = dfa::determinize(&nfa);
    let result = sim::simulate_dfa(&dfa, &alphabet, "aaaa");
    assert!(result, "DFA should accept 'aaaa'");
}

#[test]
fn test_simulate_dfa_reject() {
    let input = "a*";
    let tokens = lexer::lex(input).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    let nfa = nfa::build_nfa(&ast);
    let (dfa, alphabet) = dfa::determinize(&nfa);
    let result = sim::simulate_dfa(&dfa, &alphabet, "b");
    assert!(!result, "DFA should reject 'b'");
}

#[test]
fn test_complex_regex() {
    let input = "(a|b)*abb";
    let tokens = lexer::lex(input).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    let nfa = nfa::build_nfa(&ast);
    let (dfa, alphabet) = dfa::determinize(&nfa);
    let min_dfa = min::minimize(&dfa, &alphabet);
    assert!(sim::simulate_dfa(&min_dfa, &alphabet, "abb"));
    assert!(sim::simulate_dfa(&min_dfa, &alphabet, "aabb"));
    assert!(!sim::simulate_dfa(&min_dfa, &alphabet, "ab"));
}
