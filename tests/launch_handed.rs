//! The integration test of the launch of *product* with the *handed file* of the command line,
//! through the *workbench object*; the file-open event is tested in 'launch.rs', since each case
//! needs a Qt application of its own.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use genc3wb::bridge::editing::Editing;
use genc3wb::bridge::workbench::{self, Workbench};
use genc3wb::core::settings::{SETTINGS_FILE_NAME, Settings};

use qtbridge::QApp;

use std::fs;

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : Workbench::default, Workbench::started, workbench::hand_file,
 *                    InputGroup::load_initial */

// The front end of the case: names the workbench, which takes the handed file, tells it that the
// front end is up, shuts the build system down, and exits with 0 where the network file is the
// handed file, with 1 otherwise.
fn front_end(opened: &str) -> Vec<u8> {
    format!(
        r#"
    import QtQuick
    import genc3wb
    Item {{
        readonly property string opened: "{opened}"
        Component.onCompleted: Workbench.started()
        Timer {{
            interval: 50
            running: true
            onTriggered: {{
                const named = Workbench.networkInput.path === opened;
                // ends the build system started on the network file, so that none is left
                Workbench.network.shutDown();
                Workbench.releaseLock();
                if (!named) {{
                    console.log("path: " + Workbench.networkInput.path);
                    Qt.exit(1);
                }}
                Qt.exit(0);
            }}
        }}
    }}
"#
    )
    .into_bytes()
}

// The case needs a Qt platform, for which it chooses the offscreen one, and a working directory
// of its own, which the naming of a network file changes to the folder of that file.

#[test]
#[ignore]
fn the_handed_file_of_the_command_line_is_the_network_file_and_is_not_stored() {
    // FR-476, FR-478, IR-206: the file of the command line is the network file of the session, its
    // folder the working directory, its path not stored
    let folder = std::env::temp_dir().join(format!("genc3wb-launch-handed-{}", std::process::id()));
    let _ = fs::remove_dir_all(&folder);
    fs::create_dir_all(folder.join("second")).expect("the temporary directory can be written");
    let opened = folder.join("second").join("b.gc3n");
    fs::write(&opened, "meta compiler-compiler network\n").expect("the file is written");
    let settings_path = Settings::default_path();
    let stored_before = Settings::load(&settings_path)
        .map(|settings| settings.network_path().map(str::to_owned))
        .unwrap_or_default();
    // SAFETY: the test is the only thread of its process at this point, so that no other thread
    // reads the environment while it is written.
    unsafe { std::env::set_var("QT_QPA_PLATFORM", "offscreen") };
    workbench::hand_file(opened.to_string_lossy().into_owned());
    let exit_code = QApp::new()
        .application_name("genc3wb-test")
        .on_file_open(workbench::file_opened)
        .register::<Workbench>()
        .register::<Editing>()
        .load_qml(&front_end(&opened.to_string_lossy()))
        .run();
    let entered = std::env::current_dir()
        .ok()
        .and_then(|dir| dir.canonicalize().ok())
        == folder.join("second").canonicalize().ok();
    let stored_after = Settings::load(&settings_path)
        .map(|settings| settings.network_path().map(str::to_owned))
        .unwrap_or_default();
    let _ = fs::remove_dir_all(&folder);
    assert_eq!(
        (
            exit_code,
            stored_after == stored_before,
            entered,
            settings_path.file_name().and_then(|name| name.to_str())
        ),
        (0, true, true, Some(SETTINGS_FILE_NAME))
    );
}
