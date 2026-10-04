//! The tests of the insert key at the code editor of the front end, run by the QML test runner of Qt.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// The QML files of the front end of which a code editor consists, in 'src/qml'.
const COMPONENTS: [&str; 4] = [
    "CodeEditor.qml",
    "DiagnosticMark.qml",
    "WavyLine.qml",
    "OctetView.qml",
];

/// The stand-ins of the bridged types the code editor reaches, in 'tests/qml'.
const STAND_INS: [&str; 2] = ["Editing.qml", "Workbench.qml"];

/// The QML test file, in 'tests/qml'.
const TEST_FILE: &str = "tst_insert_key.qml";

/// The 'qmldir' of the QML module `genc3wb` the test file imports: the components of the code
/// editor, and the stand-ins as singletons.
const QMLDIR: &str = "module genc3wb
CodeEditor 1.0 CodeEditor.qml
DiagnosticMark 1.0 DiagnosticMark.qml
WavyLine 1.0 WavyLine.qml
OctetView 1.0 OctetView.qml
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

/// Runs the test function `function` of the QML test file against the QML files of the code editor
/// in the folder `case` below the temporary directory, and yields the output of the test runner
/// where a test of it failed.
///
/// * The folder holds the module `genc3wb` with the components of 'src/qml' and the stand-ins, and
///   the test file beside it; it is removed at the end.
/// * The test runner runs offscreen, with the style Basic of Qt Quick Controls.
fn failures_of(case: &str, function: &str) -> Option<String> {
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
        manifest.join("tests/qml").join(TEST_FILE),
        folder.join(TEST_FILE),
    )
    .expect("the test file is a file of 'tests/qml'");
    let output = Command::new(qt_binaries().join("qmltestrunner"))
        .env("QT_QPA_PLATFORM", "offscreen")
        .env("QT_QUICK_CONTROLS_STYLE", "Basic")
        .arg("-import")
        .arg(&folder)
        .arg("-input")
        .arg(folder.join(TEST_FILE))
        .arg(format!("InsertKey::{function}"))
        .output()
        .expect("the Qt installation holds the QML test runner");
    fs::remove_dir_all(&folder).expect("the temporary directory can be removed");
    let text = String::from_utf8_lossy(&output.stdout).into_owned()
        + &String::from_utf8_lossy(&output.stderr);
    let passed = output.status.success()
        && text.contains(&format!("PASS   : qmltestrunner::InsertKey::{function}("));
    (!passed).then_some(text)
}

/*  * validated        : ✅
 * completeness     : ✅ txt, hex and bin; a printable key, a command key stroke, Return and Enter
 *                       as the insert key
 * independence     : ✅ a temporary directory per case; every QML case prepares the code editor
 * edge cases       : ✅ Return and Enter, which have key handlers of their own in txt
 * conforms to doc  : ✅ FR-193, the key handlers of CodeEditor.qml and OctetView.qml
 * covers bridge    : Workbench::toggled_by_key, by a stand-in that toggles as it does */

// The cases need the QML test runner of the Qt installation whose qmake the build finds.

#[test]
#[ignore]
fn insert_key_is_not_entered_as_text() {
    assert_eq!(
        failures_of("insert-key-text", "test_insert_key_is_not_entered_as_text"),
        None
    );
}

#[test]
#[ignore]
fn insert_key_edits_no_digit() {
    assert_eq!(
        failures_of("insert-key-digits", "test_insert_key_edits_no_digit"),
        None
    );
}

#[test]
#[ignore]
fn insert_key_toggles_the_typing_mode() {
    assert_eq!(
        failures_of(
            "insert-key-toggle",
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
            "test_insert_key_does_not_pass_on_to_the_items_above"
        ),
        None
    );
}
