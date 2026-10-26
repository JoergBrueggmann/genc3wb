//! The round trip of the *automatic recording* of a start with the *handed file* of the command
//! line: the *workbench object* is constructed offscreen from a *settings file* the test writes
//! beside its executable, with the *handed file* left by `hand_file`, records the session the
//! test *front end* drives, and the recording is played back by a *verification run* of the
//! executable of *product* in a second process; the lines recorded, the report and the files at
//! the end are asserted. The file-open event is the case of 'recording_file_opened.rs', since
//! each case needs a Qt application of its own.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

mod round_trip;

use genc3wb::bridge::editing::Editing;
use genc3wb::bridge::workbench::{self, Workbench};

use qtbridge::QApp;

use round_trip::{
    SettingsKept, case_root, expected, front_end, new_recording, recordings, round_trip,
    write_network,
};

use std::fs;
use std::time::Duration;

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : Workbench::default with the handed file, workbench::hand_file;
 *                    InputGroup::load_initial telling the recorder; NetworkEditor::report_arrived
 *                    saving the backup; Automation::stop_recording */

// FR-476, FR-520, FR-534, FR-539, IR-206, IR-216, IR-218
// needs what 'recording_at_start.rs' needs
#[test]
#[ignore]
fn a_start_with_the_handed_file_of_the_command_line_records_its_files_and_plays_back() {
    let root = case_root("rh");
    let work = root.join("work");
    let network = write_network(&work);
    std::env::set_current_dir(&work).expect("the temporary directory can be entered");
    // SAFETY: the test is the only thread of its process at this point, so that no other thread
    // reads the environment while it is written.
    unsafe { std::env::set_var("QT_QPA_PLATFORM", "offscreen") };
    let settings = SettingsKept::write(None);
    let before = recordings();
    workbench::hand_file(network.to_string_lossy().into_owned());
    let exit_code = QApp::new()
        .application_name("genc3wb-test")
        .on_file_open(workbench::file_opened)
        .register::<Workbench>()
        .register::<Editing>()
        .load_qml(&front_end(None, true))
        .run();
    std::thread::sleep(Duration::from_millis(500));
    settings.restore();
    let script = new_recording(&before);
    let outcome = round_trip(exit_code, &script, &work);
    let _ = fs::remove_dir_all(&root);
    assert_eq!(outcome, expected(&script, &work));
}
