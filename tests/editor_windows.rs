//! The test of the *editor windows* serving the *nodes* of their files: the edit of a
//! *source input* in its *editor window* reaches the *node* and the *editor window* of the
//! *sink output* shows what the *node* derives, with no *node window* open.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use genc3wb::bridge::editing::Editing;
use genc3wb::bridge::workbench::Workbench;
use genc3wb::core::network_builder::{NetworkBuilder, StateSinks};
use genc3wb::core::text_increment::increment_between;

use qtbridge::QApp;

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// A *network file* of one *node*, which copies its *source input* to its *sink output*.
const ONE_NODE: &str = "meta compiler-compiler network n.gc3n\n\
    node copy\n  inputs\n    in.txt\n  outputs\n    out.txt\n";

/// The *meta compiler DSL* of the *node*: a syntax matching the letters of 'hello world', and a
/// generator emitting the matched text.
const COPY_META_DSL: &str = "syntax\n    root = chars, EOS\n    chars = ch, { ch }\n    \
    ch = 'h' | 'e' | 'l' | 'o' | 'w' | 'r' | 'd' | ' '\n\
    generator copy input \"in.txt\" output \"out.txt\"\n    root -> flat(#1)\n";

/// Yields the path of the build system: the environment variable `GENC3D`.
fn build_system() -> String {
    std::env::var("GENC3D").expect("GENC3D names the service executable genc3d")
}

/// Yields the sinks of a builder that tells nothing.
fn no_sinks() -> StateSinks {
    Box::new(|| Box::new(|_| {}))
}

/// Yields the centre of the vertex of each file of `wanted` as fractions of the width and the
/// height of the image, from a *build* of the network at `network_path` by a builder of its
/// own, which ends with its *build system* before the *workbench object* starts one.
fn centres_of(directory: &Path, network_path: &str, wanted: &[&str]) -> Vec<(f64, f64)> {
    let text = fs::read_to_string(network_path).expect("the network file is read");
    let mut builder = NetworkBuilder::new(directory.join("s"), no_sinks());
    builder
        .restart(&build_system(), network_path)
        .expect("the build system starts");
    let graph = builder
        .build("", &increment_between("", &text), &text)
        .graph
        .expect("the network is built");
    let centres = wanted
        .iter()
        .map(|identifier| {
            let index = graph
                .files
                .iter()
                .position(|file| file.identifier == *identifier)
                .expect("the file is one of the network");
            let vertex = graph
                .layout
                .vertices
                .iter()
                .find(|vertex| vertex.identifier == format!("f{index}"))
                .expect("the file has a vertex");
            (
                (vertex.left + vertex.width / 2.0) / graph.layout.width,
                (vertex.top + vertex.height / 2.0) / graph.layout.height,
            )
        })
        .collect();
    drop(builder);
    std::thread::sleep(Duration::from_millis(500));
    centres
}

/// Yields the *front end* of the test: names the *build system* and the *network file*,
/// provides the text of the *network file*, so that a *build* runs, activates the network once
/// it is valid, opens the *editor windows* of 'in.txt' and 'out.txt' by the double-click on
/// their vertices once it is active, appends `ddd` in the first and lets its idle expire, and
/// exits three seconds later with 0 where the second shows the appended text, with 1 where it
/// does not, and with 8 where the network does not come to be active within a minute.
fn front_end(network: &str, input: &str, output: &str, centres: &[(f64, f64)]) -> Vec<u8> {
    let (input_x, input_y) = centres[0];
    let (output_x, output_y) = centres[1];
    format!(
        r#"
import QtQuick
import QtQuick.Controls
import genc3wb

ApplicationWindow {{
    id: root

    property int step: 0
    property int ticks: 0
    property var input: null
    property var output: null

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
            if (root.step === 0 && mode === 2) {{
                Workbench.network.activate();
                root.step = 1;
            }} else if (root.step === 1 && mode === 3 && Workbench.network.graphSource.length > 0) {{
                root.step = 2;
                root.ticks = 0;
            }} else if (root.step === 2 && root.ticks >= 5) {{
                Workbench.network.openNodeAt({input_x}, {input_y});
                Workbench.network.openNodeAt({output_x}, {output_y});
                root.step = 3;
                root.ticks = 0;
            }} else if (root.step === 3 && root.ticks >= 10) {{
                root.input = Workbench.network.fileNamed("{input}");
                root.output = Workbench.network.fileNamed("{output}");
                if (root.input.path !== "{input}" || root.output.path !== "{output}") {{
                    Qt.exit(2);
                }}
                root.input.text = root.input.text + "ddd";
                root.input.idleExpired();
                root.step = 4;
                root.ticks = 0;
            }} else if (root.step === 4 && root.ticks >= 15) {{
                var shown = root.output.text.indexOf("hello worldddd") === 0;
                Workbench.network.shutDown();
                Workbench.releaseLocks();
                Qt.exit(shown ? 0 : 1);
            }}
        }}
    }}

    Timer {{
        interval: 60000
        running: true
        onTriggered: {{
            Workbench.network.shutDown();
            Workbench.releaseLocks();
            Qt.exit(8);
        }}
    }}

    Component.onCompleted: {{
        Workbench.network.tryBuildSystemPath("{build_system}");
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
 * covers bridge    : NetworkEditor::open_file, hold_nodes_of_file, hold, present_node,
 *                    NetworkEditor::output_served, NodeEditor::take_content,
 *                    InputGroup::idle_expired, DocumentRegistry::provide_peers */

// FR-290, FR-291, FR-294 to FR-296
// needs the service executable genc3d of genc³ 0.32.1.0 or later, named by the environment
// variable GENC3D, and the program dot of Graphviz; a Qt platform, for which it chooses the
// offscreen one; and a working directory of its own, since the *workbench object* takes the lock
// file of the settings file beside it, short enough for the paths of the Unix domain sockets
#[test]
#[ignore]
fn edit_of_a_source_input_reaches_the_editor_window_of_the_sink_output_without_a_node_window() {
    let folder: PathBuf = std::env::temp_dir().join(format!("g3wb-ew-{}", std::process::id()));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).expect("the temporary directory can be written");
    fs::write(folder.join("n.gc3n"), ONE_NODE).expect("the network file is written");
    fs::write(folder.join("copy.gc3"), COPY_META_DSL).expect("the meta compiler DSL is written");
    fs::write(folder.join("in.txt"), "hello world").expect("the input is written");
    fs::write(folder.join("out.txt"), "hello world").expect("the output is written");
    std::env::set_current_dir(&folder).expect("the temporary directory can be entered");
    // SAFETY: the test is the only thread of its process at this point, so that no other thread
    // reads the environment while it is written.
    unsafe { std::env::set_var("QT_QPA_PLATFORM", "offscreen") };
    let path_of = |name: &str| folder.join(name).to_string_lossy().into_owned();
    let (network, input, output) = (path_of("n.gc3n"), path_of("in.txt"), path_of("out.txt"));
    let centres = centres_of(&folder, &network, &["in.txt", "out.txt"]);
    let exit_code = QApp::new()
        .application_name("genc3wb-test")
        .register::<Workbench>()
        .register::<Editing>()
        .load_qml(&front_end(&network, &input, &output, &centres))
        .run();
    std::thread::sleep(Duration::from_millis(500));
    let stored = fs::read_to_string(folder.join("out.txt")).unwrap_or_default();
    let _ = fs::remove_dir_all(&folder);
    // the editor window shows what the node derived, and the store request wrote it (FR-109)
    assert_eq!((exit_code, stored.as_str()), (0, "hello worldddd"));
}
