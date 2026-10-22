//! The *automation scripts* under 'tests/scripts', each an *integration test* of the *front end*,
//! and the end-user scenarios under 'tests/user_scripts', maintained by the product owner:
//! *product* is started with the switch `--verify` on each script, offscreen, against the
//! `genc3d` of `GENC3D`, and its exit code and its report are asserted.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

/// The number of runs so far, so that the runs of one process, which run in parallel, use
/// temporary directories of their own.
static RUNS: AtomicUsize = AtomicUsize::new(0);

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

/// Runs the script of the folder `name` of 'tests/scripts', or `script` in place of it, as a
/// *verification run*: the folder copied into a temporary directory, the *automation
/// configuration* beside the executable of *product*, and a *settings file* beside it naming
/// the *build system*; yields the exit code, the standard output and the standard error of the
/// run.
fn verification_run(name: &str, script: Option<&str>) -> (i32, String, String) {
    verification_run_in("tests/scripts", name, script)
}

/// Runs the script of the folder `name` of `scripts`, a folder of the repository, as
/// `verification_run` does.
fn verification_run_in(scripts: &str, name: &str, script: Option<&str>) -> (i32, String, String) {
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
    // the settings file beside the executable names the build system; nothing else is kept
    // between the runs (FR-094, IR-011)
    fs::write(
        folder.join("genc3wb.yaml"),
        format!("build_system_path: {}\nautomatic: false\n", build_system()),
    )
    .expect("the settings file is written");
    let run = std::env::temp_dir().join(format!(
        "g3wb-as-{name}-{}-{}",
        std::process::id(),
        RUNS.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = fs::remove_dir_all(&run);
    copy_folder(&manifest.join(scripts).join(name), &run);
    if let Some(script) = script {
        fs::write(run.join("script.gc3s"), script).expect("the script is written");
    }
    let output = Command::new(&executable)
        .arg("--verify")
        .arg(run.join("script.gc3s"))
        .current_dir(&run)
        .env("QT_QPA_PLATFORM", "offscreen")
        .output()
        .expect("the product runs");
    let _ = fs::remove_dir_all(&run);
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
    )
}

/// Yields the lines of the report among the lines of standard output, which carries the
/// renderings of the *text increments* as well (IR-015).
fn report_lines(stdout: &str) -> Vec<&str> {
    stdout
        .lines()
        .filter(|line| {
            line.split_once(": ")
                .is_some_and(|(number, rest)| {
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
 * covers bridge    : Automation::start_verification, Automation::step, Automation::front_end_done,
 *                    Workbench::started, NetworkEditor::name_build_system, InputGroup::name_path_by_script */

// FR-497, FR-500 to FR-506, FR-511 to FR-513, FR-515, FR-517, IR-212 to IR-215
// needs the service executable genc3d of genc³ 0.36.0.0 or later, named by the environment
// variable GENC3D, with gc3 beside it, and the program dot of Graphviz; a Qt platform, for which
// it chooses the offscreen one
#[test]
#[ignore]
fn the_script_of_the_copy_node_verifies_the_output_after_an_edit() {
    let (code, stdout, stderr) = verification_run("copy_node", None);
    let lines = report_lines(&stdout);
    assert_eq!(
        (code, lines.len(), lines.last().copied()),
        (0, 14, Some("18: ok: deactivate")),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

// FR-290, FR-291, FR-294 to FR-296, FR-503, FR-505, FR-506, FR-508, FR-512
// needs what the test above needs
#[test]
#[ignore]
fn the_script_of_the_editor_windows_verifies_the_sink_output_without_a_node_window() {
    let (code, stdout, stderr) = verification_run("editor_windows", None);
    let lines = report_lines(&stdout);
    assert_eq!(
        (code, lines.len(), lines.last().copied()),
        (0, 13, Some("16: ok: close file \"work/out.txt\"")),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

// FR-486 to FR-489, FR-502, FR-510, FR-512
// needs what the test above needs
#[test]
#[ignore]
fn the_script_of_the_missing_files_cancels_and_confirms_the_creation_dialog() {
    let (code, stdout, stderr) = verification_run("missing_files", None);
    let lines = report_lines(&stdout);
    assert_eq!(
        (code, lines.len(), lines.last().copied()),
        (0, 10, Some("14: ok: close node a")),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

// FR-507 to FR-509, FR-512
// needs what the test above needs
#[test]
#[ignore]
fn the_script_of_the_settings_and_modes_plays_every_immediate_action() {
    let (code, stdout, stderr) = verification_run("settings_and_modes", None);
    let lines = report_lines(&stdout);
    assert_eq!(
        (code, lines.len(), lines.last().copied()),
        (0, 29, Some("32: ok: deactivate")),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

// FR-513, FR-514, FR-515, FR-517
// needs what the test above needs
#[test]
#[ignore]
fn a_failed_verification_ends_the_run_with_1_and_a_faulty_script_or_action_with_2() {
    let failing = "automation script\nrestore \"backup\" into \"work\"\nnetwork \"work/n.gc3n\"\n\
        verify network \"expected/out.gc3v\"\nactivate\n";
    let (failed_code, failed_out, _) = verification_run("copy_node", Some(failing));
    let faulty = "automation script\nrestore backup into work\n";
    let (faulty_code, faulty_out, _) = verification_run("copy_node", Some(faulty));
    let unknown = "automation script\nrestore \"backup\" into \"work\"\nnetwork \"work/n.gc3n\"\n\
        idle network\nopen node nobody\n";
    let (unknown_code, unknown_out, _) = verification_run("copy_node", Some(unknown));
    let failed_lines = report_lines(&failed_out);
    let unknown_lines = report_lines(&unknown_out);
    assert_eq!(
        (
            failed_code,
            failed_lines.len(),
            failed_lines.last().map(|line| line.starts_with("4: failed: verify network")),
            faulty_code,
            faulty_out.contains("the script is faulty") && faulty_out.contains("2:"),
            unknown_code,
            unknown_lines.last().map(|line| line.starts_with("5: error: open node nobody"))
        ),
        (1, 3, Some(true), 2, true, 2, Some(true)),
        "failed:\n{failed_out}\nfaulty:\n{faulty_out}\nunknown:\n{unknown_out}"
    );
}

// FR-500, FR-517, FR-519: the scenarios the product owner maintains, each recorded by product
// needs what the test above needs
#[test]
#[ignore]
fn every_user_scenario_plays_back_with_every_action_ok() {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut names: Vec<String> = fs::read_dir(manifest.join("tests/user_scripts"))
        .expect("the folder of the user scenarios exists")
        .filter_map(Result::ok)
        .filter(|entry| entry.path().join("script.gc3s").is_file())
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    names.sort();
    assert!(!names.is_empty(), "the folder holds at least one scenario");
    for name in names {
        let (code, stdout, stderr) = verification_run_in("tests/user_scripts", &name, None);
        let lines = report_lines(&stdout);
        assert_eq!(
            (code, lines.iter().all(|line| line.contains(": ok: "))),
            (0, true),
            "scenario {name}:\nstdout:\n{stdout}\nstderr:\n{stderr}"
        );
    }
}
