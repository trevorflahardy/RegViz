# Regular Expression Visualizer

A Rust-based interactive tool for visualizing and understanding the equivalence between regular expressions and finite automata, developed as a final project for COT 4210 Automata Theory & Formal Languages at USF.

## Overview

This project implements a complete pipeline for converting regular expressions into finite automata representations, demonstrating fundamental concepts from automata theory. Students and educators can use this tool to observe how regular expressions transform into NFAs and DFAs, test string membership, and explore automata behavior interactively.

### Problem Statement

While tools for visualizing regular expressions and automata exist, most are outdated, proprietary, or limited in functionality. Students often learn the theoretical equivalence between regular expressions and finite automata without a modern, hands-on way to observe the transformation process. This project addresses that gap by providing a transparent, from-scratch implementation that reveals each step of the conversion.

## Running

The project is split into two main components: a backend library and a frontend web application.

## Frontend

The frontend has been set as the default package for the workspace. To build and run the frontend, use the following commands:

Building:
```bash
cargo build
```

Running:
```bash
cargo run
```

### Backend

Building:
```bash
cargo build --package regviz_core
```

Running tests:
```bash
cargo test --package regviz_core
```

#### Backend CLI Tool

The backend also has a CLI binary tool for quick testing:
```bash
cargo run --package regviz_core --bin regviz_cli -- <regular_expression> <test_string>
```

