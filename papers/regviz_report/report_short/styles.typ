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
    size: 12pt,
  )

  // === Headings ===
  set heading(numbering: "1.")
  show heading: set block(below: 1.15em)

  // === Paragraph spacing ===
  set par(leading: 2em, justify: true, spacing: 2em)

  // === Lists ===
  show list: set block(below: 2em)

  // === Code blocks ===
  show raw.where(block: true): it => block(
    fill: luma(97%),
    inset: 10pt,
    radius: 10pt,
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
