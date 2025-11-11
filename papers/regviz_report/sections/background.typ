= Automata & Regular Expression Background
Regular expressions, NFAs, and DFAs form an equivalence class over the regular languages: every regex can be expanded into some ε-NFA via Thompson's construction; every ε-NFA can be determinized into a DFA through subset construction; and every DFA can be minimized without changing the language it recognizes. RegViz mirrors that chain step-by-step. The parser produces an AST rooted in the operators of regular algebra, the Thompson builder emits an `Nfa` with explicit ε-transitions, `dfa::determinize` applies the powerset algorithm to derive a deterministic machine, and `min::minimize` executes Hopcroft's partition refinement to collapse equivalent states. By surfacing all three artifacts, the application demonstrates why acceptance in one domain implies acceptance in the others.

To keep the focus on core automata concepts, the supported regex syntax matches what is typically covered in an introductory theory course:

- *Alphanumeric literals (`a–z`, `A–Z`, `0–9`).* Each literal expands to two NFA states connected by a symbol-labeled edge.
- *Epsilon escape (`\e`).* The lexer maps this to `Token::Epsilon`, enabling empty-string transitions and epsilon-only subexpressions.
- *Grouping via parentheses.* Balanced parentheses drive recursive descent inside the Pratt parser and later become nested bounding boxes during visualization.
- *Alternation (`+`).* Parsed as a low-precedence infix operator that creates parallel NFA branches.
- *Concatenation (implicit or explicit `.`).* Implied concatenation is synthesized by the parser when two primary expressions appear back-to-back; `.` offers an explicit variant for clarity.
- *Kleene star (`*`).* Implemented as a high-precedence postfix operator that feeds back into the inner fragment through ε-edges.
- *Optional (`?`).* Another postfix operator that chooses between the inner fragment and the empty string.

The lexer also allows escaped operator characters (e.g., `\+`, `\*`) so the same syntax can be used to demonstrate situations where symbols serve as data rather than structure. This carefully scoped grammar ensures that every feature has a direct counterpart in automata theory, making it easier to map syntax to state-machine semantics.
