use std::error::Error;

use crate::{
    puzzle::input,
    types::{ClueInfo, Direction, PuzzleState, BLANK_CELL},
};

impl PuzzleState {
    pub(crate) fn check_for_clue_number(&self, input: &str) -> Option<u8> {
        let count = input.chars().take_while(|c| c.is_ascii_digit()).count();

        if count == 0 {
            return None;
        }

        let first_word = &input[0..count];
        let second_word = input[count..].trim();

        // Means that the user is trying to reveal a clue by cluenumber, not solve a clue
        if first_word == "7"
            && !second_word.is_empty()
            && second_word.chars().all(|c| c.is_ascii_digit())
        {
            return None;
        }

        first_word.parse::<u8>().ok().filter(|num| {
            self.clues_info.across.contains_key(num) || self.clues_info.down.contains_key(num)
        })
    }

    pub(crate) fn get_clue_so_far(&self, number: u8, direction: Direction) -> (String, String) {
        let clue_info = match direction {
            Direction::Across => self
                .clues_info
                .across
                .get(&number)
                .ok_or("Clue number not found in across clues")
                .unwrap(),
            Direction::Down => self
                .clues_info
                .down
                .get(&number)
                .ok_or("Clue number not found in down clues")
                .unwrap(),
        };

        let word_so_far: String = match direction {
            Direction::Across => self.puzzle.grid.blank[clue_info.y]
                [clue_info.x..(clue_info.x + clue_info.length)]
                .to_string(),
            Direction::Down => self
                .puzzle
                .grid
                .blank
                .iter()
                .skip(clue_info.y)
                .take(clue_info.length)
                .map(|row| row.chars().nth(clue_info.x).unwrap_or(BLANK_CELL))
                .collect(),
        };
        let solution_word: String = match direction {
            Direction::Across => self.puzzle.grid.solution[clue_info.y]
                [clue_info.x..(clue_info.x + clue_info.length)]
                .to_string(),
            Direction::Down => self
                .puzzle
                .grid
                .solution
                .iter()
                .skip(clue_info.y)
                .take(clue_info.length)
                .map(|row| row.chars().nth(clue_info.x).unwrap_or(BLANK_CELL))
                .collect(),
        };

        (word_so_far, solution_word)
    }

    pub(crate) fn pick_direction_clue_info(
        &self,
        number: u8,
        guess: Option<&str>,
    ) -> Result<(Direction, &ClueInfo), Box<dyn Error>> {
        let across_clue = self.clues_info.across.get(&number);
        let down_clue = self.clues_info.down.get(&number);

        match (across_clue, down_clue) {
            (Some(clue), None) => Ok((Direction::Across, clue)),
            (None, Some(clue)) => Ok((Direction::Down, clue)),
            (Some(a_clue), Some(d_clue)) => {
                let (a_word_so_far, a_solution_word) =
                    self.get_clue_so_far(number, Direction::Across);
                let (d_word_so_far, d_solution_word) =
                    self.get_clue_so_far(number, Direction::Down);

                match (
                    a_word_so_far == a_solution_word,
                    d_word_so_far == d_solution_word,
                ) {
                    (true, false) => Ok((Direction::Down, d_clue)),
                    (false, true) => Ok((Direction::Across, a_clue)),
                    (true, true) => Err("Both across and down clues are already solved.".into()),
                    (false, false) => {
                        if let Some(guess) = guess {
                            match (
                                guess.len() == a_clue.length,
                                guess.len() == d_clue.length,
                            ) {
                                (true, false) => return Ok((Direction::Across, a_clue)),
                                (false, true) => return Ok((Direction::Down, d_clue)),
                                // Should both of these cases just continue, and solve_clue would
                                // return the wrong length error?
                                (true, true) => {
                                    // Continue to ask the user for direction choice
                                }
                                (false, false) => {
                                    return Err("Your guess does not match the length of either across or down clues.".into())
                                }
                            }
                        }

                        println!("Clue number {number} exists in both across and down clues. Please choose clue direction:");
                        println!("1. Across clue");
                        println!("2. Down clue");

                        let choice: Direction = input()?;

                        match choice {
                            Direction::Across => Ok((Direction::Across, a_clue)),
                            Direction::Down => Ok((Direction::Down, d_clue)),
                        }
                    }
                }
            }
            (None, None) => Err("Clue number not found in either across or down clues".into()),
        }
    }
}
