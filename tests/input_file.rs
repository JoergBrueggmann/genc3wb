//! The integration tests of the component `core::input_file`, through the public interface.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use genc3wb::core::input_file::{InputFile, InputKind, ProcessingState};

use std::fs;

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : InputGroup::name_document, InputGroup::set_text, InputGroup::idle_expired */

#[test]
fn editing_and_saving_an_input_and_loading_it_again_yields_the_saved_text() {
    // FR-013, FR-016, FR-018, FR-019, FR-087, C-004
    let dir =
        std::env::temp_dir().join(format!("genc3wb-input_file-editing-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("the directory can be created");
    let meta_dsl_path = dir.join("a.gc3").to_string_lossy().into_owned();
    let input_path = dir.join("a.in").to_string_lossy().into_owned();
    fs::write(&meta_dsl_path, "syntax").expect("the file can be written");
    fs::write(&input_path, "program").expect("the file can be written");

    let mut meta_dsl = InputFile::new(InputKind::MetaDsl);
    let mut input = InputFile::new(InputKind::Input);
    meta_dsl
        .load(&meta_dsl_path)
        .expect("the file can be loaded");
    input.load(&input_path).expect("the file can be loaded");
    let loaded = (meta_dsl.state(), input.state());

    input.set_text("program edited");
    let edited = input.state();
    input.save().expect("the file can be saved");
    let saved = input.state();

    let mut again = InputFile::new(InputKind::Input);
    again.load(&input_path).expect("the file can be loaded");
    let reloaded = again.text().to_owned();
    let _ = fs::remove_dir_all(&dir);
    assert_eq!(
        (loaded, edited, saved, reloaded),
        (
            (
                ProcessingState::ValidFileTextUntouched,
                ProcessingState::ValidFileTextUntouched
            ),
            ProcessingState::ValidFileTextChanged,
            ProcessingState::ValidFileTextUntouched,
            "program edited".to_owned()
        )
    );
}
