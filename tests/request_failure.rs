//! The test of the *request failure* kept apart from the *error message* of a *build*: on a
//! network whose files exist and whose *meta compiler DSL* the *node* refuses, the user's
//! *activate request* is refused, then an edit of the *network file* has a *build* succeed
//! after the refusal; the failure stays in the status bar in place of the *network mode*, the
//! graph of the *build* is presented and the *error message* stays empty (FR-261, FR-541).
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

mod start_cases;

use genc3wb::bridge::editing::Editing;
use genc3wb::bridge::workbench::Workbench;
use genc3wb::core::settings::Settings;

use qtbridge::QApp;

use start_cases::{BROKEN_META_DSL, build_system, case_root, write_network_with_dsl};

use std::fs;
use std::path::Path;

/// Yields the *front end* of the test: names the *build system* and the *network file*, expires
/// the idle, waits until the graph is presented and the *product*'s own try has failed, selects
/// `act` as the user, waits until that *request* is answered, appends a comment to the
/// *network file* and expires the idle, so that a *build* succeeds after the refusal, and exits
/// three seconds later with 0 where the *request failure* is shown, the mode is *valid
/// network*, the graph is presented and the *error message* is empty, with 1 otherwise, with 3
/// where the user's *request* was not refused, and with 8 where a step does not come within a
/// minute.
fn front_end(network: &Path) -> Vec<u8> {
    format!(
        r#"
import QtQuick
import QtQuick.Controls
import genc3wb

ApplicationWindow {{
    id: root

    property int step: 0
    property int ticks: 0
    property string refused: ""

    function leave(code) {{
        console.log("mode " + Workbench.network.networkMode + ", graph " + (Workbench.network.graphSource.length > 0) + ", error '" + Workbench.network.errorMessage + "', request failure '" + Workbench.network.requestFailure + "' at step " + root.step);
        Workbench.network.shutDown();
        Workbench.releaseLock();
        Qt.exit(code);
    }}

    Timer {{
        interval: 200
        running: true
        repeat: true
        onTriggered: {{
            root.ticks += 1;
            var network = Workbench.network;
            if (root.step === 0) {{
                if (root.ticks === 1) {{
                    Workbench.networkInput.idleExpired();
                }} else if (network.graphSource.length > 0 && !network.activating && network.requestFailure.length > 0) {{
                    // the own try failed; the user's request is refused the same way
                    network.activate();
                    root.step = 1;
                    root.ticks = 0;
                }}
            }} else if (root.step === 1 && root.ticks >= 3 && !network.activating) {{
                root.refused = network.requestFailure;
                if (root.refused.length === 0) {{
                    root.leave(3);
                }}
                // a build that succeeds after the refusal
                Workbench.networkInput.text = Workbench.networkInput.text + "// edited\n";
                Workbench.networkInput.idleExpired();
                root.step = 2;
                root.ticks = 0;
            }} else if (root.step === 2 && root.ticks >= 15) {{
                var ok = network.networkMode === 2
                    && network.graphSource.length > 0
                    && network.errorMessage.length === 0
                    && network.requestFailure === root.refused;
                root.leave(ok ? 0 : 1);
            }}
        }}
    }}

    Timer {{
        interval: 60000
        running: true
        onTriggered: root.leave(8)
    }}

    Component.onCompleted: {{
        Workbench.network.buildSystemPath = "{build_system}";
        Workbench.networkInput.path = "{network}";
    }}
}}
"#,
        build_system = build_system(),
        network = network.display(),
    )
    .into_bytes()
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : NetworkEditor::activate, activate_preparing clearing the request failure,
 *                    report_arrived on Report::Activated with an error, take_request_failure,
 *                    report_arrived on Report::Built leaving the request failure */

// FR-261, FR-541, FR-074, FR-078; the design, § Presentation derived from the state
// needs the service executable genc3d of genc³ 0.36.0.0 or later, named by the environment
// variable GENC3D, and the program dot of Graphviz; a Qt platform, for which it chooses the
// offscreen one; and a working directory of its own, short enough for the paths of the Unix
// domain sockets; it writes the settings file beside the executable, which it restores
#[test]
#[ignore]
fn a_refused_request_of_the_user_stays_in_the_status_bar_after_a_build_succeeds() {
    let root = case_root("rf");
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
        .load_qml(&front_end(&network))
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
