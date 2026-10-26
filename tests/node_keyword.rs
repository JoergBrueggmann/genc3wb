//! The test of the scenario of the *software problem report* of 2026-10-25, the *network mode*
//! along it: the *workbench object* is driven offscreen, the network loaded and activated by the
//! *product* itself, the keyword `node` of the *network file* broken by typed characters, on
//! which the network is an *invalid network*, and mended by deletions, on which it is a *valid
//! network* and stays not active, since the *product* activates a network itself the first time
//! it is valid alone (FR-534); the script 'tests/scripts/node_keyword' plays the same scenario
//! and verifies the text, which is what a script observes. The settings file beside the
//! executable is restored.
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

/// A *network file* of one *node* `copy`, whose files exist.
const ONE_NODE: &str = "meta compiler-compiler network\n\
    \tnode copy\n\t\tinputs\n\t\t\tin.txt\n\t\toutputs\n\t\t\tout.txt\n";

/// The *meta compiler DSL* of the *node* `copy`.
const COPY_META_DSL: &str = "syntax\n    Root = Chars, EOS\n    Chars = Ch, { Ch }\n    \
    Ch = 'h' | 'e' | 'l' | 'o' | 'w' | 'r' | 'd' | ' '\ngenerator\n    Root -> flat(#0)\n";

/// Yields the path of the build system: the environment variable `GENC3D`.
fn build_system() -> String {
    std::env::var("GENC3D").expect("GENC3D names the service executable genc3d")
}

/// Yields the *front end* of the test: names the *build system* and the *network file*, expires
/// the idle, so that the text is provided, waits until the *product* has activated the network
/// and presents the graph, breaks the keyword `node` by two characters
/// and expires the idle, waits until the network is an *invalid network*, mends the keyword by
/// the text of the file and expires the idle, waits until the network is a *valid network*, and
/// watches the mode for two seconds: exits with 0 where it stays a *valid network*, with 1 where
/// it comes to be active, with 3 where the broken network is not told invalid within fifteen
/// seconds, and with 8 where a step does not come within a minute.
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
            if (root.step === 0 && root.ticks === 1) {{
                // the first idle expiry provides the text of the network file
                Workbench.networkInput.idleExpired();
            }} else if (root.step === 0 && mode === 3 && Workbench.network.graphSource.length > 0) {{
                Workbench.networkInput.text = Workbench.networkInput.text.replace("node copy", "nodexx copy");
                Workbench.networkInput.idleExpired();
                root.step = 1;
                root.ticks = 0;
            }} else if (root.step === 1) {{
                if (mode === 1) {{
                    Workbench.networkInput.text = Workbench.networkInput.text.replace("nodexx copy", "node copy");
                    Workbench.networkInput.idleExpired();
                    root.step = 2;
                    root.ticks = 0;
                }} else if (root.ticks >= 75) {{
                    root.leave(3);
                }}
            }} else if (root.step === 2 && mode === 2) {{
                root.step = 3;
                root.ticks = 0;
            }} else if (root.step === 3) {{
                if (mode !== 2) {{
                    root.leave(1);
                }} else if (root.ticks >= 10) {{
                    root.leave(0);
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
 * covers bridge    : NetworkEditor::set_network_path, report_arrived, activate_on_valid;
 *                    InputGroup::set_path, set_text, idle_expired */

// FR-229, FR-258, FR-534
// needs the service executable genc3d of genc³ 0.36.0.0 or later, named by the environment
// variable GENC3D, and the program dot of Graphviz; a Qt platform, for which it chooses the
// offscreen one; and a working directory of its own, short enough for the paths of the Unix
// domain sockets; it writes the settings file beside the executable, which it restores
#[test]
#[ignore]
fn a_broken_keyword_node_makes_the_network_invalid_and_the_mended_one_valid_not_active() {
    let root: PathBuf = std::env::temp_dir().join(format!("g3wb-nk-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let work = root.join("work");
    fs::create_dir_all(&work).expect("the temporary directory can be written");
    fs::write(work.join("n.gc3n"), ONE_NODE).expect("the network file is written");
    fs::write(work.join("copy.gc3"), COPY_META_DSL).expect("the meta compiler DSL is written");
    fs::write(work.join("in.txt"), "hello world").expect("the input is written");
    fs::write(work.join("out.txt"), "hello world").expect("the output is written");
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
        .load_qml(&front_end(&work.join("n.gc3n").to_string_lossy()))
        .run();
    std::thread::sleep(Duration::from_millis(500));
    match settings_before {
        Some(octets) => fs::write(&settings_path, octets).expect("the settings are restored"),
        None => {
            let _ = fs::remove_file(&settings_path);
        }
    }
    let text = fs::read_to_string(work.join("n.gc3n")).unwrap_or_default();
    let _ = fs::remove_dir_all(&root);
    assert_eq!((exit_code, text.as_str()), (0, ONE_NODE));
}
