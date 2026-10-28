//! What the round-trip tests of the *recording* share, one test per way a *recording* can begin:
//! the files of a network of one *node*, the *settings file* beside the test executable, the
//! test *front end* that records a session, the *automatic recording* found in the *recording
//! folder*, the lines expected of it, the *verification run* of the executable of *product* on
//! it in a second process, and the files at the end.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>
#![allow(dead_code)]

use genc3wb::core::launch;
use genc3wb::core::recording::{AUTOMATIC_FILE_NAME, RECORDING_FOLDER_NAME, relative};
use genc3wb::core::settings::Settings;

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A *network file* of one *node*, which copies its *source input* to its *sink output*.
pub const ONE_NODE: &str = "meta compiler-compiler network\n\
    \tnode copy\n\t\tinputs\n\t\t\tin.txt\n\t\toutputs\n\t\t\tout.txt\n";

/// The *meta compiler DSL* of the *node*: a syntax matching the letters of 'hello world', and a
/// generator emitting the matched text.
pub const COPY_META_DSL: &str = "syntax\n    Root = Chars, EOS\n    Chars = Ch, { Ch }\n    \
    Ch = 'h' | 'e' | 'l' | 'o' | 'w' | 'r' | 'd' | ' '\n\
    generator\n    Root -> flat(#0)\n";

/// The content of the *input* and of the *output* at the beginning.
pub const HELLO: &[u8] = b"hello world";

/// Yields the path of the build system: the environment variable `GENC3D`.
pub fn build_system() -> String {
    std::env::var("GENC3D").expect("GENC3D names the service executable genc3d")
}

/// Yields the root of the case: a folder of its own under the temporary directory, short enough
/// for the paths of the Unix domain sockets, emptied.
pub fn case_root(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("g3wb-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    root
}

/// Writes the network of one *node* into `work`: the *network file* 'n.gc3n', the *meta compiler
/// DSL* 'copy.gc3', the *input* 'in.txt' and the *output* 'out.txt'; yields the path of the
/// *network file*.
pub fn write_network(work: &Path) -> PathBuf {
    fs::create_dir_all(work).expect("the work folder can be written");
    fs::write(work.join("n.gc3n"), ONE_NODE).expect("the network file is written");
    fs::write(work.join("copy.gc3"), COPY_META_DSL).expect("the meta compiler DSL is written");
    fs::write(work.join("in.txt"), HELLO).expect("the input is written");
    fs::write(work.join("out.txt"), HELLO).expect("the output is written");
    work.join("n.gc3n")
}

/// The *settings file* beside the test executable, written for the case and put back afterwards.
pub struct SettingsKept {
    path: PathBuf,
    before: Option<Vec<u8>>,
}

impl SettingsKept {
    /// Writes the *settings file* the *workbench object* reads at its construction: the *build
    /// system*, the *automatic setting* off, and the *network file* `network` where one is named
    /// at the start.
    pub fn write(network: Option<&Path>) -> Self {
        let path = Settings::default_path();
        let before = fs::read(&path).ok();
        let mut text = format!(
            "tab_size: 4\nautomatic: false\nbuild_system_path: {}\n",
            build_system()
        );
        if let Some(network) = network {
            text.push_str(&format!("network_path: {}\n", network.display()));
        }
        fs::write(&path, text).expect("the settings file is written");
        SettingsKept { path, before }
    }

    /// Puts the *settings file* back as it was.
    pub fn restore(self) {
        match self.before {
            Some(octets) => fs::write(&self.path, octets).expect("the settings are restored"),
            None => {
                let _ = fs::remove_file(&self.path);
            }
        }
    }
}

/// Yields the folders of the *recording folder*, which lies beside the folder of the executable
/// (IR-216), as `start_folder` places a *recording*.
pub fn recordings() -> BTreeSet<PathBuf> {
    let executable_folder = launch::executable_folder();
    let beside = executable_folder
        .parent()
        .unwrap_or(&executable_folder)
        .join(RECORDING_FOLDER_NAME);
    fs::read_dir(beside)
        .map(|entries| {
            entries
                .flatten()
                .map(|entry| entry.path())
                .filter(|path| path.is_dir())
                .collect()
        })
        .unwrap_or_default()
}

/// Yields the gc3s-file of the *automatic recording* the case started: the one folder of the
/// *recording folder* that `before` does not hold.
pub fn new_recording(before: &BTreeSet<PathBuf>) -> PathBuf {
    let mut added: Vec<PathBuf> = recordings().difference(before).cloned().collect();
    assert_eq!(added.len(), 1, "one recording was started: {added:?}");
    added.remove(0).join(AUTOMATIC_FILE_NAME)
}

/// Yields the *front end* of a case: a step machine of one timer that waits until a *network
/// file* is named, from the settings, from the command line or from the file-open event, or
/// names `named_at_start` itself as the dialog does, expires the idle, which provides the text
/// and has the *build* of the *product* save the backup, render the graph and yield the *node
/// descriptions*, waits until the *product* has activated the network of its own, opens the
/// *node* once the graph is presented, focuses its *input*, places the cursor,
/// appends `ddd` and expires the idle, stores the *input* and the *output* as *expected files*,
/// closes the *node window*, stops the *recording* and exits with 0; with 2 where the *node
/// editor* is not named, and 8 where a step does not come within a minute. `started` has it
/// tell the *workbench object* that the *front end* is up, so that a file-open event posted
/// reaches it in the *start-up phase*.
pub fn front_end(named_at_start: Option<&Path>, started: bool) -> Vec<u8> {
    let name = named_at_start
        .map(|path| format!("Workbench.networkInput.path = \"{}\";", path.display()))
        .unwrap_or_default();
    let started = if started { "Workbench.started();" } else { "" };
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

    function leave(code) {{
        console.log("mode " + Workbench.network.networkMode + ", path '" + Workbench.networkInput.path + "' at step " + root.step);
        Workbench.network.shutDown();
        Workbench.releaseLock();
        Qt.exit(code);
    }}

    Timer {{
        interval: 200
        running: true
        repeat: true
        onTriggered: {{
            root.ticks += 1;
            var mode = Workbench.network.networkMode;
            if (root.step === 0 && Workbench.networkInput.path.length > 0) {{
                // the first idle expiry provides the text of the network file, as the idle
                // timer of Main.qml does, on which the build of the product saves the backup,
                // renders the graph and yields the node descriptions, and the product tries
                // its own activation once those and the valid mode are known (FR-534)
                Workbench.networkInput.idleExpired();
                root.step = 2;
            }} else if (root.step === 2 && mode === 3 && Workbench.network.graphSource.length > 0) {{
                Workbench.network.openNodeNamed("copy");
                root.step = 3;
                root.ticks = 0;
            }} else if (root.step === 3 && root.ticks >= 10) {{
                root.node = Workbench.network.nodeNamed("copy");
                if (root.node.name !== "copy") {{
                    root.leave(2);
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
                root.step = 5;
                root.ticks = 0;
            }} else if (root.step === 5 && root.ticks >= 5) {{
                Workbench.automation.stopRecording();
                root.leave(0);
            }}
        }}
    }}

    Timer {{
        interval: 60000
        running: true
        onTriggered: root.leave(8)
    }}

    Component.onCompleted: {{
        {started}
        {name}
    }}
}}
"#
    )
    .into_bytes()
}

/// Yields the lines expected of the *automatic recording* of a case after its comment line and
/// its head line: `init`, the *settings file* saved, the *build system* in force, the four files
/// of the network saved from the first *network response*, `network`, the idle expiry, the
/// activation of the *product*,
/// and the actions of the *front end*; the paths relative to the folder of `script`.
pub fn expected_lines(script: &Path, work: &Path) -> Vec<String> {
    let folder = script.parent().expect("the script lies in a folder");
    let rel = |name: &str| relative(folder, &work.join(name).to_string_lossy());
    vec![
        "init".to_owned(),
        "settings file \"backup/1/1.bak\"".to_owned(),
        format!("build system \"{}\"", relative(folder, &build_system())),
        format!("restore \"backup/2/1.bak\" into \"{}\"", rel("n.gc3n")),
        format!("restore \"backup/2/2.bak\" into \"{}\"", rel("copy.gc3")),
        format!("restore \"backup/2/3.bak\" into \"{}\"", rel("in.txt")),
        format!("restore \"backup/2/4.bak\" into \"{}\"", rel("out.txt")),
        format!("network \"{}\"", rel("n.gc3n")),
        "idle network".to_owned(),
        "activate".to_owned(),
        "open node copy".to_owned(),
        "focus node copy input 0".to_owned(),
        "cursor node copy input 0 1:12".to_owned(),
        "edit node copy input 0 1:12 0 \"ddd\"".to_owned(),
        "idle node copy input 0".to_owned(),
        "verify node copy input 0 \"usage-0.gc3v\"".to_owned(),
        "focus node copy output 0".to_owned(),
        "verify node copy output 0 \"usage-1.gc3v\"".to_owned(),
        "close node copy".to_owned(),
    ]
}

/// Plays the script at `script` by a *verification run* of the executable of *product*, in the
/// folder of the script, the *automation configuration* beside the executable and a *settings
/// file* beside it naming the *build system*; yields the exit code and the lines of the report,
/// and the whole output for a message.
pub fn verification_run(script: &Path) -> (i32, Vec<String>, String) {
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
        .current_dir(script.parent().expect("the script lies in a folder"))
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

/// Yields the files of the network that the session changes: the *input* and the *output*.
pub fn end_state(work: &Path) -> (Vec<u8>, Vec<u8>) {
    (
        fs::read(work.join("in.txt")).unwrap_or_default(),
        fs::read(work.join("out.txt")).unwrap_or_default(),
    )
}

/// The outcome of a case, compared as one value.
#[derive(Debug, PartialEq)]
pub struct RoundTrip {
    /// the exit code of the *front end* of the recording
    pub exit_code: i32,
    /// whether the comment line names *product*
    pub comment: bool,
    /// the head line
    pub head: Option<String>,
    /// the lines of the actions recorded
    pub actions: Vec<String>,
    /// the files at the end of the recording
    pub recorded: (Vec<u8>, Vec<u8>),
    /// the exit code of the playback
    pub playback_code: i32,
    /// the lines of the report of the playback that are not ok
    pub not_ok: Vec<String>,
    /// the number of lines of the report
    pub reported: usize,
    /// the files at the end of the playback
    pub played: (Vec<u8>, Vec<u8>),
}

/// Reads the *recording* at `script` after the *front end* exited with `exit_code`, plays it
/// back and yields the outcome: the files of `work` are read after the recording and after the
/// playback, the recording folder removed at the end.
pub fn round_trip(exit_code: i32, script: &Path, work: &Path) -> RoundTrip {
    let text = fs::read_to_string(script).unwrap_or_default();
    let lines: Vec<&str> = text.lines().collect();
    let recorded = end_state(work);
    let (playback_code, report, _output) = verification_run(script);
    let played = end_state(work);
    if let Some(folder) = script.parent() {
        let _ = fs::remove_dir_all(folder);
    }
    RoundTrip {
        exit_code,
        comment: lines
            .first()
            .is_some_and(|line| line.starts_with("// Recorded by genc³wb 0.43.0.0 on ")),
        head: lines.get(1).map(|line| (*line).to_owned()),
        actions: lines
            .iter()
            .skip(2)
            .map(|line| (*line).to_owned())
            .collect(),
        recorded,
        playback_code,
        not_ok: report
            .iter()
            .filter(|line| !line.contains(": ok: "))
            .cloned()
            .collect(),
        reported: report.len(),
        played,
    }
}

/// Yields the outcome expected of a case whose *front end* exited with 0: every action ok and the
/// *input* and the *output* `hello worldddd` after the recording and after the playback alike.
pub fn expected(script: &Path, work: &Path) -> RoundTrip {
    let actions = expected_lines(script, work);
    let changed = (b"hello worldddd".to_vec(), b"hello worldddd".to_vec());
    RoundTrip {
        exit_code: 0,
        comment: true,
        head: Some("automation script".to_owned()),
        reported: actions.len(),
        actions,
        recorded: changed.clone(),
        playback_code: 0,
        not_ok: Vec::new(),
        played: changed,
    }
}
