use std::io::Write;
use std::{fs, os::unix::net::UnixStream};

use puz_parse::Puzzle;
use typst::{
    comemo,
    foundations::{Dict, IntoValue},
};
use typst_as_lib::TypstEngine;
use typst_pdf::PdfOptions;

use crate::puzzle::get_puz_json;
use crate::types::{CluesInfo, PdfStyle, PuzzleState, BLANK_CELL};

static HELPERS_FILE: &str = include_str!("../templates/helpers.typ");
static TEMPLATE_NORMAL_FILE: &str = include_str!("../templates/template.typ");
static TEMPLATE_LARGER_FILE: &str = include_str!("../templates/template-larger.typ");
static TEMPLATE_LANDSCAPE_FILE: &str = include_str!("../templates/template-landscape.typ");
static PDF_OUTPUT_PATH: &str = "./crossword.pdf";

static FONT_REGULAR: &[u8] = include_bytes!("../fonts/Helvetica/Helvetica.ttf");
static FONT_BOLD: &[u8] = include_bytes!("../fonts/Helvetica/Helvetica-Bold.ttf");
static FONT_OBLIQUE: &[u8] = include_bytes!("../fonts/Helvetica/Helvetica-Oblique.ttf");

static SOCKET_PATH: &str = "/tmp/fancy-cat.sock";

pub struct PdfCompiler {
    engine: TypstEngine<typst_as_lib::TypstTemplateMainFile>,
}

impl PdfCompiler {
    pub fn new(pdf_style: PdfStyle) -> Self {
        let build = || TypstEngine::builder().fonts([FONT_REGULAR, FONT_BOLD, FONT_OBLIQUE]);
        let main_file = match pdf_style {
            PdfStyle::Normal => TEMPLATE_NORMAL_FILE,
            PdfStyle::Larger => TEMPLATE_LARGER_FILE,
            PdfStyle::Landscape => TEMPLATE_LANDSCAPE_FILE,
        };
        Self {
            engine: build()
                .main_file(main_file)
                .with_static_source_file_resolver([("helpers.typ", HELPERS_FILE)])
                .build(),
        }
    }

    pub fn compile_pdf(&self, state: &PuzzleState) -> Result<(), Box<dyn std::error::Error>> {
        let inputs: Dict = [
            (
                "crossword_json".into(),
                get_puz_json(&state.puzzle).unwrap().into_value(),
            ),
            ("nord_colors".into(), state.args.nord_colors.into_value()),
            (
                "hide_completed_clues".into(),
                state.args.hide_completed_clues.into_value(),
            ),
            (
                "clues_info".into(),
                state.clues_info.get_clues_json().unwrap().into_value(),
            ),
        ]
        .into_iter()
        .collect();

        // let mut edited_puzzle = state.puzzle.clone();
        // edit_clue_text(&mut edited_puzzle, &state.clues_info);
        // let json = get_puz_json(&edited_puzzle).unwrap();

        let world = self
            .engine
            .world_builder()
            .with_inputs(inputs)
            .build()
            .unwrap();

        let doc = typst::compile(&world)
            .output
            .expect("typst::compile() returned an error!");

        comemo::evict(30);

        let pdf = typst_pdf::pdf(&doc, &PdfOptions::default()).expect("Could not generate pdf");

        fs::write(PDF_OUTPUT_PATH, pdf).expect("Could not write pdf");

        if state.args.connect_to_socket {
            let mut stream = match UnixStream::connect(SOCKET_PATH) {
                Ok(stream) => stream,
                Err(e) => {
                    eprintln!("{e}, continuing without writing to socket.");
                    return Ok(());
                }
            };
            // Doesn't acc matter what I write, just something
            stream.write_all(b"w")?;
        }

        Ok(())
    }
}

// pub fn compile_pdf_world(puzzle: &PuzzleState) {
//     let template = TypstEngine::builder()
//         .main_file(TEMPLATE_FILE)
//         .search_fonts_with(TypstKitFontOptions::default())
//         .build();
//
//     let json = puzzle.get_puz_json().unwrap();
//
//     let mut sys_dict: Dict = Dict::new();
//     sys_dict.insert("crossword_json".into(), json.into_value());
//
//     let world = template.world_builder().with_inputs(sys_dict);
//
//     let built = world.build().unwrap();
//
//     let doc = typst::compile(&built).output.expect("compile failed");
//     let options = Default::default();
//
//     let pdf = typst_pdf::pdf(&doc, &options).expect("Could not generate pdf");
//     fs::write(OUTPUT_PATH, pdf).expect("Could not write pdf")
// }

#[allow(dead_code)]
// Append `*` to partially/fully solved clues
fn edit_clue_text(puzzle: &mut Puzzle, clues_info: &CluesInfo) {
    puzzle.clues.across.iter_mut().for_each(|(k, v)| {
        let clue_info = clues_info.across.get(&(u8::try_from(*k).unwrap())).unwrap();

        if puzzle.grid.blank[clue_info.y][clue_info.x..(clue_info.x + clue_info.length)]
            .chars()
            .any(|c| c != BLANK_CELL)
        {
            *v = format!("{v} *");
        }
    });

    puzzle.clues.down.iter_mut().for_each(|(k, v)| {
        let clue_info = clues_info.down.get(&(u8::try_from(*k).unwrap())).unwrap();

        if puzzle
            .grid
            .blank
            .iter()
            .skip(clue_info.y)
            .take(clue_info.length)
            .map(|row| row.chars().nth(clue_info.x).unwrap_or(BLANK_CELL))
            .any(|c| c != BLANK_CELL)
        {
            *v = format!("{v} *");
        }
    });
}
