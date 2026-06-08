use clap::{ArgAction::SetTrue, Parser};
use std::{fs::File, path::PathBuf, str::FromStr};

use crate::types::PuzzleState;

mod pdfgen;
mod puzzle;
mod types;

static JSON_OUTPUT_PATH: &str = "src/output.json";

fn input<T: FromStr>() -> Result<T, <T as FromStr>::Err> {
    let mut input: String = String::with_capacity(64);

    std::io::stdin()
        .read_line(&mut input)
        .expect("Input could not be read");

    input.trim().parse()
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = JSON_OUTPUT_PATH, help="Path to where json should be saved")]
    json_output_path: Option<PathBuf>,

    #[arg(short, long, help = "Path to .puz file to be read")]
    // Change to path_buf
    puzzle_file_path: std::path::PathBuf,

    #[arg(short, long, action = SetTrue, help = "Just write to json file and exit")]
    write_to_json_only: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut state = PuzzleState::new(args.puzzle_file_path, args.json_output_path.unwrap())?;

    if File::open(&state.json_output_path).is_ok() {
    } else {
        println!("JSON file does not exist. Creating a new one...");
        state.write_puzzle_to_json()?;
    }

    let read_puzzle = state.read_puzzle_from_json()?;

    if read_puzzle.info.title == state.puzzle.info.title {
        state.puzzle.grid.blank.clone_from(&read_puzzle.grid.blank);
    } else {
        eprintln!("Warning: The puzzle title in the JSON file does not match the original puzzle. Overwriting JSON file with blank puzzle data...");
        state.write_puzzle_to_json()?;
    }

    if args.write_to_json_only {
        return Ok(());
    } else {
        state.solve_puzzle()?;
    }

    Ok(())
}
