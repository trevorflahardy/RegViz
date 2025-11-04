= Frontend Visualization System
== Application Architecture & Rendering Infrastructure
The `regviz_app` frontend is built on top of the Iced reactive GUI framework, and its root state container, `App`, mirrors the core data required for visualization. The application instantiates a split `pane_grid`, reserving roughly thirty-five percent of the horizontal viewport for the control column (`PaneContent::Controls`) while the remainder hosts whichever visualization pane is active. Default zooming behaviour is driven by `DEFAULT_ZOOM_FACTOR` from `app::constants`, with user overrides clamped between $0.25x$ and $4x$ to guarantee consistent rendering on desktop and on the WASM build orchestrated by `Trunk.toml`. All typography and surface styles flow from `AppTheme::Dark`, which exposes palette helpers such as `bg_mid`, `text_secondary`, and `accent`; the palette is shared by both native and web builds because the theme module implements Iced's `theme::Base` trait.

State transitions are funneled through a single `App::update` method, which reacts to strongly typed `Message` variants emitted by the view. The pattern keeps the update graph explicit: regex edits trigger `lex_and_parse`, simulation edits rebuild traces, zoom adjustments clamp to the supported interval, and pane resize events propagate directly into the `pane_grid` state. The structure is idiomatic Iced; @fig:app-update illustrates how each branch delegates to a targeted handler without mutating unrelated state.

#figure(
  ```rust
  pub fn update(&mut self, message: Message) -> Task<Message> {
      match message {
          Message::Input(InputMessage::Changed(value)) => {
              self.handle_input_changed(value);
          }
          Message::Simulation(SimulationMessage::StepForward) => {
              self.handle_simulation_step_forward();
          }
          Message::View(ViewMessage::ZoomChanged(value)) => {
              self.handle_zoom_changed(value);
          }
          Message::PaneGrid(PaneGridMessage::Resized(event)) => {
              self.panes.resize(event.split, event.ratio);
          }
      }
      ().into()
  }
  ```,
  caption: [Simplified message dispatcher in `App::update`],
) <fig:app-update>

The update loop mediates between presentation and the backend compiler by refreshing `BuildArtifacts` on every valid regex edit. When the user first requests DFA simulation, the frontend determinises the Thompson NFA lazily via `regviz_core::core::dfa::determinize`, caches the resulting automaton inside the `build_artifacts`, and subsequent runs reuse the fully built structure. Simulation input undergoes symbol validation against the alphabet extracted at parse time, preventing illegal characters from reaching the tracing logic.

Rendering is factored through a reusable graph abstraction. The `Graph` trait normalizes access to nodes, edges, and bounding boxes regardless of the underlying artifact (NFA, DFA, or AST). A generic `GraphCanvas` couples the trait with a pluggable `LayoutStrategy`, computes a zoom level that fits the current layout into the viewport, and instantiates themed draw contexts. Each scene reuses deterministic styling: `color_for_box` produces stable hues per bounding box identifier, node fills/shadows derive from the central theme, and common zoom-aware helpers (for example, scaled arrow heads) live in `graph::edge`. This separation lets the same canvas implementation serve desktop and WASM targets with no conditional rendering code.

== NFA Visualization
The NFA view is layered on top of the backend Thompson construction via `VisualNfa`, which pairs the compiled automaton with a `Highlights` structure produced by the simulation engine. Each `GraphNode` carries a `StateHighlight` that toggles between neutral, active, and rejected fills, while `GraphEdge` instances record whether a transition was traversed during the current step. The layout is delegated to `NfaLayoutStrategy`, a hierarchy-aware algorithm that respects bounding boxes emitted by the compiler. The strategy walks a `BoxHierarchy`, lays out literals, alternations, and unary operators with dedicated rules, normalizes coordinates into positive space, and produces `PositionedNode`/`PositionedEdge` records ready for drawing.

Step-by-step playback is supported by `build_nfa_trace`. The trace builder precomputes epsilon closures, records symbol transitions, and snapshots the active state set after each consumed character. It also annotates each step with the exact edges traversed, enabling the canvas to highlight the complete path (including silent ε moves) during playback. @fig:nfa-trace shows how symbol and epsilon transitions combine during trace construction.

#figure(
  ```rust
  for (idx, symbol) in symbols.iter().enumerate() {
      let mut traversed = HashSet::new();
      for state in &current {
          for transition in nfa.transitions(*state) {
              if transition.label == EdgeLabel::Sym(*symbol) {
                  traversed.insert(EdgeHighlight::new(*state, transition.to, EdgeLabel::Sym(*symbol)));
              }
          }
      }
      let moved = sim::move_on(&current, *symbol, nfa);
      // epsilon closure after consuming the symbol …
  }
  ```,
  caption: [Trace builder loop collecting symbol transitions],
) <fig:nfa-trace>

During rendering, curved bypass and loop-back edges placed by Thompson's star/plus expansions ensure that closure operators remain intelligible. Arrowheads and label text are scaled by the runtime zoom factor, so the NFA remains legible whether the user zooms in on a nested alternation or zooms out to inspect the global structure. Figure 4.2 (to be captured) illustrates the highlighting overlay as the simulation advances through $(a+b)c$.

== Bounding Box System Logic
Bounding boxes are propagated from the backend as `regviz_core::automaton::BoundingBox` records and converted into lightweight `GraphBox` structures. Each box carries its semantic kind (`Literal`, `Concat`, `Alternation`, `KleeneStar`, `KleenePlus`, `Optional`) plus the states emitted while that operator was active. The frontend maintains a `BoxVisibility` bitset that mirrors the toggle buttons in the control panel; handlers simply flip the relevant boolean and the layout strategy filters the boxes before drawing. Colors come from `color_for_box`, a deterministic hash that stabilizes hues across runs, while labels are rendered using the shared canvas font to retain legibility.

`NfaLayoutStrategy` is the core of the system. It constructs a `BoxHierarchy`, recursively lays out children before parents, and applies operator-specific geometry—concatenations line up fragments horizontally with `INLINE_GAP_X` spacing, alternations stack vertically with `BRANCH_GAP_Y`, and unary operators surround their child with entry/exit states. A `BoundsTracker` aggregates the extents of nodes and boxes, enforcing consistent padding (`LAYOUT_PADDING_RATIO`) so that the canvas' fit-to-screen zoom remains smooth. Orphan states (those not covered by any bounding box) fall back to deterministic placements to avoid drifting as the user toggles visibility. Because curves and arrowheads are computed with the same zoom-aware helpers described earlier, the semantic intent of each operator remains visible even when boxes are hidden.

== DFA Visualization
Deterministic automata reuse the same graph infrastructure but swap in `VisualDfa`, which packages the DFA's transition table, alphabet, and highlight overlay. DFA construction is demand-driven: the first request to view or simulate a DFA triggers determinisation, after which `build_dfa_trace` supplies the highlight data required for playback. Because DFAs lack regex operator metadata, the layout strategy shifts to `DfaLayoutStrategy`, a breadth-first layering pass that arranges states in columns by distance from the start state, ensures reproducible ordering with sorted adjacency lists, and pads the final bounding rectangle so zooming preserves whitespace.

Bidirectional transitions receive special treatment in the label layer. If two edges connect the same pair of states in opposite directions, the helper `adjust_bidirectional_labels` assigns opposing `LabelBias` values so that one caption sits above the baseline while the other shifts below (or left/right in vertical edges). The transitions themselves remain straight, which avoids the visual overload of opposing curves while still exposing both labels. Highlight updates feed directly into the control panel's summary text, giving users deterministic feedback during simulation (`Step i / N`, consumed symbol, acceptance status).

== AST Visualization
The AST visualization adapts the parser's recursive `regviz_core::parser::Ast` into the graph abstraction through `AstGraph::new`. Nodes are emitted with operator glyphs (`·`, `+`, `*`, `?`, quoted literals, or `ε`), and edges capture parent-child relationships with simple left/right annotations. Layout is delegated to `TreeLayoutStrategy`, a top-down binary tree algorithm that assigns depth levels, spaces siblings evenly using a fixed `NODE_WIDTH`, and recenters each level to keep the tree balanced. Because ASTs do not carry bounding boxes, the layout omits that layer entirely and renders against the same themed canvas, giving users a consistent zooming and panning experience across all visualizations. @fig:ast-visualization showcases the AST for `((a+b)c)*` with node labels and the shared theme.

#figure(
  image("../images/ast_visualization.png", width: 60%),
  caption: [AST visualization for `((a+b)c)*`],
) <fig:ast-visualization>

== User Interaction & Workflow
The control pane presents a deliberately linear workflow. At the top, the regex editor feeds `InputMessage::Changed` events into the parser and reports live status: successful parses show alphabet and state counts, while parser errors are surfaced inline with themed warning text. A row of preset buttons seeds common expressions, a dedicated section captures the simulation input string (disabled until a regex builds successfully), and validation errors are rendered beneath the field if the string uses out-of-alphabet symbols. Bounding-box toggles stay visible whenever the NFA view is active, greying out when the user switches to DFA or AST. Simulation controls adapt to readiness—buttons promote to the accent-coloured “primary” style when stepping is possible and fall back to neutral tones otherwise.

The visualization pane hosts the active graph canvas and accepts dynamic resizing through the `pane_grid` splitter. View-mode controls sit alongside the canvas and route through `ViewMessage::SelectRightPaneMode`, allowing users to flip between NFA, DFA, and AST perspectives without re-parsing. Zoom is managed via both the slider (bounded by `MIN_ZOOM_FACTOR`/`MAX_ZOOM_FACTOR`) and fit-to-screen recalculations triggered whenever the layout changes. Across all panes, the dark theme harmonizes typography, surface colours, and focus rings, ensuring that highlights, bounding boxes, and simulation cues remain readable in native windows and in the WASM build rendered via Trunk.
