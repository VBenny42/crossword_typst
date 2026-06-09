use std::fs;

use typst::foundations::{Dict, IntoValue};
use typst_as_lib::{typst_kit_options::TypstKitFontOptions, TypstEngine};

use crate::types::PuzzleState;

static TEMPLATE_FILE: &str = include_str!("../templates/template.typ");
static PDF_OUTPUT_PATH: &str = "./target/crossword.pdf";

pub fn compile_pdf(puzzle: &PuzzleState) {
    let template = TypstEngine::builder()
        .main_file(TEMPLATE_FILE)
        .search_fonts_with(TypstKitFontOptions::default())
        .build();

    let json = puzzle.get_puz_json().unwrap();

    let inputs: Dict = [
        ("crossword_json".into(), json.into_value()),
        ("nord_colors".into(), puzzle.nord_colors.into_value()),
        (
            "hide_completed_clues".into(),
            puzzle.hide_completed_clues.into_value(),
        ),
    ]
    .into_iter()
    .collect();

    let doc = template
        .compile_with_input(inputs)
        .output
        .expect("typst::compile() returned an error!");

    let options = Default::default();

    let pdf = typst_pdf::pdf(&doc, &options).expect("Could not generate pdf");
    fs::write(PDF_OUTPUT_PATH, pdf).expect("Could not write pdf")
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
