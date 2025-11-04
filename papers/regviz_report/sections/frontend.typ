= Frontend Visualization System
== Application Architecture & Rendering Infrastructure
Describe the Iced-based state container (App), pane layout, default zoom/visibility settings, and theme palette supporting both desktop and WASM builds.
Outline the message/update loop, input parsing hook, simulation validation, and lazy DFA generation bridging the UI with backend artifacts.
Explain the graph abstraction layer: the Graph trait, reusable GraphCanvas, deterministic color styling, and layout-strategy plug-ins shared by all views.

== NFA Visualization
Detail how VisualNfa combines backend NFAs with per-step highlights, including active-state coloring and traversed-edge marking emitted by simulation traces.
Present the simulation trace builder that records epsilon closures, symbol transitions, and acceptance states for step-by-step playback (feeding both highlighting and status messages).

== Bounding Box System Logic
Track how backend-generated bounding boxes feed the UI via GraphBox, including deterministic colors, labels, and user-controlled visibility flags.
Describe the hierarchical layout strategy: building BoxHierarchy, recursively composing operator-specific fragments, normalizing coordinates, stacking root boxes, and computing padded extents for rendering.
Explain bounding-box extent calculations, depth-based drawing order, fallback placement for orphan states, and how BoxVisibility toggles map to UI controls.
Highlight edge styling choices—curved bypass/loop arrows for closure operators and arrow-head rendering tied to active edges—to maintain semantic clarity during playback.

== DFA Visualization
Show how VisualDfa packages deterministic transition tables with highlights and alphabet labels, while falling back gracefully if determinization has not been computed yet.
Describe the layered BFS layout that assigns levels, normalizes coordinates, pads bounds, and renders the DFA without bounding boxes.
Note reuse of simulation traces to highlight deterministic traversal paths and update summary messaging in the control panel.

== AST Visualization
Explain the conversion from parser ASTs into GraphNode/GraphEdge sequences and the binary tree layout that balances depth and spacing for readability.
Discuss how the AST canvas shares zoom controls and theming with other views while omitting bounding boxes for clarity.

== User Interaction & Workflow
Detail the left-pane experience: regex input with live status, sample presets, simulation string entry, bounding-box toggles, and step controls that adapt to readiness and validation states.
Describe right-pane view switching (NFA/DFA/AST), zoom slider, pane resizing, and theme-aware styling that ties the application together.
