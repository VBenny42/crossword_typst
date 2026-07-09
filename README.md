# crossword_typst

A crossword PDF generator and solver. Continuously regenerates a PDF as you
solve the puzzle.

## Usage

```
crossword_typst --puzzle-file-path <PUZZLE_FILE_PATH> [OPTIONS]
```

## Options

| Flag                          | Short | Description                                                                                  | Default                                                           |
| ----------------------------- | ----- | -------------------------------------------------------------------------------------------- | ----------------------------------------------------------------- |
| `--puzzle-file-path`          | `-p`  | Path to `.puz` file to be read                                                               | _(required)_                                                      |
| `--output-path`               |       | Path to where output should be saved                                                         | Platform app data directory (extension follows `--output-format`) |
| `--output-format`             | `-o`  | Format to write puzzle state to: `json` or `puz`                                             | `puz`                                                             |
| `--update-puz-file`           | `-u`  | Update the same `.puz` file that's being read. Writing to puz does not recalculate checksums | `false`                                                           |
| `--pdf-style`                 |       | PDF layout style: `Normal`, `Larger`, or `Landscape`                                         | `Normal`                                                          |
| `--write-only`                | `-w`  | Write puzzle to output and exit without generating PDF                                       | `false`                                                           |
| `--nord-colors`               | `-n`  | Compile the PDF using Nord color scheme                                                      | `false`                                                           |
| `--hide-completed-clues`      |       | Hide clues that have been fully filled in                                                    | `false`                                                           |
| `--show-clue-length`          |       | Show the word length next to each clue in the PDF                                            | `false`                                                           |
| `--show-correct-letters-only` |       | Only show letters that have been correctly guessed                                           | `false`                                                           |
| `--help`                      | `-h`  | Print help                                                                                   |                                                                   |

> **Note:** `--update-puz-file` only takes effect when `--output-format` is
> `puz`.

## Examples

Run with a puzzle file:

```sh
cargo run -r -- -p "examples/Newsday - 20250429 - 42925 SECURITY NUMBERS.puz"
```

Generate PDF with Nord colors and larger layout:

```sh
crossword_typst -p puzzle.puz -n --pdf-style Larger
```

Just write puzzle data to JSON without generating a PDF:

```sh
crossword_typst -p puzzle.puz -w
```

## Notes

At least rust 1.96 is needed to run.

Also if output format is `puz`, the output `.puz` file will not have the correct
checksums. To fix that, run
`python validation/calculate_checksum_puz.py <file-to-recalculate-checksum>.py`
