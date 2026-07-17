use std::{borrow::Cow, cmp::Ordering, error::Error};

use crate::{
    puzzle::input,
    types::{ClueInfo, CluesInfo, Direction, PuzzleState, BLANK_CELL},
};

impl CluesInfo {
    pub fn get_clues_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(&self)
    }

    pub fn get_clue_info(&self, clue_number: u8, direction: Direction) -> Option<&ClueInfo> {
        match direction {
            Direction::Across => self.across.get(&clue_number),
            Direction::Down => self.down.get(&clue_number),
        }
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

    pub(crate) fn is_clue_solved(&self, number: u8, direction: Direction) -> bool {
        let clue_info = match direction {
            Direction::Across => self
                .clues_info
                .across
                .get(&number)
                .expect("Clue number not found in across clues"),
            Direction::Down => self
                .clues_info
                .down
                .get(&number)
                .expect("Clue number not found in down clues"),
        };

        match direction {
            Direction::Across => self.puzzle.grid.blank[clue_info.y]
                [clue_info.x..(clue_info.x + clue_info.length)]
                .chars()
                .zip(
                    self.puzzle.grid.solution[clue_info.y]
                        [clue_info.x..(clue_info.x + clue_info.length)]
                        .chars(),
                )
                .all(|(blank_char, solution_char)| blank_char == solution_char),
            Direction::Down => self
                .puzzle
                .grid
                .blank
                .iter()
                .skip(clue_info.y)
                .take(clue_info.length)
                .map(|row| row.as_bytes()[clue_info.x] as char)
                .zip(
                    self.puzzle
                        .grid
                        .solution
                        .iter()
                        .skip(clue_info.y)
                        .take(clue_info.length)
                        .map(|row| row.as_bytes()[clue_info.x] as char),
                )
                .all(|(blank_char, solution_char)| blank_char == solution_char),
        }
    }

    pub(crate) fn get_clue_so_far(
        &self,
        number: u8,
        direction: Direction,
    ) -> (Cow<'_, str>, Cow<'_, str>) {
        let clue_info = match direction {
            Direction::Across => self
                .clues_info
                .across
                .get(&number)
                .expect("Clue number not found in across clues"),
            Direction::Down => self
                .clues_info
                .down
                .get(&number)
                .expect("Clue number not found in down clues"),
        };

        let mut word_so_far: Cow<'_, str> = match direction {
            Direction::Across => Cow::Borrowed(
                &self.puzzle.grid.blank[clue_info.y][clue_info.x..(clue_info.x + clue_info.length)],
            ),
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
        let solution_word: Cow<'_, str> = match direction {
            Direction::Across => Cow::Borrowed(
                &self.puzzle.grid.solution[clue_info.y]
                    [clue_info.x..(clue_info.x + clue_info.length)],
            ),
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

        let mut changed = false;
        let mapped_to_blank = word_so_far
            .chars()
            .zip(solution_word.chars())
            .map(|(wsf, sw)| {
                if wsf != sw && wsf != BLANK_CELL {
                    changed = true;
                    BLANK_CELL
                } else {
                    wsf
                }
            })
            .collect();

        if changed {
            word_so_far = mapped_to_blank;
        }

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

        let a_clue = across_clue.expect("Across clue should be Some by this point");
        let d_clue = down_clue.expect("Down clue should be Some by this point");

        let across_clue_solved = self.is_clue_solved(number, Direction::Across);
        let down_clue_solved = self.is_clue_solved(number, Direction::Down);

        match (across_clue_solved, down_clue_solved) {
            (true, false) => return Ok((Direction::Down, d_clue)),
            (false, true) => return Ok((Direction::Across, a_clue)),
            (true, true) => return Err("Both across and down clues are already solved.".into()),
            (false, false) => {
                // Move onto checking actual guess
            }
        }

        if let Some(guess) = guess {
            let (a_word_so_far, a_solution_word) = self.get_clue_so_far(number, Direction::Across);
            let (d_word_so_far, d_solution_word) = self.get_clue_so_far(number, Direction::Down);

            let a_interweave_count =
                a_word_so_far.chars().filter(|c| *c == BLANK_CELL).count() == guess.len();
            let d_interweave_count =
                d_word_so_far.chars().filter(|c| *c == BLANK_CELL).count() == guess.len();

            let mut a_guess: Cow<str> = Cow::Borrowed(guess);
            let mut d_guess: Cow<str> = Cow::Borrowed(guess);

            // Guess can possibly be interweaved to get the an actual guess if it is less than the
            // needed length
            match (a_interweave_count, d_interweave_count) {
                (true, false) => return Ok((Direction::Across, a_clue)),
                (false, true) => return Ok((Direction::Down, d_clue)),
                (false, false) => {
                    // Move on, but don't modify a_guess or d_guess,
                    // since they can't be interweaved
                }
                (true, true) => {
                    // Check for closeness
                    // Use interweaved guesses once they both have same length
                    a_guess = Cow::Owned(interweave_guess(&a_word_so_far, guess));
                    d_guess = Cow::Owned(interweave_guess(&d_word_so_far, guess));
                }
            }

            match (
                a_guess.len() == a_clue.length,
                d_guess.len() == d_clue.length,
            ) {
                (true, false) => return Ok((Direction::Across, a_clue)),
                (false, true) => return Ok((Direction::Down, d_clue)),
                (false, false) => {
                    if !a_interweave_count && !d_interweave_count {
                        return Err(
                            "Your guess does not match the length of either across or down clues."
                                .into(),
                        );
                    }
                    // Continue to check for closeness
                }
                (true, true) => {
                    // Check which one is closer to the actual answer
                }
            }

            let across_closeness = a_guess
                .chars()
                .zip(a_solution_word.chars())
                .filter(|(guess, solution)| guess.eq_ignore_ascii_case(solution))
                .count();
            let down_closeness = d_guess
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
            if self.is_clue_solved(clue_num, Direction::Across) {
                let clue_info = self
                    .clues_info
                    .across
                    .get_mut(&clue_num)
                    .expect("Always going to exist");
                clue_info.solved = true;
                // Should reset to false if not actually solved but marked as solved
            } else if self
                .clues_info
                .across
                .get(&clue_num)
                .expect("Will always exist")
                .solved
            {
                let clue_info = self
                    .clues_info
                    .across
                    .get_mut(&clue_num)
                    .expect("Always going to exist");
                clue_info.solved = false;
            }
        }

        let down_clues = self.clues_info.down.keys().copied().collect::<Vec<_>>();
        for clue_num in down_clues {
            if self.is_clue_solved(clue_num, Direction::Down) {
                let clue_info = self
                    .clues_info
                    .down
                    .get_mut(&clue_num)
                    .expect("Always going to exist");
                clue_info.solved = true;
            } else if self
                .clues_info
                .down
                .get(&clue_num)
                .expect("Will always exist")
                .solved
            {
                let clue_info = self
                    .clues_info
                    .down
                    .get_mut(&clue_num)
                    .expect("Always going to exist");
                clue_info.solved = false;
            }
        }
    }
}

pub fn interweave_guess(word_so_far: &str, guess: &str) -> String {
    let mut interweaved = String::new();

    let mut guess = guess.chars();

    for ch in word_so_far.chars() {
        if ch == BLANK_CELL {
            interweaved.push(
                guess
                    .next()
                    .expect("guess length should be the same count as the blank cells"),
            );
        } else {
            interweaved.push(ch);
        }
    }

    interweaved
}
