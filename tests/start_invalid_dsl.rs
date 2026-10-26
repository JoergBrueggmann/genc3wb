//! The test of a start on a network whose files exist and whose *meta compiler DSL* the *node*
//! refuses: the *product* tries its own activation, since no file is missing, the *build
//! system* refuses it with the invalid *node*, and the failure is shown in the status bar in
//! place of the *network mode* while the graph stays presented and the *error message* of the
//! *build* stays empty (FR-541).
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

mod start_cases;

use genc3wb::bridge::editing::Editing;
use genc3wb::bridge::workbench::Workbench;
use genc3wb::core::settings::Settings;

use qtbridge::QApp;

use start_cases::{BROKEN_META_DSL, Expected, case_root, front_end, write_network_with_dsl};

use std::fs;

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : NetworkEditor::try_own_activation with every file present, activate_directly,
 *                    report_arrived on Report::Activated with an error, take_request_failure */

// FR-534, FR-541, FR-258
// needs what 'start_missing_files_idle_late.rs' needs
#[test]
#[ignore]
fn a_refused_own_activation_is_told_in_the_status_bar_while_the_graph_stays() {
    let root = case_root("si");
    let work = root.join("work");
    let network = write_network_with_dsl(&work, BROKEN_META_DSL);
    std::env::set_current_dir(&work).expect("the temporary directory can be entered");
    // SAFETY: the test is the only thread of its process at this point, so that no other thread
    // reads the environment while it is written.
    unsafe { std::env::set_var("QT_QPA_PLATFORM", "offscreen") };
    let settings_path = Settings::default_path();
    let settings_before = fs::read(&settings_path).ok();
    let exit_code = QApp::new()
        .application_name("genc3wb-test")
        .register::<Workbench>()
        .register::<Editing>()
        .load_qml(&front_end(&network, 1, 40, &Expected::ValidWithFailedTry))
        .run();
    match settings_before {
        Some(octets) => fs::write(&settings_path, octets).expect("the settings are restored"),
        None => {
            let _ = fs::remove_file(&settings_path);
        }
    }
    let _ = fs::remove_dir_all(&root);
    assert_eq!(exit_code, 0);
}
