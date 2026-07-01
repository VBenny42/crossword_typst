# crossword_typst

A crossword PDF generator and solver. Continuously regenerates a PDF as you
solve the puzzle.

## Usage

```
crossword_typst --puzzle-file-path <PUZZLE_FILE_PATH> [OPTIONS]
```

## Options

| Flag                          | Short | Description                                          | Default                     |
| ----------------------------- | ----- | ---------------------------------------------------- | --------------------------- |
| `--puzzle-file-path`          | `-p`  | Path to `.puz` file to be read                       | _(required)_                |
| `--json-output-path`          | `-j`  | Path to where JSON state should be saved             | Platform app data directory |
| `--pdf-style`                 |       | PDF layout style: `Normal`, `Larger`, or `Landscape` | `Normal`                    |
| `--write-to-json-only`        | `-w`  | Write puzzle to JSON and exit without generating PDF | `false`                     |
| `--nord-colors`               | `-n`  | Compile the PDF using Nord color scheme              | `false`                     |
| `--hide-completed-clues`      |       | Hide clues that have been fully filled in            | `false`                     |
| `--show-clue-length`          |       | Show the word length next to each clue in the PDF    | `false`                     |
| `--show-correct-letters-only` |       | Only show letters that have been correctly guessed   | `false`                     |
| `--help`                      | `-h`  | Print help                                           |                             |

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
