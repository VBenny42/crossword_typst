#import sys: inputs

#import "helpers.typ"

#set page(paper: "us-letter")
#set page(margin: (left: 0.25in, right: 0.25in, top: 0.5in, bottom: 0.5in))
#set text(size: 14pt)
#set text(font: "Helvetica")


// white and black colors
#let background_color = white
#let foreground_color = black
#let red_color = red

#if inputs.nord_colors == "true" or inputs.nord_colors == true {
  // // nord colors
  // background_color = rgb("#2E3440")
  // foreground_color = rgb("#D8DEE9")
  // red_color = rgb("#81A1C1")
  // // #let red_color = rgb("#BF616A")

  // // nord inverted colors
  background_color = rgb(209, 203, 192)
  foreground_color = rgb(40, 33, 23)
  red_color = rgb(127, 94, 63)
}

#set page(fill: background_color)
#set text(fill: foreground_color)

#let BLANK_CELL = "-"
#let BLACK_CELL = "."


#let crossword(puzzle, clues_info) = {
  let puzzle_grid = puzzle.grid.at("blank")

  // Allotted space should be 3/4 of usable page width
  let puzzle_width = (8.5in - 0.5in) * (3 / 4)
  let box_unit = puzzle_width / puzzle.info.width

  let wrong_letter_exists = false
  let space_exists = false

  // Build a dict of "x,y" -> clue number
  let number_to_coord = (:)
  let coord_to_number = (:)
  let n = 1

  let filled_cells = 0
  let all_cells = 0

  if clues_info != none {
    for (num, info) in clues_info.at("across").pairs() {
      number_to_coord.insert(str(info.x) + "," + str(info.y), num)
      coord_to_number.insert(num, (info.x, info.y))
    }
    for (num, info) in clues_info.at("down").pairs() {
      number_to_coord.insert(str(info.x) + "," + str(info.y), num)
      coord_to_number.insert(num, (info.x, info.y))
    }
  }

  for y in range(puzzle.info.height) {
    for x in range(puzzle.info.width) {
      let cell = puzzle_grid.at(y).clusters().at(x)
      if cell == BLACK_CELL { continue }

      all_cells += 1

      let solution_cell = puzzle.grid.solution.at(y).clusters().at(x)

      if cell == BLANK_CELL {
        space_exists = true
      }

      if cell != BLANK_CELL and cell != solution_cell {
        wrong_letter_exists = true
      }

      if cell == solution_cell {
        filled_cells += 1
      }

      if clues_info == none {
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
  }

  let is_finished = not space_exists and not wrong_letter_exists

  let percent_complete = if all_cells > 0 {
    (filled_cells / all_cells) * 100
  } else { 0 }

  set page(
    header: if is_finished {
      [#puzzle.info.title #h(1fr) _Finished_]
    } else if wrong_letter_exists == true {
      [#puzzle.info.title #h(1fr) #text(
          fill: red_color,
          weight: "bold",
          "WRONG GUESS EXISTS",
        )]
    } else if percent_complete > 0 {
      grid(
        columns: (1fr, auto),
        puzzle.info.title,
        helpers.progress(
          percent_complete / 100,
          width: 11em,
          height: 0.7em,
          fg: foreground_color,
          bg: background_color,
        ),
      )
    } else {
      puzzle.info.title
    },
    footer: if puzzle.info.notes != none {
      [#puzzle.info.author #h(1fr) #text(style: "italic", puzzle.info.notes)]
    } else { puzzle.info.author },
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
      let word_solved = false

      if clues_info == none {
        let clue_coord = coord_to_number.at(clue_num, default: none)
        let (x, y) = clue_coord

        while x <= puzzle.info.width {
          let next_cell = puzzle_grid.at(y).clusters().at(x)
          if next_cell == BLANK_CELL {
            break
          }
          if next_cell == BLACK_CELL or x + 1 == puzzle.info.width {
            word_solved = true
            break
          }
          x += 1
        }
      } else {
        word_solved = clues_info.at("across").at(clue_num).solved
      }

      let clue_text = text(size: 9pt)[*#clue.at(0).* #clue.at(1)]

      if word_solved {
        if (
          (
            inputs.at("hide_completed_clues", default: false)
              and (
                inputs.hide_completed_clues == "true"
                  or inputs.hide_completed_clues == true
              )
          )
            and not is_finished
        ) { continue }
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

  let down_clues = [
    ==== Down
    #for clue in sorted_down {
      let clue_num = clue.at(0)
      let word_solved = false

      if clues_info == none {
        let clue_coord = coord_to_number.at(clue_num, default: none)
        let (x, y) = clue_coord

        while y <= puzzle.info.height {
          let next_cell = puzzle_grid.at(y).clusters().at(x)
          if next_cell == BLANK_CELL {
            break
          }
          if next_cell == BLACK_CELL or y + 1 == puzzle.info.height {
            word_solved = true
            break
          }
          y += 1
        }
      } else {
        word_solved = clues_info.at("down").at(clue_num).solved
      }

      let clue_text = text(size: 9pt)[*#clue.at(0).* #clue.at(1)]

      if word_solved {
        if (
          (
            inputs.hide_completed_clues == "true"
              or inputs.hide_completed_clues == true
          )
            and not is_finished
        ) { continue }
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
    columns: (1fr, 3fr),
    gutter: 0.05in,

    {
      across_clues
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
                  stroke: (paint: foreground_color, thickness: 1pt),

                  {
                    if cell == BLACK_CELL {
                      continue
                    }
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
                  },
                ),
              )
            }
          }
        ),
      )

      columns(3)[#down_clues]
    },
  )
}

#let puzzle_json = json(bytes(inputs.crossword_json))

#let clues_info_json = if "clues_info" in inputs {
  json(bytes(inputs.clues_info))
} else {
  none
}

#crossword(puzzle_json, clues_info_json)
