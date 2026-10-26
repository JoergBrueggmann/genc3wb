//! The test of the line in progress of a *playback* and of the *playback delay* in-process: a
//! script of immediate actions is played back with a delay of 0.283 seconds, the *front end*
//! samples the line the *player* reports for the status bar, and the lines, their order, the
//! time the *playback* took and the delay kept through the `settings` line are asserted
//! (FR-500, FR-509, FR-543, FR-545).
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

/// The script: the head line and five actions on the lines 2 to 6, the `settings` line among
/// them switching the delay off, which the *playback* does not take.
const SCRIPT: &str = "automation script\ninit\ntyping mode insert\n\
settings idle 2.0 long idle 16.0 automatic off tab size 4 spaces off silent creation off delay off\n\
typing mode overwrite\ntyping mode insert\n";

/// Yields the *front end* of the test: starts the *playback* of `script` once the window is up,
/// samples the line in progress every 20 milliseconds, noting every change, and exits when the
/// *playback* has finished: with 0 where the lines came as 2 to 6 with their texts, where none
/// is reported after the end, where the *playback* took at least the four delays of 0.283
/// seconds and where the delay of the settings is still 0.283 seconds; with 1 otherwise, with 2
/// where the report holds a line that is not ok, and with 8 where the *playback* does not end
/// within a minute.
fn front_end(script: &Path) -> Vec<u8> {
    format!(
        r#"
import QtQuick
import QtQuick.Controls
import genc3wb

ApplicationWindow {{
    id: root

    property var numbers: []
    property var texts: []
    property int lastNumber: 0
    property double began: 0

    function leave(code) {{
        console.log("numbers " + JSON.stringify(root.numbers) + ", texts " + JSON.stringify(root.texts) + ", delay " + Workbench.playbackDelay + ", report '" + Workbench.automation.report + "'");
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

    Timer {{
        interval: 20
        running: true
        repeat: true
        onTriggered: {{
            const number = Workbench.automation.currentLineNumber;
            if (number !== root.lastNumber) {{
                root.lastNumber = number;
                if (number > 0) {{
                    root.numbers.push(number);
                    root.texts.push(Workbench.automation.currentLine);
                }}
            }}
        }}
    }}

    Connections {{
        target: Workbench.automation

        function onFinished(code) {{
            const took = Date.now() - root.began;
            const report = Workbench.automation.report.split("\n");
            if (report.some(line => line.indexOf(": ok: ") < 0)) {{
                root.leave(2);
            }}
            const expectedNumbers = [2, 3, 4, 5, 6];
            const expectedTexts = ["init", "typing mode insert",
                                   "settings idle 2.0 long idle 16.0 automatic off tab size 4 spaces off silent creation off delay off",
                                   "typing mode overwrite", "typing mode insert"];
            const ok = code === 0
                && JSON.stringify(root.numbers) === JSON.stringify(expectedNumbers)
                && JSON.stringify(root.texts) === JSON.stringify(expectedTexts)
                && Workbench.automation.currentLineNumber === 0
                && Workbench.automation.currentLine === ""
                && took >= 1100
                && Workbench.playbackDelay === 283;
            console.log("took " + took + " ms");
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
        root.began = Date.now();
        Workbench.automation.play("{script}");
    }}
}}
"#,
        script = script.display(),
    )
    .into_bytes()
}

/*  * validated        : ✅
 * completeness     : ✅ the first action without a delay, every other after one; the line and
 *                       its number during an action and none between the actions and after
 *                       the end; the `settings` line leaving the delay
 * independence     : ✅ a folder of its own, the settings file restored
 * edge cases       : ✅ the last action, after which no line is reported
 * conforms to doc  : ✅ FR-500, FR-509, FR-543, FR-545
 * covers bridge    : Automation::play, Automation::step with the delay, current_line_number,
 *                    current_line, Workbench::apply_settings leaving the delay */

// FR-500, FR-509, FR-543, FR-545
// needs the service executable genc3d of genc³ 0.36.0.0 or later, named by the environment
// variable GENC3D, with gc3 beside it; a Qt platform, for which it chooses the offscreen one;
// and a working directory of its own; it writes the settings file and the automation
// configuration beside the executable, the settings file restored
#[test]
#[ignore]
fn the_line_in_progress_is_reported_and_every_action_but_the_first_waits_the_delay() {
    let root = case_root("pl");
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
            "build_system_path: {}\nautomatic: false\nplayback_delay: 0.28\n",
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
