use cot_4210_final_proj::core::{lexer, nfa, parser};

#[test]
fn test_nfa_simple() {
    let input = "a";
    let tokens = lexer::lex(input).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    let nfa = nfa::build_nfa(&ast);
    assert!(nfa.states.len() > 0);
    assert!(nfa.edges.len() > 0);
}

#[test]
fn test_nfa_complex() {
    let input = "(a|b)*abb";
    let tokens = lexer::lex(input).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    let nfa = nfa::build_nfa(&ast);
    assert!(nfa.states.len() > 0);
    assert!(nfa.edges.len() > 0);
    assert!(nfa.accepts.len() > 0);
}
