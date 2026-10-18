//! The integration test of the creation of a *network file* from the *network template*,
//! through the *input group* of the *network file*: the file that cannot be written is
//! reported, the file written becomes the *network file*.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use genc3wb::bridge::editing::Editing;
use genc3wb::bridge::workbench::Workbench;
use genc3wb::core::input_file::NETWORK_TEMPLATE;
use genc3wb::core::settings::Settings;

use qtbridge::QApp;

use std::fs;

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : InputGroup::create_file, InputGroup::file_creation_failed,
 *                    InputGroup::name_path */

// The front end of the case: tells the workbench that it is up, asks for a network file in a
// folder that does not exist, which has to fail with a message naming the file, then for one in
// the folder of the case, which has to become the network file; a second later, so that the
// build system the named file starts is up and is ended rather than left behind, shuts the
// build system down, releases the lock, and exits with 0 where both held, with a code naming
// the first failure otherwise.
fn front_end(failing: &str, created: &str) -> Vec<u8> {
    format!(
        r#"
    import QtQuick
    import genc3wb
    Item {{
        readonly property string failing: "{failing}"
        readonly property string created: "{created}"
        property string failure: ""
        Component.onCompleted: Workbench.started()
        Connections {{
            target: Workbench.networkInput
            function onFileCreationFailed(message) {{
                failure = message;
            }}
        }}
        property int code: -1
        Timer {{
            interval: 50
            running: true
            onTriggered: {{
                const before = Workbench.networkInput.path;
                Workbench.networkInput.createFile(failing);
                const reported = failure.indexOf(failing) >= 0;
                const unchanged = Workbench.networkInput.path === before;
                Workbench.networkInput.createFile(created);
                const named = Workbench.networkInput.path === created;
                const shown = Workbench.networkInput.text.indexOf("node A") >= 0;
                if (!reported) {{
                    console.log("failure: " + failure);
                    code = 1;
                }} else if (!unchanged) {{
                    code = 2;
                }} else if (!named) {{
                    console.log("path: " + Workbench.networkInput.path);
                    code = 3;
                }} else if (!shown) {{
                    code = 4;
                }} else {{
                    code = 0;
                }}
                endTimer.start();
            }}
        }}
        Timer {{
            id: endTimer
            interval: 1000
            onTriggered: {{
                Workbench.network.shutDown();
                Workbench.releaseLock();
                Qt.exit(code);
            }}
        }}
    }}
"#
    )
    .into_bytes()
}

// The case needs a Qt platform, for which it chooses the offscreen one; naming the created file
// stores its path in the settings file beside the executable, which the case restores.

#[test]
#[ignore]
fn a_network_file_created_from_the_template_is_written_and_named_and_a_failure_is_reported() {
    // FR-482, FR-484, FR-485, IR-209: the file is written with the template and becomes the
    // network file; a file that cannot be written is reported with its path, nothing named
    let folder = std::env::temp_dir().join(format!("genc3wb-template-{}", std::process::id()));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(&folder).expect("the temporary directory can be written");
    let failing = folder.join("missing").join("a.gc3n");
    let created = folder.join("b.gc3n");
    let settings_path = Settings::default_path();
    let settings_before = fs::read(&settings_path).ok();
    // SAFETY: the test is the only thread of its process at this point, so that no other thread
    // reads the environment while it is written.
    unsafe { std::env::set_var("QT_QPA_PLATFORM", "offscreen") };
    let exit_code = QApp::new()
        .application_name("genc3wb-test")
        .register::<Workbench>()
        .register::<Editing>()
        .load_qml(&front_end(
            &failing.to_string_lossy(),
            &created.to_string_lossy(),
        ))
        .run();
    let content = fs::read(&created).ok();
    let failing_exists = failing.exists();
    match settings_before {
        Some(octets) => fs::write(&settings_path, octets).expect("the settings are restored"),
        None => {
            let _ = fs::remove_file(&settings_path);
        }
    }
    let _ = fs::remove_dir_all(&folder);
    assert_eq!(
        (exit_code, content.as_deref(), failing_exists),
        (0, Some(NETWORK_TEMPLATE.as_bytes()), false)
    );
}
