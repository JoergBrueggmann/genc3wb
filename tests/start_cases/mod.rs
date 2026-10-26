//! What the tests of a start share whose outcomes may arrive in either order: a network of one
//! *node* written complete, without its files, or with an invalid *meta compiler DSL*, and the
//! test *front end* that watches the network editor for a while, expires the idle at a tick of
//! its choice, and exits with 0 where the presentation at the end is the one expected.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
#![allow(dead_code)]

use std::fs;
use std::path::{Path, PathBuf};

/// A *network file* of one *node*, which copies its *source input* to its *sink output*.
pub const ONE_NODE: &str = "meta compiler-compiler network\n\
    \tnode copy\n\t\tinputs\n\t\t\tin.txt\n\t\toutputs\n\t\t\tout.txt\n";

/// The *meta compiler DSL* of the *node*, valid.
pub const COPY_META_DSL: &str = "syntax\n    Root = Chars, EOS\n    Chars = Ch, { Ch }\n    \
    Ch = 'h' | 'e' | 'l' | 'o' | 'w' | 'r' | 'd' | ' '\n\
    generator\n    Root -> flat(#0)\n";

/// A *meta compiler DSL* the *node* refuses: a rule without its right-hand side.
pub const BROKEN_META_DSL: &str = "syntax\n    Root =\n";

/// Yields the path of the build system: the environment variable `GENC3D`.
pub fn build_system() -> String {
    std::env::var("GENC3D").expect("GENC3D names the service executable genc3d")
}

/// Yields the root of the case, emptied: a folder of its own under the temporary directory,
/// short enough for the paths of the Unix domain sockets.
pub fn case_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("g3wb-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    root
}

/// Writes the *network file* alone into `work`; yields its path.
pub fn write_network_file_alone(work: &Path) -> PathBuf {
    fs::create_dir_all(work).expect("the work folder can be written");
    fs::write(work.join("n.gc3n"), ONE_NODE).expect("the network file is written");
    work.join("n.gc3n")
}

/// Writes the network complete, with the *meta compiler DSL* `dsl`; yields the path of the
/// *network file*.
pub fn write_network_with_dsl(work: &Path, dsl: &str) -> PathBuf {
    let network = write_network_file_alone(work);
    fs::write(work.join("copy.gc3"), dsl).expect("the meta compiler DSL is written");
    fs::write(work.join("in.txt"), "hello world").expect("the input is written");
    fs::write(work.join("out.txt"), "hello world").expect("the output is written");
    network
}

/// Yields the names of the files of `work`, sorted.
pub fn files_of(work: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(work)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
    names
}

/// What the *front end* expects of the network editor at the end of the watch.
pub enum Expected {
    /// a *valid network* with its graph, no *error message*, no failed own try, and no
    /// *activate request* ever pending: the own try was not made
    ValidWithoutTry,
    /// a *valid network* with its graph, no *error message*, and the failed own try as the
    /// *request failure* in the status bar
    ValidWithFailedTry,
}

/// Yields the *front end* of a case: names the *build system* and the *network file*, expires
/// the idle at the tick `idle_at` (200 ms per tick), logs the network editor every tick,
/// watches until the tick `until`, and exits with 0 where the presentation is `expected`, with
/// 1 where the mode, the graph or the messages differ, with 2 where an *activate request* was
/// pending at some tick although none was expected, and with 8 where the graph is not
/// presented at the end.
pub fn front_end(network: &Path, idle_at: u32, until: u32, expected: &Expected) -> Vec<u8> {
    let (try_allowed, failure_expected) = match expected {
        Expected::ValidWithoutTry => ("false", "false"),
        Expected::ValidWithFailedTry => ("true", "true"),
    };
    format!(
        r#"
import QtQuick
import QtQuick.Controls
import genc3wb

ApplicationWindow {{
    id: root

    property int ticks: 0
    property bool tried: false

    function leave(code) {{
        console.log("mode " + Workbench.network.networkMode + ", graph " + (Workbench.network.graphSource.length > 0) + ", error '" + Workbench.network.errorMessage + "', request failure '" + Workbench.network.requestFailure + "', tried " + root.tried + ", status '" + networkModeText(Workbench.network.networkMode) + "'");
        Workbench.network.shutDown();
        Workbench.releaseLock();
        Qt.exit(code);
    }}

    // the phrase of the status bar as Main.qml composes it (FR-258, FR-269, FR-541)
    function networkModeText(mode: int): string {{
        if (Workbench.network.activating) {{
            return "Activating network";
        }}
        if (Workbench.network.requestFailure.length > 0) {{
            return "Request failed: " + Workbench.network.requestFailure;
        }}
        return mode === 2 ? "Network valid, not active" : "" + mode;
    }}

    Timer {{
        interval: 200
        running: true
        repeat: true
        onTriggered: {{
            root.ticks += 1;
            if (Workbench.network.activating) {{
                root.tried = true;
            }}
            if (root.ticks === {idle_at}) {{
                Workbench.networkInput.idleExpired();
            }}
            if (root.ticks >= {until}) {{
                var graph = Workbench.network.graphSource.length > 0;
                if (!graph) {{
                    root.leave(8);
                }}
                if (root.tried && !{try_allowed}) {{
                    root.leave(2);
                }}
                var failed = Workbench.network.requestFailure.length > 0;
                var ok = Workbench.network.networkMode === 2
                    && Workbench.network.errorMessage.length === 0
                    && failed === {failure_expected}
                    && root.tried === {try_allowed};
                root.leave(ok ? 0 : 1);
            }}
        }}
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
