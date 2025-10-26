use regviz_core::core::{dfa, lexer, min, nfa, parser, sim};

#[test]
fn test_dfa_simple() {
    let input = "a";
    let tokens = lexer::lex(input).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    let nfa = nfa::build_nfa(&ast);
    let (dfa, alphabet) = dfa::determinize(&nfa);
    assert!(!dfa.states.is_empty());
    assert!(!alphabet.is_empty());
}

#[test]
fn test_dfa_accept() {
    let input = "a*";
    let tokens = lexer::lex(input).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    let nfa = nfa::build_nfa(&ast);
    let (dfa, alphabet) = dfa::determinize(&nfa);
    assert!(sim::simulate_dfa(&dfa, &alphabet, "aaaa"));
    assert!(sim::simulate_dfa(&dfa, &alphabet, ""));
}

#[test]
fn test_dfa_reject() {
    let input = "a*";
    let tokens = lexer::lex(input).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    let nfa = nfa::build_nfa(&ast);
    let (dfa, alphabet) = dfa::determinize(&nfa);
    assert!(!sim::simulate_dfa(&dfa, &alphabet, "b"));
    assert!(!sim::simulate_dfa(&dfa, &alphabet, "ab"));
}

#[test]
fn test_dfa_complex() {
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
