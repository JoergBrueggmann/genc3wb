//! The integration tests of the component `core::input_file`, through the public interface.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use genc3wb::core::input_file::{InputFile, InputKind, ProcessingState};
use genc3wb::core::runner;

use std::fs;

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : InputGroup::set_text, InputGroup::idle_expired, RunnerGroup::set_input_state */

#[test]
fn editing_and_saving_both_input_files_makes_the_runner_ready() {
    // FR-013, FR-016, FR-018, FR-019, FR-026
    let dir = std::env::temp_dir().join(format!(
        "genc3wb-input_file-readiness-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("the directory can be created");
    let cc_path = dir.join("syntax.cc").to_string_lossy().into_owned();
    let c_path = dir.join("program.c").to_string_lossy().into_owned();
    fs::write(&cc_path, "syntax").expect("the file can be written");
    fs::write(&c_path, "program").expect("the file can be written");

    let mut cc_input = InputFile::new(InputKind::CompilerCompilerInput);
    let mut c_input = InputFile::new(InputKind::CompilerInput);
    cc_input.load(&cc_path).expect("the file can be loaded");
    c_input.load(&c_path).expect("the file can be loaded");
    let states = |cc: &InputFile, c: &InputFile| [cc.state(), c.state()];
    assert!(runner::is_ready(true, states(&cc_input, &c_input)));

    c_input.set_text("program edited");
    assert_eq!(c_input.state(), ProcessingState::ValidFileTextChanged);
    assert!(!runner::is_ready(true, states(&cc_input, &c_input)));

    c_input.save().expect("the file can be saved");
    assert_eq!(
        fs::read_to_string(&c_path).expect("the file was written"),
        "program edited"
    );
    assert!(runner::is_ready(true, states(&cc_input, &c_input)));
    let _ = fs::remove_dir_all(&dir);
}
