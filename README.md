# Regular Expression Visualizer

A Rust-based interactive tool for visualizing and understanding the equivalence between regular expressions and finite automata, developed as a final project for COT 4210 Automata Theory & Formal Languages at USF.

## Overview

This project implements a complete pipeline for converting regular expressions into finite automata representations, demonstrating fundamental concepts from automata theory. Students and educators can use this tool to observe how regular expressions transform into NFAs and DFAs, test string membership, and explore automata behavior interactively.

### Problem Statement

While tools for visualizing regular expressions and automata exist, most are outdated, proprietary, or limited in functionality. Students often learn the theoretical equivalence between regular expressions and finite automata without a modern, hands-on way to observe the transformation process. This project addresses that gap by providing a transparent, from-scratch implementation that reveals each step of the conversion.

## Features

- **Manual Lexer & Parser**: Tokenizes and parses regular expressions without using built-in regex libraries
- **Thompson's Construction**: Converts regular expressions to ε-NFAs using Thompson's algorithm
- **Subset Construction**: Transforms NFAs into DFAs using the powerset construction
- **DFA Minimization**: Reduces DFAs to their minimal equivalent forms using Hopcroft's algorithm
- **Interactive Simulation**: Test string acceptance on NFAs and DFAs in real-time
- **Visualization**: (Planned) GUI for rendering state diagrams and exploring automata interactively

## Supported Syntax

The parser supports the following regular expression operators:

- **Characters**: `a`, `b`, `c`, etc. (literal characters)
- **Concatenation**: `ab` (implicit, sequential characters)
- **Alternation**: `a|b` (union/choice)
- **Kleene Star**: `a*` (zero or more repetitions)
- **Plus**: `a+` (one or more repetitions)
- **Optional**: `a?` (zero or one occurrence)
- **Grouping**: `(ab)*` (parentheses for precedence)
- **Escaping**: `\|`, `\*`, etc. (literal special characters)

### Example Expressions

- `a*` - Zero or more 'a's
- `(a|b)*abb` - Any string of a's and b's ending with "abb"
- `a+b?` - One or more 'a's followed by an optional 'b'
- `(ab)*|c` - Zero or more "ab" pairs, or a single 'c'

## Implementation Details

### 1. Lexer ([`src/core/lexer.rs`](src/core/lexer.rs))

Tokenizes input strings into a sequence of tokens representing characters, operators, and special symbols. Supports escape sequences for literal special characters.

**Key Function**: [`lex`](src/core/lexer.rs)
- Input: `&str` - Raw regular expression string
- Output: `Result<Vec<Token>, LexError>` - Token stream or error
- Handles: Escaping with `\`, all operators (`|`, `*`, `+`, `?`, `(`, `)`), and literal characters

### 2. Parser ([`src/core/parser.rs`](src/core/parser.rs))

Implements a recursive-descent parser that builds an abstract syntax tree (AST) from tokens. Handles operator precedence:
1. Alternation (`|`) - lowest precedence
2. Concatenation (implicit) - medium precedence
3. Postfix operators (`*`, `+`, `?`) - highest precedence

**Key Functions**:
- [`parse`](src/core/parser.rs) - Main entry point
- [`parse_regex`](src/core/parser.rs) → [`parse_alt`](src/core/parser.rs) → [`parse_concat`](src/core/parser.rs) → [`parse_repeat`](src/core/parser.rs) → [`parse_atom`](src/core/parser.rs)

**AST Types** ([`src/core/ast.rs`](src/core/ast.rs)):
- [`Ast::Char(char)`](src/core/ast.rs) - Literal character
- [`Ast::Concat(Box<Ast>, Box<Ast>)`](src/core/ast.rs) - Concatenation
- [`Ast::Alt(Box<Ast>, Box<Ast>)`](src/core/ast.rs) - Alternation
- [`Ast::Star(Box<Ast>)`](src/core/ast.rs) - Kleene star
- [`Ast::Plus(Box<Ast>)`](src/core/ast.rs) - One or more
- [`Ast::Opt(Box<Ast>)`](src/core/ast.rs) - Optional

### 3. NFA Construction ([`src/core/nfa.rs`](src/core/nfa.rs))

Uses **Thompson's Construction** to convert AST nodes into ε-NFA fragments:
- Each character creates a simple 2-state fragment
- Concatenation connects fragments via ε-transitions
- Alternation creates a new start state with ε-transitions to both branches
- Star, Plus, and Optional add appropriate ε-transition loops

**Key Types**:
- [`Nfa`](src/core/nfa.rs) - NFA representation with states, edges, and adjacency list
- [`EdgeLabel`](src/core/nfa.rs) - Either `Eps` (ε-transition) or `Sym(char)`
- [`Edge`](src/core/nfa.rs) - Directed edge with label

**Key Function**: [`build_nfa`](src/core/nfa.rs)
- Input: `&Ast` - Abstract syntax tree
- Output: `Nfa` - Complete ε-NFA

### 4. DFA Determinization ([`src/core/dfa.rs`](src/core/dfa.rs))

Applies **Subset Construction** to convert NFAs to DFAs:
- Computes ε-closures for state sets
- Tracks which NFA state subsets correspond to each DFA state
- Builds transition table based on symbol moves and ε-closures

**Key Function**: [`determinize`](src/core/dfa.rs)
- Input: `&Nfa` - Nondeterministic finite automaton
- Output: `(Dfa, Vec<char>)` - Deterministic automaton and alphabet
- Uses: [`epsilon_closure`](src/core/sim.rs) and [`move_on`](src/core/sim.rs) from simulation module

### 5. DFA Minimization ([`src/core/min.rs`](src/core/min.rs))

Implements **Hopcroft's Algorithm** to minimize DFAs:
- Partitions states into equivalence classes (accepting vs. non-accepting)
- Refines partitions by splitting based on transition behavior
- Constructs minimal DFA with one state per equivalence class

**Key Function**: [`minimize`](src/core/min.rs)
- Input: `&Dfa`, `&[char]` - DFA and alphabet
- Output: `Dfa` - Minimal equivalent DFA
- Complexity: O(n log n) where n is the number of states

### 6. Simulation ([`src/core/sim.rs`](src/core/sim.rs))

Provides functions for testing string acceptance:

**NFA Simulation**:
- [`nfa_accepts`](src/core/sim.rs) - Simulates NFA execution using ε-closure and symbol moves
- [`epsilon_closure`](src/core/sim.rs) - Computes ε-closure of state sets
- [`move_on`](src/core/sim.rs) - Computes states reachable via a symbol

**DFA Simulation**:
- [`simulate_dfa`](src/core/sim.rs) - Deterministic simulation for DFAs (faster, O(n) time)

## Educational Goals

This project serves multiple educational purposes:

1. **Hands-on Learning**: Demonstrates the practical application of theoretical concepts from automata theory
2. **Transparency**: Every step of the conversion process is visible and understandable
3. **Verification**: Students can test their understanding by comparing hand-drawn automata with generated ones
4. **Modern Tooling**: Introduces students to systems programming with Rust while exploring formal languages

## Testing & Evaluation

The project includes extensive unit and integration tests covering:

- **Lexer** ([`tests/lexer_tests.rs`](tests/lexer_tests.rs)): Token generation for simple and complex expressions
- **Parser** ([`tests/parser_tests.rs`](tests/parser_tests.rs)): AST construction for various operator combinations
- **NFA** ([`tests/nfa_tests.rs`](tests/nfa_tests.rs)): Correctness of Thompson's construction
- **DFA** ([`tests/dfa_tests.rs`](tests/dfa_tests.rs)): Determinization accuracy and state count
- **Minimization** ([`tests/minimization_tests.rs`](tests/minimization_tests.rs)): Verification that minimal DFAs accept the same language
- **Simulation** ([`tests/simulation_tests.rs`](tests/simulation_tests.rs)): String acceptance across multiple examples
- **Integration** ([`tests/core_parsing.rs`](tests/core_parsing.rs)): End-to-end pipeline tests

### Example Test Cases

From [`tests/simulation_tests.rs`](tests/simulation_tests.rs):

```rust
#[test]
fn test_simulate_dfa_complex() {
    let input = "(a|b)*abb";
    let tokens = lexer::lex(input).unwrap();
    let ast = parser::parse(&tokens).unwrap();
    let nfa = nfa::build_nfa(&ast);
    let (dfa, alphabet) = dfa::determinize(&nfa);
    let min_dfa = min::minimize(&dfa, &alphabet);

    assert!(sim::simulate_dfa(&min_dfa, &alphabet, "abb"));
    assert!(sim::simulate_dfa(&min_dfa, &alphabet, "aabb"));
    assert!(!sim::simulate_dfa(&min_dfa, &alphabet, "ab"));
}
```

### Preset Examples

The [`src/examples/presets.rs`](src/examples/presets.rs) module includes several predefined regular expressions for testing:

- **Balanced As**: `a(b|c)*a` - Strings starting and ending with 'a'
- **AB Repeats or C**: `(ab)*|c` - Zero or more "ab" pairs, or single 'c'
- **A Plus B Optional**: `a+b?` - One or more 'a's, optional 'b'
- **Ends With ABB**: `(a|b)*abb` - Strings ending with "abb"
- **Nested Choice**: `a(bc|d)+e?` - Complex nested expressions
- **Literal Pipe**: `a\|b` - Escaped special characters

## Future Enhancements

- **Interactive GUI**: Complete the visualization module using the [iced](https://iced.rs/) framework
- **Step-by-step Execution**: Show intermediate states during NFA/DFA simulation
- **Export Capabilities**: Save automata as images or DOT files for documentation
- **Extended Syntax**: Support character classes (`[a-z]`), ranges, and additional operators
- **Performance Metrics**: Display state counts, transition counts, and conversion timing
- **Regex to English**: Convert regular expressions to human-readable descriptions

## References

This project implements algorithms from:

- **Introduction to Automata Theory, Languages, and Computation** by Hopcroft, Motwani, and Ullman
- **Thompson's Construction**: Ken Thompson (1968) - "Regular Expression Search Algorithm"
- **Hopcroft's Minimization Algorithm**: John Hopcroft (1971)

## License

This project is developed for educational purposes as part of COT 4210 at the University of South Florida.

## Author

Developed by Trevor Flahardy for COT 4210 Automata Theory & Formal Languages, Fall 2025.

---

**Note**: This project is under active development. The visualization module is currently in progress and will be completed for the final demonstration.