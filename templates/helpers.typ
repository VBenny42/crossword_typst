#let progress(
  percent,
  height: 100%,
  width: 100%,
  bg: white,
  fg: black,
  stroke: 1pt + gray,
) = {
  box(
    height: height,
    width: width,
    // stroke: stroke,
    stroke: 1pt + color.mix((bg, 66%), (fg, 33%)),
    fill: bg,
    inset: 0.5pt,
    {
      if percent > 0 {
        box(height: 100%, width: width * percent, fill: fg)
      }

      let color = if percent < 0.5 { fg } else { bg }
      let inverse_color = if percent < 0.5 { bg } else { fg }

      if percent >= 0.43 and percent <= 0.56 {
        place(
          center + horizon,
          text(
            fill: inverse_color,
            stroke: 1pt + inverse_color,
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
