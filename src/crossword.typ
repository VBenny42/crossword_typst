#set page(paper: "us-letter")
#set page(margin: (left: 0.25in, right: 0.25in, top: 0.5in, bottom: 0.5in))
#set text(size: 14pt)
// #set text(font: "Helvetica")
#set text(font: "Arial")

// // white and black colors
// #let background_color = white
// #let foreground_color = black
// #let red_color = red

// nord colors
#let background_color = rgb("#2E3440")
#let foreground_color = rgb("#D8DEE9")
#let red_color = rgb("#81A1C1")
// #let red_color = rgb("#BF616A")

#set page(fill: background_color)
#set text(fill: foreground_color)



#let crossword(puzzle) = {
  let puzzle_grid = puzzle.grid.at("blank")

  let box_unit = 0.40in

  let wrong_letter_exists = false
  let space_exists = false

  // Build a dict of "x,y" -> clue number
  let number_to_coord = (:)
  let coord_to_number = (:)
  let n = 1
  for y in range(puzzle.info.height) {
    for x in range(puzzle.info.width) {
      let cell = puzzle_grid.at(y).clusters().at(x)
      if cell == "." { continue }

      let solution_cell = puzzle.grid.solution.at(y).clusters().at(x)

      if cell == " " or cell == "-" {
        space_exists = true
      }

      if cell != " " and cell != "-" and cell != solution_cell {
        wrong_letter_exists = true
      }

      let starts-across = (
        (x == 0 or puzzle_grid.at(y).clusters().at(x - 1) == ".")
          and (
            x + 1 < puzzle.info.width
              and puzzle_grid.at(y).clusters().at(x + 1) != "."
          )
      )
      let starts-down = (
        (y == 0 or puzzle_grid.at(y - 1).clusters().at(x) == ".")
          and (
            y + 1 < puzzle.info.height
              and puzzle_grid.at(y + 1).clusters().at(x) != "."
          )
      )
      if starts-across or starts-down {
        number_to_coord.insert(str(x) + "," + str(y), n)
        coord_to_number.insert(str(n), (x, y))
        n += 1
      }
    }
  }

  set page(
    header: if not space_exists and not wrong_letter_exists {
      [#puzzle.info.title #h(1fr) _Finished_]
    } else if wrong_letter_exists == true {
      [#puzzle.info.title #h(1fr) #text(
          fill: red_color,
          weight: "bold",
          "WRONG GUESS EXISTS",
        )]
    } else {
      puzzle.info.title
    },
    footer: puzzle.info.author,
  )

  let sorted_across = puzzle
    .clues
    .at("across")
    .pairs()
    .sorted(key: clue => int(clue.at(0))) // sort by clue number

  let sorted_down = puzzle
    .clues
    .at("down")
    .pairs()
    .sorted(key: clue => int(clue.at(0))) // sort by clue number

  grid(
    columns: (1fr, 3fr),
    gutter: 0in,

    {
      [
        ==== Across
        #for clue in sorted_across {
          let clue_num = clue.at(0)

          let clue_coord = coord_to_number.at(clue_num, default: none)
          let word_solved = false
          let (x, y) = clue_coord

          while x <= puzzle.info.width {
            let next_cell = puzzle_grid.at(y).clusters().at(x)
            if next_cell == "-" or next_cell == " " {
              break
            }
            if next_cell == "." or x + 1 == puzzle.info.width {
              word_solved = true
              break
            }
            x += 1
          }

          if word_solved {
            strike(
              background: true,
              stroke: (paint: red_color, thickness: 2pt),
              text(
                size: 9pt,
              )[*#clue.at(0).* #clue.at(1)],
            )
          } else {
            text(size: 9pt)[*#clue.at(0).* #clue.at(1)]
          }
          linebreak()
        }
      ]
    },

    {
      align(
        left,
        grid(
          columns: range(puzzle.info.width).map(_ => box_unit),
          rows: range(puzzle.info.height).map(_ => box_unit),
          gutter: 0in,
          inset: 0.5mm,
          ..for y in range(puzzle.info.height) {
            for x in range(puzzle.info.width) {
              let cell = puzzle_grid.at(y).clusters().at(x)
              let key = str(x) + "," + str(y)
              let num = number_to_coord.at(key, default: none)
              (
                box(
                  width: box_unit,
                  height: box_unit,
                  fill: if cell == "." { foreground_color } else {
                    background_color
                  },
                  clip: true,
                  stroke: (paint: foreground_color, thickness: 1pt),

                  {
                    if num != none {
                      place(top + left, dx: 2pt, dy: 2pt, text(
                        size: 6pt,
                        str(num),
                      ))
                    }
                    if cell != "." {
                      place(
                        horizon + center,
                        text(weight: "medium", size: 18pt, if cell == "-" {
                          ""
                        } else {
                          cell
                        }),
                      )
                    }
                  },
                ),
              )
            }
          }
        ),
      )

      columns(3)[
        ==== Down
        #for clue in sorted_down {
          let clue_num = clue.at(0)

          let clue_coord = coord_to_number.at(clue_num, default: none)
          let word_solved = false
          let (x, y) = clue_coord

          while y <= puzzle.info.height {
            let next_cell = puzzle_grid.at(y).clusters().at(x)
            if next_cell == "-" or next_cell == " " {
              break
            }
            if next_cell == "." or y + 1 == puzzle.info.height {
              word_solved = true
              break
            }
            y += 1
          }

          let clue_text = text(size: 9pt)[*#clue.at(0).* #clue.at(1)]

          if word_solved {
            strike(
              background: true,
              stroke: (paint: red_color, thickness: 2pt),
              clue_text,
            )
          } else {
            clue_text
          }
          linebreak()
        }
      ]
    },
  )
}

#let puzzle_json = json("./output.json")
#crossword(puzzle_json)
