#![warn(clippy::pedantic)]
use clap::Parser;

use std::{fs::File, str::FromStr};

use crate::types::PuzzleState;

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
    #[arg(short, long, default_value = JSON_OUTPUT_PATH)]
    json_output_path: Option<String>,

    #[arg(short, long)]
    puzzle_file_path: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut state = PuzzleState::new(&args.puzzle_file_path, &args.json_output_path.unwrap())?;

    if File::open(&state.json_output_path).is_ok() {
    } else {
        println!("JSON file does not exist. Creating a new one...");
        state.write_puzzle_to_json()?;
    }

    let read_puzzle = state.read_puzzle_from_json()?;

    if read_puzzle.info.title != state.puzzle.info.title {
        eprintln!("Warning: The puzzle title in the JSON file does not match the original puzzle. Overwriting JSON file with blank puzzle data...");
        state.write_puzzle_to_json()?;
    }

    state.puzzle.grid.blank.clone_from(&read_puzzle.grid.blank);

    state.solve_puzzle()?;

    Ok(())
}
