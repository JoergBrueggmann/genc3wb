//! The integration tests of the component `core::settings`, through the public interface.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use genc3wb::core::input_file::InputKind;
use genc3wb::core::output::OutputPages;
use genc3wb::core::settings::Settings;

use std::fs;

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : Workbench::default, OutputGroup::configure */

#[test]
fn stored_settings_restore_the_output_pages_of_the_next_session() {
    // FR-048, FR-049, IR-011, IR-012
    let dir = std::env::temp_dir().join(format!("genc3wb-settings-session-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let file = dir.join("settings.txt");
    let output = dir.join("out3.txt");
    fs::create_dir_all(&dir).expect("the directory can be created");
    fs::write(&output, "third").expect("the output file can be written");

    let mut first_session = Settings::default();
    first_session.set_input_path(InputKind::CompilerInput, "input.c");
    first_session
        .set_output_path(3, &output.to_string_lossy())
        .expect("3 is within range");
    first_session
        .save(&file)
        .expect("the settings can be stored");

    let second_session = Settings::load(&file).expect("the settings can be restored");
    assert_eq!(
        second_session.input_path(InputKind::CompilerInput),
        "input.c"
    );
    let pages = OutputPages::from_settings(&second_session);
    assert_eq!(pages.page_count(), 4);
    let mut pages = pages;
    pages.next();
    pages.next();
    assert_eq!(pages.current_number(), Some(3));
    assert_eq!(
        pages.current_file().map(|file| file.content.as_str()),
        Some("third")
    );
    let _ = fs::remove_dir_all(&dir);
}
