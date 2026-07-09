use core::fmt;
use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{Read, Write},
    path::PathBuf,
    str::FromStr,
};

use puz_parse::Puzzle;

use crate::Args;

#[derive(Clone, Copy, Debug)]
pub struct ClueInfo {
    pub length: usize,
    pub x: usize,
    pub y: usize,
}

#[derive(Debug)]
pub struct CluesInfo {
    pub across: HashMap<u8, ClueInfo>,
    pub down: HashMap<u8, ClueInfo>,
}

#[derive(Debug)]
pub struct PuzzleState {
    pub puzzle: Puzzle,
    pub clues_info: CluesInfo,

    pub args: Args,
}

pub const BLANK_CELL: char = '-';
pub const BLACK_CELL: char = '.';

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Direction {
    Across,
    Down,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Direction::Down => write!(f, "Down"),
            Direction::Across => write!(f, "Across"),
        }
    }
}

impl FromStr for Direction {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(Direction::Across),
            "2" => Ok(Direction::Down),
            _ => Err(format!("Invalid direction number: {s}")),
        }
    }
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Json,
    Puz,
}

impl FromStr for OutputFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(OutputFormat::Json),
            "puz" => Ok(OutputFormat::Puz),
            _ => Err(format!("Invalid OutputFormat: {s}")),
        }
    }
}

impl OutputFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            OutputFormat::Json => "json",
            OutputFormat::Puz => "puz",
        }
    }

    pub fn write_puzzle_to_file(
        &self,
        output_path: &PathBuf,
        puz_path: &PathBuf,
        puzzle: &Puzzle,
    ) -> Result<(), Box<dyn Error>> {
        match self {
            OutputFormat::Json => {
                let file = File::create(output_path)?;
                serde_json::to_writer(file, &puzzle)?;
            }
            OutputFormat::Puz => {
                let mut file = File::open(puz_path)?;
                let mut data = vec![];
                file.read_to_end(&mut data)?;

                let write_length: usize = (puzzle.info.width * puzzle.info.height).into();
                let start_position = 0x34 + write_length;
                let blank_string = puzzle.grid.blank.concat();

                assert_eq!(
                    blank_string.len(),
                    write_length,
                    "blank grid string length ({}) does not match expected write_length ({})",
                    blank_string.len(),
                    write_length
                );

                data[start_position..start_position + write_length]
                    .copy_from_slice(blank_string.as_bytes());

                let mut output_file = File::create(output_path)?;
                output_file.write_all(&data)?
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Copy)]
pub enum PdfStyle {
    Normal,
    Larger,
    Landscape,
}

impl FromStr for PdfStyle {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "normal" => Ok(PdfStyle::Normal),
            "larger" => Ok(PdfStyle::Larger),
            "landscape" => Ok(PdfStyle::Landscape),
            _ => Err(format!("Invalid PdfStyle: {s}")),
        }
    }
}
