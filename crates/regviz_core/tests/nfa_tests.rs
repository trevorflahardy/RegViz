use regviz_core::core::{nfa, parser};

#[test]
fn test_nfa_simple() {
    let input = "a";
    let ast = parser::Ast::build(input).unwrap();
    let nfa = nfa::build_nfa(ast);
    assert!(!nfa.states.is_empty());
    assert!(!nfa.edges.is_empty());
}

#[test]
fn test_nfa_complex() {
    let input = "(a+b)*abb";
    let ast = parser::Ast::build(input).unwrap();
    let nfa = nfa::build_nfa(ast);
    assert!(!nfa.states.is_empty());
    assert!(!nfa.edges.is_empty());
    assert!(!nfa.accepts.is_empty());
}
