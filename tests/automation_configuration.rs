//! The test of the *automation configuration* through the *basic executable* `gc3`: a script of
//! every action renders in the *canonical form* the *core* reads, line for line, and a faulty
//! script is told with its line.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use genc3wb::core::automation::{
    Action, DialogAnswer, EditorItem, EditorName, FocusTarget, ScriptError, SettingsValues,
    WindowName, gc3_beside, read_script,
};
use genc3wb::core::octet_view::EditorMode;
use genc3wb::core::recording::line_of;

use std::fs;
use std::path::{Path, PathBuf};

/// A script holding every action of the notation once.
const SCRIPT: &str = "// a script of every action\nautomation script\n\
    restore \"backup\" into \"src\"\nbuild system \"../bin/genc3d\"\nnetwork \"src/n.gc3n\"\n\
    idle network\n\nactivate\nopen node copy\nanswer creation create silent\n\
    edit node copy input 0 1:12 0 \"ddd\\n\\\"q\\\"\"\nidle node copy input 0\n\
    mode node copy input 0 hex\ndigits node copy input 0 hex 4 2 \"AB\"\ncursor node copy dsl 3:4\n\
    select file \"src/in.txt\" 1:1 2:5\nfocus node copy\nfocus main\nfocus node copy diagnostics\n\
    long idle network\nsettings idle 2.0 long idle 16 automatic off tab size 4 spaces on silent creation off delay 3\n\
    insert key \"Ctrl+I\"\ntyping mode overwrite\nanswer save yes\nanswer creation cancel\n\
    answer creation failure continue\nundo node copy input 0\nredo node copy input 0\n\
    open file \"src/out.txt\"\nverify node copy output 0 \"expected/out.gc3v\"\nclose node copy\n\
    close file \"src/out.txt\"\ndeactivate\n";

/// Yields `gc3` beside the build system of `GENC3D`, and the configuration of 'src'.
fn tools() -> (PathBuf, PathBuf) {
    let build_system = std::env::var("GENC3D").expect("GENC3D names the service executable genc3d");
    (
        gc3_beside(&build_system),
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src/automation.gc3"),
    )
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : Automation::play */

// FR-493 to FR-497, IR-212, IR-213
// needs the basic executable gc3 of genc³ 0.36.0.0 or later beside the genc3d named by the
// environment variable GENC3D
#[test]
#[ignore]
fn every_action_is_read_from_the_canonical_form_gc3_renders() {
    let (gc3, configuration) = tools();
    let folder = std::env::temp_dir().join(format!("g3wb-ac-{}", std::process::id()));
    fs::create_dir_all(&folder).expect("the temporary directory can be created");
    let path = folder.join("every.gc3s");
    fs::write(&path, SCRIPT).expect("the script is written");
    let script = read_script(&path, &gc3, &configuration);
    let faulty = read_script(&folder.join("none.gc3s"), &gc3, &configuration);
    fs::write(&path, "automation script\nopen node copy\nedit network 1:1 0 unquoted\n")
        .expect("the script is written");
    let unquoted = read_script(&path, &gc3, &configuration);
    let _ = fs::remove_dir_all(&folder);
    let script = script.expect("the script of every action is read");
    let lines: Vec<usize> = script.steps.iter().map(|step| step.line).collect();
    assert_eq!(
        (
            script.steps.len(),
            lines.first().copied(),
            lines.last().copied(),
            &script.steps[7].action,
            &script.steps[6].action,
            script.steps[2].text.as_str()
        ),
        (
            30,
            Some(3),
            Some(33),
            &Action::Edit {
                editor: EditorName::Node {
                    name: "copy".to_owned(),
                    item: EditorItem::Input(0)
                },
                line: 1,
                column: 12,
                range: 0,
                text: "ddd\n\"q\"".to_owned()
            },
            &Action::Answer(DialogAnswer::Creation {
                create: true,
                silent: true
            }),
            "network \"src/n.gc3n\""
        )
    );
    assert!(
        matches!(faulty, Err(ScriptError::NotRun(_))),
        "a script that does not exist: {faulty:?}"
    );
    match unquoted {
        Err(ScriptError::Faulty(diagnosis)) => assert!(
            diagnosis.contains("3:"),
            "the diagnosis names the line: {diagnosis}"
        ),
        other => panic!("an unquoted text is a fault of the script: {other:?}"),
    }
}

// FR-519, FR-496
// needs what the test above needs
#[test]
#[ignore]
fn every_line_the_recorder_renders_reads_back_into_its_action() {
    let (gc3, configuration) = tools();
    let folder = std::env::temp_dir().join(format!("g3wb-rt-{}", std::process::id()));
    fs::create_dir_all(&folder).expect("the temporary directory can be created");
    let input = EditorName::Node {
        name: "copy".to_owned(),
        item: EditorItem::Input(0),
    };
    let file = EditorName::File {
        path: folder.join("src").join("in.txt").to_string_lossy().into_owned(),
    };
    let actions = vec![
        Action::Restore {
            backup: folder.join("backup").to_string_lossy().into_owned(),
            source: folder.join("src").to_string_lossy().into_owned(),
        },
        Action::BuildSystem(
            folder
                .parent()
                .expect("the temporary directory has a parent")
                .join("bin")
                .join("genc3d")
                .to_string_lossy()
                .into_owned(),
        ),
        Action::Network(folder.join("src").join("n.gc3n").to_string_lossy().into_owned()),
        Action::NewNetwork(folder.join("new.gc3n").to_string_lossy().into_owned()),
        Action::Idle(EditorName::Network),
        Action::Activate,
        Action::OpenNode("copy".to_owned()),
        Action::Answer(DialogAnswer::Creation {
            create: true,
            silent: true,
        }),
        Action::Edit {
            editor: input.clone(),
            line: 1,
            column: 12,
            range: 0,
            text: "ddd\n\"q\"\t\\\r".to_owned(),
        },
        Action::Idle(input.clone()),
        Action::Mode {
            editor: input.clone(),
            mode: EditorMode::Hex,
        },
        Action::Digits {
            editor: input.clone(),
            mode: EditorMode::Bin,
            start: 4,
            removed: 2,
            text: "01".to_owned(),
        },
        Action::Cursor {
            editor: EditorName::Node {
                name: "copy".to_owned(),
                item: EditorItem::MetaDsl,
            },
            line: 3,
            column: 4,
        },
        Action::Select {
            editor: file.clone(),
            from: (1, 1),
            to: (2, 5),
        },
        Action::Focus(FocusTarget::Window(WindowName::Node("copy".to_owned()))),
        Action::Focus(FocusTarget::Window(WindowName::Main)),
        Action::Focus(FocusTarget::Editor(EditorName::Node {
            name: "copy".to_owned(),
            item: EditorItem::Diagnostics,
        })),
        Action::Focus(FocusTarget::Editor(file.clone())),
        Action::LongIdle(EditorName::Network),
        Action::Settings(SettingsValues {
            idle_time: 2000,
            long_idle_time: 16000,
            automatic: false,
            tab_size: 4,
            tabs_to_spaces: true,
            silent_creation: false,
            playback_delay: 3,
        }),
        Action::InsertKey("Ctrl+I".to_owned()),
        Action::TypingMode(false),
        Action::TypingMode(true),
        Action::Answer(DialogAnswer::Save(true)),
        Action::Answer(DialogAnswer::Save(false)),
        Action::Answer(DialogAnswer::Creation {
            create: true,
            silent: false,
        }),
        Action::Answer(DialogAnswer::Creation {
            create: false,
            silent: false,
        }),
        Action::Answer(DialogAnswer::CreationFailure(true)),
        Action::Answer(DialogAnswer::CreationFailure(false)),
        Action::Undo(input.clone()),
        Action::Redo(input.clone()),
        Action::OpenFile(folder.join("src").join("out.txt").to_string_lossy().into_owned()),
        Action::Verify {
            editor: EditorName::Node {
                name: "copy".to_owned(),
                item: EditorItem::Output(0),
            },
            file: folder.join("every-0.gc3v").to_string_lossy().into_owned(),
        },
        Action::CloseNode("copy".to_owned()),
        Action::CloseFile(folder.join("src").join("out.txt").to_string_lossy().into_owned()),
        Action::Deactivate,
    ];
    let text = actions
        .iter()
        .map(|action| line_of(action, &folder) + "\n")
        .collect::<String>();
    let path = folder.join("every.gc3s");
    fs::write(&path, format!("// rendered by the recorder\nautomation script\n{text}"))
        .expect("the script is written");
    let script = read_script(&path, &gc3, &configuration);
    let _ = fs::remove_dir_all(&folder);
    let script = script.expect("the rendered script is read");
    let read: Vec<Action> = script.steps.into_iter().map(|step| step.action).collect();
    assert_eq!(read, actions, "the rendering:\n{text}");
}
