// styles.typ — RegViz Project Report Styling

// === Document layout ===
#set page(
  paper: "us-letter",
  margin: (top: 1in, bottom: 1in, left: 1.25in, right: 1.25in),
  numbering: "1.",
)

// === Font & text defaults ===
#set text(
  font: "New Computer Modern",
  size: 11pt,
)

// === Headings ===
#set heading(numbering: "1.")
#show heading.where(level: 1): it => block(spacing: 1em)[
  #text(size: 16pt, weight: "bold")[#it]
]
#show heading.where(level: 2): it => block(spacing: 0.75em)[
  #text(size: 13pt, weight: "semibold")[#it]
]
#show heading.where(level: 3): it => block(spacing: 0.5em)[
  #text(size: 11pt, weight: "medium", style: "italic")[#it]
]

// === Paragraph spacing ===
#set par(leading: 0.65em, justify: true, spacing: 0.8em)

// === Lists ===
#set list(indent: 2em, spacing: 0.4em)

// === Figure & table captions ===
#show figure.caption: it => [
  #set text(size: 10pt, weight: "semibold", style: "italic")
  #it
]

// === Code blocks ===
#show raw.where(block: true): it => block(
  fill: luma(95%),
  inset: 6pt,
  radius: 2pt,
  width: 100%,
)[#it]

// === Metadata placeholders ===
#let title-style(body) = text(size: 24pt, weight: "bold")[#body]
#let subtitle-style(body) = text(size: 16pt)[#body]
#let author-style(body) = text(size: 12pt, weight: "medium")[#body]

// === Title page macro ===
#let title-page(title, subtitle, authors) = [
  #align(center + horizon)[
    #title-style[#title]
    #v(1em)
    #subtitle-style[#subtitle]
    #v(2em)
    #author-style[#authors]
    #v(1em)
    #text(size: 11pt)[#datetime.today().display("[month repr:long] [day], [year]")]
  ]
]
