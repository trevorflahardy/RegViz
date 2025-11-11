// styles.typ — RegViz Project Report Styling

#let apply-styles(body) = {
  // === Document layout ===
  set page(
    paper: "us-letter",
    margin: 1in,
    numbering: "1",
    header: [
      #set text(size: 8pt)
      #align(right)[COT 4210 - Automata Theory and Formal Languages, Fall 2025]
    ],
  )

  // === Font & text defaults ===

  set text(
    font: "New Computer Modern",
    size: 11pt,
  )

  // === Headings ===
  set heading(numbering: "1.")

  // === Paragraph spacing ===
  set par(leading: 0.65em, justify: true, spacing: 1.2em)

  // === Lists ===
  set list(indent: 2em, spacing: 0.4em)

  // === Code blocks ===
  show raw.where(block: true): it => block(
    fill: luma(95%),
    inset: 6pt,
    radius: 2pt,
    width: 100%,
  )[#it]

  body
}


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
