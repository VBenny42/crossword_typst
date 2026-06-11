//// For use with typst watch

#set page(paper: "us-letter", flipped: true)
#set page(margin: (left: 0.25in, right: 0.25in, top: 0.5in, bottom: 0.5in))
#set text(size: 14pt)
// #set text(font: "Helvetica")
#set text(font: "Arial")

// // white and black colors
// #let background_color = white
// #let foreground_color = black
// #let red_color = red

// // nord inverted colors
#let background_color = rgb(209, 203, 192)
#let foreground_color = rgb(40, 33, 23)
#let red_color = rgb(127, 94, 63)

// // nord colors
// #let background_color = rgb("#2E3440")
// #let foreground_color = rgb("#D8DEE9")
// #let red_color = rgb("#81A1C1")
// // #let red_color = rgb("#BF616A")

#set page(fill: background_color)
#set text(fill: foreground_color)

#let BLANK_CELL = "-"
#let BLACK_CELL = "."

#let crossword(puzzle) = {
  let puzzle_grid = puzzle.grid.at("blank")

  // Allotted space should be 3/4 of usable page width
  let puzzle_width = (11in - 0.5in) * 0.6
  let box_unit = puzzle_width / puzzle.info.width

  let wrong_letter_exists = false
  let space_exists = false

  // Build a dict of "x,y" -> clue number
  let number_to_coord = (:)
  let coord_to_number = (:)
  let n = 1
  for y in range(puzzle.info.height) {
    for x in range(puzzle.info.width) {
      let cell = puzzle_grid.at(y).clusters().at(x)
      if cell == BLACK_CELL { continue }

      let solution_cell = puzzle.grid.solution.at(y).clusters().at(x)

      if cell == BLANK_CELL {
        space_exists = true
      }

      if cell != BLANK_CELL and cell != solution_cell {
        wrong_letter_exists = true
      }

      let starts-across = (
        (x == 0 or puzzle_grid.at(y).clusters().at(x - 1) == BLACK_CELL)
          and (
            x + 1 < puzzle.info.width
              and puzzle_grid.at(y).clusters().at(x + 1) != BLACK_CELL
          )
      )
      let starts-down = (
        (y == 0 or puzzle_grid.at(y - 1).clusters().at(x) == BLACK_CELL)
          and (
            y + 1 < puzzle.info.height
              and puzzle_grid.at(y + 1).clusters().at(x) != BLACK_CELL
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

  let across_clues = [
    ==== Across
    #for clue in sorted_across {
      let clue_num = clue.at(0)

      let clue_coord = coord_to_number.at(clue_num, default: none)
      let word_solved = false
      let (x, y) = clue_coord

      while x <= puzzle.info.width {
        let next_cell = puzzle_grid.at(y).clusters().at(x)
        // let solution_cell = puzzle.grid.solution.at(y).clusters().at(x)
        // if next_cell == BLANK_CELL or next_cell != solution_cell {
        if next_cell == BLANK_CELL {
          break
        }
        if next_cell == BLACK_CELL or x + 1 == puzzle.info.width {
          word_solved = true
          break
        }
        x += 1
      }

      if word_solved {
        // continue
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

  let down_clues = [
    ==== Down
    #for clue in sorted_down {
      let clue_num = clue.at(0)

      let clue_coord = coord_to_number.at(clue_num, default: none)
      let word_solved = false
      let (x, y) = clue_coord

      while y <= puzzle.info.height {
        let next_cell = puzzle_grid.at(y).clusters().at(x)
        // let solution_cell = puzzle.grid.solution.at(y).clusters().at(x)
        // if next_cell == BLANK_CELL or next_cell != solution_cell {
        if next_cell == BLANK_CELL {
          break
        }
        if next_cell == BLACK_CELL or y + 1 == puzzle.info.height {
          word_solved = true
          break
        }
        y += 1
      }

      let clue_text = text(size: 9pt)[*#clue.at(0).* #clue.at(1)]

      if word_solved {
        // continue
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

  grid(
    columns: (0.75fr, 0.75fr, auto),
    gutter: 0in,

    {
      across_clues
    },
    {
      down_clues
    },

    {
      align(
        center,
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
                  fill: if cell == BLACK_CELL { foreground_color } else {
                    background_color
                  },
                  clip: true,
                  stroke: (paint: foreground_color, thickness: 1pt),

                  {
                    if cell != BLACK_CELL {
                      place(
                        horizon + center,
                        text(weight: "medium", size: box_unit * 0.625, if cell
                          == BLANK_CELL {
                          ""
                        } else {
                          cell
                        }),
                      )
                      if puzzle.extensions.circles != none {
                        if puzzle.extensions.circles.at(y).at(x) {
                          place(center + horizon, circle(
                            radius: (box_unit / 2) - 1pt,
                            stroke: (
                              paint: foreground_color.lighten(60%),
                              thickness: 1pt,
                              dash: "densely-dotted",
                            ),
                          ))
                        }
                      }
                      if num != none {
                        place(
                          top + left,
                          dx: box_unit * 0.05,
                          dy: box_unit * 0.05,
                          text(
                            size: box_unit * 0.20,
                            str(num),
                          ),
                        )
                      }
                    }
                  },
                ),
              )
            }
          }
        ),
      )
    },
  )
}

#let puzzle_json = json("../target/output.json")
#crossword(puzzle_json)
