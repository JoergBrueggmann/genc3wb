//! The test of the `delay` action in-process: a script with `delay 0.5` between immediate
//! actions is played back without a *playback delay* and timed, and a script with `delay 5`,
//! beyond the bound, fails its action (FR-514, FR-547).
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

/// The first script: a pause of half a second between two immediate actions; and the second, a
/// pause beyond the bound of four seconds.
const SCRIPT: &str = "automation script\ninit\ndelay 0.5\ntyping mode insert\n";
const SCRIPT_BEYOND: &str = "automation script\ninit\ndelay 5\ntyping mode insert\n";

/// Yields the *front end* of the test: starts the *playback* of `script` once the window is up,
/// and, when it has finished, the one of `beyond`; exits with 0 where the first report holds
/// `2: ok: init`, `3: ok: delay 0.5` and `4: ok: typing mode insert` with the code 0 and took at
/// least half a second, and the second holds `2: ok: init` and an error line for `delay 5` and
/// nothing after, with the code 2; with 1 otherwise, and with 8 where a *playback* does not end
/// within a minute.
fn front_end(script: &Path, beyond: &Path) -> Vec<u8> {
    format!(
        r#"
import QtQuick
import QtQuick.Controls
import genc3wb

ApplicationWindow {{
    id: root

    property int playbacks: 0
    property double began: 0
    property bool firstOk: false

    function leave(code) {{
        console.log("report '" + Workbench.automation.report + "', playbacks " + root.playbacks);
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

    Connections {{
        target: Workbench.automation

        function onFinished(code) {{
            const took = Date.now() - root.began;
            const report = Workbench.automation.report;
            console.log("playback " + root.playbacks + " took " + took + " ms, code " + code + ", report '" + report + "'");
            if (root.playbacks === 0) {{
                root.firstOk = code === 0
                    && report === "2: ok: init\n3: ok: delay 0.5\n4: ok: typing mode insert"
                    && took >= 500;
                root.playbacks = 1;
                root.began = Date.now();
                Workbench.automation.play("{beyond}");
                return;
            }}
            const lines = report.split("\n");
            const ok = root.firstOk
                && code === 2
                && lines.length === 2
                && lines[0] === "2: ok: init"
                && lines[1].indexOf("3: error: delay 5: ") === 0
                && lines[1].indexOf("exceeds 4 seconds") > 0;
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
        beyond = beyond.display(),
    )
    .into_bytes()
}

/*  * validated        : ✅
 * completeness     : ✅ a delay within the bound, waited and completed; one beyond it, failed
 *                       and ending the playback
 * independence     : ✅ a folder of its own, the settings file restored
 * edge cases       : ✅ the bound exceeded by one second; the delay without a unit
 * conforms to doc  : ✅ FR-514, FR-547
 * covers bridge    : Automation::step for Delay: Waiting::until, allowance_of, the readiness
 *                    failing beyond MAX_DELAY_ACTION */

// FR-514, FR-547
// needs the service executable genc3d of genc³ 0.36.0.0 or later, named by the environment
// variable GENC3D, with gc3 beside it; a Qt platform, for which it chooses the offscreen one;
// and a working directory of its own; it writes the settings file and the automation
// configuration beside the executable, the settings file restored
#[test]
#[ignore]
fn a_delay_action_waits_its_time_and_one_beyond_four_seconds_fails() {
    let root = case_root("pd");
    fs::create_dir_all(&root).expect("the temporary directory can be created");
    let script = root.join("script.gc3s");
    fs::write(&script, SCRIPT).expect("the script is written");
    let beyond = root.join("beyond.gc3s");
    fs::write(&beyond, SCRIPT_BEYOND).expect("the script is written");
    std::env::set_current_dir(&root).expect("the temporary directory can be entered");
    // SAFETY: the test is the only thread of its process at this point, so that no other thread
    // reads the environment while it is written.
    unsafe { std::env::set_var("QT_QPA_PLATFORM", "offscreen") };
    let settings_path = Settings::default_path();
    let settings_before = fs::read(&settings_path).ok();
    fs::write(
        &settings_path,
        format!("build_system_path: {}\nautomatic: false\n", build_system()),
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
        .load_qml(&front_end(&script, &beyond))
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
