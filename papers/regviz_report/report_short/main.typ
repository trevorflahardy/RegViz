#import "styles.typ": *

#show: apply-styles


#title-page(
  "RegViz: A Rust-Based Regular Expression Visualizer",
  "Implementation and Analysis of Automata Construction and Visualization",
  "The University of South Florida, COT 4210 - Automata Theory and Formal Languages, Fall 2025
  Trevor Flahardy, Hung Tran",
)

#pagebreak()

// The table of contents
#outline()
#pagebreak()


= Abstract
RegViz is a Rust application designed to make the relationship between regular expressions and finite automata concrete and interactive. The tool takes a user-provided expression, constructs its abstract syntax tree (AST), builds the corresponding Thompson ε-NFA, determinizes it into a DFA, and optionally minimizes it. Each step is visualized through a responsive GUI that displays operator structure, state transitions, and simulation traces. The goal was not to create a full-scale compiler, but to deliver a transparent learning environment that helps students see how the formal constructs covered in class materialize during execution. This report summarizes the task, implementation approach, results, and unexpected insights gained throughout the development process.


= Introduction & Motivation
Students typically encounter regular expressions, NFAs, and DFAs as isolated topics, even though the equivalence between them is a core concept in automata theory. Most existing visualizers either hide intermediate steps or are too limited to be useful for understanding the mechanical details of each construction. Our objective was to build a tool that exposes every stage: tokens, parsing decisions, NFA fragments, determinized states, and minimized partitions.

The motivation was simple: provide a system where a user can type a regex and immediately see how its structure determines the automaton that recognizes its language. Instead of static diagrams, learners interact with real, executable artifacts—bridging theory and practice.

= Background
Regular expressions, ε-NFAs, and DFAs describe the same class of languages. Thompson's construction converts a regex into an ε-NFA; subset construction removes nondeterminism; Hopcroft's algorithm minimizes the DFA. These transformations formed the conceptual backbone of RegViz. Only a subset of regex syntax was incorporated—literals, alternation (`+`), concatenation, parentheses, Kleene star (`*`), and optional (`?`)—mirroring the constructs covered in class and directly mapping to automata-level operations.

= Implementation Overview
== Pipeline Summary
The RegViz backend (`regviz_core`) implements a compilation pipeline with five stages:
1. *Lexical analysis:* Converts the raw input string into a token stream with proper handling of literals, escaped operators, and epsilon (`\e`).
2. *Pratt parser:* Builds an AST that encodes operator precedence, associativity, and implicit concatenation.
3. *Thompson construction:* Translates AST nodes into NFA fragments, complete with ε-transitions and bounding-box metadata for visualization.
4. *Subset construction:* Converts the ε-NFA into an equivalent DFA by computing ε-closures and move operations.
5. *Hopcroft minimization:* Partitions DFA states and merges indistinguishable states to obtain the minimal machine.

The frontend (`regviz_app`) presents these stages through an interactive interface. Users can switch between AST, NFA, DFA, and Min-DFA views, supply test strings, and step through automaton execution.

== Key Design Choices
To keep the project aligned with course content, the implementation avoids shortcuts or high-level regex engines. Thompson construction, subset construction, and minimization were all built manually from scratch. The application also emits bounding-box metadata—each operator in the AST corresponds to a colored region in the NFA visualization—making it easier for students to connect grammar structure to automaton layout.

= Summary of Results
RegViz successfully demonstrates the full lifecycle of converting a regular expression into its corresponding automata. The core deliverables achieved include:

- *Reliable tokenization and Pratt parsing*, with detailed error messages that highlight the precise character index of malformed input.
- *Fully functional Thompson NFA construction*, producing correct ε-edge structures for concatenation, alternation, Kleene star, and optional.
- *Determinization and minimization outputs* that behave exactly as expected from the algorithms taught in class.
- *Interactive visualization*, where AST nodes, NFA fragments, and DFA transitions update immediately as the user types.
- *Simulation tools* that highlight the active states and traversed transitions step-by-step for both NFAs and DFAs.

Across all tests—including nested operators, ambiguous cases, and malformed patterns—the pipeline behaved as intended. The minimizer consistently reduced redundant states and confirmed equivalence across different but structurally similar expressions (e.g., `a*` vs `(a*)*`).

= Surprising Observations
During development, several insights emerged:

1. *Implicit concatenation is deceptively complex.*
  Users expect `ab` to behave like `a.b`, which requires the parser to synthesize infix operators on the fly. Getting precedence and associativity correct required careful handling.

2. *Epsilon-closure strongly shapes DFA structure.*
  For patterns involving stars or optional operators, ε-closure dramatically reduces the number of transitions needed in the determinized machine. Seeing this in real time made the relationship between ε-paths and determinism much clearer.

3. *Minimization often reveals identical languages behind very different-looking regexes.*
  Expressions like `(a+aa)` and `aa` reduce to the same minimal DFA. This was a useful demonstration of why regular expressions are often more redundant than they appear.

4. *Visualization forces correctness.*
  Any bug in state ordering, ε-closure, or transition generation becomes visually obvious. Rendering the machine was effectively another layer of debugging—something not usually encountered in abstract automata coursework.

5. *Students engage more deeply when they can “see” determinism.*
  When stepping through the DFA version of a pattern they already simulated on the NFA, the equivalence becomes self-evident rather than a theorem taken on trust.

= Conclusion
RegViz illustrates the entire equivalence chain -- regex → ε-NFA → DFA → minimal DFA -- through an implementation written from first principles. The tool provides a transparent, inspectable view of automata construction, helping students connect textbook definitions to concrete runtime behavior. By exposing every stage, simulating test strings, and visually correlating operators with their automaton effects, RegViz reduces the abstraction barrier that often makes automata theory difficult for beginners.

The project succeeds as an educational companion: it is simple enough for new learners to follow, yet detailed enough to faithfully represent the algorithms taught in COT 4210. Through building and using this tool, the tight relationship between syntax and machine behavior becomes far more intuitive, reinforcing the foundational ideas of regular languages and automata.
