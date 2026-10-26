//! The test of the stop of a *playback* from the menu in-process: a script of immediate actions
//! is played back with a delay of 1.131 seconds, the *front end* stops it as the menu item does
//! once the second action is in progress, and the report, the exit code and that nothing
//! follows are asserted (FR-500, FR-515, FR-546).
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

mod start_cases;

use genc3wb::bridge::editing::Editing;
use genc3wb::bridge::workbench::Workbench;
use genc3wb::core::automation::CONFIGURATION_FILE_NAME;
use genc3wb::core::settings::Settings;

use qtbridge::QApp;

use start_cases::{build_system, case_root};

use std::fs;
use std::path::Path;

/// The script: the head line and five immediate actions on the lines 2 to 6.
const SCRIPT: &str = "automation script\ninit\ntyping mode insert\ntyping mode overwrite\n\
typing mode insert\ntyping mode overwrite\n";

/// Yields the *front end* of the test: starts the *playback* of `script` once the window is up,
/// stops it as the menu item does as soon as the line 3 is in progress, which the *player* takes
/// up at once and performs after its delay, and exits when the *playback* has finished: with 0
/// where the report holds the lines `2: ok: init` and `3: stopped: typing mode insert` alone,
/// the code is 2, no line is in progress and the *playback* no longer runs; with 1 otherwise,
/// and with 8 where the *playback* does not end within a minute.
fn front_end(script: &Path) -> Vec<u8> {
    format!(
        r#"
import QtQuick
import QtQuick.Controls
import genc3wb

ApplicationWindow {{
    id: root

    property bool stopped: false

    function leave(code) {{
        console.log("report '" + Workbench.automation.report + "', running " + Workbench.automation.running + ", line " + Workbench.automation.currentLineNumber);
        Workbench.network.shutDown();
        Workbench.releaseLock();
        Qt.exit(code);
    }}

    // steps the player while the playback runs, as Main.qml does (FR-500)
    Timer {{
        interval: 50
        repeat: true
        running: Workbench.automation.running
        onTriggered: Workbench.automation.step()
    }}

    // stops the playback as the menu item does, once the second action is in progress (FR-546)
    Timer {{
        interval: 20
        running: true
        repeat: true
        onTriggered: {{
            if (!root.stopped && Workbench.automation.currentLineNumber === 3) {{
                root.stopped = true;
                Workbench.automation.stop();
            }}
        }}
    }}

    Connections {{
        target: Workbench.automation

        function onFinished(code) {{
            const expected = "2: ok: init\n3: stopped: typing mode insert";
            const ok = code === 2
                && Workbench.automation.report === expected
                && Workbench.automation.currentLineNumber === 0
                && !Workbench.automation.running;
            root.leave(ok ? 0 : 1);
        }}
    }}

    Timer {{
        interval: 60000
        running: true
        onTriggered: root.leave(8)
    }}

    Component.onCompleted: {{
        Workbench.started();
        Workbench.automation.play("{script}");
    }}
}}
"#,
        script = script.display(),
    )
    .into_bytes()
}

/*  * validated        : ✅
 * completeness     : ✅ a stop while an action waits its delay: the action completed and
 *                       reported as stopped, the next not begun, the exit code 2
 * independence     : ✅ a folder of its own, the settings file restored
 * edge cases       : ✅ the stop during the delay before the action, the longest wait of a step
 * conforms to doc  : ✅ FR-500, FR-515, FR-546
 * covers bridge    : Automation::stop, Automation::record turning Completed into Stopped,
 *                    Automation::finish with the exit code of a stopped playback */

// FR-500, FR-515, FR-546
// needs the service executable genc3d of genc³ 0.36.0.0 or later, named by the environment
// variable GENC3D, with gc3 beside it; a Qt platform, for which it chooses the offscreen one;
// and a working directory of its own; it writes the settings file and the automation
// configuration beside the executable, the settings file restored
#[test]
#[ignore]
fn a_stopped_playback_completes_the_action_in_progress_and_ends_with_2() {
    let root = case_root("ps");
    fs::create_dir_all(&root).expect("the temporary directory can be created");
    let script = root.join("script.gc3s");
    fs::write(&script, SCRIPT).expect("the script is written");
    std::env::set_current_dir(&root).expect("the temporary directory can be entered");
    // SAFETY: the test is the only thread of its process at this point, so that no other thread
    // reads the environment while it is written.
    unsafe { std::env::set_var("QT_QPA_PLATFORM", "offscreen") };
    let settings_path = Settings::default_path();
    let settings_before = fs::read(&settings_path).ok();
    fs::write(
        &settings_path,
        format!(
            "build_system_path: {}\nautomatic: false\nplayback_delay: 1.131\n",
            build_system()
        ),
    )
    .expect("the settings file is written");
    let configuration = settings_path
        .parent()
        .expect("the settings file lies in a folder")
        .join(CONFIGURATION_FILE_NAME);
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/automation.gc3"),
        &configuration,
    )
    .expect("the automation configuration is copied beside the executable");
    let exit_code = QApp::new()
        .application_name("genc3wb-test")
        .register::<Workbench>()
        .register::<Editing>()
        .load_qml(&front_end(&script))
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
