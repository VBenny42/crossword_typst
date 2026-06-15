use puz_parse::{parse_file, Puzzle};
use std::{collections::HashMap, error::Error, fs::File, path::PathBuf};

use crate::{input, pdfgen::compile_pdf, types};
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
                println!("{}: {e}", $msg);
                continue;
            }
        }
    };
}

impl PuzzleState {
    pub fn new(args: &crate::Args) -> Result<Self, Box<dyn Error>> {
        let mut puzzle = initialize_puzzle(&args.puzzle_file_path)?;
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

        let json_output_path = args.json_output_path.as_ref().unwrap().clone();

        Ok(PuzzleState {
            puzzle,
            clues_info,
            puzzle_path: args.puzzle_file_path.clone(),
            json_output_path,
            nord_colors: args.nord_colors,
            hide_completed_clues: args.hide_completed_clues,
            show_clue_length: args.show_clue_length,
            pdf_style: args.pdf_style.unwrap(),
        })
    }

    pub fn read_puzzle_from_json(&self) -> Result<Puzzle, Box<dyn Error>> {
        let file = File::open(&self.json_output_path)?;
        let reader = std::io::BufReader::new(file);
        let puzzle = serde_json::from_reader(reader)?;
        Ok(puzzle)
    }

    pub fn write_puzzle_to_json(&self) -> Result<(), Box<dyn std::error::Error>> {
        let file = File::create(&self.json_output_path)?;
        serde_json::to_writer(file, &self.puzzle)?;
        Ok(())
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

        if self.show_clue_length {
            println!("{}. {}, {}. `{word_so_far}`", number, clue_text, direction);
        } else {
            println!(
                "{}. {} ({}), {}. `{word_so_far}`",
                number, clue_text, clue_info.length, direction
            );
        }

        let guess: String = if let Some(pass) = passed_guess {
            pass.to_uppercase()
        } else {
            println!("Input your guess:");
            input::<String>()?.to_uppercase()
        };

        if guess.len() != clue_info.length {
            println!(
                "Your guess must be {} characters long. Please try again.",
                clue_info.length
            );
            return Ok(());
        }

        if guess
            .chars()
            .zip(word_so_far.chars())
            .any(|(a, b)| a != b && b != BLANK_CELL)
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

    pub fn solve_puzzle(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut should_compile = false;

        loop {
            if self.puzzle.grid.blank == self.puzzle.grid.solution {
                println!("Congratulations! You've solved the puzzle!");
                self.write_puzzle_to_json()?;
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
7. Exit"
            );

            let choice: Result<String, _> = input();
            match choice.as_deref() {
                Ok("1") => {
                    println!("You chose to solve an across clue.");
                    solve_clue_input!(self, Direction::Across);
                    should_compile = true;
                }
                Ok("2") => {
                    println!("You chose to solve a down clue.");
                    solve_clue_input!(self, Direction::Down);
                    should_compile = true;
                }
                Ok("3") => {
                    println!("Overwriting JSON file with blank puzzle data...");
                    let blank_puzzle = initialize_puzzle(&self.puzzle_path)?;
                    should_compile = true;
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

                    self.remove_clue_answer(clue_number, direction)?;

                    should_compile = true;
                }
                Ok("6") => {
                    println!("Removing all wrong answers from the puzzle...");
                    if self.remove_wrong_answers() {
                        println!("No wrong answers to remove!");
                    } else {
                        println!("Wrong answers found and removed.");
                        should_compile = true;
                    }
                }
                Ok("7") => {
                    println!("Exiting...");
                    self.write_puzzle_to_json()?;
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

                    should_compile = true;
                }
                Ok(s) => println!("Invalid choice, please try again. {s}"),
                Err(e) => println!("Invalid input, please enter a number. Error: {e}"),
            }

            if should_compile {
                compile_pdf(self);
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

    for y in 0..puzzle.info.height {
        for x in 0..puzzle.info.width {
            let cell = puzzle.grid.blank[y as usize]
                .chars()
                .nth(x as usize)
                .unwrap_or(BLANK_CELL);

            if cell == BLACK_CELL {
                continue; // Skip black squares
            }
            let mut is_clue_start = false;

            // Check for across clue
            if (x == 0
                || puzzle.grid.blank[y as usize]
                    .chars()
                    .nth((x - 1) as usize)
                    .unwrap_or(BLANK_CELL)
                    == BLACK_CELL)
                && (x + 1 < puzzle.info.width
                    && puzzle.grid.blank[y as usize]
                        .chars()
                        .nth((x + 1) as usize)
                        .unwrap_or(BLANK_CELL)
                        != BLACK_CELL)
            {
                let clue_length = (0..puzzle.info.width)
                    .take_while(|i| {
                        x + *i < puzzle.info.width
                            && puzzle.grid.blank[y as usize]
                                .chars()
                                .nth((x + *i) as usize)
                                .unwrap_or(BLANK_CELL)
                                != BLACK_CELL
                    })
                    .count();

                is_clue_start = true;

                across_clues.insert(
                    clue_number,
                    ClueInfo {
                        length: clue_length,
                        x: x as usize,
                        y: y as usize,
                    },
                );
            }
            // Check for down clue
            if (y == 0
                || puzzle.grid.blank[(y - 1) as usize]
                    .chars()
                    .nth(x as usize)
                    .unwrap_or(BLANK_CELL)
                    == BLACK_CELL)
                && (y + 1 < puzzle.info.height
                    && puzzle.grid.blank[(y + 1) as usize]
                        .chars()
                        .nth(x as usize)
                        .unwrap_or(BLANK_CELL)
                        != BLACK_CELL)
            {
                let clue_length = (0..puzzle.info.height)
                    .take_while(|i| {
                        y + *i < puzzle.info.height
                            && puzzle.grid.blank[(y + *i) as usize]
                                .chars()
                                .nth(x as usize)
                                .unwrap_or(BLANK_CELL)
                                != BLACK_CELL
                    })
                    .count();

                is_clue_start = true;

                down_clues.insert(
                    clue_number,
                    ClueInfo {
                        length: clue_length,
                        x: x as usize,
                        y: y as usize,
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
