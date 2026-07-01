use core::fmt;
use std::{collections::HashMap, str::FromStr};

use puz_parse::Puzzle;

use crate::Args;

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

    pub args: Args,
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
        match s {
            "1" => Ok(Direction::Across),
            "2" => Ok(Direction::Down),
            _ => Err(format!("Invalid direction number: {}", s)),
        }
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
            _ => Err(format!("Invalid PdfStyle: {}", s)),
        }
    }
}
