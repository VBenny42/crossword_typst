use clap::{ArgAction::SetTrue, Parser};
use directories::ProjectDirs;
use std::{
    fs::{self},
    path::PathBuf,
    str::FromStr,
};

use crate::{
    pdfgen::compile_pdf,
    types::{PdfStyle, PuzzleState},
};

mod pdfgen;
mod puzzle;
mod types;

fn default_json_path() -> PathBuf {
    ProjectDirs::from("com", "VBenny42", "crossword_typst")
        .map(|dirs| {
            let data_dir = dirs.data_dir().to_path_buf();
            fs::create_dir_all(&data_dir).ok();
            data_dir.join("output.json")
        })
        .unwrap_or_else(|| PathBuf::from("output.json"))
}

fn input<T: FromStr>() -> Result<T, <T as FromStr>::Err> {
    let mut input: String = String::with_capacity(64);

    std::io::stdin()
        .read_line(&mut input)
        .expect("Input could not be read");

    input.trim().parse()
}

#[derive(Parser, Debug)]
pub struct Args {
    #[arg(short, long, default_value_os_t = default_json_path(), help="Path to where json should be saved")]
    json_output_path: PathBuf,

    #[arg(short, long, help = "Path to .puz file to be read")]
    puzzle_file_path: std::path::PathBuf,

    #[arg(short, long, action = SetTrue, help = "Just write to json file and exit")]
    write_to_json_only: bool,

    #[arg(short, long, action = SetTrue, help = "Compile the PDF with nord colors")]
    nord_colors: bool,

    #[arg(long, action = SetTrue, help = "Hide completed clues in the PDF")]
    hide_completed_clues: bool,

    #[arg(short, long, action = SetTrue, help = "Show word length for a clue in the PDF")]
    show_clue_length: bool,

    #[arg(
        long,
        default_value = "Normal",
        help = "Style of PDF to be generated. Can be Normal, Larger or Landscape"
    )]
    pdf_style: PdfStyle,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut state = PuzzleState::new(&args)?;

    if args.write_to_json_only {
        return Ok(());
    }
    compile_pdf(&state);
    state.solve_puzzle()?;

    Ok(())
}
