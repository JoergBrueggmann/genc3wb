//! The test of a *network file* named while another one is named: the *build system* started on
//! it activates the network at its start-up, and the network editor transmits nothing of the
//! *network file* named before, so that the network stays an *active network*; the settings
//! file beside the executable is restored.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use genc3wb::bridge::editing::Editing;
use genc3wb::bridge::workbench::Workbench;
use genc3wb::core::settings::Settings;

use qtbridge::QApp;

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

/// The first *network file*: one *node* whose *meta compiler DSL* does not exist, so that the
/// activation at start-up ends with the *node* invalid and the network stays a *valid network*.
const FIRST: &str = "meta compiler-compiler network\n\
    node a\n  inputs\n    a.in\n  outputs\n    x.txt\n";

/// The second *network file*: one *node* `copy`, whose files exist, so that the activation at
/// start-up makes the network an *active network*.
const SECOND: &str = "meta compiler-compiler network\n\
    node copy\n  inputs\n    in.txt\n  outputs\n    out.txt\n";

/// The *meta compiler DSL* of the *node* `copy`.
const COPY_META_DSL: &str = "syntax\n    Root = Chars, EOS\n    Chars = Ch, { Ch }\n    \
    Ch = 'h' | 'e' | 'l' | 'o' | 'w' | 'r' | 'd' | ' '\ngenerator\n    Root -> flat(#0)\n";

/// Yields the path of the build system: the environment variable `GENC3D`.
fn build_system() -> String {
    std::env::var("GENC3D").expect("GENC3D names the service executable genc3d")
}

/// Yields the *front end* of the test: names the *build system* and the first *network file*,
/// provides its text, so that a *build* runs and the network is told valid, not active; names
/// the second *network file* and provides its text one second later, as the *idle time* would,
/// and watches the *network mode* for three seconds from then on: exits with 0 where the
/// network is an *active network* at the end of the three seconds, with 1 where it is not, and
/// with 8 where the first network does not come to be valid within a minute.
fn front_end(first: &str, second: &str) -> Vec<u8> {
    format!(
        r#"
import QtQuick
import QtQuick.Controls
import genc3wb

ApplicationWindow {{
    id: root

    property int step: 0
    property int ticks: 0

    function leave(code) {{
        console.log("mode " + Workbench.network.networkMode + " at step " + root.step);
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
            var mode = Workbench.network.networkMode;
            if (root.step === 0) {{
                if (root.ticks === 1) {{
                    Workbench.networkInput.idleExpired();
                }} else if (mode === 2 && Workbench.network.graphSource.length > 0) {{
                    root.step = 1;
                    root.ticks = 0;
                    Workbench.networkInput.path = "{second}";
                }}
            }} else if (root.step === 1) {{
                if (root.ticks === 5) {{
                    Workbench.networkInput.idleExpired();
                }} else if (root.ticks === 20) {{
                    root.leave(mode === 3 ? 0 : 1);
                }}
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
        Workbench.networkInput.path = "{first}";
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
 * covers bridge    : NetworkEditor::set_network_path, restart, apply_increment;
 *                    InputGroup::set_path, idle_expired */

// FR-067, FR-070, FR-229
// needs the service executable genc3d of genc³ 0.36.0.0 or later, named by the environment
// variable GENC3D, and the program dot of Graphviz; a Qt platform, for which it chooses the
// offscreen one; and a working directory of its own, short enough for the paths of the Unix
// domain sockets; it writes the settings file beside the executable, which it restores
#[test]
#[ignore]
fn a_network_file_named_after_another_one_is_active() {
    let folder: PathBuf = std::env::temp_dir().join(format!("g3wb-sn-{}", std::process::id()));
    let _ = fs::remove_dir_all(&folder);
    let first = folder.join("first");
    let second = folder.join("second");
    fs::create_dir_all(&first).expect("the first folder can be written");
    fs::create_dir_all(&second).expect("the second folder can be written");
    fs::write(first.join("first.gc3n"), FIRST).expect("the first network file is written");
    fs::write(second.join("n.gc3n"), SECOND).expect("the second network file is written");
    fs::write(second.join("copy.gc3"), COPY_META_DSL).expect("the meta DSL is written");
    fs::write(second.join("in.txt"), "hello").expect("the input is written");
    std::env::set_current_dir(&folder).expect("the temporary directory can be entered");
    let settings_path = Settings::default_path();
    let settings_before = fs::read(&settings_path).ok();
    // SAFETY: the test is the only thread of its process at this point, so that no other thread
    // reads the environment while it is written.
    unsafe { std::env::set_var("QT_QPA_PLATFORM", "offscreen") };
    let first_path = first.join("first.gc3n").to_string_lossy().into_owned();
    let second_path = second.join("n.gc3n").to_string_lossy().into_owned();
    let exit_code = QApp::new()
        .application_name("genc3wb-test")
        .register::<Workbench>()
        .register::<Editing>()
        .load_qml(&front_end(&first_path, &second_path))
        .run();
    std::thread::sleep(Duration::from_millis(500));
    match settings_before {
        Some(octets) => fs::write(&settings_path, octets).expect("the settings are restored"),
        None => {
            let _ = fs::remove_file(&settings_path);
        }
    }
    let _ = fs::remove_dir_all(&folder);
    assert_eq!(exit_code, 0);
}
