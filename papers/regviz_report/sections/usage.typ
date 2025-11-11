= Usage Scenarios & Pedagogical Flow
RegViz is designed around an instructor-friendly narrative in which every learner can replicate the same observations. A typical session proceeds as follows.

== 1. Author & Validate
The class begins by entering a pattern such as `(a+b)*c` into the left pane. As soon as the final parenthesis is typed, `parser::Ast::build` re-runs, and the status line reports both the alphabet and the number of NFA states produced. If a typo occurs—for example, an unmatched `)`—the `BuildError` emitted by the parser highlights the exact character index inside the input scrollbox, making it clear why the regex is invalid before any automata are generated.

== 2. Inspect Structural Views
Learners then toggle between the AST and NFA buttons in the bottom-right selector. The AST view, powered by `TreeLayoutStrategy`, reveals operator precedence: alternation nodes sit above concatenations, and postfix operators emphasize their operands with bounding boxes. Switching to the NFA view overlays those same operator boxes on top of the Thompson automaton, lending visual continuity between grammar-level reasoning and state-machine construction. When discussing determinism, the DFA and Min DFA buttons reuse the same canvas but swap in `VisualDfa` data built from cached `BuildArtifacts`.

== 3. Simulate Test Strings
With the structure understood, the instructor supplies a test string (e.g., `abbc`). The input is validated against the alphabet extracted from the NFA; if stray characters are included, the helper text explains which symbols are unsupported. Otherwise, `build_nfa_trace` or `build_dfa_trace` produces a sequence of `SimulationStep`s, and the transport controls become active. Each click on “Next” advances the cursor, recolors active states, and highlights the transitions that consumed the current character.

#figure(
  ```rust
  SimulationStep::new(
      idx + 1,
      Some(*symbol),
      next.clone(),
      traversed,
      accepting,
  )
  ```
  ,
  caption: [Per-symbol snapshots returned by `build_nfa_trace`],
) <fig:usage-trace>

== 4. Interpret Outcomes
The summary lines under the controls report the current prefix, the set of active states (formatted as `{0, 1, 5}` for NFAs), and whether the automaton is in an accepting configuration. Acceptance and rejection messages use the same terminology students see in lecture (“Input string is accepted.”). By linking textual feedback with graphical cues, RegViz clarifies why a string is accepted or rejected rather than reducing simulation to a binary verdict.

This flow scales from quick in-class demonstrations to lab assignments: students can follow the same four steps with their own patterns and capture screenshots of each view to document their reasoning.
