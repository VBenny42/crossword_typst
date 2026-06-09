use core::fmt;
use std::{collections::HashMap, path::PathBuf, str::FromStr};

use puz_parse::Puzzle;

#[derive(Debug)]
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
    pub puzzle_path: PathBuf,
    pub json_output_path: PathBuf,

    pub nord_colors: bool,
    pub hide_completed_clues: bool,
}

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
        match s.trim() {
            "1" => Ok(Direction::Across),
            "2" => Ok(Direction::Down),
            _ => Err(format!("Invalid direction number: {}", s.trim())),
        }
    }
}
