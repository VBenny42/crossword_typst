use std::error::Error;

use crate::puzzle::input;
use crate::types::{Direction, PuzzleState, BLANK_CELL};

impl PuzzleState {
    pub(crate) fn remove_wrong_answers(&mut self) -> bool {
        let old_blank = self.puzzle.grid.blank.clone();

        self.puzzle.grid.blank = self
            .puzzle
            .grid
            .blank
            .iter()
            .zip(&self.puzzle.grid.solution)
            .map(|(blank_row, solution_row)| {
                blank_row
                    .chars()
                    .zip(solution_row.chars())
                    .map(
                        |(blank, solution)| {
                            if blank == solution {
                                blank
                            } else {
                                BLANK_CELL
                            }
                        },
                    )
                    .collect()
            })
            .collect();

        old_blank == self.puzzle.grid.blank
    }

    pub(crate) fn remove_clue_answer(&mut self, number: u8) -> Result<(), Box<dyn Error>> {
        let (direction, clue_info) = match (
            self.clues_info.across.get(&number),
            self.clues_info.down.get(&number),
        ) {
            (Some(clue), None) => (Direction::Across, clue),
            (None, Some(clue)) => (Direction::Down, clue),
            (Some(a_clue), Some(d_clue)) => {
                let (a_word_so_far, _) = self.get_clue_so_far(number, Direction::Across);
                let (d_word_so_far, _) = self.get_clue_so_far(number, Direction::Down);

                // Only first of across filled in, then remove whole down, and vice versa.
                // If both have partial fills, ask the user which one to remove.
                match (
                    a_word_so_far[1..].chars().all(|c| c == BLANK_CELL),
                    d_word_so_far[1..].chars().all(|c| c == BLANK_CELL),
                ) {
                    (true, false) => (Direction::Down, d_clue),
                    (false, true) => (Direction::Across, a_clue),
                    _ => {
                        println!("Clue number {number} exists in both across and down clues. Please choose which one to remove:");
                        println!("1. Across clue");
                        println!("2. Down clue");

                        let choice: Direction = input()?;

                        match choice {
                            Direction::Across => (Direction::Across, a_clue),
                            Direction::Down => (Direction::Down, d_clue),
                        }
                    }
                }
            }
            (None, None) => {
                return Err("Clue number not found in either across or down clues".into());
            }
        };

        for (y, row) in self.puzzle.grid.blank.iter_mut().enumerate() {
            let mut chars: Vec<char> = row.chars().collect();
            for (x, c) in chars.iter_mut().enumerate() {
                let in_word = (direction == Direction::Across
                    && y == clue_info.y
                    && x >= clue_info.x
                    && x < (clue_info.x + clue_info.length))
                    || (direction == Direction::Down
                        && x == clue_info.x
                        && y >= clue_info.y
                        && y < (clue_info.y + clue_info.length));

                if in_word {
                    *c = BLANK_CELL
                }
            }
            *row = chars.into_iter().collect();
        }

        Ok(())
    }

    pub(crate) fn reveal_clue_answer(&mut self, number: u8) -> Result<(), Box<dyn Error>> {
        let (direction, clue_info) = self.select_clue(number, None)?;
        let clue_info = *clue_info;

        for (y, row) in self.puzzle.grid.blank.iter_mut().enumerate() {
            let mut chars: Vec<char> = row.chars().collect();
            for (x, c) in chars.iter_mut().enumerate() {
                let in_word = (direction == Direction::Across
                    && y == clue_info.y
                    && x >= clue_info.x
                    && x < (clue_info.x + clue_info.length))
                    || (direction == Direction::Down
                        && x == clue_info.x
                        && y >= clue_info.y
                        && y < (clue_info.y + clue_info.length));

                if in_word {
                    *c = self.puzzle.grid.solution[y].chars().nth(x).unwrap();
                }
            }
            *row = chars.into_iter().collect();
        }

        Ok(())
    }
}
