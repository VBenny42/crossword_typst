use std::fs;

use puz_parse::Puzzle;
use typst::foundations::{Dict, IntoValue};
use typst_as_lib::{typst_kit_options::TypstKitFontOptions, TypstEngine};
use typst_pdf::PdfOptions;

use crate::{
    puzzle::{get_puz_json, BLANK_CELL},
    types::{CluesInfo, PdfStyle, PuzzleState},
};

static TEMPLATE_NORMAL_FILE: &str = include_str!("../templates/template.typ");
static TEMPLATE_LARGER_FILE: &str = include_str!("../templates/template-larger.typ");
static TEMPLATE_LANDSCAPE_FILE: &str = include_str!("../templates/template-landscape.typ");
static PDF_OUTPUT_PATH: &str = "./target/crossword.pdf";

pub fn compile_pdf(state: &PuzzleState) {
    let template_file = match state.pdf_style {
        PdfStyle::Normal => TEMPLATE_NORMAL_FILE,
        PdfStyle::Larger => TEMPLATE_LARGER_FILE,
        PdfStyle::Landscape => TEMPLATE_LANDSCAPE_FILE,
    };

    let template = TypstEngine::builder()
        .main_file(template_file)
        .search_fonts_with(TypstKitFontOptions::default())
        .build();

    // let mut edited_puzzle = state.puzzle.clone();
    // edit_clue_text(&mut edited_puzzle, &state.clues_info);
    // let json = get_puz_json(&edited_puzzle).unwrap();

    let json = get_puz_json(&state.puzzle).unwrap();

    let inputs: Dict = [
        ("crossword_json".into(), json.into_value()),
        ("nord_colors".into(), state.nord_colors.into_value()),
        (
            "hide_completed_clues".into(),
            state.hide_completed_clues.into_value(),
        ),
    ]
    .into_iter()
    .collect();

    let doc = template
        .compile_with_input(inputs)
        .output
        .expect("typst::compile() returned an error!");

    let pdf = typst_pdf::pdf(&doc, &PdfOptions::default()).expect("Could not generate pdf");
    fs::write(PDF_OUTPUT_PATH, pdf).expect("Could not write pdf");
}

#[allow(dead_code)]
fn edit_clue_text(puzzle: &mut Puzzle, clues_info: &CluesInfo) {
    puzzle.clues.across.iter_mut().for_each(|(k, v)| {
        let clue_info = clues_info.across.get(&(*k as u8)).unwrap();

        if puzzle.grid.blank[clue_info.y][clue_info.x..(clue_info.x + clue_info.length)]
            .chars()
            .any(|c| c != BLANK_CELL)
        {
            *v = format!("{v} *")
        }
    });

    puzzle.clues.down.iter_mut().for_each(|(k, v)| {
        let clue_info = clues_info.down.get(&(*k as u8)).unwrap();

        if puzzle
            .grid
            .blank
            .iter()
            .skip(clue_info.y)
            .take(clue_info.length)
            .map(|row| row.chars().nth(clue_info.x).unwrap_or(BLANK_CELL))
            .any(|c| c != BLANK_CELL)
        {
            *v = format!("{v} *")
        }
    });
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
