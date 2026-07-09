use puz_parse::{parse, Puzzle};
use std::{
    collections::HashMap,
    error::Error,
    fmt,
    fs::{self, File},
    path::PathBuf,
    str::FromStr,
};

use crate::types::{ClueInfo, CluesInfo, OutputFormat, PuzzleState, BLACK_CELL, BLANK_CELL};

mod clues;
mod remove;
mod solve;

impl PuzzleState {
    pub fn new(args: &crate::Args) -> Result<Self, Box<dyn Error>> {
        let mut puzzle = initialize_puzzle(&args.puzzle_file_path)?;

        match args.output_format {
            OutputFormat::Json => {
                if File::open(&args.output_path).is_ok() {
                } else {
                    eprintln!("JSON file does not exist. Creating a new one...");
                    args.output_format.write_puzzle_to_file(
                        &args.output_path,
                        &args.puzzle_file_path,
                        &puzzle,
                    )?;
                }

                let read_puzzle = read_puzzle_from_json(&args.output_path)?;

                if read_puzzle.info.title == puzzle.info.title {
                    puzzle.grid.blank.clone_from(&read_puzzle.grid.blank);
                } else {
                    eprintln!("Warning: The puzzle title in the JSON file does not match the original puzzle. Overwriting JSON file with blank puzzle data...");
                    args.output_format.write_puzzle_to_file(
                        &args.output_path,
                        &args.puzzle_file_path,
                        &puzzle,
                    )?;
                }
            }
            OutputFormat::Puz => {
                // Already reads from a puz file in initialize_puzzle
            }
        }

        let clues_info = extract_clue_info(&puzzle);

        if args.show_clue_length {
            puzzle.clues.across.iter_mut().for_each(|(k, v)| {
                let clue_info = clues_info.across.get(&(u8::try_from(*k).unwrap())).unwrap();
                *v = format!("{v} ({})", clue_info.length);
            });
            puzzle.clues.down.iter_mut().for_each(|(k, v)| {
                let clue_info = clues_info.down.get(&(u8::try_from(*k).unwrap())).unwrap();
                *v = format!("{v} ({})", clue_info.length);
            });
        }

        Ok(Self {
            puzzle,
            clues_info,
            args: args.clone(),
        })
    }
}

impl fmt::Display for PuzzleState {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        writeln!(f)?;
        for row in &self.puzzle.grid.blank {
            writeln!(f, "{row}")?;
        }
        Ok(())
    }
}

fn initialize_puzzle(file_path: &PathBuf) -> Result<Puzzle, Box<dyn Error>> {
    let file = fs::File::open(file_path)?;
    let result = parse(file)?;

    for warning in &result.warnings {
        println!("Warning: {warning}");
    }

    Ok(result.result)
}

fn input<T: FromStr>() -> Result<T, <T as FromStr>::Err> {
    let mut input: String = String::with_capacity(64);

    std::io::stdin()
        .read_line(&mut input)
        .expect("Input could not be read");

    input.trim().parse()
}

fn extract_clue_info(puzzle: &Puzzle) -> CluesInfo {
    let mut across_clues = HashMap::new();
    let mut down_clues = HashMap::new();

    let mut clue_number = 1;

    for y in 0..puzzle.info.height as usize {
        for x in 0..puzzle.info.width as usize {
            let cell = puzzle.grid.blank[y].chars().nth(x).unwrap_or(BLANK_CELL);

            if cell == BLACK_CELL {
                continue; // Skip black squares
            }
            let mut is_clue_start = false;

            // Check for across clue
            if (x == 0
                || puzzle.grid.blank[y]
                    .chars()
                    .nth(x - 1)
                    .unwrap_or(BLANK_CELL)
                    == BLACK_CELL)
                && (x + 1 < puzzle.info.width as usize
                    && puzzle.grid.blank[y]
                        .chars()
                        .nth(x + 1)
                        .unwrap_or(BLANK_CELL)
                        != BLACK_CELL)
            {
                let clue_length = (0..puzzle.info.width as usize)
                    .take_while(|i| {
                        x + *i < (puzzle.info.width as usize)
                            && puzzle.grid.blank[y]
                                .chars()
                                .nth(x + *i)
                                .unwrap_or(BLANK_CELL)
                                != BLACK_CELL
                    })
                    .count();

                is_clue_start = true;

                across_clues.insert(
                    clue_number,
                    ClueInfo {
                        length: clue_length,
                        x,
                        y,
                    },
                );
            }
            // Check for down clue
            if (y == 0
                || puzzle.grid.blank[y - 1]
                    .chars()
                    .nth(x)
                    .unwrap_or(BLANK_CELL)
                    == BLACK_CELL)
                && (y + 1 < puzzle.info.height as usize
                    && puzzle.grid.blank[y + 1]
                        .chars()
                        .nth(x)
                        .unwrap_or(BLANK_CELL)
                        != BLACK_CELL)
            {
                let clue_length = (0..puzzle.info.height as usize)
                    .take_while(|i| {
                        y + *i < puzzle.info.height as usize
                            && puzzle.grid.blank[y + *i]
                                .chars()
                                .nth(x)
                                .unwrap_or(BLANK_CELL)
                                != BLACK_CELL
                    })
                    .count();

                is_clue_start = true;

                down_clues.insert(
                    clue_number,
                    ClueInfo {
                        length: clue_length,
                        x,
                        y,
                    },
                );
            }

            if is_clue_start {
                clue_number += 1;
            }
        }
    }

    CluesInfo {
        across: across_clues,
        down: down_clues,
    }
}

pub fn get_puz_json(puzzle: &Puzzle) -> Result<String, serde_json::Error> {
    serde_json::to_string(&puzzle)
}

fn read_puzzle_from_json(json_output_path: &PathBuf) -> Result<Puzzle, Box<dyn Error>> {
    let file = File::open(json_output_path)?;
    let reader = std::io::BufReader::new(file);
    let puzzle = serde_json::from_reader(reader)?;
    Ok(puzzle)
}
