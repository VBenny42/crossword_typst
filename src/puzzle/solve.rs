use std::error::Error;

use crate::pdfgen::PdfCompiler;
use crate::puzzle::clues::interweave_guess;
use crate::puzzle::input;
use crate::types::{Direction, PuzzleState, BLACK_CELL, BLANK_CELL};

macro_rules! try_or_continue {
    ($expr:expr, $msg:expr) => {
        match $expr {
            Ok(val) => val,
            Err(e) => {
                println!("{} {e}", $msg);
                continue;
            }
        }
    };
}

macro_rules! solve_clue_input {
    ($self:expr, $direction:expr) => {
        println!("Please enter the clue number [+ guess]:");

        let input_string: String = try_or_continue!(input(), "Invalid input. Error:");

        let mut split = input_string.split_whitespace();

        let clue_number = try_or_continue!(
            split
                .next()
                .expect("Expected at least one word")
                .parse::<u8>(),
            "Invalid input, please enter a number. Error:"
        );
        try_or_continue!(
            $self.solve_clue(clue_number, $direction, split.next()),
            "Solve:"
        );
    };
}

#[derive(Default)]
struct LastSolve {
    clue_number: u8,
    direction: Direction,
    max_length: usize,
    set: bool,
}

impl PuzzleState {
    pub fn solve_puzzle(&mut self) -> Result<(), Box<dyn Error>> {
        let mut should_recompile = false;

        let compiler = PdfCompiler::new(self.args.pdf_style);

        self.update_clues_status();
        compiler.compile_pdf(self)?;

        println!("Can solve clues by entering the clue number followed by your guess, e.g., `<CLUENUMBER> <GUESS>`.");
        println!("Can also pick a clue to solve by entering the clue number on its own, e.g., `<CLUENUMBER>`, and then entering your guess when prompted.");
        println!("This only works if the clue is not any number between 1 and 8, which are reserved for the menu options.");
        println!("Can also reveal a clue by entering the clue number followed by your guess, e.g., `7 <CLUENUMBER>`.");
        println!("Can also type the guess for last solved clue and the solver will try and use it, e.g. `<GUESS>`.");

        let mut last_solve = LastSolve::default();

        loop {
            if self.puzzle.grid.blank == self.puzzle.grid.solution {
                println!("Congratulations! You've solved the puzzle!");
                break;
            }

            println!(
                "Your choices are:
1. Solve an across clue
2. Solve a down clue
3. Overwrite file with blank puzzle data
4. Print the current state of the puzzle
5. Remove a clue's answer from the puzzle
6. Remove all wrong answers from the puzzle
7. Reveal a clue in the puzzle
8. Exit"
            );

            let choice: String = input()?;
            match choice.as_str() {
                "1" => {
                    println!("You chose to solve an across clue.");
                    solve_clue_input!(self, Direction::Across);
                    should_recompile = true;
                }
                "2" => {
                    println!("You chose to solve a down clue.");
                    solve_clue_input!(self, Direction::Down);
                    should_recompile = true;
                }
                "3" => {
                    println!("Overwriting file with blank puzzle data...");
                    let blank_puzzle_grid = self
                        .puzzle
                        .grid
                        .blank
                        .iter()
                        .map(|row| {
                            row.chars()
                                .map(|c| {
                                    if c == BLACK_CELL {
                                        BLACK_CELL
                                    } else {
                                        BLANK_CELL
                                    }
                                })
                                .collect()
                        })
                        .collect();

                    self.puzzle.grid.blank.clone_from(&blank_puzzle_grid);
                    should_recompile = true;
                }
                "4" => {
                    println!("Current state of the puzzle: {self}");
                }
                "5" => {
                    println!("You chose to remove a clue's answer. Please enter the clue number:");
                    let clue_number: u8 = try_or_continue!(input(), "Invalid digit:");

                    try_or_continue!(self.remove_clue_answer(clue_number), "Error removing clue:");

                    should_recompile = true;
                }
                "6" => {
                    println!("Removing all wrong answers from the puzzle...");
                    if self.remove_wrong_answers() {
                        println!("No wrong answers to remove!");
                    } else {
                        println!("Wrong answers found and removed.");
                        should_recompile = true;
                    }
                }
                "7" => {
                    println!("You chose to reveal a clue's answer. Please enter the clue number:");
                    let clue_number: u8 = try_or_continue!(input(), "Invalid digit:");

                    try_or_continue!(
                        self.reveal_clue_answer(clue_number),
                        "Error revealing clue:"
                    );

                    should_recompile = true;
                }
                "8" => {
                    println!("Exiting...");
                    self.args.output_format.write_puzzle_to_file(
                        &self.args.output_path,
                        &self.args.puzzle_file_path,
                        &self.puzzle,
                        &self.clues_info,
                    )?;
                    break;
                }
                s if let Some((clue_number, guess)) = self.parse_clue_input(s) => {
                    let (direction, clue_info) = try_or_continue!(
                        self.select_clue(clue_number, guess),
                        "Error picking clue direction:"
                    );

                    last_solve.set = true;
                    last_solve.clue_number = clue_number;
                    last_solve.max_length = clue_info.length;
                    last_solve.direction = direction;

                    try_or_continue!(self.solve_clue(clue_number, direction, guess), "Solve:");

                    should_recompile = true;
                }
                s if s.starts_with('7') => {
                    let mut split = s[1..].split_whitespace();

                    let clue_number: u8 =
                        try_or_continue!(split.next().unwrap().parse(), "Invalid clue number:");

                    try_or_continue!(
                        self.reveal_clue_answer(clue_number),
                        "Error revealing clue:"
                    );

                    should_recompile = true;
                }
                s if s.is_ascii() && last_solve.set && s.len() <= last_solve.max_length => {
                    // Just need to check if length is less than last clue's total length
                    // solve_clue will error out if it's bad input
                    // The ascii check is just nice sanity check
                    try_or_continue!(
                        self.solve_clue(last_solve.clue_number, last_solve.direction, Some(s)),
                        "Solve retry:"
                    );

                    let (word_so_far, _) =
                        self.get_clue_so_far(last_solve.clue_number, last_solve.direction);
                    let any_blanks = word_so_far.chars().any(|c| c == BLANK_CELL);

                    // If there any blanks left after solve, the last solve should still be set
                    last_solve.set = any_blanks;

                    should_recompile = true;
                }
                s => println!("Invalid choice, please try again. {s}"),
            }

            if should_recompile {
                let start = std::time::Instant::now();

                self.update_clues_status();

                compiler.compile_pdf(self)?;
                self.args.output_format.write_puzzle_to_file(
                    &self.args.output_path,
                    &self.args.puzzle_file_path,
                    &self.puzzle,
                    &self.clues_info,
                )?;

                println!("Took: {:?}", start.elapsed());

                should_recompile = false;
            }
        }

        Ok(())
    }

    fn solve_clue(
        &mut self,
        number: u8,
        direction: Direction,
        passed_guess: Option<&str>,
    ) -> Result<(), Box<dyn Error>> {
        let clue_info = match direction {
            Direction::Across => self
                .clues_info
                .across
                .get(&number)
                .ok_or_else(|| format!("Clue number {number} not found in across clues"))?,
            Direction::Down => self
                .clues_info
                .down
                .get(&number)
                .ok_or_else(|| format!("Clue number {number} not found in down clues"))?,
        };

        let clues = match direction {
            Direction::Across => &self.puzzle.clues.across,
            Direction::Down => &self.puzzle.clues.down,
        };

        let clue_text = clues.get(&u16::from(number)).map_or_else(
            || format!("Unknown clue {number}"),
            std::clone::Clone::clone,
        );

        let (word_so_far, solution_word) = self.get_clue_so_far(number, direction);

        if self.args.show_clue_length {
            println!("{number}. {clue_text}, {direction}. `{word_so_far}`");
        } else {
            println!(
                "{}. {} ({}), {}. `{word_so_far}`",
                number, clue_text, clue_info.length, direction
            );
        }

        if word_so_far == solution_word {
            return Err("You already solved this clue!".into());
        }

        let mut guess: String = if let Some(passed_guess) = passed_guess {
            passed_guess.to_uppercase()
        } else {
            println!("Input your guess:");
            input::<String>()?.to_uppercase()
        };

        if guess.len() != clue_info.length {
            if word_so_far.chars().filter(|c| *c == BLANK_CELL).count() == guess.len() {
                guess = interweave_guess(&word_so_far, &guess);
            } else {
                return Err(format!(
                "Your guess must be {} characters long. Please try again. Your guess length: {}",
                clue_info.length,
                guess.len()
            )
                .into());
            }
        }

        if guess
            .chars()
            .zip(word_so_far.chars())
            .any(|(a, b)| a != b && b != BLANK_CELL)
            && guess != solution_word
        {
            println!(
                "At least one of the letters in your guess differs from the solved clue so far."
            );
            println!("Are you sure you want to add it? y/n");
            match input::<String>()?.trim() {
                "y" => {}
                "n" => {
                    println!("Cancelling solve.");
                    return Ok(());
                }
                _ => {
                    println!("Invalid input. Cancelling solve.");
                    return Ok(());
                }
            }
        }

        if self.args.show_correct_letters_only {
            guess = guess
                .chars()
                .zip(solution_word.chars())
                .map(|(a, b)| if a == b { a } else { BLANK_CELL })
                .collect();

            if guess == word_so_far {
                return Err("Your guess did not add any new correct letters.".into());
            }
        }

        for (i, ch) in guess.chars().enumerate() {
            let (x, y) = match direction {
                Direction::Across => (clue_info.x + i, clue_info.y),
                Direction::Down => (clue_info.x, clue_info.y + i),
            };

            // SAFETY: ch is guaranteed to be valid UTF-8
            unsafe {
                self.puzzle.grid.blank[y].as_bytes_mut()[x] = ch as u8;
            }
        }

        Ok(())
    }
}
