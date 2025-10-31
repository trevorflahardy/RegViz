use regviz_core::core::{BuildArtifacts, nfa::Nfa, parser};

use super::state::App;

impl App {
    /// Attempts to lex and parse the current input, updating build artifacts or error state.
    ///
    /// This function is called whenever the user changes the input text. It performs:
    /// 1. Lexical analysis (tokenization)
    /// 2. Syntax analysis (AST construction)
    /// 3. NFA construction from the AST
    /// 4. Alphabet extraction
    ///
    /// On success, `build_artifacts` is populated and `error` is cleared.
    /// On failure, `error` is set and `build_artifacts` is cleared.
    pub fn lex_and_parse(&mut self) {
        // Try to lex the input into tokens
        match parser::Ast::build(self.input.trim()) {
            Ok(ast) => {
                let nfa = Nfa::build(&ast);
                let alphabet = nfa.alphabet();
                self.build_artifacts = Some(BuildArtifacts {
                    ast,
                    nfa,
                    alphabet,
                    dfa: None,
                    min_dfa: None,
                });
                self.error = None;
                self.simulation.reset_cursor();
                self.refresh_simulation_trace();
            }
            Err(e) => {
                // Lex error
                self.error = Some(format!("Build error: {}", e));
                self.build_artifacts = None;
                self.simulation.clear_trace();
                self.simulation_error = None;
            }
        }
    }
}
