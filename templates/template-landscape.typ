#import sys: inputs

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

#let progress(
  percent,
  height: 100%,
  width: 100%,
  bg: background_color,
  fg: foreground_color,
  stroke: 1pt + gray,
) = {
  box(
    height: height,
    width: width,
    stroke: stroke,
    fill: bg,
    {
      if percent > 0 {
        box(height: 100%, width: width * percent, fill: fg)
      }

      let color = if percent < 0.5 { fg } else { bg }
      let inverse_color = if percent < 0.5 { bg } else { fg }

      if percent >= 0.44 and percent <= 0.56 {
        place(
          center + horizon,
          text(
            fill: inverse_color,
            stroke: 2pt + inverse_color,
            size: 9pt,
            weight: "bold",
            str(int(percent * 100)) + "%",
          ),
        )
      }
      place(
        center + horizon,
        text(
          fill: color,
          size: 9pt,
          weight: "bold",
          str(int(percent * 100)) + "%",
        ),
      )
    },
  )
}

#let crossword(puzzle) = {
  let puzzle_grid = puzzle.grid.at("blank")

  let puzzle_width = (11in - 0.5in) * 0.7
  let puzzle_height = (8.5in - 1in)
  let box_unit = calc.min(
    puzzle_width / puzzle.info.width,
    puzzle_height / puzzle.info.height,
  )

  let wrong_letter_exists = false
  let space_exists = false

  // Build a dict of "x,y" -> clue number
  let number_to_coord = (:)
  let coord_to_number = (:)
  let n = 1

  let filled_cells = 0
  let all_cells = 0

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
      // Technically I don't need to check if cell is a solution cell,
      // as the wrong letter text would show up in the header instead of the progress bar,
      // but I want to be explicit about it
      if cell == solution_cell {
        filled_cells += 1
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
      set text(size: 14pt)
      grid(
        columns: (1fr, auto),
        puzzle.info.title,
        progress(percent_complete / 100, width: 11em, height: 0.7em),
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
    .sorted(key: clue => int(clue.at(0)))

  let sorted_down = puzzle
    .clues
    .at("down")
    .pairs()
    .sorted(key: clue => int(clue.at(0)))

  let across_clues_array = ()
  let down_clues_array = ()

  for clue in sorted_across {
    let clue_num = clue.at(0)

    let clue_coord = coord_to_number.at(clue_num, default: none)
    let word_solved = false
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

    let clue_coord = coord_to_number.at(clue_num, default: none)
    let word_solved = false
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

  across_clues_array = across_clues_array.sorted(key: x => x.at("solved"))
  down_clues_array = down_clues_array.sorted(key: x => x.at("solved"))

  layout(size => {
    let available_height = size.height

    let grid_width = box_unit * puzzle.info.width
    let clue_col_width = (size.width - grid_width - 0.02in) * (0.75 / 1.5)

    let across_split_index = across_clues_array.len()
    // Assuming a minimum of 15 items can always fit
    let start = calc.min(19, across_clues_array.len())
    // across_split_index = start

    for i in range(start, across_clues_array.len() + 1) {
      let test_content = [==== Across
        #(across_clues_array.slice(0, i).map(x => x.at("content")).join())]

      let h = measure(test_content, width: clue_col_width).height

      if h > available_height {
        across_split_index = i - 1
        break
      }
    }

    let down_split_index = down_clues_array.len()
    start = calc.min(19, down_clues_array.len())
    // down_split_index = start

    for i in range(start, down_clues_array.len() + 1) {
      let test_content = [==== Down
        #(down_clues_array.slice(0, i).map(x => x.at("content")).join())]

      let h = measure(test_content, width: clue_col_width).height

      if h > available_height {
        down_split_index = i - 1
        break
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
#crossword(puzzle_json)
