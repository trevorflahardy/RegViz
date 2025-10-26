use regviz_core::core::{dfa, min, nfa, parser};

#[test]
fn test_minimize_simple() {
    let input = "a*";
    let ast = parser::Ast::build(input).unwrap();
    let nfa = nfa::build_nfa(&ast);
    let (dfa, alphabet) = dfa::determinize(&nfa);
    let min_dfa = min::minimize(&dfa, &alphabet);
    assert!(!min_dfa.states.is_empty());
}

#[test]
fn test_minimize_complex() {
    let input = "(a|b)*abb";
    let ast = parser::Ast::build(input).unwrap();
    let nfa = nfa::build_nfa(&ast);
    let (dfa, alphabet) = dfa::determinize(&nfa);
    let min_dfa = min::minimize(&dfa, &alphabet);
    assert!(!min_dfa.states.is_empty());
    // Should accept 'abb' and 'aabb', but not 'ab'
}
