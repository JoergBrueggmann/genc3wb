//! The tests of the insert key at the code editor of the front end and of the position switch, run by the
//! QML test runner of Qt.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The QML files of the front end of which a code editor and a position switch consist, in
/// 'src/qml'.
const COMPONENTS: [&str; 5] = [
    "CodeEditor.qml",
    "DiagnosticMark.qml",
    "WavyLine.qml",
    "OctetView.qml",
    "PositionSwitch.qml",
];

/// The stand-ins of the bridged types the code editor reaches, in 'tests/qml'.
const STAND_INS: [&str; 2] = ["Editing.qml", "Workbench.qml"];

/// The QML test file of the insert key, in 'tests/qml', with the name of its test case.
const INSERT_KEY: (&str, &str) = ("tst_insert_key.qml", "InsertKey");

/// The QML test file of the position switch, in 'tests/qml', with the name of its test case.
const POSITION_SWITCH: (&str, &str) = ("tst_position_switch.qml", "PositionSwitch");

/// The 'qmldir' of the QML module `genc3wb` the test file imports: the components of the code
/// editor, and the stand-ins as singletons.
const QMLDIR: &str = "module genc3wb
CodeEditor 1.0 CodeEditor.qml
DiagnosticMark 1.0 DiagnosticMark.qml
WavyLine 1.0 WavyLine.qml
OctetView 1.0 OctetView.qml
PositionSwitch 1.0 PositionSwitch.qml
singleton Editing 1.0 Editing.qml
singleton Workbench 1.0 Workbench.qml
";

/// Yields the folder of the executables of the Qt that *product* is built against, as its `qmake`
/// names it: the one named by `QMAKE`, or found on the path, as the build script takes it.
fn qt_binaries() -> PathBuf {
    let qmake = std::env::var("QMAKE").unwrap_or_else(|_| "qmake".to_owned());
    let output = Command::new(&qmake)
        .args(["-query", "QT_INSTALL_BINS"])
        .output()
        .expect("the test needs the qmake of a Qt installation");
    PathBuf::from(String::from_utf8_lossy(&output.stdout).trim())
}

/// Runs the test function `function` of the QML test file `test`, named with its test case,
/// against the QML files of the components in the folder `case` below the temporary directory, and
/// yields the output of the test runner where a test of it failed.
///
/// * The folder holds the module `genc3wb` with the components of 'src/qml' and the stand-ins, and
///   the test file beside it; it is removed at the end.
/// * The test runner runs offscreen, with the style Basic of Qt Quick Controls.
fn failures_of(case: &str, test: (&str, &str), function: &str) -> Option<String> {
    let (test_file, test_case) = test;
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let folder = std::env::temp_dir().join(format!("genc3wb-{case}-{}", std::process::id()));
    let module = folder.join("genc3wb");
    fs::create_dir_all(&module).expect("the temporary directory can be written");
    for file in COMPONENTS {
        fs::copy(manifest.join("src/qml").join(file), module.join(file))
            .expect("the component is a file of 'src/qml'");
    }
    for file in STAND_INS {
        fs::copy(manifest.join("tests/qml").join(file), module.join(file))
            .expect("the stand-in is a file of 'tests/qml'");
    }
    fs::write(module.join("qmldir"), QMLDIR).expect("the temporary directory can be written");
    fs::copy(
        manifest.join("tests/qml").join(test_file),
        folder.join(test_file),
    )
    .expect("the test file is a file of 'tests/qml'");
    let output = Command::new(qt_binaries().join("qmltestrunner"))
        .env("QT_QPA_PLATFORM", "offscreen")
        .env("QT_QUICK_CONTROLS_STYLE", "Basic")
        .arg("-import")
        .arg(&folder)
        .arg("-input")
        .arg(folder.join(test_file))
        .arg(format!("{test_case}::{function}"))
        .output()
        .expect("the Qt installation holds the QML test runner");
    fs::remove_dir_all(&folder).expect("the temporary directory can be removed");
    let text = String::from_utf8_lossy(&output.stdout).into_owned()
        + &String::from_utf8_lossy(&output.stderr);
    let passed = output.status.success()
        && text.contains(&format!("PASS   : qmltestrunner::{test_case}::{function}("));
    (!passed).then_some(text)
}

/*  * validated        : ✅
 * completeness     : ✅ txt, hex and bin; a printable key, a command key stroke, Return and Enter
 *                       as the insert key; the position switch tapped, disabled, and its tooltip
 * independence     : ✅ a temporary directory per case; every QML case prepares its items
 * edge cases       : ✅ Return and Enter, which have key handlers of their own in txt; a tap on
 *                       the highlighted position, which moves the highlight not
 * conforms to doc  : ✅ FR-193, the key handlers of CodeEditor.qml and OctetView.qml; FR-259,
 *                       FR-265, FR-266, FR-272, IR-031, IR-039, PositionSwitch.qml
 * covers bridge    : Workbench::toggled_by_key, by a stand-in that toggles as it does */

// The cases need the QML test runner of the Qt installation whose qmake the build finds.

#[test]
#[ignore]
fn insert_key_is_not_entered_as_text() {
    assert_eq!(
        failures_of(
            "insert-key-text",
            INSERT_KEY,
            "test_insert_key_is_not_entered_as_text"
        ),
        None
    );
}

#[test]
#[ignore]
fn insert_key_edits_no_digit() {
    assert_eq!(
        failures_of(
            "insert-key-digits",
            INSERT_KEY,
            "test_insert_key_edits_no_digit"
        ),
        None
    );
}

#[test]
#[ignore]
fn insert_key_toggles_the_typing_mode() {
    assert_eq!(
        failures_of(
            "insert-key-toggle",
            INSERT_KEY,
            "test_insert_key_toggles_the_typing_mode"
        ),
        None
    );
}

#[test]
#[ignore]
fn insert_key_does_not_pass_on_to_the_items_above() {
    assert_eq!(
        failures_of(
            "insert-key-passing",
            INSERT_KEY,
            "test_insert_key_does_not_pass_on_to_the_items_above"
        ),
        None
    );
}

#[test]
#[ignore]
fn tap_on_a_position_emits_its_index() {
    assert_eq!(
        failures_of(
            "position-switch-tap",
            POSITION_SWITCH,
            "test_tap_on_a_position_emits_its_index"
        ),
        None
    );
}

#[test]
#[ignore]
fn disabled_position_switch_emits_nothing() {
    assert_eq!(
        failures_of(
            "position-switch-disabled",
            POSITION_SWITCH,
            "test_disabled_switch_emits_nothing"
        ),
        None
    );
}

#[test]
#[ignore]
fn highlighted_position_follows_the_owner_not_the_tap() {
    assert_eq!(
        failures_of(
            "position-switch-highlight",
            POSITION_SWITCH,
            "test_highlight_follows_selected_not_the_tap"
        ),
        None
    );
}

#[test]
#[ignore]
fn tooltip_of_a_position_switch_is_the_text_given() {
    assert_eq!(
        failures_of(
            "position-switch-tooltip",
            POSITION_SWITCH,
            "test_tooltip_is_the_text_given"
        ),
        None
    );
}
