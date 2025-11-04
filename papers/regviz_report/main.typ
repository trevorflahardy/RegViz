#import "styles.typ": *
#set page(paper: "us-letter", margin: 1in)
#set text(font: "New Computer Modern", size: 11pt)

#show heading.where(level: 1): it => block(spacing: 1em)[#it]

#title-page(
  "RegViz: A Rust-Based Regular Expression Visualizer",
  "Implementation and Analysis of Automata Construction and Visualization",
  "Trevor Flahardy, Hung Tran",
)

#pagebreak()

#include "sections/abstract.typ"
#include "sections/introduction.typ"
#include "sections/background.typ"
#include "sections/core.typ"
#include "sections/frontend.typ"
#include "sections/build_deploy.typ"
#include "sections/usage.typ"
#include "sections/future.typ"
#include "sections/conclusion.typ"
