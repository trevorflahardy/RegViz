= Future Work & Extensions
RegViz already exposes a substantial portion of the regex-to-automata toolchain, yet several extensions would deepen both capability and pedagogy.

== Syntax Surface Area
The automaton model anticipates richer operators than the current parser accepts. `BoxKind::KleenePlus` already exists, but the AST never emits it; adding a `+` postfix operator (distinct from alternation) would allow students to contrast one-or-more repetition with `*` in both the AST and the NFA. Similarly, character classes or predefined shorthand tokens could be layered on top of the lexer by desugaring them into alternations before passing the stream to the parser, preserving the simplicity of the Thompson builder.

== Guided Explanations
While errors are annotated with exact character indices, the UI could attach contextual hints pulled directly from `ParseErrorKind` and `LexErrorKind`, e.g., suggesting “Add an operand after '+'” instead of a generic message. On the acceptance side, each `SimulationStep` already knows which edges were traversed, so the UI could surface textual traces (“Consumed 'b' via state 2 → 5”) to accompany the visual highlights. These explanations would help students who are still internalizing the symbolic notation.

== Collaborative & Exportable Artifacts
Instructors frequently request sharable diagrams for slides or homework keys. Because every `GraphCanvas` ends in an Iced `Canvas`, it is already possible to render to an off-screen target; adding an “Export SVG/PNG” button would make it trivial to capture the current zoom, pan, and highlight state. A complementary feature could serialize `BuildArtifacts` (AST + NFA + DFA) to JSON so learners can document their reasoning or compare results between classmates.

== Beyond Deterministic Automata
Finally, the architecture leaves room for additional analyses layered on top of the minimized DFA. Examples include generating state equivalence proofs, constructing Myhill–Nerode distinguishability tables, or synthesizing counterexample strings when a test fails. Because the core crate cleanly separates parsing, construction, and simulation, these features can be developed incrementally without disturbing the existing UX.
