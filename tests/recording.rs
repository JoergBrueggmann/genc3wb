//! The integration test of the *recording*: the *workbench object* is driven from a test front
//! end offscreen against the `genc3d` of `GENC3D`, an *individual recording* started, the network
//! backed up, named and activated, a *node* opened, its *input* edited with the idle expired, two
//! *expected files* stored and the *recording* stopped; the script is read back through `gc3`
//! and played back by a *verification run* of the executable of *product* in a second process.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use genc3wb::bridge::editing::Editing;
use genc3wb::bridge::workbench::Workbench;
use genc3wb::core::automation::{Action, EditorItem, EditorName, gc3_beside, read_script};
use genc3wb::core::recording::relative;
use genc3wb::core::settings::Settings;

use qtbridge::QApp;

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

/// A *network file* of one *node*, which copies its *source input* to its *sink output*.
const ONE_NODE: &str = "meta compiler-compiler network\n\
    \tnode copy\n\t\tinputs\n\t\t\tin.txt\n\t\toutputs\n\t\t\tout.txt\n";

/// The *meta compiler DSL* of the *node*: a syntax matching the letters of 'hello world', and a
/// generator emitting the matched text.
const COPY_META_DSL: &str = "syntax\n    Root = Chars, EOS\n    Chars = Ch, { Ch }\n    \
    Ch = 'h' | 'e' | 'l' | 'o' | 'w' | 'r' | 'd' | ' '\n\
    generator\n    Root -> flat(#0)\n";

/// Yields the path of the build system: the environment variable `GENC3D`.
fn build_system() -> String {
    std::env::var("GENC3D").expect("GENC3D names the service executable genc3d")
}

/// Yields the *front end* of the test: a step machine of one timer that provides the text of the
/// *network file*, starts the *recording* into `script` once the network is active, which records
/// the state it finds, deactivates and activates the network, opens
/// the *node* once it is active, focuses, places the cursor, appends `ddd` to its *input* and
/// expires the idle, stores the *input* and the *output* as *expected files*, closes the
/// *node window*, appends a comment to the *network file* and stores it, deactivates, stops the
/// *recording* and exits with 0; with 2 where the
/// *node editor* is not named, and 8 where a step does not come within a minute. A network the *build system* activated at its start is deactivated first,
/// so that the activation is the user's and recorded.
fn front_end(network: &str, script: &str) -> Vec<u8> {
    format!(
        r#"
import QtQuick
import QtQuick.Controls
import genc3wb

ApplicationWindow {{
    id: root

    property int step: 0
    property int ticks: 0
    property var node: null
    property var input: null

    Timer {{
        interval: 200
        running: true
        repeat: true
        onTriggered: {{
            root.ticks += 1;
            var mode = Workbench.network.networkMode;
            if (root.step === 0) {{
                Workbench.networkInput.idleExpired();
                root.step = 1;
            }} else if (root.step === 1 && mode === 3 && Workbench.network.graphSource.length > 0) {{
                // the recording starts on the built network the product activated of its own
                // once the node descriptions and the valid mode were known (FR-534), which it
                // records as the state it finds; the network is deactivated then, so that the
                // activation that follows is the user's
                Workbench.automation.startRecording("{script}");
                Workbench.network.deactivate();
                root.step = 11;
            }} else if (root.step === 11 && mode === 2) {{
                Workbench.network.activate();
                root.step = 2;
            }} else if (root.step === 2 && mode === 3 && Workbench.network.graphSource.length > 0) {{
                Workbench.network.openNodeNamed("copy");
                root.step = 3;
                root.ticks = 0;
            }} else if (root.step === 3 && root.ticks >= 10) {{
                root.node = Workbench.network.nodeNamed("copy");
                if (root.node.name !== "copy") {{
                    Qt.exit(2);
                }}
                root.input = root.node.inputAt(0);
                root.input.recordFocus();
                root.input.recordCursor(11, 11, 11);
                root.input.text = root.input.text + "ddd";
                root.input.idleExpired();
                root.step = 4;
                root.ticks = 0;
            }} else if (root.step === 4 && root.ticks >= 15) {{
                Workbench.automation.storeFocused();
                root.node.output.recordFocus(false);
                Workbench.automation.storeFocused();
                Workbench.network.closeNode("copy");
                // the network file edited as well: at playback it is loaded before the restore
                // puts the backup on disk, so that the restore must load it anew (FR-511)
                Workbench.networkInput.recordFocus();
                Workbench.networkInput.text = Workbench.networkInput.text + "// end\n";
                Workbench.networkInput.idleExpired();
                root.step = 5;
                root.ticks = 0;
            }} else if (root.step === 5 && root.ticks >= 15) {{
                Workbench.automation.storeFocused();
                Workbench.network.deactivate();
                root.step = 6;
                root.ticks = 0;
            }} else if (root.step === 6 && (mode === 2 || root.ticks >= 25)) {{
                Workbench.automation.stopRecording();
                Workbench.network.shutDown();
                Workbench.releaseLock();
                Qt.exit(0);
            }}
        }}
    }}

    Timer {{
        interval: 60000
        running: true
        onTriggered: {{
            Workbench.network.shutDown();
            Workbench.releaseLock();
            Qt.exit(8);
        }}
    }}

    Component.onCompleted: {{
        Workbench.network.buildSystemPath = "{build_system}";
        Workbench.networkInput.path = "{network}";
    }}
}}
"#,
        build_system = build_system()
    )
    .into_bytes()
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

/// Plays the script at `script` by a *verification run* of the executable of *product*, in the
/// folder `folder`, the *automation configuration* beside the executable and a *settings file*
/// beside it naming the *build system*; yields the exit code and the lines of the report.
fn verification_run(script: &Path, folder: &Path) -> (i32, Vec<String>, String) {
    let manifest = Path::new(env!("CARGO_MANIFEST_DIR"));
    let executable = PathBuf::from(env!("CARGO_BIN_EXE_genc3wb"));
    let beside = executable
        .parent()
        .expect("the executable lies in a folder");
    fs::copy(
        manifest.join("src/automation.gc3"),
        beside.join("automation.gc3"),
    )
    .expect("the automation configuration is copied beside the executable");
    fs::write(
        beside.join("genc3wb.yaml"),
        format!("build_system_path: {}\nautomatic: false\n", build_system()),
    )
    .expect("the settings file is written");
    let output = Command::new(&executable)
        .arg("--verify")
        .arg(script)
        .current_dir(folder)
        .env("QT_QPA_PLATFORM", "offscreen")
        .output()
        .expect("the product runs");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let lines = stdout
        .lines()
        .filter(|line| {
            line.split_once(": ").is_some_and(|(number, rest)| {
                number.parse::<usize>().is_ok()
                    && (rest.starts_with("ok: ")
                        || rest.starts_with("failed: ")
                        || rest.starts_with("error: "))
            })
        })
        .map(str::to_owned)
        .collect();
    (
        output.status.code().unwrap_or(-1),
        lines,
        stdout + &String::from_utf8_lossy(&output.stderr),
    )
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : Automation::start_recording, Automation::stop_recording, Automation::back_up,
 *                    Automation::store_focused, InputGroup::record_cursor, InputGroup::record_focus,
 *                    InputGroup::set_text, InputGroup::idle_expired, OutputGroup::record_focus,
 *                    NetworkEditor::activate, deactivate, open_node_preparing, close_node,
 *                    NodeEditor::open_node, Workbench::default */

// FR-519, FR-521 to FR-523, FR-525, FR-526, FR-528 to FR-531, IR-217, IR-218
// needs the service executable genc3d of genc³ 0.36.0.0 or later, named by the environment
// variable GENC3D, with gc3 beside it, and the program dot of Graphviz; a Qt platform, for which
// it chooses the offscreen one; and a working directory of its own, since the *workbench object*
// takes the lock file of the network file, short enough for the paths of the Unix domain sockets
#[test]
#[ignore]
fn a_recorded_session_reads_back_through_gc3_and_plays_back_with_every_action_ok() {
    let root: PathBuf = std::env::temp_dir().join(format!("g3wb-rc-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    let folder = root.join("s");
    let work = folder.join("work");
    fs::create_dir_all(&work).expect("the temporary directory can be written");
    fs::write(work.join("n.gc3n"), ONE_NODE).expect("the network file is written");
    fs::write(work.join("copy.gc3"), COPY_META_DSL).expect("the meta compiler DSL is written");
    fs::write(work.join("in.txt"), "hello world").expect("the input is written");
    fs::write(work.join("out.txt"), "hello world").expect("the output is written");
    std::env::set_current_dir(&work).expect("the temporary directory can be entered");
    // SAFETY: the test is the only thread of its process at this point, so that no other thread
    // reads the environment while it is written.
    unsafe { std::env::set_var("QT_QPA_PLATFORM", "offscreen") };
    let path_of = |path: &Path| path.to_string_lossy().into_owned();
    let script = folder.join("script.gc3s");
    // the settings file beside the test executable, saved by the recording as its first file
    let settings_path = Settings::default_path();
    let settings_before = fs::read(&settings_path).ok();
    fs::write(&settings_path, "tab_size: 4\n").expect("the settings file is written");
    let exit_code = QApp::new()
        .application_name("genc3wb-test")
        .register::<Workbench>()
        .register::<Editing>()
        .load_qml(&front_end(
            &path_of(&work.join("n.gc3n")),
            &path_of(&script),
        ))
        .run();
    std::thread::sleep(Duration::from_millis(500));
    match settings_before {
        Some(octets) => fs::write(&settings_path, octets).expect("the settings are restored"),
        None => {
            let _ = fs::remove_file(&settings_path);
        }
    }
    let text = fs::read_to_string(&script).unwrap_or_default();
    let lines: Vec<&str> = text.lines().collect();
    let read = read_script(
        &script,
        &gc3_beside(&build_system()),
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src/automation.gc3"),
    );
    let actions: Vec<Action> = read
        .as_ref()
        .map(|script| {
            script
                .steps
                .iter()
                .map(|step| step.action.clone())
                .collect()
        })
        .unwrap_or_default();
    let expected_0 = fs::read(folder.join("script-0.gc3v")).unwrap_or_default();
    let expected_1 = fs::read(folder.join("script-1.gc3v")).unwrap_or_default();
    let expected_2 = fs::read(folder.join("script-2.gc3v")).unwrap_or_default();
    let saved_network =
        fs::read_to_string(folder.join("backup").join("2").join("1.bak")).unwrap_or_default();
    let saved_settings =
        fs::read_to_string(folder.join("backup").join("1").join("1.bak")).unwrap_or_default();
    // a copy of the scenario, since the playback restores the work folder from the backup
    let replay = root.join("replay");
    copy_folder(&folder, &replay);
    let (code, report, output) = verification_run(&replay.join("script.gc3s"), &replay);
    // the files at the end of the playback are those the recording ended in
    let output_after = fs::read(replay.join("work").join("out.txt")).unwrap_or_default();
    let _ = fs::remove_dir_all(&root);
    let input = EditorName::Node {
        name: "copy".to_owned(),
        item: EditorItem::Input(0),
    };
    let build_system_line = format!("build system \"{}\"", relative(&folder, &build_system()));
    let expected: Vec<&str> = [
        "init",
        "settings file \"backup/1/1.bak\"",
        build_system_line.as_str(),
        "restore \"backup/2/1.bak\" into \"work/n.gc3n\"",
        "restore \"backup/2/2.bak\" into \"work/copy.gc3\"",
        "restore \"backup/2/3.bak\" into \"work/in.txt\"",
        "restore \"backup/2/4.bak\" into \"work/out.txt\"",
        "network \"work/n.gc3n\"",
        "activate",
        "typing mode insert",
        "deactivate",
        "activate",
        "open node copy",
        "focus node copy input 0",
        "cursor node copy input 0 1:12",
        "edit node copy input 0 1:12 0 \"ddd\"",
        "idle node copy input 0",
        "verify node copy input 0 \"script-0.gc3v\"",
        "focus node copy output 0",
        "verify node copy output 0 \"script-1.gc3v\"",
        "close node copy",
        "focus network",
        "edit network 7:1 0 \"// end\\n\"",
        "idle network",
        "verify network \"script-2.gc3v\"",
        "deactivate",
    ]
    .to_vec();
    assert_eq!(
        (
            exit_code,
            lines.len(),
            lines.get(1).copied(),
            lines[2..].to_vec()
        ),
        (0, 28, Some("automation script"), expected),
        "the recording:\n{text}"
    );
    assert_eq!(
        (
            lines[0].starts_with("// Recorded by genc³wb 0.42.0.0 on "),
            actions.len(),
            actions.get(12),
            actions.get(14),
            actions.get(15),
            expected_0.as_slice(),
            expected_1.as_slice(),
            expected_2.ends_with(b"out.txt\n// end\n"),
            saved_network.as_str(),
            // the product stored the settings before the recording began, so the copy holds them all
            saved_settings.contains("tab_size: 4\n"),
        ),
        (
            true,
            26,
            Some(&Action::OpenNode("copy".to_owned())),
            Some(&Action::Cursor {
                editor: input.clone(),
                line: 1,
                column: 12,
            }),
            Some(&Action::Edit {
                editor: input,
                line: 1,
                column: 12,
                range: 0,
                text: "ddd".to_owned(),
            }),
            b"hello worldddd".as_slice(),
            b"hello worldddd".as_slice(),
            true,
            ONE_NODE,
            true,
        ),
        "read: {read:?}"
    );
    let last_line = text.lines().count();
    assert_eq!(
        (
            code,
            report.len(),
            report.last().map(String::as_str),
            output_after.as_slice()
        ),
        (
            0,
            last_line - 2,
            Some(format!("{last_line}: ok: deactivate").as_str()),
            b"hello worldddd".as_slice()
        ),
        "the playback:\n{output}"
    );
}
