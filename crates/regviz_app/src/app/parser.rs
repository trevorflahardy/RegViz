use regviz_core::core::{BuildArtifacts, lexer, nfa, parser};

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
        // Empty input is not an error, just clears everything
        if self.input.is_empty() {
            self.error = None;
            self.build_artifacts = None;
            return;
        }

        // Try to lex the input into tokens
        match lexer::lex(self.input.trim()) {
            Ok(tokens) => {
                // Try to parse tokens into an AST
                match parser::parse(&tokens) {
                    Ok(ast) => {
                        // Build NFA from AST
                        let nfa = nfa::build_nfa(&ast);
                        let alphabet = nfa.alphabet();

                        // Success: store artifacts and clear error
                        self.build_artifacts = Some(BuildArtifacts {
                            ast,
                            nfa,
                            alphabet,
                            dfa: None,
                            min_dfa: None,
                        });
                        self.error = None;
                    }
                    Err(e) => {
                        // Parse error
                        self.error = Some(format!("Parse error: {}", e));
                        self.build_artifacts = None;
                    }
                }
            }
            Err(e) => {
                // Lex error
                self.error = Some(format!("Lex error: {}", e));
                self.build_artifacts = None;
            }
        }
    }
}
