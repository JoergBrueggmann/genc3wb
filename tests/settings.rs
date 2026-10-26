//! The integration tests of the component `core::settings`, through the public interface.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use genc3wb::core::settings::{DEFAULT_NODE_UPPER_HEIGHT, SETTINGS_FILE_NAME, Settings};

use std::fs;

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : Workbench::default, Workbench::try_set_settings,
 *                    Workbench::set_node_left_width, Workbench::set_network_left_width,
 *                    InputGroup::set_path,
 *                    NetworkEditor::set_build_system_path */

#[test]
fn the_paths_of_the_compiler_network_editor_are_restored_and_no_path_of_the_node_window() {
    // FR-090, FR-091, FR-093, FR-094, FR-140, FR-149, FR-200, FR-490, FR-492, FR-518, FR-544,
    // IR-011, IR-012: the settings file holds the two times, the automatic setting, the tab
    // size, the spaces setting, the silent creation setting, the playback delay in seconds while
    // one is waited, the playback delay dialog setting, the positions of the three splitters, the network file, the
    // build system and the insert key, and nothing else
    let dir = std::env::temp_dir().join(format!("genc3wb-settings-session-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    let file = dir.join(SETTINGS_FILE_NAME);
    fs::create_dir_all(&dir).expect("the directory can be created");

    let mut session = Settings::default();
    session.set_network_path(Some("/tmp/n.gc3n"));
    session
        .set_times(4200, 9000)
        .expect("the times satisfy the constraints");
    session.set_automatic(false);
    session
        .set_tab_size(2)
        .expect("the tab size satisfies the constraint");
    session.set_tabs_to_spaces(true);
    session.set_silent_creation(true);
    session
        .set_playback_delay(true, 5)
        .expect("the index names a value");
    session.set_playback_delay_dialog_disabled(true);
    session.set_node_left_width(640);
    session.set_network_left_width(300);
    session.save(&file).expect("the settings can be stored");
    let content = fs::read_to_string(&file).expect("the settings file was written");
    let restored = Settings::load(&file).expect("the settings can be restored");
    let _ = fs::remove_dir_all(&dir);
    assert_eq!(content.lines().count(), 14, "one line per key");
    assert!(
        content.contains("playback_delay: 0.283\n"),
        "the delay is written in seconds"
    );
    assert_eq!(
        (
            restored.network_path(),
            restored.build_system_path(),
            restored.idle_time(),
            restored.long_idle_time(),
            restored.automatic(),
            restored.tab_size()
        ),
        (
            Some("/tmp/n.gc3n"),
            Settings::default_build_system_path().as_str(),
            4200,
            9000,
            false,
            2
        )
    );
    assert_eq!(
        (
            restored.tabs_to_spaces(),
            restored.silent_creation(),
            restored.playback_delay(),
            restored.playback_delay_index(),
            restored.playback_delay_dialog_disabled(),
            restored.node_left_width(),
            restored.node_upper_height(),
            restored.network_left_width()
        ),
        (
            true,
            true,
            283,
            5,
            true,
            640,
            DEFAULT_NODE_UPPER_HEIGHT,
            300
        )
    );
}
