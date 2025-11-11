= RegViz Core Implementation
== Module Structure & Build Artifacts
The RegViz core is organized as a Rust workspace with a clear separation of concerns across specialized modules. The crate's source tree is structured as follows:

```
src/
├── core/
│   ├── automaton.rs    // Core automaton types, edges, and bounding-box metadata
│   ├── dfa.rs          // Deterministic finite automaton (subset construction)
│   ├── lexer.rs        // Lexical analysis / tokenization
│   ├── min.rs          // DFA minimization (Hopcroft's algorithm)
│   ├── mod.rs          // Module exports and the BuildArtifacts container
│   ├── nfa.rs          // Thompson NFA construction with bounding-box metadata
│   ├── parser.rs       // Pratt parser and AST representation
│   └── sim.rs          // Automaton simulation utilities (NFA & DFA)
├── errors.rs           // Error and diagnostic types
├── lib.rs              // Crate root and public re-exports
└── main.rs             // CLI runner / example usage
```

At the heart of the module interface sits the `BuildArtifacts` container, defined in `core/mod.rs`. This structure serves as the holder for the various stages of the regex compilation pipeline:

1. *AST*: The abstract syntax tree produced by the Pratt parser.
2. *NFA*: The Thompson NFA constructed from the AST, complete with bounding-box metadata.
3. *Alphabet*: The sorted list of symbols extracted from the NFA.
4. *DFA*: An optional deterministic finite automaton derived from the NFA via subset construction.
5. *Minimized DFA*: An optional minimized version of the DFA using Hopcroft's algorithm.


#figure(
  ```rust
  pub struct BuildArtifacts {
      pub ast: Ast,
      pub nfa: Nfa,
      pub alphabet: Vec<char>,
      pub dfa: Option<Dfa>,
      pub min_dfa: Option<Dfa>,
  }
  ```,
)

Notably, while defined in the core crate, `BuildArtifacts` serves purely as an interface boundary—the core crate's own modules do not consume it internally, instead passing individual AST/NFA/DFA structures directly between functions. This keeps the core library focused on transformation logic while `BuildArtifacts` acts as a convenient packaging mechanism for external consumers.

The `BuildArtifacts` design allows for *lazy computation*: while the AST, NFA, and extracted alphabet are always present (as they form the foundational stages of any regex compilation), the DFA and minimized DFA are optional fields. DFA and Minimizied DFA are only computed when actually needed — important for interactive visualization workflows where users may inspect intermediate representations without requiring full automaton determinization.

The container provides a straightforward constructor `BuildArtifacts::new(ast, nfa, alphabet)` that initializes the mandatory fields while leaving DFA slots unpopulated. Downstream code can populate these on-demand by invoking the determinization and minimization routines independently, maintaining flexibility in the compilation pipeline.

== Lexical Analysis

The lexer transforms raw regex strings into a token stream suitable for parsing. Implemented in `lexer.rs`, it defines a two-tier token taxonomy that separates structural elements from operators and literals.

The `Token` enum captures six fundamental categories:

- *`Epsilon`* — represents the empty string (entered as `\e`)
- *`Literal(char)`* — alphanumeric characters or escaped special symbols
- *`Op(OpToken)`* — the four supported operators: `+` (alternation), `*` (Kleene star), `.` (explicit concatenation), and `?` (optional)
- *`LParen`* / *`RParen`* — grouping delimiters
- *`Eof`* — sentinel marking input exhaustion

The lexer processes input character-by-character, maintaining precise character indices for downstream error reporting. Its escape mechanism uses backslash notation: `\e` denotes epsilon, while `\+`, `\*`, `\.`, `\?`, `\(`, `\)` allow these meta-characters to appear as literals in the pattern. A trailing backslash triggers a `DanglingEscape` error, while non-alphanumeric, non-operator characters (excluding whitespace, which is silently skipped) produce an `InvalidCharacter` error.

The `Lexer` struct stores tokens in reverse order to support efficient stack-like consumption via `advance()` and `peek()` methods. Each token is paired with its original character index from the input string, enabling the parser to attach precise source locations to syntax errors. The lexer reports the total character count alongside the EOF token, facilitating end-of-input error positioning.

Token display formatting uses Unicode symbols for readability—epsilon renders as `ε` and EOF as `<EOF>`—while operator tokens preserve their ASCII representations.

The test suite validates tokenization rules across five scenarios:
1. *Basic operator sequences* (`a+b*c`) — verifies correct operator classification and index tracking
2. *Parenthesized expressions* (`(a.b)+c`) — confirms delimiter recognition and nesting preparation
3. *Escape handling* (`a\+b\*c`) — ensures meta-characters become literals when escaped
4. *Invalid character rejection* (`a+b$c`) — validates error reporting with correct position for disallowed symbols
5. *Dangling escape detection* (`a+b\`) — catches incomplete escape sequences at input boundaries

This comprehensive coverage ensures the lexer reliably transforms valid regex syntax into a well-formed token stream while providing actionable error diagnostics for malformed input.

== Pratt Parser & Operator Precedence

The parser transforms token streams into abstract syntax trees using a Pratt parsing algorithm that elegantly handles operator precedence and associativity. Implemented in `parser.rs`, it supports both explicit operators (consumed from the token stream) and implicit concatenation (synthesized without token consumption).

=== AST Representation & Display

The `Ast` enum defines a six-node taxonomy representing the complete structure of regular expressions:

- *`Epsilon`* — the empty string (ε)
- *`Atom(char)`* — literal characters
- *`Concat(Box<Ast>, Box<Ast>)`* — sequential composition of two sub-expressions
- *`Alt(Box<Ast>, Box<Ast>)`* — choice between two alternatives
- *`Star(Box<Ast>)`* — zero-or-more repetition (Kleene star)
- *`Opt(Box<Ast>)`* — zero-or-one occurrence (optional)

Binary operators (`Concat` and `Alt`) box their children to maintain constant enum size, while unary operators (`Star` and `Opt`) similarly box their single operand.

The `Display` trait implementation renders ASTs in unambiguous S-expression format, using prefix notation with explicit operator symbols:
- Concatenation: `(. a b)`
- Alternation: `(+ a b)`
- Kleene star: `(* a)`
- Optional: `(? a)`
- Atoms and epsilon render directly: `a` and `ε`

This canonical representation eliminates ambiguity in nested structures—`(. (. a b) c)` clearly shows left-associative concatenation—and serves as the foundation for test assertions. For example, the pattern `a*b+c*` produces `(+ (. (* a) b) (* c))`, explicitly encoding the precedence and associativity rules.

=== Operator Binding Powers

The parser defines binding powers that encode precedence relationships:

- *Alternation (`+`)* — left binding power 1, right binding power 2 (lowest precedence, right-associative)
- *Concatenation (`.` or implicit)* — left binding power 3, right binding power 4 (middle precedence, right-associative)
- *Postfix operators (`*`, `?`)* — binding power 5 (highest precedence)

This hierarchy ensures that `a+b*c` parses as `(+ a (* b c))` (star binds tighter than concatenation, concatenation binds tighter than alternation) and that `a+b+c` becomes `(+ (+ a b) c)` (right-associativity produces left-leaning trees).

=== Implicit Concatenation

A distinguishing feature of this parser is its automatic insertion of concatenation operators. When the parser encounters a literal, epsilon, or left parenthesis after completing a left-hand expression, it synthesizes an implicit `OpToken::Dot` without consuming any token. This allows `ab` to parse identically to `a.b`, producing `(. a b)`. The logic treats prefix operators specially: if the next token could begin a new expression (e.g., a hypothetical prefix operator), implicit concatenation applies rather than treating it as an infix operator.

=== Two-Phase Parsing Loop

The `parse` function operates in two phases:

1. *Primary Parsing* — consumes the next token to establish an initial left-hand side (`lhs`):
  - Literals and epsilon become `Ast::Atom` and `Ast::Epsilon` nodes
  - Left parentheses trigger recursive sub-expression parsing, expecting a matching right parenthesis
  - Prefix operators (currently unsupported but structurally accommodated) would recursively parse their operands

2. *Pratt Loop* — repeatedly inspects the next token without consuming it, deciding whether to:
  - Apply a postfix operator (checked first)
  - Apply an infix operator (explicit or implicit concatenation)
  - Stop parsing when the next token's binding power falls below `min_bp` or upon encountering a delimiter (closing parenthesis or EOF)

The parser checks postfix operators before infix operators, as postfixes naturally bind tighter than any infix operator. Implicit concatenation is treated as an infix operator with its own binding powers.

=== Error Reporting with Source Positions

Every parse error carries the character index where the problem occurred, enabling precise error messages:

- `UnexpectedPrefixOperator` — reports operators like `+` or `.` appearing in prefix position
- `MismatchedLeftParen` — detects unclosed parentheses, reporting the token found instead of `)`
- `EmptyParentheses` — rejects patterns like `()` or `a+()`
- `RightParenWithoutLeft` — catches stray closing parentheses
- `UnexpectedEof` — identifies incomplete expressions like `a+` or `ab(`

=== Test Coverage

The parser test suite comprises 24 test cases across seven categories:

1. *Basic structures* — empty input, single literals, nested parentheses
2. *Parenthesis errors* — mismatched, empty, or missing delimiters
3. *Concatenation* — implicit (`ab`) and explicit (`a.b`) forms, multi-operand chains
4. *Alternation* — single and chained alternations, mixed with concatenation
5. *Postfix operators* — Kleene star and optional, including nested applications (`a**`)
6. *Complex combinations* — mixed infix/postfix precedence tests like `a*b+c*` and `(a+b)*c`
7. *Epsilon handling* — epsilon in concatenation, alternation, star, and parentheses
8. *Error cases* — prefix operator misuse, unexpected EOF, parenthesis mismatches

This comprehensive coverage validates precedence rules, associativity, implicit concatenation logic, and error diagnostics across common and edge-case patterns.

== Thompson NFA Construction with Metadata

The NFA builder implements Thompson's construction algorithm with an additional metadata layer that tracks the hierarchical structure of AST operators. Implemented in `nfa.rs`, it produces NFAs augmented with bounding boxes that record which states belong to which AST sub-expression—critical information for interactive visualization.

=== Fragment-Based Construction with Bounding Boxes

The `Builder` struct maintains a `box_stack` that tracks the current nesting context during construction. Every fragment-building operation wraps its logic in `with_box(kind, f)`, which:

1. Creates a new `BoundingBox` record with a unique ID, linking it to the current parent box (if any)
2. Pushes the box onto the stack, making it the active context
3. Executes the fragment-building closure `f`, during which all `new_state()` calls automatically register themselves in this box's state list
4. Pops the box from the stack upon completion

This design ensures that the final `Nfa` contains a `boxes` vector describing the complete parent-child hierarchy of AST nodes, with each box storing the exact set of states created for that operator. The six `BoxKind` variants (`Literal`, `Concat`, `Alternation`, `KleeneStar`, `Optional`, and a root variant) mirror the AST taxonomy, enabling visualizers to draw operator boundaries around their constituent states.

=== Thompson Fragment Generation

Each AST node type produces a `Fragment` structure containing a unique `start` and `accept` state ID. The builder guarantees that each fragment has exactly one accepting state, simplifying composition:

*Epsilon* (`ε`) — allocates a single state that serves as both start and accept, with no outgoing edges. This represents the empty string.

#figure(
  image("../images/nfa/epsilon.png", width: 60%),
  caption: [NFA for `'\e'`],
) <fig:nfa-epsilon>

*Literal* (`Atom(c)`) — creates two states connected by a single edge labeled with the character `c`: `start --c--> accept`.

#figure(
  image("../images/nfa/literal.png", width: 60%),
  caption: [NFA for `'a'`],
) <fig:nfa-literal>

*Concatenation* (`Concat(lhs, rhs)`) — builds both sub-fragments independently, then joins them with an epsilon transition from the left fragment's accept state to the right fragment's start state: `left_accept --ε--> right_start`. The resulting fragment's start is `left_start` and accept is `right_accept`.

#figure(
  image("../images/nfa/concatenation.png", width: 60%),
  caption: [NFA for `'ab'`],
) <fig:nfa-concatenation>

*Alternation* (`Alt(lhs, rhs)`) — constructs both branches, then creates new start and accept states. The new start epsilon-transitions to both branch starts, and both branch accepts epsilon-transition to the new accept state.

#figure(
  image("../images/nfa/alternation.png", width: 60%),
  caption: [NFA for `'a+b'`],
) <fig:nfa-alternation>

*Kleene Star* (`Star(inner)`) — wraps the inner fragment with new start and accept states. Four epsilon edges implement zero-or-more semantics:
- `start --ε--> inner_start` (enter loop)
- `start --ε--> accept` (skip loop, accept empty)
- `inner_accept --ε--> inner_start` (repeat loop)
- `inner_accept --ε--> accept` (exit loop)

#figure(
  image("../images/nfa/star.png", width: 60%),
  caption: [NFA for `'a*'`],
) <fig:nfa-kleene-star>

*Optional* (`Opt(inner)`) — similar to star but omits the back-loop edge:
- `start --ε--> inner_start` (take the optional path)
- `start --ε--> accept` (skip it, accept empty)
- `inner_accept --ε--> accept` (exit after one match)

#figure(
  image("../images/nfa/optional.png", width: 60%),
  caption: [NFA for `'a?'`],
) <fig:nfa-optional>

=== Adjacency Representation and Finalization

The builder maintains a `Vec<Vec<Transition>>` adjacency list, where each state's outgoing transitions are stored in a vector. The `Transition` struct pairs a destination state ID with an `EdgeLabel` (either `Sym(char)` for literal symbols or `Eps` for epsilon transitions).

Upon finalization, the builder:
1. Sorts each state's adjacency list by destination for consistency
2. Flattens the adjacency structure into a concrete `edges` vector for uniform iteration
3. Retains both representations—the flattened list for display and the adjacency lists for efficient traversal during simulation

Each state also carries its parent `box_id`, linking it back to the bounding box metadata for visualization queries like "which states belong to this star operator?"

=== Alphabet Extraction

The `alphabet()` method scans all transitions, collecting characters that appear as edge labels (excluding epsilon). It returns a sorted `Vec<char>` that serves multiple purposes:
- Gates simulation algorithms—only symbols in the alphabet can drive transitions
- Defines DFA column indices—the determinization process builds transition tables indexed by this alphabet
- Validates input strings—characters outside the alphabet immediately reject

This sorted alphabet becomes a foundational contract between NFA construction and downstream DFA operations.

=== Test Coverage

The NFA test suite validates six construction scenarios with precise state and edge counting:

1. *Epsilon* — confirms single-state, zero-edge structure
2. *Single character* — verifies two-state literal fragment with one symbol edge
3. *Concatenation* (`ab`) — checks epsilon-joined fragments with four states, three edges
4. *Alternation* (`a+b`) — validates diamond structure with six states, six edges (four epsilon, two symbol)
5. *Kleene star* (`a*`) — asserts four-state, five-edge structure with correct loop-back edges
6. *Optional* (`a?`, `\e?`) — confirms three/four-state structures with bypass epsilon edges
7. *Complex nesting* (`(ab+c)*`) — tests composed operators with ten states, twelve edges

Each test inspects the exact adjacency structure, verifying that edge labels and destinations match Thompson's construction rules. This comprehensive coverage ensures both correctness and structural consistency across all operator combinations.

== DFA Determinization

The determinization process transforms NFAs into equivalent DFAs through subset construction, eliminating nondeterminism by treating sets of NFA states as single DFA states. Implemented in `dfa.rs`, the algorithm produces a `Dfa` structure with an explicit transition table indexed by state and alphabet symbol.

=== Subset Construction Algorithm

The `Determinizer` struct orchestrates the conversion through a worklist algorithm:

1. *Initialization* — computes the epsilon-closure of the NFA's start state to seed the DFA's start state (ID 0). This closure becomes the first entry in both the state map and work queue.

2. *State Exploration* — pops an NFA state subset from the queue and processes each alphabet symbol:
  - Computes the `move` operation: collects all NFA states reachable from the current subset via edges labeled with the symbol
  - Applies epsilon-closure to the moved set, yielding the next DFA state's corresponding NFA subset
  - Looks up or inserts this subset in the state map, assigning it a unique DFA state ID
  - Records the transition in the `trans` table at `[current_state][symbol_index]`

3. *Termination* — continues until the queue empties, guaranteeing all reachable DFA states have been explored.

The algorithm uses an `IndexMap` to maintain insertion order for deterministic state ID assignment—the first subset encountered always receives ID 0, the second ID 1, and so forth. This ensures consistent DFA structures across multiple runs for the same NFA.

=== Epsilon-Closure Seeding

The start state construction illustrates epsilon-closure's critical role: the NFA's start state (possibly with outgoing epsilon transitions) expands into a set containing itself plus all states reachable via epsilon paths. For example, a Kleene star's start state immediately includes both the inner fragment's start and the outer accept state via the bypass epsilon edge.

Every transition computation repeats this closure operation after the symbol-labeled move, ensuring the DFA "resolves" all epsilon ambiguity before recording the next state.

=== Transition Table Materialization

The `trans` field stores a two-dimensional structure: `Vec<Vec<Option<StateId>>>`, where `trans[s][i]` holds the destination state when DFA state `s` reads the `i`-th alphabet symbol. A `None` entry indicates a transition to an implicit dead state (no valid continuation).

The `ensure_capacity` method grows this table on-demand as new DFA states are discovered, initializing each row with `None` values matching the alphabet size. This lazy allocation handles unpredictable state space growth during subset construction.

=== Accepting-State Detection

The `collect_accepting` method scans the state map after construction completes, marking any DFA state whose corresponding NFA subset contains at least one NFA accepting state. For instance, in the DFA for `a*`, state 0 is accepting because its NFA subset includes the star's accept state (reachable via epsilon from the start).

This set-intersection logic naturally handles multiple accepting states arising from alternation—the DFA for `a+b` produces two accepting states, one for each branch.

=== DFA Structure

The final `Dfa` carries:
- `states` — a dense sequence `[0, 1, 2, ..., n-1]` where `n` is the number of reachable subsets
- `start` — always 0 by construction
- `accepts` — the list of accepting state IDs
- `trans` — the complete transition table
- `alphabet` — the sorted symbol list inherited from the NFA

Dead states are implicit—any `None` transition represents rejection without allocating an explicit error state.

=== Test Coverage

The determinization test suite validates five core patterns:

1. *Epsilon* (`""`) — single accepting start state, empty alphabet, no transitions
2. *Literal* (`a`) — two states, one accepting, single symbol transition
3. *Concatenation* (`ab`) — three states forming a linear chain, third state accepting
4. *Alternation* (`a+b`) — three states with two accepting (one per branch), split from start
5. *Kleene star* (`a*`) — two states both accepting, self-loop on state 1 for repetition

Each test inspects the exact transition table structure, verifying that state counts, accepting sets, and edge destinations match theoretical subset construction outcomes. This coverage ensures correctness across the fundamental operator types that compose arbitrary regular expressions.

== DFA Minimization

DFA minimization reduces automaton size by merging indistinguishable states—those that exhibit identical behavior for all possible input continuations. Implemented in `min.rs`, the algorithm applies Hopcroft's partition refinement strategy to produce a canonical minimal DFA equivalent to the input.

=== Initial Partition

The `PartitionRefinement` struct begins by splitting states into two coarse blocks:
- *Accepting block* — all states in `dfa.accepts`
- *Rejecting block* — all other states

This fundamental distinction ensures accepting and non-accepting states never merge, as they differ in their terminal behavior (accept vs. reject). The algorithm maintains a `state_class` vector mapping each state to its current partition index, enabling constant-time class lookups during refinement.

=== Worklist Initialization

For each initial partition block and each alphabet symbol, the algorithm enqueues a `(block_index, symbol_index)` pair into the worklist. This seeding strategy ensures all potential distinguishing transitions are examined. The worklist drives the refinement loop—each entry represents a hypothesis: "states transitioning into this block on this symbol might need separation."

=== Partition Refinement Loop

The core algorithm processes worklist entries until exhaustion:

1. *Collect involved states* — for a given `(block_idx, symbol_idx)` pair, identify all states whose transition on `symbol_idx` lands in `block_idx`. These "involved" states form a distinguishing criterion.

2. *Split existing blocks* — iterate through all current partitions, checking if each block contains both involved and non-involved states:
  - If homogeneous (all involved or all non-involved), the block remains intact
  - If mixed, split into two new blocks: `in_part` (involved) and `out_part` (non-involved)
  - Replace the original block with `in_part` and append `out_part` as a new partition
  - Update `state_class` mappings for all affected states

3. *Enqueue new splits* — for each newly created block, add `(new_block_idx, symbol)` entries to the worklist for all alphabet symbols. This ensures subsequent iterations refine based on the updated partition structure.

The algorithm selectively enqueues only the *smaller* of the two split blocks (an optimization from Hopcroft's original paper) to bound the number of worklist operations, though the current implementation enqueues the smaller block's index for all symbols.

=== Termination and Stability

Refinement terminates when the worklist empties, indicating no further distinguishable state pairs exist. At this point, the partition structure is stable—states within the same block are provably equivalent under all input suffixes.

=== Minimized DFA Construction

The `build_minimized` method reconstructs the DFA from the final partition structure:

*Transition table* — for each partition block, select an arbitrary representative state (the first element) and copy its transitions, remapping destination states through `state_class` to reference partition indices rather than original state IDs.

*Accepting states* — mark any partition containing at least one original accepting state as an accepting partition. This preserves acceptance behavior under state merging.

*Start state* — map the original start state through `state_class` to find its partition index, which becomes the minimized DFA's start state.

*State numbering* — assign sequential IDs `[0, 1, 2, ..., n-1]` to the `n` final partitions, producing a compact state space.

The resulting `Dfa` shares the original alphabet and maintains equivalent language recognition, but eliminates redundant states.

=== Edge Cases

For DFAs with 0 or 1 states, the algorithm short-circuits and returns a clone, as no meaningful minimization is possible. This prevents degenerate edge cases in the partition initialization.

=== Test Coverage

The minimization test suite validates correctness across 14 scenarios:

1. *Equivalent patterns* (`a+a*` vs `a*`, `(a|aa)*` vs `a*`) — confirms patterns with identical languages minimize to the same state count
2. *State count verification* (`aa*` → 2 states, `a*` → 1 state) — ensures expected minimal sizes
3. *Acceptance preservation* — verifies start state acceptance for patterns like `a*` and `\e`
4. *Complex merging* (`(a+b)(a+b)`) — tests reduction from product construction bloat
5. *Behavioral equivalence* (`a*b*`, `(a*)*`) — comprehensive input acceptance validation
6. *Already minimal* (`ab`) — confirms stable behavior on minimal inputs
7. *Optional patterns* (`a?`) — validates both-accepting-state scenarios
8. *Nested redundancy* (`(a*)*`) — detects and eliminates compositional duplication
9. *Disjoint branches* (`a+b`) — ensures independent accept states remain distinct when appropriate
10. *Duplicate paths* (`(aa+aa)` → `aa`) — collapses structurally redundant alternations

Each test combines state count assertions with behavioral validation through the `dfa_accepts` helper, ensuring both structural correctness and semantic equivalence. This dual-checking strategy guards against minimization bugs that preserve state counts but corrupt language recognition.

== Error Handling & Diagnostics

The error system provides structured, position-annotated diagnostics that enable precise feedback in both CLI and GUI contexts. Implemented in `errors.rs` using the `thiserror` crate, the hierarchy distinguishes lexical from syntactic failures while preserving exact error locations for source highlighting.

=== Error Hierarchy

The system defines a three-tier structure:

*`BuildError`* — the top-level enum aggregating all compilation failures:
- *`Lex(LexError)`* — wraps tokenization errors
- *`Parse(ParseError)`* — wraps syntax tree construction errors

The `#[from]` attribute enables automatic conversion via Rust's `?` operator, allowing `Ast::build()` to return a single unified error type while internally handling distinct failure modes.

*`LexError`* — pairs a character index (`at: usize`) with a `LexErrorKind`:
- *`DanglingEscape`* — detects trailing backslash with no subsequent character (e.g., `a+b\`)
- *`InvalidCharacter(char)`* — reports disallowed symbols with the offending character (e.g., `a+b$c` → `'$'`)

The error message template `"{kind} at index {at}"` produces output like `"invalid character '$' at index 3"`, directly usable in terminal output or UI overlays.

*`ParseError`* — similarly combines position (`at`) with a `ParseErrorKind` enum:
- *`UnexpectedEof`* — identifies incomplete expressions (e.g., `a+`, `(ab`)
- *`UnexpectedPrefixOperator(OpToken)`* — catches operators like `+` or `*` in prefix position where literals are expected
- *`MismatchedLeftParen { other: Token }`* — reports unclosed parentheses, noting what token appeared instead of `)`
- *`RightParenWithoutLeft`* — detects stray closing delimiters
- *`ParenthesesWithInvalidExp`* — rejects parenthesis group with no content or invalid expression (e.g., `()`, `(a+)`)

Each variant stores relevant context—`MismatchedLeftParen` preserves the unexpected token, while `UnexpectedPrefixOperator` captures the operator itself—enabling detailed error messages like `"expected an expression after the operator '+' at index 0"`.

=== Character Index Tracking

Both error types store `at` as a *character* index (not byte index), ensuring correct positioning even with multi-byte UTF-8 characters. The lexer pairs every token with its original character index during tokenization, and the parser propagates these indices when constructing error objects. This guarantees that error positions align with visual character offsets in input strings containing emojis or non-ASCII symbols.

For example, in the input `"🎉+b$"`, the invalid character `$` appears at character index 3 (not byte index 7), matching user perception.

=== UI Integration

The error structure directly supports the GUI's error display mechanism:

- The `at` field enables the error box to highlight the problematic character with a red background
- The `kind` enum's `Display` implementation provides human-readable messages
- The `BuildError` wrapper allows the application to present lexical and syntactic errors uniformly without type-switching

The report's earlier error-box implementation leverages these indices to position visual indicators (previously arrows, now background highlighting) at the exact failure point, creating immediate visual feedback for malformed patterns.

=== Error Message Quality

The `InvalidCharacter` variant includes a comprehensive message listing allowed characters—alphanumerics and reserved symbols (`\e`, `(`, `)`, `+`, `*`, `.`, `?`)—guiding users toward valid syntax. Similarly, `UnexpectedPrefixOperator` contextualizes the failure by explaining that an expression was expected after the operator, distinguishing prefix from infix/postfix usage.

These domain-specific messages reduce user confusion compared to generic "syntax error" reports, particularly for users unfamiliar with formal regex notation.

=== Exhaustive Error Taxonomy

The lexer and parser test suites collectively exercise all error variants:
- Lexer tests verify `DanglingEscape` and `InvalidCharacter` detection with exact position checks
- Parser tests validate all five `ParseErrorKind` cases across diverse input patterns, ensuring the error system handles both simple failures (single misplaced operator) and complex nesting errors (multiple unclosed parentheses)

This comprehensive coverage guarantees that every reachable error path produces actionable diagnostics with correct source positions.

=== Error examples

The following examples illustrate common lexer and parser diagnostics as they appear in the error overlay. Each figure shows the input pattern and the diagnostic produced by the core tool.

#grid(
  columns: 2,
  gutter: 1em,

  // 1
  figure(
    image("../images/errors/unexpected_prefix.png", width: 80%),
    caption: [Unexpected prefix operator],
  ),

  // 2
  figure(
    image("../images/errors/unexpected_eof.png", width: 80%),
    caption: [Unexpected end-of-input],
  ),

  // 3
  figure(
    image("../images/errors/mismatched_left_paren.png", width: 80%),
    caption: [Mismatched left parenthesis],
  ),

  // 4
  figure(
    image("../images/errors/right_paren_without_left.png", width: 80%),
    caption: [Stray right parenthesis],
  ),

  // 5
  figure(
    image("../images/errors/parens_with_no_exp.png", width: 80%),
    caption: [Empty parentheses],
  ),

  // 6
  figure(
    image("../images/errors/parens_with_invalid_exp.png", width: 80%),
    caption: [Parentheses containing an invalid expression],
  ),

  // 7
  figure(
    image("../images/errors/invalid_character.png", width: 80%),
    caption: [Invalid character in pattern — e.g. `$`],
  ),

  // 8
  figure(
    image("../images/errors/dangling_escape.png", width: 80%),
    caption: [Dangling escape at end of input — trailing `\`],
  ),
)


== Command-Line Interface

The CLI entry point, implemented in `main.rs`, provides a user-friendly interface for compiling regular expressions and inspecting their automaton representations. It accepts a regex pattern as input, constructs the corresponding AST, NFA, and optionally DFA and minimized DFA, and outputs relevant statistics about each stage.
The CLI supports the following features:

- Usage: `regviz <pattern> [input-string]` — the first argument is the regex pattern to compile; the optional second argument is a test string to simulate against the resulting automata.
- Robust parsing: the tool invokes `Ast::build(pattern)` which performs lexing and Pratt parsing; parse or lex errors are printed as `Build error: ...` with position information.
- NFA construction: after a successful parse the program builds a Thompson NFA via `Nfa::build(&ast)` and prints a short summary line showing the number of states, the start state index, number of accepting states, and total edges (e.g., `NFA: states=4 start=2 accepts=1 edges=5`).
- DFA determinization: the program determinizes the NFA with `dfa::determinize(&nfa)` and prints DFA statistics including number of states, start, number of accepting states, and the alphabet (e.g., `DFA: states=2 start=0 accepts=0 alphabet=['a']`).
- Optional simulation: when an input string is supplied, the CLI runs both the NFA and DFA simulations using `sim::nfa_accepts(&nfa, &s)` and `sim::simulate_dfa(&dfa, &s)`, printing whether each automaton accepts the string.
- Human-friendly output: the CLI prints the original pattern and the `Ast`'s S-expression form at the top (`Pattern: ...` and `AST: ...`), aiding quick inspection and debugging.

Example session:

#figure(
  ```bash
  $ cargo run "a*"
  Pattern: a*
  AST: (* a)
  NFA: states=4 start=2 accepts=1 edges=5
  DFA: states=2 start=0 accepts=2 alphabet=['a']

  $ cargo run "a*" "aaa"
  Pattern: a*
  AST: (* a)
  NFA: states=4 start=2 accepts=1 edges=5
  DFA: states=2 start=0 accepts=2 alphabet=['a']
  Input: "aaa"
  NFA accepts: true
  DFA accepts: true
  ```,
  caption: [Example CLI session compiling and simulating the pattern `a*`],
)
