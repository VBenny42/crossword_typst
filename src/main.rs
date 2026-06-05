use clap::Parser;
use puz_parse::{parse_file, Puzzle};
use std::{collections::HashMap, fs::File, str::FromStr};

static JSON_OUTPUT_PATH: &str = "src/output.json";

fn initialize_puzzle(file_path: &str) -> Result<Puzzle, Box<dyn std::error::Error>> {
    let puzzle = parse_file(file_path)?;
    Ok(puzzle)
}

fn write_puzzle_to_json(
    puzzle: &Puzzle,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(output_path)?;
    serde_json::to_writer(file, puzzle)?;
    Ok(())
}

fn read_puzzle_from_json(input_path: &str) -> Result<Puzzle, Box<dyn std::error::Error>> {
    let file = File::open(input_path)?;
    let reader = std::io::BufReader::new(file);
    let puzzle = serde_json::from_reader(reader)?;
    Ok(puzzle)
}

#[allow(dead_code)]
#[derive(Debug)]
struct ClueInfo {
    length: u8,
    x: u8,
    y: u8,
}

#[allow(dead_code)]
#[derive(Debug)]
struct CluesInfo {
    across: HashMap<u8, ClueInfo>,
    down: HashMap<u8, ClueInfo>,
}

#[allow(dead_code)]
#[derive(Debug)]
struct PuzzleState {
    puzzle: Puzzle,
    clues_info: CluesInfo,
    puzzle_path: String,
    json_output_path: String,
}

fn extract_clue_info(puzzle: &Puzzle) -> Result<CluesInfo, Box<dyn std::error::Error>> {
    let mut across_clues = HashMap::new();
    let mut down_clues = HashMap::new();

    let mut clue_number = 1;

    for y in 0..puzzle.info.height {
        for x in 0..puzzle.info.width {
            let cell = puzzle.grid.blank[y as usize]
                .chars()
                .nth(x as usize)
                .unwrap_or(' ');

            if cell == '.' {
                continue; // Skip black squares
            }
            let mut is_clue_start = false;

            // Check for across clue
            if (x == 0
                || puzzle.grid.blank[y as usize]
                    .chars()
                    .nth((x - 1) as usize)
                    .unwrap_or(' ')
                    == '.')
                && (x + 1 < puzzle.info.width
                    && puzzle.grid.blank[y as usize]
                        .chars()
                        .nth((x + 1) as usize)
                        .unwrap_or(' ')
                        != '.')
            {
                let clue_length = (0..)
                    .take_while(|i| {
                        x + *i < puzzle.info.width
                            && puzzle.grid.blank[y as usize]
                                .chars()
                                .nth((x + *i) as usize)
                                .unwrap_or(' ')
                                != '.'
                    })
                    .count() as u8;

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
                || puzzle.grid.blank[(y - 1) as usize]
                    .chars()
                    .nth(x as usize)
                    .unwrap_or(' ')
                    == '.')
                && (y + 1 < puzzle.info.height
                    && puzzle.grid.blank[(y + 1) as usize]
                        .chars()
                        .nth(x as usize)
                        .unwrap_or(' ')
                        != '.')
            {
                let clue_length = (0..)
                    .take_while(|i| {
                        y + *i < puzzle.info.height
                            && puzzle.grid.blank[(y + *i) as usize]
                                .chars()
                                .nth(x as usize)
                                .unwrap_or(' ')
                                != '.'
                    })
                    .count() as u8;

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

    Ok(CluesInfo {
        across: across_clues,
        down: down_clues,
    })
}

fn input<T: FromStr>() -> Result<T, <T as FromStr>::Err> {
    let mut input: String = String::with_capacity(64);

    std::io::stdin()
        .read_line(&mut input)
        .expect("Input could not be read");

    input.trim().parse()
}

fn solve_clue(
    number: u8,
    direction: &str,
    state: &mut PuzzleState,
) -> Result<(), Box<dyn std::error::Error>> {
    let clue_info = match direction {
        "across" => state
            .clues_info
            .across
            .get(&number)
            .ok_or("Clue number not found in across clues")?,
        "down" => state
            .clues_info
            .down
            .get(&number)
            .ok_or("Clue number not found in down clues")?,
        _ => return Err("Invalid direction".into()),
    };

    let clue_text = match direction {
        "across" => state
            .puzzle
            .clues
            .across
            .get(&(number as u16))
            .map(|s| s.as_str())
            .unwrap_or("Unknown clue"),
        "down" => state
            .puzzle
            .clues
            .down
            .get(&(number as u16))
            .map(|s| s.as_str())
            .unwrap_or("Unknown clue"),
        _ => "Unknown clue",
    };

    println!(
        // "Clue {} {} has length {}, with text {}. Input your guess:",
        "{}. {} ({}), {}. Input your guess:",
        number, clue_text, clue_info.length, direction
    );

    let guess = input::<String>()?.to_uppercase();

    if guess.len() as u8 != clue_info.length {
        println!(
            "Your guess must be {} characters long. Please try again.",
            clue_info.length
        );
        return Ok(());
    }

    state.puzzle.grid.blank = state
        .puzzle
        .grid
        .blank
        .iter()
        .enumerate()
        .map(|(y, row)| {
            row.chars()
                .enumerate()
                .map(|(x, c)| {
                    if direction == "across"
                        && y == clue_info.y as usize
                        && x >= clue_info.x as usize
                        && x < (clue_info.x + clue_info.length) as usize
                    {
                        guess.chars().nth(x - clue_info.x as usize).unwrap_or(c)
                    } else if direction == "down"
                        && x == clue_info.x as usize
                        && y >= clue_info.y as usize
                        && y < (clue_info.y + clue_info.length) as usize
                    {
                        guess.chars().nth(y - clue_info.y as usize).unwrap_or(c)
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
    number: u8,
    direction: u8,
    state: &mut PuzzleState,
) -> Result<(), Box<dyn std::error::Error>> {
    let clue_info = match direction {
        1 => state
            .clues_info
            .across
            .get(&number)
            .ok_or("Clue number not found in across clues")?,
        2 => state
            .clues_info
            .down
            .get(&number)
            .ok_or("Clue number not found in down clues")?,
        _ => return Err("Invalid direction".into()),
    };

    state.puzzle.grid.blank = state
        .puzzle
        .grid
        .blank
        .iter()
        .enumerate()
        .map(|(y, row)| {
            row.chars()
                .enumerate()
                .map(|(x, c)| {
                    if (direction == 1
                        && y == clue_info.y as usize
                        && x >= clue_info.x as usize
                        && x < (clue_info.x + clue_info.length) as usize)
                        || (direction == 2
                            && x == clue_info.x as usize
                            && y >= clue_info.y as usize
                            && y < (clue_info.y + clue_info.length) as usize)
                    {
                        ' '
                    } else {
                        c
                    }
                })
                .collect()
        })
        .collect();

    Ok(())
}

fn remove_wrong_answers(state: &mut PuzzleState) -> bool {
    let old_blank = state.puzzle.grid.blank.clone();

    state.puzzle.grid.blank = state
        .puzzle
        .grid
        .blank
        .iter()
        .enumerate()
        .map(|(y, row)| {
            row.chars()
                .enumerate()
                .map(|(x, c)| {
                    if state.puzzle.grid.solution[y].chars().nth(x).unwrap_or(' ') != c {
                        ' '
                    } else {
                        c
                    }
                })
                .collect()
        })
        .collect();

    old_blank == state.puzzle.grid.blank
}

fn print_puzzle(state: &PuzzleState) {
    for row in &state.puzzle.grid.blank {
        println!("{}", row);
    }
}

fn solve_puzzle(state: &mut PuzzleState) -> Result<(), Box<dyn std::error::Error>> {
    loop {
        if state.puzzle.grid.blank == state.puzzle.grid.solution {
            println!("Congratulations! You've solved the puzzle!");
            break;
        }

        println!("Your choices are:");
        println!("1. Solve an across clue");
        println!("2. Solve a down clue");
        println!("3. Overwrite JSON file with blank puzzle data");
        println!("4. Print the current state of the puzzle");
        println!("5. Remove a clue's answer from the puzzle");
        println!("6. Remove all wrong answers from the puzzle");
        println!("7. Exit");

        let choice: Result<u8, _> = input();
        match choice {
            Ok(1) => {
                println!("You chose to solve an across clue. Please enter the clue number:");
                let clue_number: u8 = match input() {
                    Ok(num) => num,
                    Err(e) => {
                        println!("Invalid input, please enter a number. Error: {}", e);
                        continue;
                    }
                };
                match solve_clue(clue_number, "across", state) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error solving clue: {}", e);
                        continue;
                    }
                }

                write_puzzle_to_json(&state.puzzle, &state.json_output_path)?;
            }
            Ok(2) => {
                println!("You chose to solve a down clue. Please enter the clue number:");
                let clue_number: u8 = match input() {
                    Ok(num) => num,
                    Err(e) => {
                        println!("Invalid input, please enter a number. Error: {}", e);
                        continue;
                    }
                };
                match solve_clue(clue_number, "down", state) {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Error solving clue: {}", e);
                        continue;
                    }
                }

                write_puzzle_to_json(&state.puzzle, &state.json_output_path)?;
            }
            Ok(3) => {
                println!("Overwriting JSON file with blank puzzle data...");
                let blank_puzzle = initialize_puzzle(&state.puzzle_path)?;
                write_puzzle_to_json(&blank_puzzle, &state.json_output_path)?;
                state.puzzle.grid.blank = blank_puzzle.grid.blank.clone();
            }
            Ok(4) => {
                println!("Current state of the puzzle:");
                print_puzzle(state);
            }
            Ok(5) => {
                println!("You chose to remove a clue's answer. Please enter the clue number:");
                let clue_number: u8 = input()?;

                println!("1. Remove an across clue");
                println!("2. Remove a down clue");

                let direction: u8 = input()?;
                remove_clue_answer(clue_number, direction, state)?;

                write_puzzle_to_json(&state.puzzle, &state.json_output_path)?;
            }
            Ok(6) => {
                println!("Removing all wrong answers from the puzzle...");
                match remove_wrong_answers(state) {
                    true => println!("No wrong answers to remove!"),
                    false => {
                        println!("Wrong answers found and removed.");
                        write_puzzle_to_json(&state.puzzle, &state.json_output_path)?;
                    }
                }
            }
            Ok(7) => {
                println!("Exiting...");
                break;
            }
            Ok(_) => println!("Invalid choice, please try again."),
            Err(e) => println!("Invalid input, please enter a number. Error: {}", e),
        }
    }

    Ok(())
}

#[derive(Parser, Debug)]
struct Args {
    #[arg(short, long, default_value = JSON_OUTPUT_PATH)]
    json_output_path: Option<String>,

    #[arg(short, long)]
    puzzle_file_path: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut state = PuzzleState {
        puzzle: initialize_puzzle(&args.puzzle_file_path)?,
        clues_info: extract_clue_info(&initialize_puzzle(&args.puzzle_file_path)?)?,
        puzzle_path: args.puzzle_file_path.clone(),
        json_output_path: args.json_output_path.clone().unwrap(),
    };

    match File::open(&state.json_output_path) {
        Ok(_) => {}
        Err(_) => {
            println!("JSON file does not exist. Creating a new one...");
            write_puzzle_to_json(&state.puzzle, &state.json_output_path)?;
        }
    }

    let read_puzzle = read_puzzle_from_json(&state.json_output_path)?;

    if read_puzzle.info.title != state.puzzle.info.title {
        eprintln!("Warning: The puzzle title in the JSON file does not match the original puzzle. Overwriting JSON file with blank puzzle data...");
        write_puzzle_to_json(&state.puzzle, &state.json_output_path)?;
    }

    state.puzzle.grid.blank = read_puzzle.grid.blank.clone();

    solve_puzzle(&mut state)?;

    Ok(())
}
