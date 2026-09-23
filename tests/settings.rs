//! The integration tests of the component `core::settings`, through the public interface.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use genc3wb::core::settings::{DEFAULT_BUILD_SYSTEM_PATH, SETTINGS_FILE_NAME, Settings};

use std::fs;

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : Workbench::default, InputGroup::set_path, NetworkEditor::set_build_system_path */

#[test]
fn the_paths_of_the_compiler_network_editor_are_restored_and_no_path_of_the_node_window() {
    // FR-090, FR-091, FR-093, FR-094, IR-011, IR-012: the settings file holds the two times, the
    // automatic setting, the network file and the build system, and nothing else
    let dir = std::env::temp_dir().join(format!("genc3wb-settings-session-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let file = dir.join(SETTINGS_FILE_NAME);
    fs::create_dir_all(&dir).expect("the directory can be created");

    let mut session = Settings::default();
    session.set_network_path("/tmp/n.gc3n");
    session
        .set_times(4200, 9000)
        .expect("the times satisfy the constraints");
    session.set_automatic(false);
    session.save(&file).expect("the settings can be stored");
    let content = fs::read_to_string(&file).expect("the settings file was written");
    let restored = Settings::load(&file).expect("the settings can be restored");
    let _ = fs::remove_dir_all(&dir);
    assert_eq!(
        (
            content.lines().count(),
            restored.network_path(),
            restored.build_system_path(),
            restored.idle_time(),
            restored.long_idle_time(),
            restored.automatic()
        ),
        (
            5,
            "/tmp/n.gc3n",
            DEFAULT_BUILD_SYSTEM_PATH,
            4200,
            9000,
            false
        )
    );
}
