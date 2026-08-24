#import sys: inputs

#import "helpers.typ"

#set page(paper: "us-letter", flipped: true)
#set page(margin: (left: 0.25in, right: 0.25in, top: 0.5in, bottom: 0.5in))
#set text(size: 14pt)
#set text(font: "Helvetica")
// #set text(font: "Arial")


// white and black colors
#let background_color = white
#let foreground_color = black
#let red_color = rgb("#DA2121")

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

  let puzzle_width = (11in - 0.5in) * 0.7
  let puzzle_height = (8.5in - 1in)
  let box_unit = calc.min(
    puzzle_width / puzzle.info.width,
    puzzle_height / puzzle.info.height,
  )

  let wrong_letter_exists = false
  let space_exists = false

  let coord_to_number = (none,) * (puzzle.info.width * puzzle.info.height)
  let number_to_coord = (none,) * (puzzle.info.width * puzzle.info.height)

  let n = 1

  let filled_cells = 0
  let all_cells = 0

  let is_new_solve = (:)

  if clues_info != none {
    for (num, info) in clues_info.at("across").pairs() {
      number_to_coord.at(info.y * puzzle.info.width + info.x) = num

      coord_to_number.at(int(num)) = (info.x, info.y)
      is_new_solve.insert(num, info.new_solve)
    }
    for (num, info) in clues_info.at("down").pairs() {
      number_to_coord.at(info.y * puzzle.info.width + info.x) = num

      coord_to_number.at(int(num)) = (info.x, info.y)
      is_new_solve.insert(
        num,
        is_new_solve.at(num, default: false) or info.new_solve,
      )
    }
  }

  let grid_rows = range(puzzle.info.height).map(y => puzzle_grid
    .at(y)
    .clusters())
  let solution_rows = range(puzzle.info.height).map(y => puzzle
    .grid
    .solution
    .at(y)
    .clusters())

  for y in range(puzzle.info.height) {
    for x in range(puzzle.info.width) {
      let cell = grid_rows.at(y).at(x)
      if cell == BLACK_CELL { continue }

      all_cells += 1

      let solution_cell = solution_rows.at(y).at(x)

      if cell == BLANK_CELL {
        space_exists = true
      }

      if cell != BLANK_CELL and cell != solution_cell {
        wrong_letter_exists = true
      }
      // Technically I don't need to check if cell is a solution cell,
      // as the wrong letter text would show up in the header instead of the progress bar,
      // but I want to be explicit about it
      if cell == solution_cell {
        filled_cells += 1
      }

      if clues_info == none {
        let starts-across = (
          (x == 0 or row.at(x - 1) == BLACK_CELL)
            and (
              x + 1 < puzzle.info.width and row.at(x + 1) != BLACK_CELL
            )
        )
        let starts-down = (
          (y == 0 or grid_rows.at(y - 1).at(x) == BLACK_CELL)
            and (
              y + 1 < puzzle.info.height
                and grid_rows.at(y + 1).at(x) != BLACK_CELL
            )
        )
        if starts-across or starts-down {
          number_to_coord.at(info.y * puzzle.info.width + info.x) = num

          coord_to_number.at(num) = (info.x, info.y)
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
      context {
        let text_size = text.size
        if puzzle.info.notes.len() > 60 {
          text_size = 0.45 * text_size
        }
        grid(
          columns: 2,
          align: (left, right),
          puzzle.info.author,
          text(style: "italic", size: text_size, puzzle.info.notes),
        )
      }
    } else {
      puzzle.info.author
    },
  )

  let sorted_across = puzzle
    .clues
    .at("across")
    .pairs()
    .sorted(key: clue => int(clue.at(0)))

  let sorted_down = puzzle
    .clues
    .at("down")
    .pairs()
    .sorted(key: clue => int(clue.at(0)))

  let across_clues_array = ()
  let down_clues_array = ()

  let stars_exist = false
  let star_spots_flat = (false,) * (puzzle.info.width * puzzle.info.height)

  for clue in sorted_across {
    let clue_num = clue.at(0)
    let word_solved = false

    let has_star = clue.at(1).starts-with("*")
    if has_star {
      stars_exist = true
      let (x, y) = coord_to_number.at(int(clue_num), default: none)
      star_spots_flat.at(y * puzzle.info.width + x) = true
    }

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
          inputs.hide_completed_clues == "true"
            or inputs.hide_completed_clues == true
        )
          and not is_finished
      ) { continue }
      across_clues_array.push(
        (
          content: strike(
            background: true,
            stroke: (paint: red_color, thickness: 2pt),
            clue_text,
          )
            + linebreak(),
          solved: true,
        ),
      )
    } else {
      across_clues_array.push((
        content: clue_text + linebreak(),
        solved: false,
      ))
    }
  }

  for clue in sorted_down {
    let clue_num = clue.at(0)
    let word_solved = false

    let has_star = clue.at(1).starts-with("*")
    if has_star {
      stars_exist = true
      let (x, y) = coord_to_number.at(clue_num)
      star_spots_flat.at(y * puzzle.info.width + x) = true
    }

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
      down_clues_array.push(
        (
          content: strike(
            background: true,
            stroke: (paint: red_color, thickness: 2pt),
            clue_text,
          )
            + linebreak(),
          solved: true,
        ),
      )
    } else {
      down_clues_array.push((content: clue_text + linebreak(), solved: false))
    }
  }

  let plain-text(it) = {
    if type(it) == str {
      it
    } else if it == [ ] {
      " "
    } else if it.has("children") {
      it.children.map(plain-text).join()
    } else if it.has("body") {
      plain-text(it.body)
    } else if it.has("text") {
      if type(it.text) == str { it.text } else { plain-text(it.text) }
    } else {
      ""
    }
  }

  across_clues_array = across_clues_array.sorted(key: x => x.at("solved"))
  down_clues_array = down_clues_array.sorted(key: x => x.at("solved"))

  layout(size => {
    let available_height = size.height

    // place(rect(width: 100%, height: 100%, fill: gray))

    let grid_width = box_unit * puzzle.info.width
    let clue_col_width = (size.width - grid_width - 0.02in) * (0.75 / 1.5)

    let one_line_height = measure(
      [42],
      width: clue_col_width,
    ).height
    let two_line_height = measure(
      [42 #linebreak() 42],
      width: clue_col_width,
    ).height
    let linebreak_height = two_line_height - (one_line_height * 2)

    let min_items = 19
    let max_items = 33

    let max_lines = 33
    // guess for max chars per line
    let max_line_chars = 28

    let across_split_index = across_clues_array.len()

    if across_split_index > min_items {
      let lines_so_far = 0
      for (i, clue) in across_clues_array.enumerate() {
        let clue_chars_len = clue
          .at("content")
          .at("children")
          .at(0)
          .at("child")
          .fields()
          .at("children")
          .map(plain-text)
          .join()
          .len()

        if clue_chars_len <= max_line_chars {
          lines_so_far += 1
        } else {
          // Only call measure if there are more than min_items clues, otherwise we don't need to split
          // Most likely less than min_items clues will fit on the page
          let clue_height = measure(
            clue.at("content"),
            width: clue_col_width,
          ).height

          if clue_height > one_line_height and clue_height > two_line_height {
            // if a clue is more than 4 lines, god help us all
            if (
              clue_height
                > (two_line_height + linebreak_height + one_line_height)
            ) {
              // four-line clue
              lines_so_far += 4
            } else {
              // three-line clue
              lines_so_far += 3
            }
          } else if clue_height > one_line_height {
            // two-line clue
            lines_so_far += 2
          } else {
            // one-line clue; somehow it still fit in one line
            lines_so_far += 1
          }
        }

        if lines_so_far > max_lines {
          across_split_index = i
          break
        }
      }
    }

    let down_split_index = down_clues_array.len()

    if down_split_index > min_items {
      let lines_so_far = 0
      for (i, clue) in down_clues_array.enumerate() {
        let clue_chars_len = clue
          .at("content")
          .at("children")
          .at(0)
          .at("child")
          .fields()
          .at("children")
          .map(plain-text)
          .join()
          .len()

        if clue_chars_len > max_line_chars {
          let clue_height = measure(
            clue.at("content"),
            width: clue_col_width,
          ).height
          if clue_height > one_line_height and clue_height > two_line_height {
            // if a clue is more than 4 lines, god help us all
            if (
              clue_height
                > (two_line_height + linebreak_height + one_line_height)
            ) {
              lines_so_far += 4
            } else {
              lines_so_far += 3
            }
          } else if clue_height > one_line_height {
            lines_so_far += 2
          } else {
            lines_so_far += 1
          }
        } else {
          lines_so_far += 1
        }

        if lines_so_far > max_lines {
          down_split_index = i
          break
        }
      }
    }

    grid(
      columns: (0.75fr, 0.75fr, auto),
      gutter: 0.01in,

      {
        [==== Across
          #(
            across_clues_array
              .slice(0, across_split_index)
              .map(x => x.at("content"))
              .join()
          )]
      },
      {
        [==== Down
          #(
            down_clues_array
              .slice(
                0,
                down_split_index,
              )
              .map(x => x.at("content"))
              .join()
          )]
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
                let cell = grid_rows.at(y).at(x)
                let num = number_to_coord.at(
                  y * puzzle.info.width + x,
                  default: none,
                )
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
                              thickness: 1pt,
                              dash: "densely-dotted",
                            ),
                          ))
                        }
                      }
                      if num != none {
                        let (style, weight) = if (
                          clues_info != none
                            and is_new_solve.at(
                              str(num),
                              default: false,
                            )
                          // and not is_finished
                        ) {
                          ("italic", "bold")
                        } else {
                          ("normal", "regular")
                        }

                        place(
                          top + left,
                          dx: box_unit * 0.05,
                          dy: box_unit * 0.05,
                          text(
                            size: box_unit * 0.20,
                            style: style,
                            weight: weight,
                            str(num),
                          ),
                        )
                      }
                      if (
                        stars_exist
                          and star_spots_flat.at(y * puzzle.info.width + x)
                      ) {
                        place(
                          top + right,
                          dx: -box_unit * 0.05,
                          dy: box_unit * 0.05,
                          text(
                            size: box_unit * 0.50,
                            "*",
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
      },
    )

    if (
      across_clues_array.len() > across_split_index
        or down_clues_array.len() > down_split_index
    ) {
      columns(4)[
        #if across_clues_array.len() > across_split_index {
          [
            ==== Across (Continued)
            #(
              across_clues_array
                .slice(
                  across_split_index,
                )
                .map(x => x.at("content"))
                .join()
            )
            #colbreak()
          ]
        }
        #if down_clues_array.len() > down_split_index [
          ==== Down (Continued)
          #(
            down_clues_array
              .slice(down_split_index)
              .map(x => x.at("content"))
              .join()
          )
        ]
      ]
    }
  })
}

#let puzzle_json = json(bytes(inputs.crossword_json))

#let clues_info_json = if "clues_info" in inputs {
  json(bytes(inputs.clues_info))
} else {
  none
}

#crossword(puzzle_json, clues_info_json)
