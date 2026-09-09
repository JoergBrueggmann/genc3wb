//! The build script: gives the binaries the run path of the Qt libraries, found through qmake.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use std::process::Command;

/// Emits the run path of the Qt libraries for every binary of the package, so that the
/// application and the test executables load the Qt they were built against without
/// `DYLD_FRAMEWORK_PATH` or `LD_LIBRARY_PATH` being set.
///
/// * The Qt is the one whose `qmake` is named by `QMAKE`, or found on the path, as the crate
///   `qtbridge` finds it.
/// * Windows has no run path; there the Qt `bin` directory is on the path.
fn main() {
    println!("cargo:rerun-if-env-changed=QMAKE");
    println!("cargo:rerun-if-changed=build.rs");
    if cfg!(windows) {
        return;
    }
    let qmake = std::env::var("QMAKE").unwrap_or_else(|_| "qmake".to_owned());
    let Ok(output) = Command::new(&qmake)
        .arg("-query")
        .arg("QT_INSTALL_LIBS")
        .output()
    else {
        return;
    };
    let libs = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    if libs.is_empty() {
        return;
    }
    println!("cargo:rustc-link-arg=-Wl,-rpath,{libs}");
}
