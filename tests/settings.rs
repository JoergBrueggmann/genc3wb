//! The integration tests of the component `core::settings`, through the public interface.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use genc3wb::core::output::OutputPages;
use genc3wb::core::settings::{SETTINGS_FILE_NAME, Settings};

use std::fs;

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : Workbench::default, OutputGroup::configure */

#[test]
fn output_pages_follow_the_settings_of_the_session_and_are_not_stored() {
    // FR-031, FR-038, FR-093: the *output files* are held for the session alone
    let dir = std::env::temp_dir().join(format!("genc3wb-settings-session-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let file = dir.join(SETTINGS_FILE_NAME);
    let output = dir.join("out3.txt");
    fs::create_dir_all(&dir).expect("the directory can be created");
    fs::write(&output, "third").expect("the output file can be written");

    let mut session = Settings::default();
    session
        .set_output_path(3, &output.to_string_lossy())
        .expect("3 is within range");
    session.save(&file).expect("the settings can be stored");

    let mut pages = OutputPages::from_settings(&session);
    pages.next();
    pages.next();
    let restored = Settings::load(&file).expect("the settings can be restored");
    assert_eq!(
        (
            pages.page_count(),
            pages.current_number(),
            pages.current_file().map(|file| file.content.clone()),
            restored.output_numbers()
        ),
        (4, Some(3), Some("third".to_owned()), vec![])
    );
    let _ = fs::remove_dir_all(&dir);
}
