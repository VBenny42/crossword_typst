use std::{cmp::Ordering, error::Error};

use crate::{
    puzzle::input,
    types::{ClueInfo, CluesInfo, Direction, PuzzleState, BLANK_CELL},
};

impl CluesInfo {
    pub fn get_clues_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }
}

impl PuzzleState {
    pub(crate) fn parse_clue_input<'a>(&self, input: &'a str) -> Option<(u8, Option<&'a str>)> {
        let count = input.chars().take_while(char::is_ascii_digit).count();

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

        let clue_number = first_word.parse::<u8>().ok().filter(|num| {
            self.clues_info.across.contains_key(num) || self.clues_info.down.contains_key(num)
        })?;

        let guess = (!second_word.is_empty()).then_some(second_word);

        Some((clue_number, guess))
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
                .map(|row| row.as_bytes()[clue_info.x] as char)
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
                .map(|row| row.as_bytes()[clue_info.x] as char)
                .collect(),
        };

        (word_so_far, solution_word)
    }

    pub(crate) fn select_clue(
        &self,
        number: u8,
        guess: Option<&str>,
    ) -> Result<(Direction, &ClueInfo), Box<dyn Error>> {
        let across_clue = self.clues_info.across.get(&number);
        let down_clue = self.clues_info.down.get(&number);

        match (across_clue, down_clue) {
            (Some(clue), None) => return Ok((Direction::Across, clue)),
            (None, Some(clue)) => return Ok((Direction::Down, clue)),
            (None, None) => {
                return Err("Clue number not found in either across or down clues".into())
            }
            (Some(_), Some(_)) => {
                // Move onto seeing if one of the clues are already solved
            }
        }

        let a_clue = across_clue.unwrap();
        let d_clue = down_clue.unwrap();

        let (a_word_so_far, a_solution_word) = self.get_clue_so_far(number, Direction::Across);
        let (d_word_so_far, d_solution_word) = self.get_clue_so_far(number, Direction::Down);

        match (
            a_word_so_far == a_solution_word,
            d_word_so_far == d_solution_word,
        ) {
            (true, false) => return Ok((Direction::Down, d_clue)),
            (false, true) => return Ok((Direction::Across, a_clue)),
            (true, true) => return Err("Both across and down clues are already solved.".into()),
            (false, false) => {
                // Move onto comparing clue lengths
            }
        }

        if let Some(guess) = guess {
            // Guess can possibly be interweaved to get the an actual guess if it is less than the
            // needed length
            match (
                a_word_so_far.chars().filter(|c| *c == BLANK_CELL).count() == guess.len(),
                d_word_so_far.chars().filter(|c| *c == BLANK_CELL).count() == guess.len(),
            ) {
                (true, false) => return Ok((Direction::Across, a_clue)),
                (false, true) => return Ok((Direction::Down, d_clue)),
                _ => {
                    // Proceed to check for length and closeness
                }
            };

            // Use interweaved guesses once they are both valid
            let across_interweaved = interweave_guess(&a_word_so_far, guess);
            let down_interweaved = interweave_guess(&d_word_so_far, guess);

            match (
                across_interweaved.len() == a_clue.length,
                down_interweaved.len() == d_clue.length,
            ) {
                (true, false) => return Ok((Direction::Across, a_clue)),
                (false, true) => return Ok((Direction::Down, d_clue)),
                (false, false) => {
                    return Err(
                        "Your guess does not match the length of either across or down clues."
                            .into(),
                    )
                }
                (true, true) => {
                    // Check which one is closer to the actual answer
                }
            }

            let across_closeness = across_interweaved
                .chars()
                .zip(a_solution_word.chars())
                .filter(|(guess, solution)| guess.eq_ignore_ascii_case(solution))
                .count();
            let down_closeness = down_interweaved
                .chars()
                .zip(d_solution_word.chars())
                .filter(|(guess, solution)| guess.eq_ignore_ascii_case(solution))
                .count();

            match across_closeness.cmp(&down_closeness) {
                // It's more likely across
                Ordering::Greater => return Ok((Direction::Across, a_clue)),
                // It's more likely down
                Ordering::Less => return Ok((Direction::Down, d_clue)),
                Ordering::Equal => {
                    // Continue to ask the user for direction choice
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

    pub(crate) fn update_clues_status(&mut self) {
        let across_clues = self.clues_info.across.keys().copied().collect::<Vec<_>>();
        for clue_num in across_clues {
            let (clue_so_far, solution_word) = self.get_clue_so_far(clue_num, Direction::Across);
            if clue_so_far == solution_word {
                let clue_info = self
                    .clues_info
                    .across
                    .get_mut(&clue_num)
                    .expect("Always going to exist");
                clue_info.solved = true;
            };
        }

        let down_clues = self.clues_info.down.keys().copied().collect::<Vec<_>>();
        for clue_num in down_clues {
            let (clue_so_far, solution_word) = self.get_clue_so_far(clue_num, Direction::Down);
            if clue_so_far == solution_word {
                let clue_info = self
                    .clues_info
                    .down
                    .get_mut(&clue_num)
                    .expect("Always going to exist");
                clue_info.solved = true;
            };
        }
    }
}

pub(crate) fn interweave_guess(word_so_far: &str, guess: &str) -> String {
    let mut interweaved = String::new();

    let mut guess_iter = guess.chars();

    for ch in word_so_far.chars() {
        if ch == BLANK_CELL {
            interweaved.push(
                guess_iter
                    .next()
                    .expect("guess length should be the same count as the blank cells"),
            );
        } else {
            interweaved.push(ch);
        }
    }

    interweaved
}
