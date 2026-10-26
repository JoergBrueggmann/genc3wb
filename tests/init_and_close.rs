//! The test of the *defined beginning*: a *verification run* of the script 'init_and_close'
//! applies a saved *settings file*, recovers the files of a network, names, activates and closes
//! the network, names it again and leaves it by `init`; afterwards the folder of the network
//! holds no *lock file*, the *settings file* beside the executable names no *network file* and
//! is the one the run began with, and the file kept aside is gone.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Yields the path of the build system: the environment variable `GENC3D`.
fn build_system() -> String {
    std::env::var("GENC3D").expect("GENC3D names the service executable genc3d")
}

/// Copies the folder `from` into `to`, recursively.
fn copy_folder(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("the folder can be created");
    for entry in fs::read_dir(from).expect("the folder is read") {
        let entry = entry.expect("the entry is read");
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_folder(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), &target).expect("the file is copied");
        }
    }
}

/// Yields the lines of the report among the lines of standard output.
fn report_lines(stdout: &str) -> Vec<&str> {
    stdout
        .lines()
        .filter(|line| {
            line.split_once(": ").is_some_and(|(number, rest)| {
                number.parse::<usize>().is_ok()
                    && (rest.starts_with("ok: ")
                        || rest.starts_with("failed: ")
                        || rest.starts_with("error: "))
            })
        })
        .collect()
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : NetworkEditor::init, close_network, finish_close, name_none;
 *                    InputGroup::close_network; Workbench::apply_settings_file, restore_settings_file */

// FR-091, FR-221, FR-511, FR-535 to FR-538, IR-214, IR-219
// needs the service executable genc3d of genc³ 0.36.0.0 or later, named by the environment
// variable GENC3D, with gc3 beside it, and the program dot of Graphviz; a Qt platform, for which
// it chooses the offscreen one; it writes the settings file beside the executable
#[test]
#[ignore]
fn init_and_close_leave_no_lock_file_no_network_file_named_and_the_settings_as_before() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let executable = PathBuf::from(env!("CARGO_BIN_EXE_genc3wb"));
    let folder = executable
        .parent()
        .expect("the executable lies in a folder")
        .to_path_buf();
    fs::copy(
        manifest.join("src/automation.gc3"),
        folder.join("automation.gc3"),
    )
    .expect("the automation configuration is copied beside the executable");
    let settings = folder.join("genc3wb.yaml");
    fs::write(
        &settings,
        format!("build_system_path: {}\nautomatic: false\n", build_system()),
    )
    .expect("the settings file is written");
    let run = std::env::temp_dir().join(format!("g3wb-ic-{}", std::process::id()));
    let _ = fs::remove_dir_all(&run);
    copy_folder(&manifest.join("tests/scripts/init_and_close"), &run);
    let output = Command::new(&executable)
        .arg("--verify")
        .arg(run.join("script.gc3s"))
        .current_dir(&run)
        .env("QT_QPA_PLATFORM", "offscreen")
        .output()
        .expect("the product runs");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let locks: Vec<String> = fs::read_dir(run.join("work"))
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .filter(|name| name.ends_with(".lock"))
                .collect()
        })
        .unwrap_or_default();
    // the close of the active network stored the output the node derived (FR-267, FR-535)
    let derived = fs::read_to_string(run.join("work").join("out.txt")).unwrap_or_default();
    let stored = fs::read_to_string(&settings).unwrap_or_default();
    let aside = folder.join("genc3wb.yaml.aside").exists();
    let _ = fs::remove_dir_all(&run);
    let lines = report_lines(&stdout);
    assert_eq!(
        (
            output.status.code(),
            lines.len(),
            lines.iter().all(|line| line.contains(": ok: ")),
            locks,
            derived.as_str(),
            stored.contains("network_path"),
            stored.contains("tab_size: 6"),
            aside,
        ),
        (
            Some(0),
            13,
            true,
            Vec::<String>::new(),
            "hello world",
            false,
            false,
            false
        ),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
}
