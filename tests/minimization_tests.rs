use cot_4210_final_proj::core::{dfa, lexer, min, nfa, parser};

#[test]
fn test_minimize_simple() {
    let input = "a*";
    let tokens = lexer::lex(input).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    let nfa = nfa::build_nfa(&ast);
    let (dfa, alphabet) = dfa::determinize(&nfa);
    let min_dfa = min::minimize(&dfa, &alphabet);
    assert!(min_dfa.states.len() > 0);
}

#[test]
fn test_minimize_complex() {
    let input = "(a|b)*abb";
    let tokens = lexer::lex(input).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    let nfa = nfa::build_nfa(&ast);
    let (dfa, alphabet) = dfa::determinize(&nfa);
    let min_dfa = min::minimize(&dfa, &alphabet);
    assert!(min_dfa.states.len() > 0);
    // Should accept 'abb' and 'aabb', but not 'ab'
}
