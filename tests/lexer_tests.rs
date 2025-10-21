use regviz::core::{lexer, tokens};

#[test]
fn test_lexer_simple() {
    let input = "a";
    let tokens = lexer::lex(input).unwrap();
    assert_eq!(tokens.len(), 2); // Char('a'), Eos
    assert_eq!(tokens[0].kind, tokens::TokenKind::Char('a'));
    assert_eq!(tokens[1].kind, tokens::TokenKind::Eos);
}

#[test]
fn test_lexer_complex() {
    let input = "a|b*";
    let tokens = lexer::lex(input).unwrap();
    assert_eq!(tokens.len(), 5); // Char('a'), Or, Char('b'), Star, Eos
    assert_eq!(tokens[0].kind, tokens::TokenKind::Char('a'));
    assert_eq!(tokens[1].kind, tokens::TokenKind::Or);
    assert_eq!(tokens[2].kind, tokens::TokenKind::Char('b'));
    assert_eq!(tokens[3].kind, tokens::TokenKind::Star);
    assert_eq!(tokens[4].kind, tokens::TokenKind::Eos);
}
