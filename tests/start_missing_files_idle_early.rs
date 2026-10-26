//! The test of a start on a *network file* whose node files are missing, the first *build*
//! begun at once: the idle is expired at the first tick, so that the *node descriptions* and
//! the *network mode* arrive close together, the mode first all the same, as the *build* waits
//! for `dot`; the *product* tries no activation of its own (FR-534), and at the end the network
//! is a *valid network* with its graph, no message anywhere and no file created, as in
//! 'start_missing_files_idle_late.rs'. The *automatic recording* of the start holds the
//! *network file* and one `remove` line per missing file and no `activate`, and plays back by
//! a *verification run* with every action ok.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

mod round_trip;
mod start_cases;

use genc3wb::bridge::editing::Editing;
use genc3wb::bridge::workbench::Workbench;

use qtbridge::QApp;

use round_trip::{SettingsKept, new_recording, recordings, verification_run};
use start_cases::{Expected, case_root, files_of, front_end, write_network_file_alone};

use std::fs;
use std::time::Duration;

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : NetworkEditor::report_arrived on the build close after the state response,
 *                    try_own_activation with files missing, files_to_create; the recorder's
 *                    backup with remove lines */

// FR-534, FR-487, FR-258, FR-520, FR-539; the design, § Presentation derived from the state
// needs what 'start_missing_files_idle_late.rs' needs, with gc3 beside genc3d
#[test]
#[ignore]
fn a_network_file_without_its_files_stays_valid_with_its_graph_when_the_build_begins_at_once() {
    let root = case_root("se");
    let work = root.join("work");
    let network = write_network_file_alone(&work);
    std::env::set_current_dir(&work).expect("the temporary directory can be entered");
    // SAFETY: the test is the only thread of its process at this point, so that no other thread
    // reads the environment while it is written.
    unsafe { std::env::set_var("QT_QPA_PLATFORM", "offscreen") };
    let settings = SettingsKept::write(None);
    let before = recordings();
    let exit_code = QApp::new()
        .application_name("genc3wb-test")
        .register::<Workbench>()
        .register::<Editing>()
        .load_qml(&front_end(&network, 1, 30, &Expected::ValidWithoutTry))
        .run();
    std::thread::sleep(Duration::from_millis(500));
    settings.restore();
    let files = files_of(&work);
    let script = new_recording(&before);
    let text = fs::read_to_string(&script).unwrap_or_default();
    let lines: Vec<&str> = text.lines().collect();
    let (playback_code, report, output) = verification_run(&script);
    let files_after_playback = files_of(&work);
    if let Some(folder) = script.parent() {
        let _ = fs::remove_dir_all(folder);
    }
    let _ = fs::remove_dir_all(&root);
    let kinds: Vec<&str> = lines
        .iter()
        .skip(2)
        .map(|line| line.split([' ', '"']).next().unwrap_or(""))
        .collect();
    assert_eq!(
        (
            exit_code,
            files,
            kinds,
            lines.iter().any(|line| line.starts_with("activate")),
            playback_code,
            report
                .iter()
                .filter(|line| !line.contains(": ok: "))
                .count(),
            files_after_playback,
        ),
        (
            0,
            vec!["n.gc3n".to_owned()],
            vec![
                "init", "settings", "build", "restore", "remove", "remove", "remove", "network",
                "idle",
            ],
            false,
            0,
            0,
            vec!["n.gc3n".to_owned()],
        ),
        "the recording:\n{text}\nthe playback:\n{output}"
    );
}
