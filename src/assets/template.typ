#set text(lang: "zh", region: "cn")
#set page(
  numbering: "1 / 1",
  number-align: right,
)
#set pagebreak(weak: true)

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


#outline(depth: 6, indent: 2em)
#pagebreak()
