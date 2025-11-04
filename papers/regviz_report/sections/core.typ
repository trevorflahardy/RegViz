= RegViz Core Implementation
== Module Structure & Build Artifacts
Present the workspace layout and the BuildArtifacts container that stores ASTs, NFAs, alphabets, and lazily computed DFAs/minimized DFAs for downstream consumers.

== Lexical Analysis
Detail token categories, escape handling, and lexing errors, including unit tests that validate tokenization rules.

== Pratt Parser & Operator Precedence
Explain the Pratt parser setup with infix/postfix definitions, implicit concatenation handling, and rich parse errors with source positions.

== AST Representation & Display
Describe the AST node taxonomy (Epsilon, Atom, Concat, Alt, Star, Opt) and its S-expression formatter; note extensive parser test coverage across operator combinations and error cases.

== Thompson NFA Construction with Metadata
Show how the builder wraps fragment creation in with_box, tracking parent/child relationships and the exact states created under each AST operator to emit bounding-box metadata for visualization.
Summarize fragment generation for literals, concatenation, alternation, Kleene star, and optional constructs, emphasizing the stored start/accept structure and explicit epsilon edges.

== Alphabet Extraction & Build Pipeline
Note how NFAs derive sorted symbol alphabets to gate simulations and drive DFA tables.

== DFA Determinization
Detail the subset construction algorithm, epsilon-closure seeding, transition table materialization, and accepting-state detection.

== DFA Minimization
Outline Hopcroft's partition refinement stages, worklist management, and reconstruction of minimized transition tables.

== Automaton Simulation Algorithms
Explain NFA epsilon-closure traversal, DFA stepping, and acceptance checks reused by both the CLI and frontend simulations.

== Error Handling & Diagnostics
Cover LexError, ParseError, and the aggregated BuildError, highlighting how errors carry indices and operator context for UI feedback.

== Command-Line Interface
Summarize the CLI entry point that builds AST/NFA/DFA, reports automaton statistics, and evaluates an optional input string for quick validation workflows.

== Testing & Quality Assurance
Mention the core crate's unit tests spanning lexing, parsing, and automaton behaviors to assert correctness across common and edge-case patterns.
