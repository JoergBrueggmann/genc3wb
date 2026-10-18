//! The test of a *node* opened while the file of its *meta compiler DSL* and the file of its
//! *input* do not exist: the *input creation confirmation dialog* names both, a cancelled dialog
//! opens nothing, a confirmed one creates the folder, the *meta compiler DSL* from the *meta DSL
//! template* and the *input* empty, and the *node* is held on them, valid.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use genc3wb::bridge::editing::Editing;
use genc3wb::bridge::workbench::Workbench;
use genc3wb::core::input_file::META_DSL_TEMPLATE;

use qtbridge::QApp;

use std::fs;
use std::path::PathBuf;
use std::time::Duration;

/// A *network file* of one *node* whose *meta compiler DSL* lies in a folder that does not
/// exist and whose *input* does not exist either.
const ONE_NODE: &str = "meta compiler-compiler network\n\
    node a\n  inputs\n    a.in\n  outputs\n    x.txt\n  transformation sub/a.gc3\n";

/// Yields the path of the build system: the environment variable `GENC3D`.
fn build_system() -> String {
    std::env::var("GENC3D").expect("GENC3D names the service executable genc3d")
}

/// Yields the *front end* of the test: names the *build system* and the *network file*,
/// provides its text, so that a *build* runs and the network is told valid, its *node* being
/// invalid without its *meta compiler DSL*; opens the *node* and cancels the *input creation
/// confirmation dialog*, which has to name the *meta compiler DSL* and the *input* and open
/// nothing; opens it again and confirms, which has to create the files and present the
/// *node window*, its *meta compiler DSL* group showing the template and the *node* told valid
/// or active; exits with 0 where all held, with a code naming the first failure otherwise, and
/// with 8 where the network does not come to be valid within a minute.
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
    property int opened: 0
    property string metaDsls: ""
    property string inputs: ""
    property int code: -1

    Connections {{
        target: Workbench.network
        function onCreationAsked(metaDsls, inputs) {{
            root.asked += 1;
            root.metaDsls = metaDsls;
            root.inputs = inputs;
            Workbench.network.answerCreation(root.step === 2, false);
        }}
        function onNodeOpened(name) {{
            root.opened += 1;
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
            if (root.step === 0 && mode === 2 && Workbench.network.graphSource.length > 0) {{
                root.step = 1;
                root.ticks = 0;
                Workbench.network.openNodeNamed("a");
            }} else if (root.step === 1 && root.ticks >= 5) {{
                if (root.asked !== 1 || root.metaDsls.indexOf("sub") < 0 || root.inputs.indexOf("a.in") < 0) {{
                    root.code = 1;
                }} else if (root.opened !== 0) {{
                    root.code = 2;
                }}
                root.step = 2;
                root.ticks = 0;
                Workbench.network.openNodeNamed("a");
            }} else if (root.step === 2 && root.ticks >= 25) {{
                var node = Workbench.network.nodeNamed("a");
                var shown = node.metaDsl.text.indexOf("syntax // takes all UTF-8") === 0;
                var valid = node.nodeMode === 2 || node.nodeMode === 3;
                if (root.code < 0) {{
                    if (root.asked !== 2) {{
                        root.code = 3;
                    }} else if (root.opened !== 1) {{
                        root.code = 4;
                    }} else if (!shown) {{
                        console.log("meta DSL: " + node.metaDsl.text);
                        root.code = 5;
                    }} else if (!valid) {{
                        console.log("node mode: " + node.nodeMode);
                        root.code = 6;
                    }} else {{
                        root.code = 0;
                    }}
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
 * covers bridge    : NetworkEditor::open_node_named, open_node_preparing, files_to_create,
 *                    prepared, answer_creation, resume, hold */

// FR-486 to FR-489, IR-210
// needs the service executable genc3d of genc³ 0.36.0.0 or later, named by the environment
// variable GENC3D, and the program dot of Graphviz; a Qt platform, for which it chooses the
// offscreen one; and a working directory of its own, short enough for the paths of the Unix
// domain sockets
#[test]
#[ignore]
fn a_node_without_its_files_gets_them_once_the_dialog_is_confirmed() {
    let folder: PathBuf = std::env::temp_dir().join(format!("g3wb-md-{}", std::process::id()));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).expect("the temporary directory can be written");
    fs::write(folder.join("n.gc3n"), ONE_NODE).expect("the network file is written");
    std::env::set_current_dir(&folder).expect("the temporary directory can be entered");
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
    let created = fs::read_to_string(folder.join("sub").join("a.gc3")).ok();
    let input = fs::read(folder.join("a.in")).ok();
    let _ = fs::remove_dir_all(&folder);
    assert_eq!(
        (exit_code, created.as_deref(), input.as_deref()),
        (0, Some(META_DSL_TEMPLATE), Some(&b""[..]))
    );
}
