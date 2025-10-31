use std::env;

use regviz_core::core::{dfa, nfa, parser, sim};

fn main() {
    let mut args = env::args().skip(1);
    let pattern = match args.next() {
        Some(s) => s,
        None => {
            eprintln!("Usage: regviz <pattern> [input-string]");
            return;
        }
    };

    let input = args.next();

    // Lex
    match parser::Ast::build(&pattern) {
        Ok(ast) => {
            println!("Pattern: {}", pattern);
            println!("AST: {}", ast);

            // Build NFA
            let nfa = nfa::Nfa::build(&ast);
            println!(
                "NFA: states={} start={} accepts={} edges={}",
                nfa.states.len(),
                nfa.start,
                nfa.accepts.len(),
                nfa.edges.len()
            );

            // Determinize -> DFA
            let (dfa, alphabet) = dfa::determinize(&nfa);
            println!(
                "DFA: states={} start={} accepts={} alphabet={:?}",
                dfa.states.len(),
                dfa.start,
                dfa.accepts.len(),
                alphabet
            );

            // If user provided an input string, simulate both NFA and DFA
            if let Some(s) = input {
                let nfa_accepts = sim::nfa_accepts(&nfa, &s);
                let dfa_accepts = sim::simulate_dfa(&dfa, &alphabet, &s);
                println!("Input: {:?}", s);
                println!("NFA accepts: {}", nfa_accepts);
                println!("DFA accepts: {}", dfa_accepts);
            }
        }
        Err(e) => eprintln!("Build error: {:?}", e),
    }
}
