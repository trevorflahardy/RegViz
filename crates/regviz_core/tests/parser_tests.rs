use regviz_core::core::{ast, lexer, parser};

#[test]
fn test_parser_simple() {
    let input = "a";
    let tokens = lexer::lex(input).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    match ast {
        ast::Ast::Char('a') => {}
        _ => panic!("Expected Char('a') at root"),
    }
}

#[test]
fn test_parser_alt() {
    let input = "a|b";
    let tokens = lexer::lex(input).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    match ast {
        ast::Ast::Alt(left, right) => match (*left, *right) {
            (ast::Ast::Char('a'), ast::Ast::Char('b')) => {}
            _ => panic!("Expected Alt(a, b)"),
        },
        _ => panic!("Expected Alt node at root"),
    }
}

#[test]
fn test_parser_complex() {
    let input = "(a|b)*abb";
    let tokens = lexer::lex(input).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    // Just check it's not an error and root is Star or Concat
    match ast {
        ast::Ast::Concat(_, _) | ast::Ast::Star(_) => {}
        _ => panic!("Expected Concat or Star at root"),
    }
}
