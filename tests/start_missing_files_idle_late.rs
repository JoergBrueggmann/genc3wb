//! The test of a start on a *network file* whose node files are missing, the *state response*
//! of the *build system* arriving long before the first *build*: the idle is expired three
//! seconds after the start, so that the *network mode* is known first and the *node
//! descriptions* last; the *product* tries no activation of its own (FR-534), and at the end
//! the network is a *valid network* with its graph, no message anywhere and no file created.
//! The other order is the case of 'start_missing_files_idle_early.rs', since each needs a Qt
//! application of its own.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

mod start_cases;

use genc3wb::bridge::editing::Editing;
use genc3wb::bridge::workbench::Workbench;
use genc3wb::core::settings::Settings;

use qtbridge::QApp;

use start_cases::{Expected, case_root, files_of, front_end, write_network_file_alone};

use std::fs;

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : NetworkEditor::report_arrived on the state response before the build,
 *                    try_own_activation with files missing, files_to_create */

// FR-534, FR-487, FR-258; the design, § Presentation derived from the state
// needs the service executable genc3d of genc³ 0.36.0.0 or later, named by the environment
// variable GENC3D, and the program dot of Graphviz; a Qt platform, for which it chooses the
// offscreen one; and a working directory of its own, short enough for the paths of the Unix
// domain sockets; it writes the settings file beside the executable, which it restores
#[test]
#[ignore]
fn a_network_file_without_its_files_stays_valid_with_its_graph_when_the_mode_arrives_first() {
    let root = case_root("sl");
    let work = root.join("work");
    let network = write_network_file_alone(&work);
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
        .load_qml(&front_end(&network, 15, 40, &Expected::ValidWithoutTry))
        .run();
    match settings_before {
        Some(octets) => fs::write(&settings_path, octets).expect("the settings are restored"),
        None => {
            let _ = fs::remove_file(&settings_path);
        }
    }
    let files = files_of(&work);
    let _ = fs::remove_dir_all(&root);
    assert_eq!((exit_code, files), (0, vec!["n.gc3n".to_owned()]));
}
