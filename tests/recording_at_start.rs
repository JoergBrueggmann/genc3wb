//! The round trip of the *automatic recording* of a start with the *network file* named in the
//! *settings file*, the way the *software problem report* of 2026-10-25 began: the *workbench
//! object* is constructed offscreen from a *settings file* the test writes beside its executable,
//! as *product* does at a start, records the session the test *front end* drives, and the
//! recording is played back by a *verification run* of the executable of *product* in a second
//! process; the lines recorded, the report and the files at the end are asserted.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

mod round_trip;

use genc3wb::bridge::editing::Editing;
use genc3wb::bridge::workbench::Workbench;

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
 * covers bridge    : Workbench::default, the recorder started before the input group of the
 *                    network file loads its initial path; InputGroup::load_initial telling
 *                    the recorder; NetworkEditor::report_arrived saving the backup;
 *                    Automation::stop_recording */

// FR-520, FR-534, FR-539, IR-216, IR-218
// needs the service executable genc3d of genc³ 0.36.0.0 or later, named by the environment
// variable GENC3D, with gc3 beside it, and the program dot of Graphviz; a Qt platform, for which
// it chooses the offscreen one; and a working directory of its own, short enough for the paths
// of the Unix domain sockets; it writes the settings file beside the executable, which it
// restores
#[test]
#[ignore]
fn a_start_with_the_network_file_named_in_the_settings_records_its_files_and_plays_back() {
    let root = case_root("rs");
    let work = root.join("work");
    let network = write_network(&work);
    std::env::set_current_dir(&work).expect("the temporary directory can be entered");
    // SAFETY: the test is the only thread of its process at this point, so that no other thread
    // reads the environment while it is written.
    unsafe { std::env::set_var("QT_QPA_PLATFORM", "offscreen") };
    let settings = SettingsKept::write(Some(&network));
    let before = recordings();
    let exit_code = QApp::new()
        .application_name("genc3wb-test")
        .register::<Workbench>()
        .register::<Editing>()
        .load_qml(&front_end(None, false))
        .run();
    std::thread::sleep(Duration::from_millis(500));
    settings.restore();
    let script = new_recording(&before);
    let outcome = round_trip(exit_code, &script, &work);
    let _ = fs::remove_dir_all(&root);
    assert_eq!(outcome, expected(&script, &work));
}
