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

// Begin the main content
#include "sections/abstract.typ"
#include "sections/introduction.typ"
#include "sections/background.typ"
#include "sections/core.typ"
#include "sections/frontend.typ"
#include "sections/build_deploy.typ"
#include "sections/usage.typ"
#include "sections/future.typ"
#include "sections/conclusion.typ"
