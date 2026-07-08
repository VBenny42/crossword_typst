use std::error::Error;

use crate::pdfgen::PdfCompiler;
use crate::puzzle::{initialize_puzzle, input, write_puzzle_to_json};
use crate::types::{Direction, PuzzleState, BLANK_CELL};

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

impl PuzzleState {
    pub fn solve_puzzle(&mut self) -> Result<(), Box<dyn Error>> {
        let mut should_recompile = false;

        let compiler = PdfCompiler::new(self.args.pdf_style);
        compiler.compile_pdf(self);

        println!("Can solve clues by entering the clue number followed by your guess, e.g., `<CLUENUMBER> <ANSWER>`.");
        println!("Can also pick a clue to solve by entering the clue number on its own, e.g., `<CLUENUMBER>`, and then entering your guess when prompted.");
        println!("This only works if the clue is not any number between 1 and 8, which are reserved for the menu options.");
        println!("Can also reveal a clue by entering the clue number followed by your guess, e.g., `7 <CLUENUMBER>`.");

        loop {
            if self.puzzle.grid.blank == self.puzzle.grid.solution {
                println!("Congratulations! You've solved the puzzle!");
                break;
            }

            println!(
                "Your choices are:
1. Solve an across clue
2. Solve a down clue
3. Overwrite JSON file with blank puzzle data
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
                    println!("Overwriting JSON file with blank puzzle data...");
                    let blank_puzzle = initialize_puzzle(&self.args.puzzle_file_path)?;
                    should_recompile = true;
                    self.puzzle.grid.blank.clone_from(&blank_puzzle.grid.blank);
                }
                "4" => {
                    println!("Current state of the puzzle:");
                    self.print_puzzle();
                }
                "5" => {
                    println!("You chose to remove a clue's answer. Please enter the clue number:");
                    let clue_number: u8 = try_or_continue!(input(), "Invalid digit:");

                    try_or_continue!(self.remove_clue_answer(clue_number), "Error:");

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
                    write_puzzle_to_json(&self.args.json_output_path, &self.puzzle)?;
                    break;
                }
                // s if self.check_for_clue_number(s).is_some() => {
                s if let Some(clue_number) = self.check_for_clue_number(s) => {
                    let guess = match s.chars().any(|c| c.is_ascii_whitespace()) {
                        true => s.split_whitespace().nth(1),
                        // Supporting user typing in <CLUENUMBER><GUESS>
                        // i.e. no space(s) between
                        // Since check_for_clue_number returned true,
                        // there must be a valid clue number
                        false => {
                            let digits = if clue_number < 10 {
                                1
                            } else if clue_number < 100 {
                                2
                            } else {
                                3
                            };
                            match digits == s.len() {
                                // only number was sent
                                true => None,
                                false => Some(&s[digits..]),
                            }
                        }
                    };

                    let (direction, _) = try_or_continue!(
                        self.pick_direction_clue_info(clue_number, guess),
                        "Error picking clue direction:"
                    );

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
                s => println!("Invalid choice, please try again. {s}"),
            }

            if should_recompile {
                let start = std::time::Instant::now();

                compiler.compile_pdf(self);
                write_puzzle_to_json(&self.args.json_output_path, &self.puzzle)?;

                println!("Took: {:?}", start.elapsed());
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
                .ok_or("Clue number not found in across clues")?,
            Direction::Down => self
                .clues_info
                .down
                .get(&number)
                .ok_or("Clue number not found in down clues")?,
        };

        let clues = match direction {
            Direction::Across => &self.puzzle.clues.across,
            Direction::Down => &self.puzzle.clues.down,
        };

        let clue_text = clues
            .get(&u16::from(number))
            .map_or("Unknown clue", |s| s.as_str());

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
            return Err(format!(
                "Your guess must be {} characters long. Please try again. Your guess length: {}",
                clue_info.length,
                guess.len()
            )
            .into());
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

        self.puzzle.grid.blank = self
            .puzzle
            .grid
            .blank
            .iter()
            .enumerate()
            .map(|(y, row)| {
                row.chars()
                    .enumerate()
                    .map(|(x, c)| {
                        if direction == Direction::Across
                            && y == clue_info.y
                            && x >= clue_info.x
                            && x < (clue_info.x + clue_info.length)
                        {
                            guess.chars().nth(x - clue_info.x).unwrap_or(c)
                        } else if direction == Direction::Down
                            && x == clue_info.x
                            && y >= clue_info.y
                            && y < (clue_info.y + clue_info.length)
                        {
                            guess.chars().nth(y - clue_info.y).unwrap_or(c)
                        } else {
                            c
                        }
                    })
                    .collect()
            })
            .collect();

        Ok(())
    }
}
