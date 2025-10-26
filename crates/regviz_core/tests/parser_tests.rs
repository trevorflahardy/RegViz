use regviz_core::core::parser::Ast;

#[test]
fn test_parser_simple() {
    let input = "a";
    let ast = Ast::build(input).unwrap();
    match ast {
        Ast::Atom('a') => {}
        _ => panic!("Expected Char('a') at root"),
    }
}

#[test]
fn test_parser_alt() {
    let input = "a|b";
    let ast = Ast::build(input).unwrap();
    match ast {
        Ast::Alt(left, right) => match (*left, *right) {
            (Ast::Atom('a'), Ast::Atom('b')) => {}
            _ => panic!("Expected Alt(a, b)"),
        },
        _ => panic!("Expected Alt node at root"),
    }
}

#[test]
fn test_parser_complex() {
    let input = "(a|b)*abb";
    let ast = Ast::build(input).unwrap();
    // Just check it's not an error and root is Star or Concat
    match ast {
        Ast::Concat(_, _) | Ast::Star(_) => {}
        _ => panic!("Expected Concat or Star at root"),
    }
}
