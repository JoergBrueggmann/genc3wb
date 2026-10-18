//! The test of the network activated while the files of its *nodes* do not exist: the *input
//! creation confirmation dialog* names them before the *activate request*, a confirmed dialog with
//! 'Do not show this dialog again' creates the files, switches the *silent creation setting* on
//! and activates the network, which comes to be active; the settings file beside the executable
//! is restored.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use genc3wb::bridge::editing::Editing;
use genc3wb::bridge::workbench::Workbench;
use genc3wb::core::input_file::META_DSL_TEMPLATE;
use genc3wb::core::settings::Settings;

use qtbridge::QApp;

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

/// A *network file* of two *nodes*: the *meta compiler DSL* and the *input* of the first and the
/// *meta compiler DSL* of the second do not exist, the *connection* between them is produced.
const TWO_NODES: &str = "meta compiler-compiler network\n\
    node a\n  inputs\n    in/a.txt\n  outputs\n    x.txt\n\
    node b\n  inputs\n    x.txt\n  outputs\n    y.txt\n";

/// Yields the path of the build system: the environment variable `GENC3D`.
fn build_system() -> String {
    std::env::var("GENC3D").expect("GENC3D names the service executable genc3d")
}

/// Yields the *front end* of the test: names the *build system* and the *network file*,
/// provides its text, so that a *build* runs, the *network graph* is built and the network is
/// told valid, not active, its *nodes* being invalid without their files; selects `act`, on which the dialog has to name
/// the two *meta compiler DSLs* and the one *input* and not the *connection*, and confirms it
/// with the check box, on which the files have to be created, the *silent creation setting*
/// switched on and the network told active; exits with 0 where all held, with a code naming the
/// first failure otherwise, and with 8 where the network does not come to be active within a
/// minute.
fn front_end(network: &str) -> Vec<u8> {
    format!(
        r#"
import QtQuick
import QtQuick.Controls
import genc3wb

ApplicationWindow {{
    id: root

    property int step: 0
    property int ticks: 0
    property int asked: 0
    property string metaDsls: ""
    property string inputs: ""
    property int code: -1

    Connections {{
        target: Workbench.network
        function onCreationAsked(metaDsls, inputs) {{
            root.asked += 1;
            root.metaDsls = metaDsls;
            root.inputs = inputs;
            Workbench.network.answerCreation(true, true);
        }}
    }}

    Timer {{
        interval: 200
        running: true
        repeat: true
        onTriggered: {{
            root.ticks += 1;
            if (root.ticks === 1) {{
                Workbench.networkInput.idleExpired();
            }}
            var mode = Workbench.network.networkMode;
            if (root.step === 0 && mode === 2 && Workbench.network.graphSource.length > 0 && Workbench.network.switchable) {{
                root.step = 1;
                root.ticks = 0;
                Workbench.network.activate();
            }} else if (root.step === 1 && mode === 3) {{
                var lines = root.metaDsls.split("\n");
                if (root.asked !== 1) {{
                    root.code = 1;
                }} else if (lines.length !== 2 || lines[0].indexOf("a.gc3") < 0 || lines[1].indexOf("b.gc3") < 0) {{
                    console.log("meta DSLs: " + root.metaDsls);
                    root.code = 2;
                }} else if (root.inputs.indexOf("in/a.txt") < 0 || root.inputs.indexOf("x.txt") >= 0) {{
                    console.log("inputs: " + root.inputs);
                    root.code = 3;
                }} else if (!Workbench.silentCreation) {{
                    root.code = 4;
                }} else {{
                    root.code = 0;
                }}
                Workbench.network.shutDown();
                Workbench.releaseLock();
                Qt.exit(root.code);
            }}
        }}
    }}

    Timer {{
        interval: 60000
        running: true
        onTriggered: {{
            console.log("asked " + root.asked + ", mode " + Workbench.network.networkMode);
            Workbench.network.shutDown();
            Workbench.releaseLock();
            Qt.exit(8);
        }}
    }}

    Component.onCompleted: {{
        Workbench.network.buildSystemPath = "{build_system}";
        Workbench.networkInput.path = "{network}";
    }}
}}
"#,
        build_system = build_system()
    )
    .into_bytes()
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : NetworkEditor::activate, activate_preparing, files_to_create, prepared,
 *                    answer_creation, resume; Workbench::silent_creation */

// FR-260, FR-487 to FR-490, IR-210
// needs the service executable genc3d of genc³ 0.36.0.0 or later, named by the environment
// variable GENC3D, and the program dot of Graphviz; a Qt platform, for which it chooses the
// offscreen one; and a working directory of its own, short enough for the paths of the Unix
// domain sockets; it writes the settings file beside the executable, which it restores
#[test]
#[ignore]
fn the_network_is_activated_once_the_missing_files_of_its_nodes_are_created() {
    let folder: PathBuf = std::env::temp_dir().join(format!("g3wb-ic-{}", std::process::id()));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).expect("the temporary directory can be written");
    fs::write(folder.join("n.gc3n"), TWO_NODES).expect("the network file is written");
    std::env::set_current_dir(&folder).expect("the temporary directory can be entered");
    let settings_path = Settings::default_path();
    let settings_before = fs::read(&settings_path).ok();
    // SAFETY: the test is the only thread of its process at this point, so that no other thread
    // reads the environment while it is written.
    unsafe { std::env::set_var("QT_QPA_PLATFORM", "offscreen") };
    let network = folder.join("n.gc3n").to_string_lossy().into_owned();
    let exit_code = QApp::new()
        .application_name("genc3wb-test")
        .register::<Workbench>()
        .register::<Editing>()
        .load_qml(&front_end(&network))
        .run();
    std::thread::sleep(Duration::from_millis(500));
    let stored = fs::read_to_string(&settings_path).unwrap_or_default();
    match settings_before {
        Some(octets) => fs::write(&settings_path, octets).expect("the settings are restored"),
        None => {
            let _ = fs::remove_file(&settings_path);
        }
    }
    let a = fs::read_to_string(folder.join("a.gc3")).ok();
    let b = fs::read_to_string(folder.join("b.gc3")).ok();
    let input = fs::read(folder.join("in").join("a.txt")).ok();
    let _ = fs::remove_dir_all(&folder);
    assert_eq!(
        (
            exit_code,
            a.as_deref(),
            b.as_deref(),
            input.as_deref(),
            stored.contains("silent_creation: true")
        ),
        (
            0,
            Some(META_DSL_TEMPLATE),
            Some(META_DSL_TEMPLATE),
            Some(&b""[..]),
            true
        )
    );
}
