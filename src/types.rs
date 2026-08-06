use core::fmt;
use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    path::PathBuf,
    str::FromStr,
};

use puz_parse::Puzzle;
use serde::Serialize;

use crate::Args;

#[derive(Clone, Copy, Debug, Serialize)]
pub struct ClueInfo {
    pub length: usize,
    pub x: usize,
    pub y: usize,

    pub solved: bool,
    pub new_solve: bool,
}

#[derive(Debug, Serialize)]
pub struct CluesInfo {
    pub across: HashMap<u8, ClueInfo>,
    pub down: HashMap<u8, ClueInfo>,
}

#[derive(Debug)]
pub struct PuzzleState {
    pub puzzle: Puzzle,
    pub clues_info: CluesInfo,
    pub intersections: Intersections,

    pub args: Args,
}

pub const BLANK_CELL: char = '-';
pub const BLACK_CELL: char = '.';

#[derive(Debug, Hash, PartialEq, Eq, Clone, Copy, Default)]
pub enum Direction {
    #[default]
    Across,
    Down,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Self::Down => write!(f, "Down"),
            Self::Across => write!(f, "Across"),
        }
    }
}

impl FromStr for Direction {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1" => Ok(Self::Across),
            "2" => Ok(Self::Down),
            _ => Err(format!("Invalid direction number: {s}")),
        }
    }
}

impl Direction {
    pub fn alternate(&self) -> Self {
        match *self {
            Self::Across => Self::Down,
            Self::Down => Self::Across,
        }
    }
}

#[derive(clap::ValueEnum, Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    Json,
    #[default]
    Puz,
}

impl FromStr for OutputFormat {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Self::Json),
            "puz" => Ok(Self::Puz),
            _ => Err(format!("Invalid OutputFormat: {s}")),
        }
    }
}

impl OutputFormat {
    pub const fn extension(self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Puz => "puz",
        }
    }

    pub fn write_puzzle_to_file(
        self,
        output_path: &PathBuf,
        puz_path: &PathBuf,
        puzzle: &Puzzle,
        clues_info: &CluesInfo,
    ) -> Result<(), Box<dyn Error>> {
        match self {
            Self::Json => {
                let file = File::create(output_path)?;
                serde_json::to_writer(file, &puzzle)?;

                let clue_info_path = output_path
                    .parent()
                    .unwrap_or_else(|| std::path::Path::new("."))
                    .join("clues_info.json");
                let clue_info_file = File::create(clue_info_path)?;
                serde_json::to_writer(clue_info_file, &clues_info)?;
            }
            Self::Puz => {
                let mut puz_file = fs::read(puz_path)?;

                // .puz format specifies that solution string is at 0x34
                // and is width x length bytes long,
                // with the blank string directly after
                let write_length: usize = puzzle.info.width as usize * puzzle.info.height as usize;
                let start_position = 0x34 + write_length;

                let blank_string = puzzle.grid.blank.concat();

                assert!(
                    blank_string.is_ascii(),
                    "Blank grid should only have ascii chars"
                );
                assert_eq!(
                    blank_string.len(),
                    write_length,
                    "Blank grid string length ({}) does not match expected write_length ({})",
                    blank_string.len(),
                    write_length
                );

                puz_file[start_position..start_position + write_length]
                    .copy_from_slice(blank_string.as_bytes());

                fs::write(output_path, &puz_file)?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Copy, Default)]
pub enum PdfStyle {
    #[default]
    Normal,
    Larger,
    Landscape,
}

impl FromStr for PdfStyle {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "normal" => Ok(Self::Normal),
            "larger" => Ok(Self::Larger),
            "landscape" => Ok(Self::Landscape),
            _ => Err(format!("Invalid PdfStyle: {s}")),
        }
    }
}

pub type Intersections = Vec<Vec<Option<[u8; 2]>>>;
