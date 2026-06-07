use core::fmt;
use std::collections::HashMap;

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
    pub puzzle_path: String,
    pub json_output_path: String,
}

#[derive(Debug, PartialEq, Eq)]
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
