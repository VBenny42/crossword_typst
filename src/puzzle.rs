use puz_parse::{parse_file, Puzzle};
use std::{collections::HashMap, error::Error, fs::File, path::PathBuf};

use crate::{input, pdfgen::PdfCompiler, types};
use types::{ClueInfo, CluesInfo, Direction, PuzzleState};

pub const BLANK_CELL: char = '-';
pub const BLACK_CELL: char = '.';

macro_rules! solve_clue_input {
    ($self:expr, $direction:expr) => {
        println!("Please enter the clue number:");
        let clue_number: u8 = match input() {
            Ok(num) => num,
            Err(e) => {
                println!("Invalid input, please enter a number. Error: {e}");
                continue;
            }
        };
        match $self.solve_clue(clue_number, $direction, None) {
            Ok(()) => {}
            Err(e) => {
                println!("Error solving clue: {e}");
                continue;
            }
        }
    };
}

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

impl PuzzleState {
    pub fn new(args: &crate::Args) -> Result<Self, Box<dyn Error>> {
        let mut puzzle = initialize_puzzle(&args.puzzle_file_path)?;

        if File::open(&args.json_output_path).is_ok() {
        } else {
            eprintln!("JSON file does not exist. Creating a new one...");
            write_puzzle_to_json(&args.json_output_path, &puzzle)?;
        }

        let read_puzzle = read_puzzle_from_json(&args.json_output_path)?;

        if read_puzzle.info.title == puzzle.info.title {
            puzzle.grid.blank.clone_from(&read_puzzle.grid.blank);
        } else {
            eprintln!("Warning: The puzzle title in the JSON file does not match the original puzzle. Overwriting JSON file with blank puzzle data...");
            write_puzzle_to_json(&args.json_output_path, &puzzle)?;
        }

        let clues_info = extract_clue_info(&puzzle);

        if args.show_clue_length {
            puzzle.clues.across.iter_mut().for_each(|(k, v)| {
                let clue_info = clues_info.across.get(&(*k as u8)).unwrap();
                *v = format!("{v} ({})", clue_info.length);
            });
            puzzle.clues.down.iter_mut().for_each(|(k, v)| {
                let clue_info = clues_info.down.get(&(*k as u8)).unwrap();
                *v = format!("{v} ({})", clue_info.length);
            });
        }

        Ok(PuzzleState {
            puzzle,
            clues_info,
            args: args.clone(),
        })
    }

    fn print_puzzle(&self) {
        for row in &self.puzzle.grid.blank {
            println!("{row}");
        }
    }

    fn remove_wrong_answers(&mut self) -> bool {
        let old_blank = self.puzzle.grid.blank.clone();

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
                        if self.puzzle.grid.solution[y]
                            .chars()
                            .nth(x)
                            .unwrap_or(BLANK_CELL)
                            == c
                        {
                            c
                        } else {
                            BLANK_CELL
                        }
                    })
                    .collect()
            })
            .collect();

        old_blank == self.puzzle.grid.blank
    }

    fn solve_clue(
        &mut self,
        number: u8,
        direction: Direction,
        passed_guess: Option<&str>,
    ) -> Result<(), Box<dyn std::error::Error>> {
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

        let mut guess: String = if let Some(pass) = passed_guess {
            pass.to_uppercase()
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

    fn remove_clue_answer(
        &mut self,
        number: u8,
        direction: Direction,
    ) -> Result<(), Box<dyn std::error::Error>> {
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
                        if (direction == Direction::Across
                            && y == clue_info.y
                            && x >= clue_info.x
                            && x < (clue_info.x + clue_info.length))
                            || (direction == Direction::Down
                                && x == clue_info.x
                                && y >= clue_info.y
                                && y < (clue_info.y + clue_info.length))
                        {
                            BLANK_CELL
                        } else {
                            c
                        }
                    })
                    .collect()
            })
            .collect();

        Ok(())
    }

    fn get_clue_so_far(&self, number: u8, direction: Direction) -> (String, String) {
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

    fn reveal_clue_answer(&mut self, number: u8) -> Result<(), Box<dyn std::error::Error>> {
        let across_clue = self.clues_info.across.get(&number);
        let down_clue = self.clues_info.down.get(&number);

        // If a clue number exists for both across and down, determine which one to reveal based on the current state of the puzzle
        // If across clue is already solved, reveal down clue, and vice versa. If both are unsolved, ask the user which one to reveal.
        // If both are already solved, return an error.
        // Otherwise, reveal the clue that exists.
        let (direction, clue_info) = match (across_clue, down_clue) {
            (Some(clue), None) => (Direction::Across, clue),
            (None, Some(clue)) => (Direction::Down, clue),
            (Some(a_clue), Some(d_clue)) => {
                let (a_word_so_far, a_solution_word) =
                    self.get_clue_so_far(number, Direction::Across);
                let (d_word_so_far, d_solution_word) =
                    self.get_clue_so_far(number, Direction::Down);

                match (
                    a_word_so_far == a_solution_word,
                    d_word_so_far == d_solution_word,
                ) {
                    (true, false) => (Direction::Down, d_clue),
                    (false, true) => (Direction::Across, a_clue),
                    (true, true) => {
                        return Err("Both across and down clues are already solved.".into())
                    }
                    (false, false) => {
                        println!(
                            "Clue number {} exists in both across and down clues. Please choose which one to reveal:",
                            number,
                        );
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
                        if (direction == Direction::Across
                            && y == clue_info.y
                            && x >= clue_info.x
                            && x < (clue_info.x + clue_info.length))
                            || (direction == Direction::Down
                                && x == clue_info.x
                                && y >= clue_info.y
                                && y < (clue_info.y + clue_info.length))
                        {
                            self.puzzle.grid.solution[y].chars().nth(x).unwrap()
                        } else {
                            c
                        }
                    })
                    .collect()
            })
            .collect();

        Ok(())
    }

    pub fn solve_puzzle(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut should_recompile = false;

        let compiler = PdfCompiler::new(self.args.pdf_style);
        compiler.compile_pdf(self);

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

            let choice: Result<String, _> = input();
            match choice.as_deref() {
                Ok("1") => {
                    println!("You chose to solve an across clue.");
                    solve_clue_input!(self, Direction::Across);
                    should_recompile = true;
                }
                Ok("2") => {
                    println!("You chose to solve a down clue.");
                    solve_clue_input!(self, Direction::Down);
                    should_recompile = true;
                }
                Ok("3") => {
                    println!("Overwriting JSON file with blank puzzle data...");
                    let blank_puzzle = initialize_puzzle(&self.args.puzzle_file_path)?;
                    should_recompile = true;
                    self.puzzle.grid.blank.clone_from(&blank_puzzle.grid.blank);
                }
                Ok("4") => {
                    println!("Current state of the puzzle:");
                    self.print_puzzle();
                }
                Ok("5") => {
                    println!("You chose to remove a clue's answer. Please enter the clue number:");
                    let clue_number: u8 = try_or_continue!(input(), "Invalid digit:");

                    println!("1. Remove an across clue");
                    println!("2. Remove a down clue");

                    let direction = try_or_continue!(
                        input::<Direction>(),
                        "Invalid input, please enter 1 or 2:"
                    );

                    try_or_continue!(self.remove_clue_answer(clue_number, direction), "Error:");

                    should_recompile = true;
                }
                Ok("6") => {
                    println!("Removing all wrong answers from the puzzle...");
                    if self.remove_wrong_answers() {
                        println!("No wrong answers to remove!");
                    } else {
                        println!("Wrong answers found and removed.");
                        should_recompile = true;
                    }
                }
                Ok("7") => {
                    println!("You chose to reveal a clue's answer. Please enter the clue number:");
                    let clue_number: u8 = try_or_continue!(input(), "Invalid digit:");

                    try_or_continue!(
                        self.reveal_clue_answer(clue_number),
                        "Error revealing clue:"
                    );

                    should_recompile = true;
                }
                Ok("8") => {
                    println!("Exiting...");
                    write_puzzle_to_json(&self.args.json_output_path, &self.puzzle)?;
                    break;
                }
                Ok(s) if s.starts_with('1') || s.starts_with('2') => {
                    let direction = try_or_continue!(s[0..1].parse(), "Invalid input:");

                    let mut split = s[1..].split_whitespace();

                    let clue_number: u8 =
                        try_or_continue!(split.next().unwrap().parse(), "Invalid clue number:");

                    try_or_continue!(
                        self.solve_clue(clue_number, direction, split.next()),
                        "Error solving clue:"
                    );

                    should_recompile = true;
                }
                Ok(s) if s.starts_with('7') => {
                    let mut split = s[1..].split_whitespace();

                    let clue_number: u8 =
                        try_or_continue!(split.next().unwrap().parse(), "Invalid clue number:");

                    try_or_continue!(
                        self.reveal_clue_answer(clue_number),
                        "Error revealing clue:"
                    );

                    should_recompile = true;
                }
                Ok(s) => println!("Invalid choice, please try again. {s}"),
                Err(e) => println!("Invalid input, please enter a number. Error: {e}"),
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
}

fn initialize_puzzle(file_path: &PathBuf) -> Result<Puzzle, Box<dyn std::error::Error>> {
    let puzzle = parse_file(file_path)?;
    Ok(puzzle)
}

fn extract_clue_info(puzzle: &Puzzle) -> CluesInfo {
    let mut across_clues = HashMap::new();
    let mut down_clues = HashMap::new();

    let mut clue_number = 1;

    for y in 0..puzzle.info.height as usize {
        for x in 0..puzzle.info.width as usize {
            let cell = puzzle.grid.blank[y].chars().nth(x).unwrap_or(BLANK_CELL);

            if cell == BLACK_CELL {
                continue; // Skip black squares
            }
            let mut is_clue_start = false;

            // Check for across clue
            if (x == 0
                || puzzle.grid.blank[y]
                    .chars()
                    .nth(x - 1)
                    .unwrap_or(BLANK_CELL)
                    == BLACK_CELL)
                && (x + 1 < puzzle.info.width as usize
                    && puzzle.grid.blank[y]
                        .chars()
                        .nth(x + 1)
                        .unwrap_or(BLANK_CELL)
                        != BLACK_CELL)
            {
                let clue_length = (0..puzzle.info.width as usize)
                    .take_while(|i| {
                        x + *i < (puzzle.info.width as usize)
                            && puzzle.grid.blank[y]
                                .chars()
                                .nth(x + *i)
                                .unwrap_or(BLANK_CELL)
                                != BLACK_CELL
                    })
                    .count();

                is_clue_start = true;

                across_clues.insert(
                    clue_number,
                    ClueInfo {
                        length: clue_length,
                        x,
                        y,
                    },
                );
            }
            // Check for down clue
            if (y == 0
                || puzzle.grid.blank[y - 1]
                    .chars()
                    .nth(x)
                    .unwrap_or(BLANK_CELL)
                    == BLACK_CELL)
                && (y + 1 < puzzle.info.height as usize
                    && puzzle.grid.blank[y + 1]
                        .chars()
                        .nth(x)
                        .unwrap_or(BLANK_CELL)
                        != BLACK_CELL)
            {
                let clue_length = (0..puzzle.info.height as usize)
                    .take_while(|i| {
                        y + *i < puzzle.info.height as usize
                            && puzzle.grid.blank[y + *i]
                                .chars()
                                .nth(x)
                                .unwrap_or(BLANK_CELL)
                                != BLACK_CELL
                    })
                    .count();

                is_clue_start = true;

                down_clues.insert(
                    clue_number,
                    ClueInfo {
                        length: clue_length,
                        x,
                        y,
                    },
                );
            }

            if is_clue_start {
                clue_number += 1;
            }
        }
    }

    CluesInfo {
        across: across_clues,
        down: down_clues,
    }
}

pub fn get_puz_json(puzzle: &Puzzle) -> Result<String, serde_json::Error> {
    serde_json::to_string(&puzzle)
}

pub fn read_puzzle_from_json(json_output_path: &PathBuf) -> Result<Puzzle, Box<dyn Error>> {
    let file = File::open(json_output_path)?;
    let reader = std::io::BufReader::new(file);
    let puzzle = serde_json::from_reader(reader)?;
    Ok(puzzle)
}

pub fn write_puzzle_to_json(
    json_output_path: &PathBuf,
    puzzle: &Puzzle,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(json_output_path)?;
    serde_json::to_writer(file, &puzzle)?;
    Ok(())
}
