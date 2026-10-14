//! The test of the *bridged types* the network editor and the *node editor* hand to the *front end*
//! by their slots, across a garbage collection of the QML engine.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use genc3wb::bridge::editing::Editing;
use genc3wb::bridge::workbench::Workbench;

use qtbridge::QApp;

use std::fs;

/// The *front end* of the test: takes the *node editor* `nodeNamed` yields, its *input groups*
/// by `inputAt` and `metaDsl`, and the *input group* of an *editor window* by `fileNamed`, drops
/// them, runs the garbage collector of the engine, and takes them again one turn of the event
/// loop later; exits with 0 where every one is still there, and with the number of the first
/// one lost otherwise.
const FRONT_END: &[u8] = br#"
import QtQuick
import QtQuick.Controls
import genc3wb

ApplicationWindow {
    id: root

    function takeAndDrop() {
        var editor = Workbench.network.nodeNamed("x");
        var group = editor.inputAt(0);
        var metaDsl = editor.metaDsl;
        var output = editor.output;
        var file = Workbench.network.fileNamed("/none");
        if (!editor || !group || !metaDsl || !output || !file) {
            Qt.exit(9);
        }
    }

    function lost() {
        var editor = Workbench.network.nodeNamed("x");
        if (!editor) {
            return 1;
        }
        if (!editor.inputAt(0)) {
            return 2;
        }
        if (!editor.metaDsl) {
            return 3;
        }
        if (!editor.output) {
            return 4;
        }
        if (!Workbench.network.fileNamed("/none")) {
            return 5;
        }
        return 0;
    }

    Timer {
        id: afterCollection

        interval: 200
        onTriggered: Qt.exit(root.lost())
    }

    Timer {
        id: guard

        interval: 10000
        running: true
        onTriggered: Qt.exit(8)
    }

    Component.onCompleted: {
        root.takeAndDrop();
        gc();
        afterCollection.start();
    }
}
"#;

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : NetworkEditor::node_named, NetworkEditor::file_named, NodeEditor::input_at,
 *                    NodeEditor::meta_dsl, NodeEditor::output */

// The case needs a Qt platform, for which it chooses the offscreen one, and a working directory
// of its own, since the *workbench object* takes the lock file of the settings file beside it.

#[test]
#[ignore]
fn bridged_types_handed_by_slots_survive_a_garbage_collection() {
    // FR-001, FR-085, FR-087, FR-290, FR-291: the engine deleted what a slot handed over
    let folder =
        std::env::temp_dir().join(format!("genc3wb-network-editor-{}", std::process::id()));
    fs::create_dir_all(&folder).expect("the temporary directory can be written");
    std::env::set_current_dir(&folder).expect("the temporary directory can be entered");
    // SAFETY: the test is the only thread of its process at this point, so that no other thread
    // reads the environment while it is written.
    unsafe { std::env::set_var("QT_QPA_PLATFORM", "offscreen") };
    let exit_code = QApp::new()
        .application_name("genc3wb-test")
        .register::<Workbench>()
        .register::<Editing>()
        .load_qml(FRONT_END)
        .run();
    let _ = fs::remove_dir_all(&folder);
    assert_eq!(exit_code, 0);
}
