//! The test of the network loaded, activated by the *product* itself and closed by the close
//! button: the *workbench object* is driven offscreen, a *network file* named and its text
//! provided, the network has to come to be an *active network* without a tap on `act`, and
//! after `closeNetwork` no *network file* is named, no mode is kept, the *lock file* is gone and
//! the *settings file* names no *network file*; the settings file beside the executable is
//! restored.
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
    node copy\n  inputs\n    in.txt\n  outputs\n    out.txt\n";

/// The *meta compiler DSL* of the *node* `copy`.
const COPY_META_DSL: &str = "syntax\n    Root = Chars, EOS\n    Chars = Ch, { Ch }\n    \
    Ch = 'h' | 'e' | 'l' | 'o' | 'w' | 'r' | 'd' | ' '\ngenerator\n    Root -> flat(#0)\n";

/// Yields the path of the build system: the environment variable `GENC3D`.
fn build_system() -> String {
    std::env::var("GENC3D").expect("GENC3D names the service executable genc3d")
}

/// Yields the *front end* of the test: names the *build system* and the *network file*, provides
/// its text, waits for the network to be an *active network* without activating it, closes the
/// network by `closeNetwork`, and exits with 0 once no *network file* is named and no mode is
/// kept, with 1 where the path is still named a second after the close, and with 8 where the
/// network does not come to be active within a minute.
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
        console.log("mode " + Workbench.network.networkMode + ", path '" + Workbench.networkInput.path + "' at step " + root.step);
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
                }} else if (mode === 3) {{
                    root.step = 1;
                    root.ticks = 0;
                    Workbench.network.closeNetwork();
                }}
            }} else if (root.step === 1) {{
                if (Workbench.networkInput.path === "" && mode === 0) {{
                    root.leave(0);
                }} else if (root.ticks >= 25) {{
                    root.leave(1);
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
 * covers bridge    : NetworkEditor::report_arrived (FR-534), activate_directly, close_network,
 *                    finish_close, name_none; InputGroup::close_network */

// FR-067, FR-091, FR-221, FR-534, FR-535
// needs the service executable genc3d of genc³ 0.36.0.0 or later, named by the environment
// variable GENC3D, and the program dot of Graphviz; a Qt platform, for which it chooses the
// offscreen one; and a working directory of its own, short enough for the paths of the Unix
// domain sockets; it writes the settings file beside the executable, which it restores
#[test]
#[ignore]
fn a_loaded_network_is_activated_by_the_product_and_closed_by_the_close_button() {
    let folder: PathBuf = std::env::temp_dir().join(format!("g3wb-cn-{}", std::process::id()));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).expect("the temporary directory can be written");
    fs::write(folder.join("n.gc3n"), ONE_NODE).expect("the network file is written");
    fs::write(folder.join("copy.gc3"), COPY_META_DSL).expect("the meta DSL is written");
    fs::write(folder.join("in.txt"), "hello").expect("the input is written");
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
    let stored =
        Settings::load(&settings_path).map(|settings| settings.network_path().map(str::to_owned));
    match settings_before {
        Some(octets) => fs::write(&settings_path, octets).expect("the settings are restored"),
        None => {
            let _ = fs::remove_file(&settings_path);
        }
    }
    let lock = folder.join("n.gc3n.lock").exists();
    let _ = fs::remove_dir_all(&folder);
    assert_eq!((exit_code, stored, lock), (0, Ok(None), false));
}
