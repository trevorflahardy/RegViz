use regviz_core::core::{dfa, min, nfa, parser, sim};

#[test]
fn test_dfa_simple() {
    let input = "a";
    let ast = parser::Ast::build(input).unwrap();
    let nfa = nfa::Nfa::build(&ast);
    let dfa = dfa::determinize(&nfa);
    assert!(!dfa.states.is_empty());
    assert!(!dfa.alphabet.is_empty());
}

#[test]
fn test_dfa_accept() {
    let input = "a*";
    let ast = parser::Ast::build(input).unwrap();
    let nfa = nfa::Nfa::build(&ast);
    let dfa = dfa::determinize(&nfa);
    assert!(sim::simulate_dfa(&dfa, "aaaa"));
    assert!(sim::simulate_dfa(&dfa, ""));
}

#[test]
fn test_dfa_reject() {
    let input = "a*";
    let ast = parser::Ast::build(input).unwrap();
    let nfa = nfa::Nfa::build(&ast);
    let dfa = dfa::determinize(&nfa);
    assert!(!sim::simulate_dfa(&dfa, "b"));
    assert!(!sim::simulate_dfa(&dfa, "ab"));
}

#[test]
fn test_dfa_complex() {
    let input = "(a+b)*abb";
    let ast = parser::Ast::build(input).unwrap();
    let nfa = nfa::Nfa::build(&ast);
    let dfa = dfa::determinize(&nfa);
    let min_dfa = min::minimize(&dfa);
    assert!(sim::simulate_dfa(&min_dfa, "abb"));
    assert!(sim::simulate_dfa(&min_dfa, "aabb"));
    assert!(!sim::simulate_dfa(&min_dfa, "ab"));
}
