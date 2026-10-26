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

/// The names a script of a folder may carry, tried in this order: the one the engineer writes,
/// and the one the *automatic recording* writes (FR-520), which a folder the product owner
/// copies from the *recording folder* holds.
const SCRIPT_NAMES: [&str; 2] = ["script.gc3s", "usage.gc3s"];

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
    verification_run_configured(scripts, name, script, "")
}

/// Runs the script as `verification_run_in` does, with the lines `settings` appended to the
/// settings file written beside the executable, so that a run takes a setting of its own, the
/// *playback delay* for example. A run with settings of its own runs a copy of the executable in
/// a folder of its own, since the runs of one process run in parallel and each writes the
/// settings file beside the executable it starts, so that the file beside the shared executable
/// may hold another run's lines when this run reads it.
fn verification_run_configured(
    scripts: &str,
    name: &str,
    script: Option<&str>,
    settings: &str,
) -> (i32, String, String) {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let shared = PathBuf::from(env!("CARGO_BIN_EXE_genc3wb"));
    let own_folder = (!settings.is_empty()).then(|| {
        std::env::temp_dir().join(format!(
            "g3wb-bin-{name}-{}-{}",
            std::process::id(),
            RUNS.fetch_add(1, Ordering::SeqCst)
        ))
    });
    let executable = match &own_folder {
        Some(folder) => {
            let _ = fs::remove_dir_all(folder);
            fs::create_dir_all(folder).expect("the folder of the copy can be created");
            let copy = folder.join("genc3wb");
            fs::copy(&shared, &copy).expect("the executable is copied");
            copy
        }
        None => shared,
    };
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
        format!(
            "build_system_path: {}\nautomatic: false\n{settings}",
            build_system()
        ),
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
    // the script of the engineer is 'script.gc3s', a recording of the product owner 'usage.gc3s'
    // as the automatic recording names it (FR-520)
    let script_file = SCRIPT_NAMES
        .iter()
        .map(|name| run.join(name))
        .find(|path| path.is_file())
        .expect("the folder holds a script");
    let output = Command::new(&executable)
        .arg("--verify")
        .arg(&script_file)
        .current_dir(&run)
        .env("QT_QPA_PLATFORM", "offscreen")
        .output()
        .expect("the product runs");
    let _ = fs::remove_dir_all(&run);
    if let Some(folder) = own_folder {
        let _ = fs::remove_dir_all(folder);
    }
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
        (0, 18, Some("22: ok: deactivate")),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

// FR-067, FR-070, FR-501, FR-502, FR-512
// needs what the test above needs
#[test]
#[ignore]
fn the_script_of_a_second_network_verifies_the_output_of_its_node_after_an_edit() {
    let (code, stdout, stderr) = verification_run("second_network", None);
    let lines = report_lines(&stdout);
    assert_eq!(
        (code, lines.len(), lines.last().copied()),
        (0, 18, Some("23: ok: deactivate")),
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
        (0, 17, Some("20: ok: close file \"work/out.txt\"")),
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
        (0, 11, Some("15: ok: close node a")),
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
        (0, 33, Some("36: ok: deactivate")),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

// FR-534, FR-539 and the software problem report of 2026-10-25, whose scenario the script states
// needs what the test above needs
#[test]
#[ignore]
fn the_script_of_the_node_keyword_breaks_and_mends_the_network_file() {
    let (code, stdout, stderr) = verification_run("node_keyword", None);
    let lines = report_lines(&stdout);
    assert_eq!(
        (code, lines.len(), lines.last().copied()),
        (0, 17, Some("23: ok: verify network \"expected/n.gc3v\"")),
        "stdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

// FR-513, FR-514, FR-515, FR-517
// needs what the test above needs
#[test]
#[ignore]
fn a_failed_verification_ends_the_run_with_1_and_a_faulty_script_or_action_with_2() {
    const RESTORES: &str = "restore \"backup/n.gc3n\" into \"work/n.gc3n\"\n\
        restore \"backup/copy.gc3\" into \"work/copy.gc3\"\nrestore \"backup/in.txt\" into \"work/in.txt\"\n";
    let failing = format!(
        "automation script\n{RESTORES}network \"work/n.gc3n\"\nverify network \"expected/out.gc3v\"\nactivate\n"
    );
    let (failed_code, failed_out, _) = verification_run("copy_node", Some(&failing));
    let faulty = "automation script\nrestore backup into work\n";
    let (faulty_code, faulty_out, _) = verification_run("copy_node", Some(faulty));
    let unknown = format!(
        "automation script\n{RESTORES}network \"work/n.gc3n\"\nidle network\nopen node nobody\n"
    );
    let (unknown_code, unknown_out, _) = verification_run("copy_node", Some(&unknown));
    let failed_lines = report_lines(&failed_out);
    let unknown_lines = report_lines(&unknown_out);
    assert_eq!(
        (
            failed_code,
            failed_lines.len(),
            failed_lines
                .last()
                .map(|line| line.starts_with("6: failed: verify network")),
            faulty_code,
            faulty_out.contains("the script is faulty") && faulty_out.contains("2:"),
            unknown_code,
            unknown_lines
                .last()
                .map(|line| line.starts_with("7: error: open node nobody"))
        ),
        (1, 5, Some(true), 2, true, 2, Some(true)),
        "failed:\n{failed_out}\nfaulty:\n{faulty_out}\nunknown:\n{unknown_out}"
    );
}

/// A script of immediate actions in the folder of 'init_and_close', whose saved settings file
/// holds no delay: the delay of the run's settings file is waited before every action but the
/// first, and neither the saved settings file nor the `settings` line changes it.
const DELAYED_SCRIPT: &str = "// The playback delay of the settings file, waited before every action but \
the first and kept\n// through the saved settings file and the settings line.\nautomation script\n\
init\nsettings file \"settings.bak\"\ntyping mode insert\n\
settings idle 2.0 long idle 16.0 automatic off tab size 4 spaces off silent creation off delay off\n\
typing mode overwrite\ntyping mode insert\ntyping mode overwrite\ntyping mode insert\n";

// FR-500, FR-509, FR-537, FR-544: eight actions, seven delays of 0.566 seconds in a verification
// run, which takes the delay of the settings file, so that the run takes at least 3.96 seconds;
// the same script without a delay as the measure of the run itself, which the delays exceed by
// at least 2.5 seconds, the margin allowing for the load of the parallel runs
// needs what the test above needs
#[test]
#[ignore]
fn a_verification_run_waits_the_playback_delay_of_the_settings_file_before_every_action_but_the_first()
 {
    let undelayed_began = std::time::Instant::now();
    let (undelayed_code, undelayed_out, undelayed_err) =
        verification_run_configured("tests/scripts", "init_and_close", Some(DELAYED_SCRIPT), "");
    let undelayed = undelayed_began.elapsed();
    let delayed_began = std::time::Instant::now();
    let (code, stdout, stderr) = verification_run_configured(
        "tests/scripts",
        "init_and_close",
        Some(DELAYED_SCRIPT),
        "playback_delay: 0.57\n",
    );
    let delayed = delayed_began.elapsed();
    let lines = report_lines(&stdout);
    assert_eq!(
        (
            undelayed_code,
            code,
            lines.len(),
            lines.last().copied(),
            delayed >= std::time::Duration::from_millis(3960)
                && delayed >= undelayed + std::time::Duration::from_millis(2500)
        ),
        (0, 0, 8, Some("11: ok: typing mode insert"), true),
        "undelayed {undelayed:?}, delayed {delayed:?}\nundelayed stdout:\n{undelayed_out}\nundelayed stderr:\n{undelayed_err}\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
}

// FR-500, FR-517, FR-519: the scenarios the product owner maintains, each recorded by product
// needs what the test above needs
#[test]
#[ignore]
fn every_user_scenario_plays_back_with_every_action_ok() {
    // the folder is the product owner's: absent or empty, there is no scenario to play
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut names: Vec<String> = fs::read_dir(manifest.join("tests/user_scripts"))
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter(|entry| {
                    SCRIPT_NAMES
                        .iter()
                        .any(|name| entry.path().join(name).is_file())
                })
                .map(|entry| entry.file_name().to_string_lossy().into_owned())
                .collect()
        })
        .unwrap_or_default();
    names.sort();
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
