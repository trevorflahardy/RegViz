= Introduction & Motivation
== Motivation
Students typically encounter regular expressions, NFAs, and DFAs as separate artifacts on a chalkboard, with little tooling that lets them observe how each construct arises from the others. Existing visualizers are either closed-source, dated, or treat conversion as a black box, leaving few opportunities to inspect intermediate structures or reason about parser errors. RegViz addresses that gap with a transparent Rust pipeline: every stage—from lexer diagnostics to Hopcroft minimization—is implemented in-house and surfaced through the same interface, allowing learners to correlate theory with executable artifacts.

The project also emphasizes reproducibility. Because the backend is a normal Rust crate, the same functions that power the GUI can be imported into scripts, unit tests, or CLI experiments. Educators therefore gain both a research-grade reference implementation and an approachable visualization that can be handed to students without extra dependencies.

== Workflow Overview
RegViz presents a consistent, discoverable flow:

1. *Author a regex.* Typing into the left pane immediately re-runs `parser::Ast::build`, surfaces lexer/parse diagnostics (highlighted at the exact character index), and refreshes the alphabet extracted from the constructed NFA.
2. *Inspect structure.* Users can switch between the AST view and the automaton view via the bottom-right toggle. The AST canvas reports operator precedence through a tree layout, while the automaton canvas renders Thompson NFAs, determinized DFAs, or minimized DFAs with bounding boxes and tooltips.
3. *Simulate behavior.* Providing a test string enables the simulation controls: `build_nfa_trace` and `build_dfa_trace` precompute step-by-step snapshots, and the UI highlights active states, traversed transitions, and acceptance results in real time.

This loop turns abstract definitions into observable mechanics, reinforcing the equivalence between syntactic constructs and machine behavior.
