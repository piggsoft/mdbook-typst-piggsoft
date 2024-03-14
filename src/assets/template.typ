#set text(lang: "zh", region: "cn")
#set document(title: document_title, author: document_authors, keywords: document_keywords, date: auto)
#set page(
  numbering: "1 / 1",
  number-align: right,
)
// [
//     #set align(right)
//     #set text(13pt)
//     #counter(page).display(
//       "1 / 1",
//       both: true,
//     )
//   ]
#set pagebreak(weak: true)
#set par(justify: true)
#set heading(numbering: "1.1.")

#show outline.entry.where(level: 1):it => {
  v(11pt, weak: true)
  strong(it)
}

#show heading: it => [
  #block(above: 2em, below: 2em, it)
]
#set table(
    fill: (col, row) =>
        if calc.odd(row) {
            luma(240)
        } else {
            white
        }
)
#set quote(block: true)
#show link: underline
#show link: set text(blue)

//---------------------content start-----------------
#align(center, text(17pt)[
  *#document_title*
])

#let jb = linebreak(justify: true)
#align(right, text(15pt)[
  #let index = 4
  #while index > 0 {
    index = index - 1
    jb
  }
  #for author in document_authors {
    author
    jb
  }
])
#pagebreak()

#outline(depth: section_depth, /*indent: 2em*/)
#pagebreak()
