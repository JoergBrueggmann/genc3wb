//! The *automation script*: its reading through the *basic executable* `gc3` on the *automation
//! configuration*, the *canonical form* read into the actions, the offsets of a position, the
//! restoring of a *precondition backup*, the comparison with an *expected file*, the lines of the
//! *playback report* and the exit code of a *verification run*.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::launch;
use crate::core::octet_view::EditorMode;

use std::fmt;
use std::fs;
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::time::Duration;

// realises FR-496, IR-212
/// The name of the *automation configuration* in the folder of the executable (C-009), and the
/// name of the *basic executable* in the folder of the *build system*.
pub const CONFIGURATION_FILE_NAME: &str = "automation.gc3";
pub const GC3_FILE_NAME: &str = "gc3";

// realises FR-514
/// How long the *player* waits for the effect of an action before the action failed.
pub const ACTION_TIMEOUT: Duration = Duration::from_secs(30);

// realises FR-501, FR-504, FR-506, FR-510
/// How long nothing may be pending after an action before it is complete: the *settling*, two
/// steps of the *player*.
pub const SETTLING: Duration = Duration::from_millis(100);

/// The suffix of a *lock file*, which `restore` leaves in place.
const LOCK_SUFFIX: &str = ".lock";

// realises FR-494
/// The code editor of a *node window* an action names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorItem {
    /// the *input group* of the *meta compiler DSL*, `dsl`
    MetaDsl,
    /// the *input group* of the *input* of that number, `input <n>`
    Input(usize),
    /// the *output page* of the *output* of that number, `output <n>`
    Output(usize),
    /// the *diagnostics page*, `diagnostics`
    Diagnostics,
}

// realises FR-494
/// A code editor an action names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EditorName {
    /// the code editor of the *network file*, `network`
    Network,
    /// a code editor of the *node window* of the *node* `name`
    Node { name: String, item: EditorItem },
    /// the code editor of the *editor window* on the file at the resolved `path`
    File { path: String },
}

impl EditorName {
    // realises FR-494, FR-505, FR-508
    /// Yields the four fields by which the *front end* finds the code editor: its kind,
    /// `network`, `node` or `file`, the name of the *node* or the path of the file, the item,
    /// `dsl`, `input`, `output`, `diagnostics` or empty, and the number of the item, 0 where it
    /// has none.
    pub fn fields(&self) -> (String, String, String, i32) {
        match self {
            EditorName::Network => ("network".to_owned(), String::new(), String::new(), 0),
            EditorName::Node { name, item } => {
                let (item, index) = match item {
                    EditorItem::MetaDsl => ("dsl", 0),
                    EditorItem::Input(index) => ("input", *index),
                    EditorItem::Output(index) => ("output", *index),
                    EditorItem::Diagnostics => ("diagnostics", 0),
                };
                (
                    "node".to_owned(),
                    name.clone(),
                    item.to_owned(),
                    i32::try_from(index).unwrap_or(i32::MAX),
                )
            }
            EditorName::File { path } => ("file".to_owned(), path.clone(), String::new(), 0),
        }
    }
}

// realises FR-495, FR-508
/// A window the action `focus` names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WindowName {
    /// the main window, `main`
    Main,
    /// the *node window* of the *node* of that name
    Node(String),
}

// realises FR-495, FR-508
/// What the action `focus` gives the focus to.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FocusTarget {
    Editor(EditorName),
    Window(WindowName),
}

// realises FR-495, FR-510
/// The answer of the action `answer`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DialogAnswer {
    /// the question of FR-014: whether to save
    Save(bool),
    /// the *input creation confirmation dialog*: 'Create', with the check box checked where
    /// `silent`, or 'Cancel'
    Creation { create: bool, silent: bool },
    /// the dialog of FR-491: 'Continue' or 'Cancel'
    CreationFailure(bool),
}

// realises FR-495, FR-509
/// The values of the action `settings`, as the settings dialog accepts them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SettingsValues {
    /// the two times in milliseconds
    pub idle_time: u32,
    pub long_idle_time: u32,
    pub automatic: bool,
    pub tab_size: u32,
    pub tabs_to_spaces: bool,
    pub silent_creation: bool,
    /// the *character playback delay* in tenths of a second
    pub playback_delay: u32,
}

// realises FR-495
/// One action of an *automation script*, with its arguments: a path resolved against the folder
/// of the script, a text unescaped, a line and a column counted from 1 as the script counts them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Network(String),
    BuildSystem(String),
    NewNetwork(String),
    OpenNode(String),
    CloseNode(String),
    OpenFile(String),
    CloseFile(String),
    Activate,
    Deactivate,
    /// replace `range` characters at the position by `text`
    Edit {
        editor: EditorName,
        line: usize,
        column: usize,
        range: usize,
        text: String,
    },
    /// replace `removed` *digits* from the *digit* `start` by the *digits* of `text` in `mode`
    Digits {
        editor: EditorName,
        mode: EditorMode,
        start: usize,
        removed: usize,
        text: String,
    },
    Idle(EditorName),
    LongIdle(EditorName),
    Cursor {
        editor: EditorName,
        line: usize,
        column: usize,
    },
    Select {
        editor: EditorName,
        from: (usize, usize),
        to: (usize, usize),
    },
    Focus(FocusTarget),
    Mode {
        editor: EditorName,
        mode: EditorMode,
    },
    Undo(EditorName),
    Redo(EditorName),
    Settings(SettingsValues),
    /// the name of the *insert key* (FR-200)
    InsertKey(String),
    /// whether the *typing mode* is the insert mode
    TypingMode(bool),
    Answer(DialogAnswer),
    /// make `source` equal to `backup`, both resolved
    Restore { backup: String, source: String },
    /// compare the document of the code editor with the *expected file* at the resolved path
    Verify { editor: EditorName, file: String },
}

// realises FR-496, FR-515
/// One action of a script with the number of its line, counted from 1, and the line as written,
/// without its comment and trimmed, as the *playback report* names it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Step {
    pub line: usize,
    pub text: String,
    pub action: Action,
}

// realises FR-493, FR-497
/// An *automation script* read: the folder of its file, against which its paths were resolved,
/// and its actions in their order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Script {
    pub folder: PathBuf,
    pub steps: Vec<Step>,
}

// realises FR-497, FR-514, FR-517
/// Why a script was not read; each renders as the text of the *playback report* of a faulty
/// script.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScriptError {
    /// `gc3` is not found beside the *build system*; carries the path tried
    Gc3NotFound(String),
    /// the *automation configuration* is not in the folder of the executable; carries the path
    ConfigurationNotFound(String),
    /// the script could not be read or `gc3` could not be run; carries the reason
    NotRun(String),
    /// `gc3` told a fault of the script; carries what it wrote to its standard error
    Faulty(String),
    /// the *canonical form* holds a line the *core* does not read, the configuration and the
    /// *core* having diverged; carries the line
    Canonical(String),
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ScriptError::Gc3NotFound(path) => {
                write!(f, "the basic executable gc3 is not found at '{path}'")
            }
            ScriptError::ConfigurationNotFound(path) => {
                write!(f, "the automation configuration is not found at '{path}'")
            }
            ScriptError::NotRun(reason) => write!(f, "the script could not be read: {reason}"),
            ScriptError::Faulty(diagnosis) => write!(f, "the script is faulty:\n{diagnosis}"),
            ScriptError::Canonical(line) => write!(
                f,
                "the canonical form holds a line the workbench does not read: '{line}'"
            ),
        }
    }
}

// realises FR-513 to FR-515
/// The outcome of an action: completed, a verification that failed with its difference, or an
/// action that failed with its reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Completed,
    Failed(String),
    Error(String),
}

impl Outcome {
    /// Yields whether the *playback* goes on after the outcome: it does after `Completed` alone.
    pub fn continues(&self) -> bool {
        matches!(self, Outcome::Completed)
    }
}

// realises IR-212
/// Yields the path of the *basic executable* `gc3` in the folder of the *build system* at
/// `build_system`, as `genc3d` is found beside the executable of *product*; the name alone where
/// the path of the *build system* has no folder.
pub fn gc3_beside(build_system: &str) -> PathBuf {
    Path::new(build_system)
        .parent()
        .filter(|folder| !folder.as_os_str().is_empty())
        .map_or_else(
            || PathBuf::from(GC3_FILE_NAME),
            |folder| folder.join(GC3_FILE_NAME),
        )
}

// realises FR-496, IR-212
/// Yields the path of the *automation configuration*: `CONFIGURATION_FILE_NAME` in the folder of
/// the executable (`launch::executable_folder`).
pub fn configuration_path() -> PathBuf {
    launch::executable_folder().join(CONFIGURATION_FILE_NAME)
}

// realises FR-497, IR-212, IR-213
/// Reads the *automation script* at `path`: runs `gc3` with `-c` and `configuration`, the
/// octets of the script as its standard input, and reads the *canonical form* from its standard
/// output into the steps (`steps_of_canonical`), the paths resolved against the folder of
/// `path`, made absolute against the working directory of the moment.
///
/// # Errors
/// Returns [`ScriptError::Gc3NotFound`] and [`ScriptError::ConfigurationNotFound`] where either
/// file does not exist, [`ScriptError::NotRun`] where the script cannot be read or the process
/// cannot be run, [`ScriptError::Faulty`] with the standard error of `gc3` where it exits with
/// a failure, and [`ScriptError::Canonical`] where a line of the *canonical form* is not read.
pub fn read_script(path: &Path, gc3: &Path, configuration: &Path) -> Result<Script, ScriptError> {
    if !gc3.is_file() {
        return Err(ScriptError::Gc3NotFound(gc3.display().to_string()));
    }
    if !configuration.is_file() {
        return Err(ScriptError::ConfigurationNotFound(
            configuration.display().to_string(),
        ));
    }
    let octets = fs::read(path).map_err(|error| ScriptError::NotRun(error.to_string()))?;
    let text = String::from_utf8_lossy(&octets).into_owned();
    let folder = PathBuf::from(launch::absolute(
        &path
            .parent()
            .filter(|folder| !folder.as_os_str().is_empty())
            .map_or_else(|| ".".to_owned(), |folder| folder.display().to_string()),
    ));
    let mut child = Command::new(gc3)
        .arg("-c")
        .arg(configuration)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|error| ScriptError::NotRun(error.to_string()))?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin
            .write_all(&octets)
            .map_err(|error| ScriptError::NotRun(error.to_string()))?;
    }
    let output = child
        .wait_with_output()
        .map_err(|error| ScriptError::NotRun(error.to_string()))?;
    if !output.status.success() {
        let diagnosis = String::from_utf8_lossy(&output.stderr).trim_end().to_owned();
        return Err(ScriptError::Faulty(if diagnosis.is_empty() {
            format!("gc3 ended with {}", output.status)
        } else {
            diagnosis
        }));
    }
    let canonical = String::from_utf8_lossy(&output.stdout).into_owned();
    let steps = steps_of_canonical(&canonical, &text, &folder).map_err(ScriptError::Canonical)?;
    Ok(Script { folder, steps })
}

// realises FR-494 to FR-496
/// Yields the steps of the *canonical form* `canonical` of the script whose text is `script`,
/// the paths resolved against `folder` and the texts unescaped; the text of a step is its line
/// of `script`, trimmed, without the comment.
///
/// # Errors
/// Returns the line of `canonical` that is not read.
pub fn steps_of_canonical(
    canonical: &str,
    script: &str,
    folder: &Path,
) -> Result<Vec<Step>, String> {
    let lines: Vec<&str> = script.split('\n').collect();
    let mut steps = Vec::new();
    for (index, line) in canonical.split('\n').enumerate() {
        if line.is_empty() {
            continue;
        }
        let fields: Vec<&str> = line.split('\t').collect();
        let action = action_of_fields(&fields, folder).ok_or_else(|| line.to_owned())?;
        let text = lines
            .get(index)
            .map(|line| without_comment(line).trim().to_owned())
            .unwrap_or_default();
        steps.push(Step {
            line: index + 1,
            text,
            action,
        });
    }
    Ok(steps)
}

/// Yields the action of the fields of a line of the *canonical form*, `None` where they are not
/// read.
fn action_of_fields(fields: &[&str], folder: &Path) -> Option<Action> {
    let editor = |at: usize| editor_of_fields(fields.get(at..at + 4)?, folder);
    let number = |at: usize| fields.get(at)?.parse::<usize>().ok();
    let text = |at: usize| fields.get(at).map(|field| unescaped(field));
    let path = |at: usize| fields.get(at).map(|field| resolved(folder, &unescaped(field)));
    let switch = |at: usize| match *fields.get(at)? {
        "on" => Some(true),
        "off" => Some(false),
        _ => None,
    };
    let kind = *fields.first()?;
    let action = match kind {
        "network" => Action::Network(path(1)?),
        "build system" => Action::BuildSystem(path(1)?),
        "new network" => Action::NewNetwork(path(1)?),
        "open node" => Action::OpenNode(text(1)?),
        "close node" => Action::CloseNode(text(1)?),
        "open file" => Action::OpenFile(path(1)?),
        "close file" => Action::CloseFile(path(1)?),
        "activate" => Action::Activate,
        "deactivate" => Action::Deactivate,
        "edit" => Action::Edit {
            editor: editor(1)?,
            line: number(5)?,
            column: number(6)?,
            range: number(7)?,
            text: text(8)?,
        },
        "digits" => Action::Digits {
            editor: editor(1)?,
            mode: octet_mode_of(fields.get(5)?)?,
            start: number(6)?,
            removed: number(7)?,
            text: text(8)?,
        },
        "idle" => Action::Idle(editor(1)?),
        "long idle" => Action::LongIdle(editor(1)?),
        "cursor" => Action::Cursor {
            editor: editor(1)?,
            line: number(5)?,
            column: number(6)?,
        },
        "select" => Action::Select {
            editor: editor(1)?,
            from: (number(5)?, number(6)?),
            to: (number(7)?, number(8)?),
        },
        "focus" => Action::Focus(if fields.get(3) == Some(&"window") {
            FocusTarget::Window(match *fields.get(1)? {
                "main" => WindowName::Main,
                "node" => WindowName::Node(text(2)?),
                _ => return None,
            })
        } else {
            FocusTarget::Editor(editor(1)?)
        }),
        "mode" => Action::Mode {
            editor: editor(1)?,
            mode: match *fields.get(5)? {
                "txt" => EditorMode::Txt,
                other => octet_mode_of(other)?,
            },
        },
        "undo" => Action::Undo(editor(1)?),
        "redo" => Action::Redo(editor(1)?),
        "settings" => Action::Settings(SettingsValues {
            idle_time: milliseconds_of(fields.get(1)?)?,
            long_idle_time: milliseconds_of(fields.get(2)?)?,
            automatic: switch(3)?,
            tab_size: u32::try_from(number(4)?).ok()?,
            tabs_to_spaces: switch(5)?,
            silent_creation: switch(6)?,
            playback_delay: u32::try_from(number(7)?).ok()?,
        }),
        "insert key" => Action::InsertKey(text(1)?),
        "typing mode" => Action::TypingMode(match *fields.get(1)? {
            "insert" => true,
            "overwrite" => false,
            _ => return None,
        }),
        "answer" => Action::Answer(match *fields.get(1)? {
            "save" => DialogAnswer::Save(match *fields.get(2)? {
                "yes" => true,
                "no" => false,
                _ => return None,
            }),
            "creation" => DialogAnswer::Creation {
                create: match *fields.get(2)? {
                    "create" => true,
                    "cancel" => false,
                    _ => return None,
                },
                silent: fields.get(3) == Some(&"silent"),
            },
            "creation failure" => DialogAnswer::CreationFailure(match *fields.get(2)? {
                "continue" => true,
                "cancel" => false,
                _ => return None,
            }),
            _ => return None,
        }),
        "restore" => Action::Restore {
            backup: path(1)?,
            source: path(2)?,
        },
        "verify" => Action::Verify {
            editor: editor(1)?,
            file: path(5)?,
        },
        _ => return None,
    };
    Some(action)
}

/// Yields the code editor of four fields of the *canonical form*: the kind, the name or path,
/// the item and the number of the item.
fn editor_of_fields(fields: &[&str], folder: &Path) -> Option<EditorName> {
    let [kind, name, item, index] = fields else {
        return None;
    };
    match *kind {
        "network" => Some(EditorName::Network),
        "file" => Some(EditorName::File {
            path: resolved(folder, &unescaped(name)),
        }),
        "node" => {
            let number = || index.parse::<usize>().ok();
            let item = match *item {
                "dsl" => EditorItem::MetaDsl,
                "input" => EditorItem::Input(number()?),
                "output" => EditorItem::Output(number()?),
                "diagnostics" => EditorItem::Diagnostics,
                _ => return None,
            };
            Some(EditorName::Node {
                name: unescaped(name),
                item,
            })
        }
        _ => None,
    }
}

/// Yields the *editor mode* hex or bin of its word.
fn octet_mode_of(word: &str) -> Option<EditorMode> {
    match word {
        "hex" => Some(EditorMode::Hex),
        "bin" => Some(EditorMode::Bin),
        _ => None,
    }
}

/// Yields the milliseconds of a number of seconds with at most three decimals, as the script
/// writes a time.
fn milliseconds_of(seconds: &str) -> Option<u32> {
    let (whole, fraction) = seconds.split_once('.').unwrap_or((seconds, ""));
    let whole: u32 = whole.parse().ok()?;
    let mut digits = fraction.to_owned();
    if digits.len() > 3 || !digits.chars().all(|c| c.is_ascii_digit()) {
        return None;
    }
    while digits.len() < 3 {
        digits.push('0');
    }
    let fraction: u32 = digits.parse().ok()?;
    whole.checked_mul(1000)?.checked_add(fraction)
}

/// Yields `line` without the comment that begins at a `//` outside a text.
fn without_comment(line: &str) -> &str {
    let mut in_text = false;
    let mut escaped = false;
    let mut previous_slash = None;
    for (index, character) in line.char_indices() {
        if in_text {
            if escaped {
                escaped = false;
            } else if character == '\\' {
                escaped = true;
            } else if character == '"' {
                in_text = false;
            }
            previous_slash = None;
            continue;
        }
        match character {
            '"' => in_text = true,
            '/' => {
                if let Some(start) = previous_slash {
                    return &line[..start];
                }
                previous_slash = Some(index);
                continue;
            }
            _ => {}
        }
        previous_slash = None;
    }
    line
}

// realises FR-494
/// Yields `text` with the escapes of FR-062 replaced by the characters they stand for; an
/// escape the rendering does not define is kept as written.
pub fn unescaped(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut characters = text.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            result.push(character);
            continue;
        }
        match characters.next() {
            Some('\\') => result.push('\\'),
            Some('"') => result.push('"'),
            Some('n') => result.push('\n'),
            Some('r') => result.push('\r'),
            Some('t') => result.push('\t'),
            Some(other) => {
                result.push('\\');
                result.push(other);
            }
            None => result.push('\\'),
        }
    }
    result
}

// realises FR-494
/// Yields `path` resolved against `folder`: `path` as it is where it is absolute, and the join
/// otherwise, the segments `.` and `..` resolved lexically.
pub fn resolved(folder: &Path, path: &str) -> String {
    let given = Path::new(path);
    let joined = if given.is_absolute() {
        given.to_path_buf()
    } else {
        folder.join(given)
    };
    let mut result = PathBuf::new();
    for component in joined.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if !result.pop() {
                    result.push("..");
                }
            }
            other => result.push(other.as_os_str()),
        }
    }
    result.to_string_lossy().into_owned()
}

// realises FR-494, FR-505, FR-508
/// Yields the offset, in UTF-16 code units as a `TextArea` counts positions, of the character at
/// `line` and `column` of `text`, both counted from 1, the column in characters; the end of the
/// line for a column behind its last character, and `None` for a line beyond the last, or a line
/// or a column of 0.
pub fn offset_of_position(text: &str, line: usize, column: usize) -> Option<usize> {
    if line == 0 || column == 0 {
        return None;
    }
    let mut offset = 0;
    for (number, content) in text.split('\n').enumerate() {
        if number + 1 == line {
            for (count, character) in content.chars().enumerate() {
                if count + 1 == column {
                    return Some(offset);
                }
                offset += character.len_utf16();
            }
            return Some(offset);
        }
        offset += content.encode_utf16().count() + 1;
    }
    None
}

// realises FR-505
/// Yields the offset, in UTF-16 code units, `characters` characters after the offset `offset`
/// of `text`, the end of `text` where fewer remain.
pub fn offset_after(text: &str, offset: usize, characters: usize) -> usize {
    let mut units = 0;
    let mut remaining = characters;
    for character in text.chars() {
        if units >= offset {
            if remaining == 0 {
                return units;
            }
            remaining -= 1;
        }
        units += character.len_utf16();
    }
    units
}

// realises FR-511, IR-214
/// Makes the folder `source` hold what the folder `backup` holds: creates `source` where it does
/// not exist, removes every file and folder of `source` that `backup` does not hold, a *lock
/// file*, whose name ends in `.lock`, excepted, and copies every file and folder of `backup` into
/// it, replacing files of the same path and creating missing folders; `mirror` with the folders
/// in that order.
///
/// # Errors
/// Returns the reason where `backup` is no folder or a file cannot be copied, created or removed.
pub fn restore(backup: &Path, source: &Path) -> Result<(), String> {
    mirror(backup, source)
}

// realises FR-511, FR-530, IR-214, IR-218
/// Makes the folder `to` hold what the folder `from` holds: creates `to` where it does not
/// exist, removes every file and folder of `to` that `from` does not hold, and copies every file
/// and folder of `from` into it, replacing files of the same path and creating missing folders;
/// a *lock file*, whose name ends in `.lock`, is neither removed from `to` nor copied from
/// `from`, since a lock is held by the process that took it and is no content, and neither is
/// a file that is no regular file, a socket of a running *build system* among them.
///
/// # Errors
/// Returns the reason where `from` is no folder or a file cannot be copied, created or removed.
pub fn mirror(from: &Path, to: &Path) -> Result<(), String> {
    if !from.is_dir() {
        return Err(format!("the folder '{}' does not exist", from.display()));
    }
    let io = |what: &str, path: &Path, error: std::io::Error| {
        format!("{what} '{}': {error}", path.display())
    };
    let is_lock = |relative: &Path| {
        relative
            .file_name()
            .is_some_and(|name| name.to_string_lossy().ends_with(LOCK_SUFFIX))
    };
    fs::create_dir_all(to).map_err(|error| io("cannot create", to, error))?;
    let mut extras = Vec::new();
    walk(to, Path::new(""), &mut extras).map_err(|error| io("cannot read", to, error))?;
    for (relative, is_dir) in extras.iter().rev() {
        let counterpart = from.join(relative);
        if is_lock(relative) || counterpart.exists() && counterpart.is_dir() == *is_dir {
            continue;
        }
        let target = to.join(relative);
        let removed = if *is_dir {
            fs::remove_dir_all(&target)
        } else {
            fs::remove_file(&target)
        };
        removed.map_err(|error| io("cannot remove", &target, error))?;
    }
    let mut content = Vec::new();
    walk(from, Path::new(""), &mut content).map_err(|error| io("cannot read", from, error))?;
    for (relative, is_dir) in content {
        if is_lock(&relative) {
            continue;
        }
        let source = from.join(&relative);
        let target = to.join(&relative);
        if is_dir {
            fs::create_dir_all(&target).map_err(|error| io("cannot create", &target, error))?;
        } else {
            fs::copy(&source, &target).map_err(|error| io("cannot copy", &source, error))?;
        }
    }
    Ok(())
}

/// Collects every folder and every regular file below `root`, as its path relative to `root`
/// and whether it is a folder, a folder before its content; a socket, a pipe or a device is
/// neither content nor collected, so that the socket files of a running *build system* beside
/// a *network file* are neither copied nor removed.
fn walk(root: &Path, relative: &Path, entries: &mut Vec<(PathBuf, bool)>) -> std::io::Result<()> {
    let mut names: Vec<_> = fs::read_dir(root.join(relative))?
        .map(|entry| entry.map(|entry| entry.file_name()))
        .collect::<Result<_, _>>()?;
    names.sort();
    for name in names {
        let child = relative.join(&name);
        let Ok(metadata) = fs::metadata(root.join(&child)) else {
            continue;
        };
        let is_dir = metadata.is_dir();
        if !is_dir && !metadata.is_file() {
            continue;
        }
        entries.push((child.clone(), is_dir));
        if is_dir {
            walk(root, &child, entries)?;
        }
    }
    Ok(())
}

// realises FR-512, FR-513
/// Yields the difference between `actual` and `expected`: `None` where they are equal, the
/// offset of the first differing octet, counted from 0, as `differs at octet <n>`, or the two
/// lengths as `<n> octets, <m> expected` where one is the beginning of the other.
pub fn difference(actual: &[u8], expected: &[u8]) -> Option<String> {
    if let Some(offset) = actual
        .iter()
        .zip(expected)
        .position(|(a, e)| a != e)
    {
        return Some(format!("differs at octet {offset}"));
    }
    if actual.len() != expected.len() {
        return Some(format!(
            "{} octets, {} expected",
            actual.len(),
            expected.len()
        ));
    }
    None
}

// realises FR-515
/// Yields the line of the *playback report* for `step` with `outcome`: `<line>: ok: <text>`,
/// `<line>: failed: <text>: <difference>` or `<line>: error: <text>: <reason>`.
pub fn report_line(step: &Step, outcome: &Outcome) -> String {
    match outcome {
        Outcome::Completed => format!("{}: ok: {}", step.line, step.text),
        Outcome::Failed(difference) => {
            format!("{}: failed: {}: {difference}", step.line, step.text)
        }
        Outcome::Error(reason) => format!("{}: error: {}: {reason}", step.line, step.text),
    }
}

// realises FR-517
/// Yields the exit code of a *verification run* with `outcomes`: 2 where one is an error, 1
/// where one is a failed verification and none is an error, and 0 otherwise.
pub fn exit_code_of(outcomes: &[Outcome]) -> i32 {
    if outcomes
        .iter()
        .any(|outcome| matches!(outcome, Outcome::Error(_)))
    {
        2
    } else if outcomes
        .iter()
        .any(|outcome| matches!(outcome, Outcome::Failed(_)))
    {
        1
    } else {
        0
    }
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : Automation::play, Automation::start_verification, Automation::step */
#[cfg(test)]
mod tests {
    use super::*;

    /// The *canonical form* of the script `SCRIPT`, as the *automation configuration* renders it.
    const CANONICAL: &str = "\n\nrestore\tbackup\tsrc\nbuild system\t../bin/genc3d\nnetwork\tsrc/n.gc3n\n\
        idle\tnetwork\t\t\t\n\nactivate\nopen node\tcopy\nanswer\tcreation\tcreate\tsilent\n\
        edit\tnode\tcopy\tinput\t0\t1\t12\t0\tddd\\n\\\"q\\\"\nidle\tnode\tcopy\tinput\t0\n\
        mode\tnode\tcopy\tinput\t0\thex\ndigits\tnode\tcopy\tinput\t0\thex\t4\t2\tAB\n\
        cursor\tnode\tcopy\tdsl\t\t3\t4\nselect\tfile\tsrc/in.txt\t\t\t1\t1\t2\t5\n\
        focus\tnode\tcopy\twindow\t\nfocus\tmain\t\twindow\t\nfocus\tnode\tcopy\tdiagnostics\t\n\
        long idle\tnetwork\t\t\t\nsettings\t2.0\t16\toff\t4\ton\toff\t3\ninsert key\tCtrl+I\n\
        typing mode\toverwrite\nanswer\tsave\tyes\nanswer\tcreation\tcancel\t\n\
        answer\tcreation failure\tcontinue\nundo\tnode\tcopy\tinput\t0\nredo\tnode\tcopy\tinput\t0\n\
        open file\tsrc/out.txt\nverify\tnode\tcopy\toutput\t0\texpected/out.gc3v\n\
        close node\tcopy\nclose file\tsrc/out.txt\ndeactivate\n";

    const SCRIPT: &str = "// a script\nautomation script\nrestore \"backup\" into \"src\"\n\
        build system \"../bin/genc3d\"\nnetwork \"src/n.gc3n\"   // the network\nidle network\n\n\
        activate\nopen node copy\nanswer creation create silent\n\
        edit node copy input 0 1:12 0 \"ddd\\n\\\"q\\\"\"\nidle node copy input 0\n\
        mode node copy input 0 hex\ndigits node copy input 0 hex 4 2 \"AB\"\ncursor node copy dsl 3:4\n\
        select file \"src/in.txt\" 1:1 2:5\nfocus node copy\nfocus main\nfocus node copy diagnostics\n\
        long idle network\nsettings idle 2.0 long idle 16 automatic off tab size 4 spaces on silent creation off delay 3\n\
        insert key \"Ctrl+I\"\ntyping mode overwrite\nanswer save yes\nanswer creation cancel\n\
        answer creation failure continue\nundo node copy input 0\nredo node copy input 0\n\
        open file \"src/out.txt\"\nverify node copy output 0 \"expected/out.gc3v\"\nclose node copy\n\
        close file \"src/out.txt\"\ndeactivate\n";

    fn folder() -> PathBuf {
        PathBuf::from("/scripts/run")
    }

    fn copy_input() -> EditorName {
        EditorName::Node {
            name: "copy".to_owned(),
            item: EditorItem::Input(0),
        }
    }

    #[test]
    fn the_canonical_form_is_read_into_the_actions_with_their_lines() {
        // FR-494, FR-495, FR-496
        let steps = steps_of_canonical(CANONICAL, SCRIPT, &folder()).expect("the form is read");
        let actions: Vec<&Action> = steps.iter().map(|step| &step.action).collect();
        let lines: Vec<usize> = steps.iter().map(|step| step.line).collect();
        assert_eq!(steps.len(), 30);
        assert_eq!(
            &lines[..9],
            &[3, 4, 5, 6, 8, 9, 10, 11, 12],
            "the lines of the script, the blank line passed over"
        );
        assert_eq!(
            (actions[0], actions[1], actions[2], actions[3], actions[4]),
            (
                &Action::Restore {
                    backup: "/scripts/run/backup".to_owned(),
                    source: "/scripts/run/src".to_owned()
                },
                &Action::BuildSystem("/scripts/bin/genc3d".to_owned()),
                &Action::Network("/scripts/run/src/n.gc3n".to_owned()),
                &Action::Idle(EditorName::Network),
                &Action::Activate
            )
        );
        assert_eq!(
            (actions[5], actions[6], actions[7]),
            (
                &Action::OpenNode("copy".to_owned()),
                &Action::Answer(DialogAnswer::Creation {
                    create: true,
                    silent: true
                }),
                &Action::Edit {
                    editor: copy_input(),
                    line: 1,
                    column: 12,
                    range: 0,
                    text: "ddd\n\"q\"".to_owned()
                }
            )
        );
        assert_eq!(
            (actions[9], actions[10], actions[11], actions[12]),
            (
                &Action::Mode {
                    editor: copy_input(),
                    mode: EditorMode::Hex
                },
                &Action::Digits {
                    editor: copy_input(),
                    mode: EditorMode::Hex,
                    start: 4,
                    removed: 2,
                    text: "AB".to_owned()
                },
                &Action::Cursor {
                    editor: EditorName::Node {
                        name: "copy".to_owned(),
                        item: EditorItem::MetaDsl
                    },
                    line: 3,
                    column: 4
                },
                &Action::Select {
                    editor: EditorName::File {
                        path: "/scripts/run/src/in.txt".to_owned()
                    },
                    from: (1, 1),
                    to: (2, 5)
                }
            )
        );
        assert_eq!(
            (actions[13], actions[14], actions[15], actions[16]),
            (
                &Action::Focus(FocusTarget::Window(WindowName::Node("copy".to_owned()))),
                &Action::Focus(FocusTarget::Window(WindowName::Main)),
                &Action::Focus(FocusTarget::Editor(EditorName::Node {
                    name: "copy".to_owned(),
                    item: EditorItem::Diagnostics
                })),
                &Action::LongIdle(EditorName::Network)
            )
        );
        assert_eq!(
            (actions[17], actions[18], actions[19]),
            (
                &Action::Settings(SettingsValues {
                    idle_time: 2000,
                    long_idle_time: 16000,
                    automatic: false,
                    tab_size: 4,
                    tabs_to_spaces: true,
                    silent_creation: false,
                    playback_delay: 3
                }),
                &Action::InsertKey("Ctrl+I".to_owned()),
                &Action::TypingMode(false)
            )
        );
        assert_eq!(
            (actions[20], actions[21], actions[22]),
            (
                &Action::Answer(DialogAnswer::Save(true)),
                &Action::Answer(DialogAnswer::Creation {
                    create: false,
                    silent: false
                }),
                &Action::Answer(DialogAnswer::CreationFailure(true))
            )
        );
        assert_eq!(
            (actions[25], actions[26], actions[27], actions[28], actions[29]),
            (
                &Action::OpenFile("/scripts/run/src/out.txt".to_owned()),
                &Action::Verify {
                    editor: EditorName::Node {
                        name: "copy".to_owned(),
                        item: EditorItem::Output(0)
                    },
                    file: "/scripts/run/expected/out.gc3v".to_owned()
                },
                &Action::CloseNode("copy".to_owned()),
                &Action::CloseFile("/scripts/run/src/out.txt".to_owned()),
                &Action::Deactivate
            )
        );
    }

    #[test]
    fn the_text_of_a_step_is_its_line_without_the_comment() {
        // FR-515
        let steps = steps_of_canonical(CANONICAL, SCRIPT, &folder()).expect("the form is read");
        assert_eq!(
            (steps[2].text.as_str(), steps[7].text.as_str()),
            (
                "network \"src/n.gc3n\"",
                "edit node copy input 0 1:12 0 \"ddd\\n\\\"q\\\"\""
            )
        );
    }

    #[test]
    fn a_line_of_the_canonical_form_that_is_not_read_is_the_error() {
        // FR-497
        let unknown = steps_of_canonical("\nwhistle\tloud\n", "automation script\nwhistle\n", &folder());
        let short = steps_of_canonical("\nedit\tnode\tcopy\n", "automation script\nedit\n", &folder());
        assert_eq!(
            (unknown, short),
            (
                Err("whistle\tloud".to_owned()),
                Err("edit\tnode\tcopy".to_owned())
            )
        );
    }

    #[test]
    fn the_escapes_of_a_text_are_resolved_and_unknown_ones_kept() {
        // FR-494, FR-062
        assert_eq!(
            unescaped("a\\nb\\tc\\\\d\\\"e\\rf\\xg\\"),
            "a\nb\tc\\d\"e\rf\\xg\\"
        );
    }

    #[test]
    fn a_path_is_resolved_against_the_folder_lexically() {
        // FR-494
        assert_eq!(
            (
                resolved(&folder(), "src/n.gc3n"),
                resolved(&folder(), "../bin/./genc3d"),
                resolved(&folder(), "/abs/x"),
                resolved(Path::new("rel"), "../../up")
            ),
            (
                "/scripts/run/src/n.gc3n".to_owned(),
                "/scripts/bin/genc3d".to_owned(),
                "/abs/x".to_owned(),
                "../up".to_owned()
            )
        );
    }

    #[test]
    fn a_position_is_an_offset_in_utf16_units_and_a_column_behind_the_line_is_its_end() {
        // FR-494, FR-505, FR-508
        let text = "ab\n𝄞cd\n";
        assert_eq!(
            (
                offset_of_position(text, 1, 1),
                offset_of_position(text, 1, 3),
                offset_of_position(text, 1, 9),
                offset_of_position(text, 2, 2),
                offset_of_position(text, 2, 4),
                offset_of_position(text, 3, 1),
                offset_of_position(text, 4, 1),
                offset_of_position(text, 0, 1),
                offset_of_position("", 1, 1)
            ),
            (
                Some(0),
                Some(2),
                Some(2),
                Some(5),
                Some(7),
                Some(8),
                None,
                None,
                Some(0)
            )
        );
    }

    #[test]
    fn the_offset_after_some_characters_counts_characters_not_units() {
        // FR-505
        let text = "𝄞cd";
        assert_eq!(
            (
                offset_after(text, 0, 1),
                offset_after(text, 0, 2),
                offset_after(text, 2, 5),
                offset_after("", 0, 3)
            ),
            (2, 3, 4, 0)
        );
    }

    #[test]
    fn a_difference_names_the_first_octet_or_the_lengths() {
        // FR-512, FR-513
        assert_eq!(
            (
                difference(b"abc", b"abc"),
                difference(b"abd", b"abc"),
                difference(b"ab", b"abc"),
                difference(b"abcd", b"abc"),
                difference(b"", b"")
            ),
            (
                None,
                Some("differs at octet 2".to_owned()),
                Some("2 octets, 3 expected".to_owned()),
                Some("4 octets, 3 expected".to_owned()),
                None
            )
        );
    }

    #[test]
    fn the_report_lines_and_the_exit_code_follow_the_outcomes() {
        // FR-515, FR-517
        let step = Step {
            line: 7,
            text: "activate".to_owned(),
            action: Action::Activate,
        };
        assert_eq!(
            (
                report_line(&step, &Outcome::Completed),
                report_line(&step, &Outcome::Failed("differs at octet 2".to_owned())),
                report_line(&step, &Outcome::Error("no node".to_owned())),
                exit_code_of(&[]),
                exit_code_of(&[Outcome::Completed]),
                exit_code_of(&[Outcome::Completed, Outcome::Failed(String::new())]),
                exit_code_of(&[Outcome::Failed(String::new()), Outcome::Error(String::new())]),
                Outcome::Completed.continues(),
                Outcome::Failed(String::new()).continues()
            ),
            (
                "7: ok: activate".to_owned(),
                "7: failed: activate: differs at octet 2".to_owned(),
                "7: error: activate: no node".to_owned(),
                0,
                0,
                1,
                2,
                true,
                false
            )
        );
    }

    #[test]
    fn the_fields_of_an_editor_name_are_the_four_the_front_end_takes() {
        // FR-494
        assert_eq!(
            (
                EditorName::Network.fields(),
                copy_input().fields(),
                EditorName::File {
                    path: "/p".to_owned()
                }
                .fields(),
                EditorName::Node {
                    name: "a".to_owned(),
                    item: EditorItem::Diagnostics
                }
                .fields()
            ),
            (
                ("network".to_owned(), String::new(), String::new(), 0),
                ("node".to_owned(), "copy".to_owned(), "input".to_owned(), 0),
                ("file".to_owned(), "/p".to_owned(), String::new(), 0),
                (
                    "node".to_owned(),
                    "a".to_owned(),
                    "diagnostics".to_owned(),
                    0
                )
            )
        );
    }

    #[test]
    fn gc3_is_found_beside_the_build_system() {
        // IR-212
        assert_eq!(
            (
                gc3_beside("/dist/bin/genc3d"),
                gc3_beside("genc3d"),
                configuration_path().file_name().map(|n| n.to_os_string())
            ),
            (
                PathBuf::from("/dist/bin/gc3"),
                PathBuf::from("gc3"),
                Some(CONFIGURATION_FILE_NAME.into())
            )
        );
    }

    #[test]
    fn a_missing_gc3_or_configuration_is_told_before_anything_runs() {
        // FR-497
        let missing = Path::new("/nowhere/at/all/gc3");
        let folder = std::env::temp_dir().join(format!("genc3wb-auto-cfg-{}", std::process::id()));
        fs::create_dir_all(&folder).expect("the temporary directory can be created");
        let configuration = folder.join(CONFIGURATION_FILE_NAME);
        fs::write(&configuration, "syntax\n").expect("the file can be written");
        let no_gc3 = read_script(Path::new("s.gc3s"), missing, &configuration);
        let no_configuration = read_script(Path::new("s.gc3s"), &configuration, missing);
        let _ = fs::remove_dir_all(&folder);
        assert_eq!(
            (no_gc3, no_configuration),
            (
                Err(ScriptError::Gc3NotFound(missing.display().to_string())),
                Err(ScriptError::ConfigurationNotFound(
                    missing.display().to_string()
                ))
            )
        );
    }

    #[test]
    fn restoring_makes_the_source_equal_to_the_backup_and_keeps_lock_files() {
        // FR-511, IR-214
        let root = std::env::temp_dir().join(format!("genc3wb-auto-restore-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let backup = root.join("backup");
        let source = root.join("src");
        fs::create_dir_all(backup.join("sub")).expect("the backup can be created");
        fs::create_dir_all(source.join("old")).expect("the source can be created");
        fs::write(backup.join("a.txt"), "A").expect("written");
        fs::write(backup.join("sub").join("b.txt"), "B").expect("written");
        fs::write(source.join("a.txt"), "old A").expect("written");
        fs::write(source.join("extra.txt"), "gone").expect("written");
        fs::write(source.join("old").join("x.txt"), "gone").expect("written");
        fs::write(source.join("n.gc3n.lock"), "held").expect("written");
        fs::write(backup.join("other.lock"), "stale").expect("written");
        let restored = restore(&backup, &source);
        let missing_backup = restore(&root.join("none"), &source);
        let a = fs::read_to_string(source.join("a.txt")).ok();
        let b = fs::read_to_string(source.join("sub").join("b.txt")).ok();
        let extra = source.join("extra.txt").exists();
        let old = source.join("old").exists();
        let lock = fs::read_to_string(source.join("n.gc3n.lock")).ok();
        let stale = source.join("other.lock").exists();
        let created = restore(&backup, &root.join("fresh")).map(|()| root.join("fresh/a.txt").is_file());
        let _ = fs::remove_dir_all(&root);
        assert_eq!(
            (restored, missing_backup.is_err(), a, b, extra, old, lock, stale, created),
            (
                Ok(()),
                true,
                Some("A".to_owned()),
                Some("B".to_owned()),
                false,
                false,
                Some("held".to_owned()),
                false,
                Ok(true)
            )
        );
    }

    #[test]
    fn a_time_is_read_in_seconds_with_decimals() {
        // FR-495
        assert_eq!(
            (
                milliseconds_of("2.0"),
                milliseconds_of("16"),
                milliseconds_of("0.25"),
                milliseconds_of("1.2345"),
                milliseconds_of("x")
            ),
            (Some(2000), Some(16000), Some(250), None, None)
        );
    }

    #[test]
    fn a_comment_is_cut_outside_a_text_alone() {
        // FR-493, FR-515
        assert_eq!(
            (
                without_comment("activate // go"),
                without_comment("network \"a//b\" // c"),
                without_comment("edit x 1:1 0 \"q\\\"//\""),
                without_comment("plain")
            ),
            (
                "activate ",
                "network \"a//b\" ",
                "edit x 1:1 0 \"q\\\"//\"",
                "plain"
            )
        );
    }
}
