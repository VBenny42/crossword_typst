use crate::types::{OutputFormat, PdfStyle, PuzzleState};
use clap::{parser::ValueSource, ArgAction::SetTrue, CommandFactory, FromArgMatches, Parser};
use directories::ProjectDirs;
use std::{fs, path::PathBuf};

mod pdfgen;
mod puzzle;
mod types;

fn default_output_path(format: OutputFormat) -> PathBuf {
    let filename = format!("output.{}", format.extension());
    ProjectDirs::from("com", "VBenny42", "crossword_typst").map_or_else(
        || PathBuf::from(&filename),
        |dirs| {
            let data_dir = dirs.data_dir().to_path_buf();
            fs::create_dir_all(&data_dir).ok();
            data_dir.join(&filename)
        },
    )
}

#[derive(Parser, Debug, Clone)]
pub struct Args {
    #[arg(
        long,
        default_value_os_t = default_output_path(OutputFormat::Json),
        help = "Path to where output should be saved"
    )]
    output_path: PathBuf,

    #[arg(short, long, help = "Path to .puz file to be read")]
    puzzle_file_path: PathBuf,

    #[arg(short, long, action = SetTrue, help = "Just write the output file and exit")]
    write_only: bool,

    #[arg(
        short,
        long,
        value_enum,
        default_value_t = OutputFormat::Puz,
        help = "Format to write puzzle state to: json or puz"
    )]
    output_format: OutputFormat,

    #[arg(short, long, action = SetTrue, help = "Update the same .puz file that's being read. Writing to puz does not recalculate checksums")]
    update_puz_file: bool,

    #[arg(short, long, action = SetTrue, help = "Compile the PDF with nord colors")]
    nord_colors: bool,

    #[arg(long, action = SetTrue, help = "Hide completed clues in the PDF")]
    hide_completed_clues: bool,

    #[arg(long, action = SetTrue, help = "Show word length for a clue in the PDF")]
    show_clue_length: bool,

    #[arg(long, action = SetTrue, help = "Only show correct letters from guessed clues")]
    show_correct_letters_only: bool,

    #[arg(
        long,
        default_value = "Normal",
        help = "Style of PDF to be generated. Can be Normal, Larger or Landscape"
    )]
    pdf_style: PdfStyle,
}

impl Args {
    /// Parses CLI args, re-deriving `output_path`'s default to match
    /// `output_format` if the user didn't explicitly pass --output-path.
    pub fn parse_with_format_aware_default() -> Self {
        let command = Self::command().mut_arg("output_path", |a| {
a.hide_default_value(true).help(format!(
            "Path to where output should be saved [default: {} (extension follows --output-format)]",
            default_output_path(OutputFormat::Puz).display()
        ))
        });

        let matches = command.get_matches();
        let mut args = Self::from_arg_matches(&matches).unwrap_or_else(|e| e.exit());

        if matches.value_source("output_path") == Some(ValueSource::DefaultValue) {
            args.output_path = default_output_path(args.output_format);
        }

        if args.update_puz_file {
            if args.output_format == OutputFormat::Puz {
                args.output_path = args.puzzle_file_path.clone();
            } else {
                eprintln!("Output format is not puz, doing nothing");
            }
        }

        args
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse_with_format_aware_default();

    let mut state = PuzzleState::new(&args)?;

    if args.write_only {
        return Ok(());
    }

    state.solve_puzzle()?;

    Ok(())
}
