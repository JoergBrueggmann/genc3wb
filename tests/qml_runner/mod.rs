//! The runner of the QML tests of the front end: a test function of a QML test file of 'tests/qml',
//! run by the QML test runner of Qt against the components of 'src/qml' and the stand-ins of the
//! bridged types of 'tests/qml', shared by the integration tests of the front end.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
/// against the QML files `components` of 'src/qml' and the stand-ins `stand_ins` of 'tests/qml',
/// registered by `qmldir` as the module `genc3wb`, in the folder `case` below the temporary
/// directory, and yields the output of the test runner where a test of it failed.
///
/// * The folder holds the module `genc3wb` with the components and the stand-ins, and the test
///   file beside it; it is removed at the end.
/// * The test runner runs offscreen, with the style Basic of Qt Quick Controls.
pub fn failures_of(
    case: &str,
    components: &[&str],
    stand_ins: &[&str],
    qmldir: &str,
    test: (&str, &str),
    function: &str,
) -> Option<String> {
    let (test_file, test_case) = test;
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let folder = std::env::temp_dir().join(format!("genc3wb-{case}-{}", std::process::id()));
    let module = folder.join("genc3wb");
    fs::create_dir_all(&module).expect("the temporary directory can be written");
    for file in components {
        fs::copy(manifest.join("src/qml").join(file), module.join(file))
            .expect("the component is a file of 'src/qml'");
    }
    for file in stand_ins {
        fs::copy(manifest.join("tests/qml").join(file), module.join(file))
            .expect("the stand-in is a file of 'tests/qml'");
    }
    fs::write(module.join("qmldir"), qmldir).expect("the temporary directory can be written");
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
