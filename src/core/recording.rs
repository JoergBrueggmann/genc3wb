//! The *recorder*: the *automatic recording* and the *individual recording* of an
//! *automation script*, every action rendered in the *automation notation* and written as it
//! arises, the *recorded text* of every code editor from which the `edit` lines are derived, the
//! cursor and selection lines coalesced, the *expected files* and the *precondition backup*.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::automation::{
    Action, DialogAnswer, EditorItem, EditorName, FocusTarget, SettingsValues, WindowName,
};
use crate::core::file_lock::iso_of_unix_seconds;
use crate::core::octet_view::EditorMode;
use crate::core::settings::playback_delay_text;
use crate::core::text_increment::increment_between;

use std::fs::{self, File};
use std::io::Write;
use std::path::{Component, Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

// realises FR-520, IR-216
/// The name of the *recording folder* beside the folder of the executable, the prefix of the
/// folder of one start in it, and the name of the file of the *automatic recording*.
pub const RECORDING_FOLDER_NAME: &str = "rec";
pub const START_FOLDER_PREFIX: &str = "rec_";
pub const AUTOMATIC_FILE_NAME: &str = "usage.gc3s";

// realises FR-531
/// The suffix of an *expected file*.
pub const EXPECTED_SUFFIX: &str = "gc3v";

// realises FR-493, FR-519
/// The head line of an *automation script*.
const HEAD_LINE: &str = "automation script";

// realises FR-519 to FR-533
/// The *recorder*.
#[derive(Debug, Default)]
pub struct Recorder {
    /// the *recording* that runs, `None` while none does
    recording: Option<Recording>,
    /// whether a *playback* runs, during which nothing is recorded (FR-533)
    suspended: bool,
    /// the failure of the last start of a *recording* or write of an *expected file*, for the
    /// status bar; empty where the last succeeded (FR-532)
    failure: String,
    /// the code editor that holds or held the keyboard focus last and its cursor, kept across
    /// the recordings, so that an *individual recording* records the focus and the cursor it
    /// finds (FR-540)
    focused: Option<(EditorName, usize, usize)>,
}

// realises FR-540
/// The state an *individual recording* finds, gathered by the *player* from the network editor,
/// the *workbench object* and the *recorder*, and recorded as the commands that recover it.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct FoundState {
    /// the *build system* in force, recorded first, since a *playback* names it anew (FR-520)
    pub build_system: String,
    /// the *network file* named, empty where none is
    pub network: String,
    /// the files the network works on, from the last *network response*, for the backup
    pub files: Vec<String>,
    /// whether the network is an *active network*; `None` where no mode is kept
    pub active: Option<bool>,
    /// the *nodes* whose *node windows* are presented, in the order they were opened
    pub node_windows: Vec<String>,
    /// the files whose *editor windows* are presented, in the order they were opened
    pub editor_windows: Vec<String>,
    /// per code editor whose text differs from the content of its file: the editor, the content
    /// of the file and the text
    pub edited: Vec<(EditorName, String, String)>,
    /// whether the *typing mode* is the insert mode
    pub insert_mode: bool,
}

// realises FR-519, FR-522, FR-525, FR-529, FR-531
/// One *recording*.
#[derive(Debug)]
struct Recording {
    /// the gc3s-file, opened for writing, and its path
    file: File,
    path: PathBuf,
    /// the folder of the gc3s-file, against which the paths are written, and the name of the file
    /// without its suffix, by which the *expected files* are named
    folder: PathBuf,
    name: String,
    /// the number of *expected files* written by this *recording*
    expected: usize,
    /// the number of *backup folders* taken by this *recording* (IR-218)
    backups: usize,
    /// the `network` or `new network` line held back until the first *build* after the loading
    /// has answered and the *precondition backup* is saved (FR-539)
    held: Option<Action>,
    /// the lines of the actions that arose while a line is held back, written after it
    deferred: Vec<String>,
    /// the code editors the user edited, in the order of their first edit, each with its
    /// *recorded text* and its current text
    texts: Vec<EditedText>,
    /// the cursor or selection line held back, so that the next one of the same code editor
    /// replaces it; written before any other line
    pending: Option<Action>,
    /// the code editor that holds or held the keyboard focus last, for 'Store to verify on
    /// replay'
    focused: Option<EditorName>,
}

// realises FR-522, FR-523
/// The texts of one code editor.
#[derive(Debug, Clone, PartialEq, Eq)]
struct EditedText {
    editor: EditorName,
    recorded: String,
    current: String,
}

impl Recorder {
    // realises FR-519, FR-520, FR-528
    /// Starts a *recording* into the file at `path`, created or truncated, its folders created:
    /// writes `comment_line` of `version` and `time`, the head line and `network <path>` where
    /// `network` names a *network file*; a *recording* that runs is stopped first.
    ///
    /// # Errors
    /// Returns the reason where the folders or the file cannot be created, kept as the failure.
    pub fn start(
        &mut self,
        path: &Path,
        version: &str,
        time: &str,
        settings: &Path,
    ) -> Result<(), String> {
        self.stop();
        let created = (|| {
            if let Some(folder) = path.parent() {
                fs::create_dir_all(folder)
                    .map_err(|error| format!("cannot create '{}': {error}", folder.display()))?;
            }
            File::create(path)
                .map_err(|error| format!("cannot create '{}': {error}", path.display()))
        })();
        let file = match created {
            Ok(file) => file,
            Err(reason) => {
                self.failure = reason.clone();
                return Err(reason);
            }
        };
        let folder = path.parent().map(Path::to_path_buf).unwrap_or_default();
        let name = path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_default();
        self.recording = Some(Recording {
            file,
            path: path.to_path_buf(),
            folder,
            name,
            expected: 0,
            backups: 0,
            held: None,
            deferred: Vec::new(),
            texts: Vec::new(),
            pending: None,
            focused: None,
        });
        self.failure.clear();
        self.write_line(&comment_line(version, time));
        self.write_line(HEAD_LINE);
        self.write_line("init");
        if settings.is_file() {
            match self.save_file(settings) {
                Ok(saved) => {
                    let line = self.line_of(&Action::SettingsFile(saved));
                    self.write_line(&line);
                }
                Err(reason) => self.failure = reason,
            }
        }
        Ok(())
    }

    // realises FR-539
    /// Holds the `network` or `new network` line `action` back until `back_up_network`, since
    /// the files of the network are known once the first *build* has answered; a line held
    /// before is written first, where a *build* never answered it.
    pub fn hold_network_line(&mut self, action: &Action) {
        if !self.active() {
            return;
        }
        self.flush_held();
        if let Some(recording) = &mut self.recording {
            recording.held = Some(action.clone());
        }
    }

    // realises FR-539, IR-218
    /// Saves the *precondition backup* of the network: `network_file` and each of `files` that
    /// exists copied to the next *backup folder* as '<m>.bak', one `restore` line per file that
    /// exists and one `remove` line per file that does not, in that order, then the line held
    /// back; a file that cannot be copied gets no line and its reason is kept as the failure.
    /// Nothing while no line is held back, so that a *build* after an edit saves nothing anew.
    pub fn back_up_network(&mut self, network_file: &str, files: &[String]) {
        if !self.active()
            || self
                .recording
                .as_ref()
                .is_none_or(|recording| recording.held.is_none())
        {
            return;
        }
        self.flush_edits();
        self.flush_pending();
        if let Some(recording) = &mut self.recording {
            recording.backups += 1;
        }
        let mut paths: Vec<String> = Vec::with_capacity(files.len() + 1);
        paths.push(network_file.to_owned());
        for file in files {
            if !paths.contains(file) {
                paths.push(file.clone());
            }
        }
        for file in paths {
            let path = Path::new(&file);
            let action = if path.is_file() {
                match self.save_file(path) {
                    Ok(saved) => Action::Restore {
                        saved,
                        file: file.clone(),
                    },
                    Err(reason) => {
                        self.failure = reason;
                        continue;
                    }
                }
            } else if path.exists() {
                continue;
            } else {
                Action::Remove(file.clone())
            };
            let line = self.line_of(&action);
            self.write_now(&line);
        }
        self.flush_held();
    }

    // realises FR-520, FR-528, FR-540
    /// Records the state an *individual recording* finds: `build system <path>` of the *build
    /// system* in force, as the *automatic recording* records it at the start, then the backup of
    /// its network and `network <path>`, `activate` or `deactivate` by `active`, `open node` and
    /// `open file` per window, `edit` per edited code editor by `increment_between` of the
    /// content of its file and its text, `typing mode`, and `focus` and `cursor` of the code
    /// editor kept as focused; nothing beyond the *build system* where `state` names no network.
    pub fn record_state(&mut self, state: &FoundState) {
        if !self.active() {
            return;
        }
        if !state.build_system.is_empty() {
            self.record(&Action::BuildSystem(state.build_system.clone()));
        }
        if state.network.is_empty() {
            return;
        }
        self.hold_network_line(&Action::Network(state.network.clone()));
        self.back_up_network(&state.network, &state.files);
        match state.active {
            Some(true) => self.record(&Action::Activate),
            Some(false) => self.record(&Action::Deactivate),
            None => {}
        }
        for name in &state.node_windows {
            self.record(&Action::OpenNode(name.clone()));
        }
        for path in &state.editor_windows {
            self.record(&Action::OpenFile(path.clone()));
        }
        for (editor, content, text) in &state.edited {
            let increment = increment_between(content, text);
            let (line, column) = position_of_offset(content, increment.position);
            self.record(&Action::Edit {
                editor: editor.clone(),
                line,
                column,
                range: increment.range,
                text: increment.text,
            });
            self.reset_text(editor, text);
        }
        self.record(&Action::TypingMode(state.insert_mode));
        if let Some((editor, line, column)) = self.focused.clone() {
            self.record(&Action::Focus(FocusTarget::Editor(editor.clone())));
            self.record(&Action::Cursor {
                editor,
                line,
                column,
            });
        }
    }

    /// Copies the file at `source` into the current *backup folder* 'backup/<n>' beside the
    /// gc3s-file as the next '<m>.bak', the folder created where it does not exist, the first
    /// backup folder taken where none is; yields the absolute path of the copy.
    ///
    /// # Errors
    /// Returns the reason where the folder cannot be created or the file cannot be copied.
    fn save_file(&mut self, source: &Path) -> Result<String, String> {
        let Some(recording) = &mut self.recording else {
            return Err("no recording runs".to_owned());
        };
        if recording.backups == 0 {
            recording.backups = 1;
        }
        let folder = recording
            .folder
            .join("backup")
            .join(recording.backups.to_string());
        fs::create_dir_all(&folder)
            .map_err(|error| format!("cannot create '{}': {error}", folder.display()))?;
        let count = fs::read_dir(&folder)
            .map(|entries| entries.filter_map(Result::ok).count())
            .unwrap_or(0);
        let target = folder.join(format!("{}.bak", count + 1));
        fs::copy(source, &target)
            .map_err(|error| format!("cannot copy '{}': {error}", source.display()))?;
        Ok(target.to_string_lossy().into_owned())
    }

    /// Writes the `network` or `new network` line held back, where there is one, and then the
    /// lines held back behind it.
    fn flush_held(&mut self) {
        let held = self
            .recording
            .as_mut()
            .and_then(|recording| recording.held.take());
        if let Some(action) = held {
            let line = self.line_of(&action);
            self.write_now(&line);
            let deferred = self
                .recording
                .as_mut()
                .map(|recording| std::mem::take(&mut recording.deferred))
                .unwrap_or_default();
            for line in deferred {
                self.write_now(&line);
            }
        }
    }

    // realises FR-529
    /// Writes the pending `edit` lines and the pending cursor or selection line and closes the
    /// file; nothing where none runs.
    pub fn stop(&mut self) {
        self.flush_held();
        if self.recording.is_none() {
            return;
        }
        self.flush_edits();
        self.flush_pending();
        self.recording = None;
    }

    // realises FR-528, FR-529, FR-532
    /// Yields whether a *recording* runs.
    pub fn is_running(&self) -> bool {
        self.recording.is_some()
    }

    /// Yields the path of the gc3s-file of the *recording* that runs.
    pub fn path(&self) -> Option<&Path> {
        self.recording
            .as_ref()
            .map(|recording| recording.path.as_path())
    }

    /// Yields the failure of the last start of a *recording* or write of an *expected file*,
    /// empty where it succeeded.
    pub fn failure(&self) -> &str {
        &self.failure
    }

    // realises FR-532
    /// Keeps `reason` as the failure of the last write of an *expected file*: the code editor
    /// to store could not be read.
    pub fn note_failure(&mut self, reason: String) {
        self.failure = reason;
    }

    // realises FR-533
    /// Suspends the *recorder* while a *playback* runs, and resumes it.
    pub fn suspend(&mut self, suspended: bool) {
        self.suspended = suspended;
    }

    // realises FR-521, FR-524, FR-527
    /// Records `action` as a line: the pending `edit` lines of every code editor and the pending
    /// cursor or selection line written first; `close node` and `close file` forget the focused
    /// code editor of that window. Nothing while no *recording* runs or the *recorder* is
    /// suspended, as for every other function that records.
    pub fn record(&mut self, action: &Action) {
        if !self.active() {
            return;
        }
        self.flush_edits();
        self.flush_pending();
        let line = self.line_of(action);
        self.write_line(&line);
        if let Some(recording) = &mut self.recording {
            let closed = match (action, &recording.focused) {
                (Action::CloseNode(name), Some(EditorName::Node { name: focused, .. })) => {
                    name == focused
                }
                (Action::CloseFile(path), Some(EditorName::File { path: focused })) => {
                    path == focused
                }
                _ => false,
            };
            if closed {
                recording.focused = None;
            }
        }
    }

    // realises FR-522
    /// Takes that the text of the code editor `editor` changed from `before` to `after` by the
    /// user: its *recorded text* is `before` where the editor was not edited yet, its current
    /// text becomes `after`, and the pending cursor or selection line is written, since a text
    /// change ends a sequence of cursor moves.
    pub fn text_changed(&mut self, editor: &EditorName, before: &str, after: &str) {
        if !self.active() {
            return;
        }
        self.flush_pending();
        let entry = self.entry(editor, before);
        entry.current = after.to_owned();
    }

    // realises FR-525
    /// Holds a cursor line of `editor` back as the pending one, replacing the pending one; the
    /// pending `edit` lines are written first.
    pub fn cursor(&mut self, editor: &EditorName, line: usize, column: usize) {
        if let Some((focused, kept_line, kept_column)) = &mut self.focused
            && focused == editor
        {
            *kept_line = line;
            *kept_column = column;
        }
        self.hold(Action::Cursor {
            editor: editor.clone(),
            line,
            column,
        });
    }

    // realises FR-525
    /// Holds a selection line of `editor` back as the pending one, as `cursor` does.
    pub fn select(&mut self, editor: &EditorName, from: (usize, usize), to: (usize, usize)) {
        self.hold(Action::Select {
            editor: editor.clone(),
            from,
            to,
        });
    }

    // realises FR-522, FR-523
    /// Takes the expiry of the *idle time* of `editor`: writes the pending `edit` lines, and
    /// `idle <editor>` where the expiry `provided` a *text increment* or octets, which is what
    /// a *playback* of the line does again.
    pub fn idle(&mut self, editor: &EditorName, provided: bool) {
        if !self.active() {
            return;
        }
        self.flush_edits();
        if provided {
            self.record(&Action::Idle(editor.clone()));
        }
    }

    // realises FR-523
    /// Takes the expiry of the *long idle time* of `editor`: records `long idle <editor>` where
    /// `completed` a *placeholder*, the *recorded text* becoming `text`.
    pub fn long_idle(&mut self, editor: &EditorName, completed: bool, text: &str) {
        if !self.active() || !completed {
            return;
        }
        self.record(&Action::LongIdle(editor.clone()));
        self.reset_text(editor, text);
    }

    // realises FR-524
    /// Records a `digits`, `undo` or `redo` action of its editor, whose *recorded text* and
    /// current text become `text` after it.
    pub fn record_octet_edit(&mut self, action: &Action, text: &str) {
        if !self.active() {
            return;
        }
        let editor = match action {
            Action::Digits { editor, .. } | Action::Undo(editor) | Action::Redo(editor) => {
                editor.clone()
            }
            _ => return,
        };
        self.record(action);
        self.reset_text(&editor, text);
    }

    // realises FR-526, FR-531
    /// Records `focus <target>`, and keeps the code editor where the target is one.
    pub fn focus(&mut self, target: &FocusTarget) {
        if !self.active() {
            return;
        }
        self.record(&Action::Focus(target.clone()));
        if let (FocusTarget::Editor(editor), Some(recording)) = (target, &mut self.recording) {
            recording.focused = Some(editor.clone());
            match &self.focused {
                Some((focused, _, _)) if focused == editor => {}
                _ => self.focused = Some((editor.clone(), 1, 1)),
            }
        }
    }

    // realises FR-531
    /// Yields the code editor that holds or held the keyboard focus last.
    pub fn focused_editor(&self) -> Option<EditorName> {
        self.recording
            .as_ref()
            .and_then(|recording| recording.focused.clone())
    }

    // realises FR-531, IR-218
    /// Writes `octets` into the next *expected file* `<name>-<n>.gc3v` beside the gc3s-file, the
    /// pending `edit` lines written first, and records `verify <editor> <path>`; yields the path.
    ///
    /// # Errors
    /// Returns the reason where no *recording* runs or the file cannot be written, kept as the
    /// failure.
    pub fn store_expected(
        &mut self,
        editor: &EditorName,
        octets: &[u8],
    ) -> Result<PathBuf, String> {
        if !self.active() {
            let reason = "no recording runs".to_owned();
            self.failure = reason.clone();
            return Err(reason);
        }
        self.flush_edits();
        let path = {
            let recording = self
                .recording
                .as_ref()
                .expect("a recording runs while the recorder is active");
            recording.folder.join(format!(
                "{}-{}.{EXPECTED_SUFFIX}",
                recording.name, recording.expected
            ))
        };
        if let Err(error) = fs::write(&path, octets) {
            let reason = format!("cannot write '{}': {error}", path.display());
            self.failure = reason.clone();
            return Err(reason);
        }
        if let Some(recording) = &mut self.recording {
            recording.expected += 1;
        }
        self.failure.clear();
        self.record(&Action::Verify {
            editor: editor.clone(),
            file: path.to_string_lossy().into_owned(),
        });
        Ok(path)
    }

    /// Yields whether a *recording* runs and the *recorder* is not suspended.
    fn active(&self) -> bool {
        self.recording.is_some() && !self.suspended
    }

    /// Yields the line of `action` against the folder of the *recording*.
    fn line_of(&self, action: &Action) -> String {
        let folder = self
            .recording
            .as_ref()
            .map(|recording| recording.folder.as_path())
            .unwrap_or_else(|| Path::new(""));
        line_of(action, folder)
    }

    /// Writes `line` and a line feed to the file and flushes it, or holds it back while a
    /// `network` line is held back, so that it follows that line (FR-539); a line that cannot be
    /// written is passed over, the next one being tried again.
    fn write_line(&mut self, line: &str) {
        if let Some(recording) = &mut self.recording {
            if recording.held.is_some() {
                recording.deferred.push(line.to_owned());
                return;
            }
            let _ = writeln!(recording.file, "{line}");
            let _ = recording.file.flush();
        }
    }

    /// Writes `line` and a line feed to the file and flushes it, whether a line is held back or
    /// not: the `restore` and `remove` lines of a backup and the line held back itself.
    fn write_now(&mut self, line: &str) {
        if let Some(recording) = &mut self.recording {
            let _ = writeln!(recording.file, "{line}");
            let _ = recording.file.flush();
        }
    }

    /// Holds `action`, a cursor or selection line, back as the pending one, the pending `edit`
    /// lines written first.
    fn hold(&mut self, action: Action) {
        if !self.active() {
            return;
        }
        self.flush_edits();
        if let Some(recording) = &mut self.recording {
            recording.pending = Some(action);
        }
    }

    /// Writes the pending cursor or selection line, where there is one.
    fn flush_pending(&mut self) {
        let pending = self
            .recording
            .as_mut()
            .and_then(|recording| recording.pending.take());
        if let Some(action) = pending {
            let line = self.line_of(&action);
            self.write_line(&line);
        }
    }

    /// Writes the `edit` line of every code editor whose current text differs from its
    /// *recorded text*, in the order of the editors, the *recorded text* becoming the current.
    fn flush_edits(&mut self) {
        let edits: Vec<Action> = {
            let Some(recording) = &mut self.recording else {
                return;
            };
            recording
                .texts
                .iter_mut()
                .filter(|entry| entry.recorded != entry.current)
                .map(|entry| {
                    let increment = increment_between(&entry.recorded, &entry.current);
                    let (line, column) = position_of_offset(&entry.recorded, increment.position);
                    entry.recorded.clone_from(&entry.current);
                    Action::Edit {
                        editor: entry.editor.clone(),
                        line,
                        column,
                        range: increment.range,
                        text: increment.text,
                    }
                })
                .collect()
        };
        for edit in edits {
            let line = self.line_of(&edit);
            self.write_line(&line);
        }
    }

    /// Yields the entry of `editor`, created with the *recorded text* `before` where none is.
    fn entry(&mut self, editor: &EditorName, before: &str) -> &mut EditedText {
        let recording = self
            .recording
            .as_mut()
            .expect("the recorder is active while an entry is taken");
        let index = match recording
            .texts
            .iter()
            .position(|entry| entry.editor == *editor)
        {
            Some(index) => index,
            None => {
                recording.texts.push(EditedText {
                    editor: editor.clone(),
                    recorded: before.to_owned(),
                    current: before.to_owned(),
                });
                recording.texts.len() - 1
            }
        };
        &mut recording.texts[index]
    }

    /// Makes `text` the *recorded text* and the current text of `editor`.
    fn reset_text(&mut self, editor: &EditorName, text: &str) {
        if self.recording.is_none() {
            return;
        }
        let entry = self.entry(editor, text);
        entry.recorded = text.to_owned();
        entry.current = text.to_owned();
    }
}

// realises FR-519
/// Yields the line of `action` in the *automation notation*, without its line feed: the inverse
/// of the reading of the *canonical form*, a text escaped by `escaped`, a path written by
/// `relative` against `folder`, a time in seconds with one decimal.
pub fn line_of(action: &Action, folder: &Path) -> String {
    let path = |path: &str| format!("\"{}\"", escaped(&relative(folder, path)));
    let text = |text: &str| format!("\"{}\"", escaped(text));
    let editor = |editor: &EditorName| editor_text(editor, folder);
    let position = |(line, column): (usize, usize)| format!("{line}:{column}");
    let switch = |on: bool| if on { "on" } else { "off" };
    match action {
        Action::Network(p) => format!("network {}", path(p)),
        Action::BuildSystem(p) => format!("build system {}", path(p)),
        Action::NewNetwork(p) => format!("new network {}", path(p)),
        Action::OpenNode(name) => format!("open node {name}"),
        Action::CloseNode(name) => format!("close node {name}"),
        Action::OpenFile(p) => format!("open file {}", path(p)),
        Action::CloseFile(p) => format!("close file {}", path(p)),
        Action::Activate => "activate".to_owned(),
        Action::Deactivate => "deactivate".to_owned(),
        Action::Edit {
            editor: e,
            line,
            column,
            range,
            text: t,
        } => format!(
            "edit {} {} {range} {}",
            editor(e),
            position((*line, *column)),
            text(t)
        ),
        Action::Digits {
            editor: e,
            mode,
            start,
            removed,
            text: t,
        } => format!(
            "digits {} {} {start} {removed} {}",
            editor(e),
            mode_text(*mode),
            text(t)
        ),
        Action::Idle(e) => format!("idle {}", editor(e)),
        Action::Delay(milliseconds) => format!("delay {milliseconds} ms"),
        Action::LongIdle(e) => format!("long idle {}", editor(e)),
        Action::Cursor {
            editor: e,
            line,
            column,
        } => format!("cursor {} {}", editor(e), position((*line, *column))),
        Action::Select {
            editor: e,
            from,
            to,
        } => format!("select {} {} {}", editor(e), position(*from), position(*to)),
        Action::Focus(FocusTarget::Editor(e)) => format!("focus {}", editor(e)),
        Action::Focus(FocusTarget::Window(WindowName::Main)) => "focus main".to_owned(),
        Action::Focus(FocusTarget::Window(WindowName::Node(name))) => format!("focus node {name}"),
        Action::Mode { editor: e, mode } => format!("mode {} {}", editor(e), mode_text(*mode)),
        Action::Undo(e) => format!("undo {}", editor(e)),
        Action::Redo(e) => format!("redo {}", editor(e)),
        Action::Settings(values) => settings_text(values, switch),
        Action::InsertKey(name) => format!("insert key {}", text(name)),
        Action::TypingMode(insert) => format!(
            "typing mode {}",
            if *insert { "insert" } else { "overwrite" }
        ),
        Action::Answer(DialogAnswer::Save(save)) => {
            format!("answer save {}", if *save { "yes" } else { "no" })
        }
        Action::Answer(DialogAnswer::Creation { create, silent }) => match (create, silent) {
            (true, true) => "answer creation create silent".to_owned(),
            (true, false) => "answer creation create".to_owned(),
            (false, _) => "answer creation cancel".to_owned(),
        },
        Action::Answer(DialogAnswer::CreationFailure(proceed)) => format!(
            "answer creation failure {}",
            if *proceed { "continue" } else { "cancel" }
        ),
        Action::Init => "init".to_owned(),
        Action::CloseNetwork => "close network".to_owned(),
        Action::SettingsFile(p) => format!("settings file {}", path(p)),
        Action::Restore { saved, file } => format!("restore {} into {}", path(saved), path(file)),
        Action::Remove(p) => format!("remove {}", path(p)),
        Action::Verify { editor: e, file } => format!("verify {} {}", editor(e), path(file)),
    }
}

/// Yields the `settings` line of `values`.
fn settings_text(values: &SettingsValues, switch: impl Fn(bool) -> &'static str) -> String {
    format!(
        "settings idle {} long idle {} automatic {} tab size {} spaces {} silent creation {} delay {}",
        seconds_text(values.idle_time),
        seconds_text(values.long_idle_time),
        switch(values.automatic),
        values.tab_size,
        switch(values.tabs_to_spaces),
        switch(values.silent_creation),
        values
            .playback_delay
            .map_or_else(|| "off".to_owned(), playback_delay_text)
    )
}

/// Yields `milliseconds` as seconds with one decimal, rounded down to a tenth.
fn seconds_text(milliseconds: u32) -> String {
    format!("{}.{}", milliseconds / 1000, milliseconds % 1000 / 100)
}

/// Yields the name of a code editor as FR-494 states it, a path written against `folder`.
fn editor_text(editor: &EditorName, folder: &Path) -> String {
    match editor {
        EditorName::Network => "network".to_owned(),
        EditorName::Node { name, item } => match item {
            EditorItem::MetaDsl => format!("node {name} dsl"),
            EditorItem::Input(index) => format!("node {name} input {index}"),
            EditorItem::Output(index) => format!("node {name} output {index}"),
            EditorItem::Diagnostics => format!("node {name} diagnostics"),
        },
        EditorName::File { path } => format!("file \"{}\"", escaped(&relative(folder, path))),
    }
}

/// Yields the word of an *editor mode*.
fn mode_text(mode: EditorMode) -> &'static str {
    match mode {
        EditorMode::Txt => "txt",
        EditorMode::Hex => "hex",
        EditorMode::Bin => "bin",
    }
}

// realises FR-519
/// Yields `text` with a backslash, a double quote, a line feed, a carriage return and a tab
/// written as the escapes of FR-062.
pub fn escaped(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            other => result.push(other),
        }
    }
    result
}

// realises FR-519
/// Yields `path` relative to `folder`, both absolute: the segments of `path` behind the common
/// prefix, preceded by one `..` per segment of `folder` behind it, joined by `/`; `path` as it is
/// where it is not absolute or the two share no root; `.` where they are equal.
pub fn relative(folder: &Path, path: &str) -> String {
    let target = Path::new(path);
    if !target.is_absolute() || !folder.is_absolute() {
        return path.to_owned();
    }
    let base: Vec<Component> = folder.components().collect();
    let goal: Vec<Component> = target.components().collect();
    let is_root = |component: &Component| !matches!(component, Component::Normal(_));
    let base_root: Vec<&Component> = base.iter().take_while(|c| is_root(c)).collect();
    let goal_root: Vec<&Component> = goal.iter().take_while(|c| is_root(c)).collect();
    if base_root != goal_root {
        return path.to_owned();
    }
    let common = base
        .iter()
        .zip(goal.iter())
        .take_while(|(a, b)| a == b)
        .count();
    let mut segments: Vec<String> =
        std::iter::repeat_n("..".to_owned(), base.len() - common).collect();
    segments.extend(
        goal[common..]
            .iter()
            .map(|component| component.as_os_str().to_string_lossy().into_owned()),
    );
    if segments.is_empty() {
        ".".to_owned()
    } else {
        segments.join("/")
    }
}

// realises FR-522
/// Yields the line and the column, both counted from 1, of the character at the offset
/// `characters` of `text`, counted in characters; the end of the text for an offset beyond it.
pub fn position_of_offset(text: &str, characters: usize) -> (usize, usize) {
    position_where(text, |_, index| index < characters)
}

// realises FR-525
/// Yields the line and the column, both counted from 1, of the character at the offset `units`
/// of `text`, counted in UTF-16 code units as a `TextArea` counts positions; the end of the text
/// for an offset beyond it.
pub fn position_of_utf16_offset(text: &str, units: usize) -> (usize, usize) {
    let mut passed = 0;
    position_where(text, move |character, _| {
        passed += character.len_utf16();
        passed <= units
    })
}

/// Yields the line and the column behind the characters of `text` for which `before` holds,
/// taken from the start.
fn position_where(text: &str, mut before: impl FnMut(char, usize) -> bool) -> (usize, usize) {
    let (mut line, mut column) = (1, 1);
    for (index, character) in text.chars().enumerate() {
        if !before(character, index) {
            break;
        }
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    (line, column)
}

// realises FR-520, IR-216
/// Yields the folder of a start in the *recording folder* beside `executable_folder`:
/// `rec/rec_<now>`, `now` being a name `local_time_name` yields.
pub fn start_folder(executable_folder: &Path, now: &str) -> PathBuf {
    executable_folder
        .parent()
        .unwrap_or(executable_folder)
        .join(RECORDING_FOLDER_NAME)
        .join(format!("{START_FOLDER_PREFIX}{now}"))
}

// realises FR-520, IR-216
/// Yields the local date and time as `YYYY-MM-DD_HH-MM-SS`, by the program `date`, and the UTC of
/// `iso_of_unix_seconds` in the same form where it cannot be run.
pub fn local_time_name() -> String {
    let local = Command::new("date")
        .arg("+%Y-%m-%d_%H-%M-%S")
        .output()
        .ok()
        .filter(|output| output.status.success())
        .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
        .filter(|name| name.len() == 19);
    local.unwrap_or_else(|| {
        let seconds = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|since| since.as_secs())
            .unwrap_or(0);
        let iso = iso_of_unix_seconds(seconds);
        let (date, clock) = iso.split_once('T').unwrap_or((&iso, ""));
        format!("{date}_{}", clock.trim_end_matches('Z').replace(':', "-"))
    })
}

// realises FR-519
/// Yields the comment line of a *recording*: `// Recorded by genc³wb <version> on <date> at
/// <time>`, the date and the time taken from a name `local_time_name` yields.
pub fn comment_line(version: &str, time: &str) -> String {
    let (date, clock) = time.split_once('_').unwrap_or((time, ""));
    format!(
        "// Recorded by genc³wb {version} on {date} at {}",
        clock.replace('-', ":")
    )
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : Automation::start_recording, Automation::stop_recording,
 *                    Automation::store_focused, Automation::record_window_focus,
 *                    InputGroup::record_cursor, InputGroup::record_focus, InputGroup::record_mode,
 *                    OutputGroup::record_focus, OutputGroup::record_mode, Workbench::default */
#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::automation::{read_script, resolved, steps_of_canonical, unescaped};

    fn folder() -> PathBuf {
        PathBuf::from("/scripts/run")
    }

    fn copy_input() -> EditorName {
        EditorName::Node {
            name: "copy".to_owned(),
            item: EditorItem::Input(0),
        }
    }

    fn temp(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("genc3wb-rec-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        root
    }

    fn lines_of(path: &Path) -> Vec<String> {
        fs::read_to_string(path)
            .expect("the recording is read")
            .lines()
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn every_action_is_rendered_in_the_notation() {
        // FR-519, FR-521, FR-524, FR-527
        let folder = folder();
        let values = SettingsValues {
            idle_time: 2000,
            long_idle_time: 16000,
            automatic: false,
            tab_size: 4,
            tabs_to_spaces: true,
            silent_creation: false,
            playback_delay: Some(283),
        };
        let actions = vec![
            Action::Network("/scripts/run/src/n.gc3n".to_owned()),
            Action::BuildSystem("/scripts/bin/genc3d".to_owned()),
            Action::NewNetwork("/scripts/run/new.gc3n".to_owned()),
            Action::OpenNode("copy".to_owned()),
            Action::CloseNode("copy".to_owned()),
            Action::OpenFile("/scripts/run/src/in.txt".to_owned()),
            Action::CloseFile("/scripts/run/src/in.txt".to_owned()),
            Action::Activate,
            Action::Deactivate,
            Action::Edit {
                editor: copy_input(),
                line: 1,
                column: 12,
                range: 0,
                text: "ddd\n\"q\"\t\\".to_owned(),
            },
            Action::Digits {
                editor: copy_input(),
                mode: EditorMode::Hex,
                start: 4,
                removed: 2,
                text: "AB".to_owned(),
            },
            Action::Idle(EditorName::Network),
            Action::LongIdle(EditorName::Network),
            Action::Cursor {
                editor: EditorName::Node {
                    name: "copy".to_owned(),
                    item: EditorItem::MetaDsl,
                },
                line: 3,
                column: 4,
            },
            Action::Select {
                editor: EditorName::File {
                    path: "/scripts/run/src/in.txt".to_owned(),
                },
                from: (1, 1),
                to: (2, 5),
            },
            Action::Focus(FocusTarget::Window(WindowName::Node("copy".to_owned()))),
            Action::Focus(FocusTarget::Window(WindowName::Main)),
            Action::Focus(FocusTarget::Editor(EditorName::Node {
                name: "copy".to_owned(),
                item: EditorItem::Diagnostics,
            })),
            Action::Mode {
                editor: EditorName::Node {
                    name: "copy".to_owned(),
                    item: EditorItem::Output(1),
                },
                mode: EditorMode::Bin,
            },
            Action::Undo(copy_input()),
            Action::Redo(copy_input()),
            Action::Settings(values),
            Action::InsertKey("Ctrl+I".to_owned()),
            Action::TypingMode(false),
            Action::Answer(DialogAnswer::Save(true)),
            Action::Answer(DialogAnswer::Creation {
                create: true,
                silent: true,
            }),
            Action::Answer(DialogAnswer::Creation {
                create: true,
                silent: false,
            }),
            Action::Answer(DialogAnswer::Creation {
                create: false,
                silent: false,
            }),
            Action::Answer(DialogAnswer::CreationFailure(true)),
            Action::Init,
            Action::CloseNetwork,
            Action::Delay(1500),
            Action::SettingsFile("/scripts/run/backup/1/1.bak".to_owned()),
            Action::Remove("/scripts/run/src/out.txt".to_owned()),
            Action::Restore {
                saved: "/scripts/run/backup/2/3.bak".to_owned(),
                file: "/scripts/work/net/letters.txt".to_owned(),
            },
            Action::Verify {
                editor: EditorName::Node {
                    name: "copy".to_owned(),
                    item: EditorItem::Output(0),
                },
                file: "/scripts/run/run-0.gc3v".to_owned(),
            },
        ];
        let lines: Vec<String> = actions
            .iter()
            .map(|action| line_of(action, &folder))
            .collect();
        assert_eq!(
            lines,
            vec![
                "network \"src/n.gc3n\"",
                "build system \"../bin/genc3d\"",
                "new network \"new.gc3n\"",
                "open node copy",
                "close node copy",
                "open file \"src/in.txt\"",
                "close file \"src/in.txt\"",
                "activate",
                "deactivate",
                "edit node copy input 0 1:12 0 \"ddd\\n\\\"q\\\"\\t\\\\\"",
                "digits node copy input 0 hex 4 2 \"AB\"",
                "idle network",
                "long idle network",
                "cursor node copy dsl 3:4",
                "select file \"src/in.txt\" 1:1 2:5",
                "focus node copy",
                "focus main",
                "focus node copy diagnostics",
                "mode node copy output 1 bin",
                "undo node copy input 0",
                "redo node copy input 0",
                "settings idle 2.0 long idle 16.0 automatic off tab size 4 spaces on silent creation off delay 0.28",
                "insert key \"Ctrl+I\"",
                "typing mode overwrite",
                "answer save yes",
                "answer creation create silent",
                "answer creation create",
                "answer creation cancel",
                "answer creation failure continue",
                "init",
                "close network",
                "delay 1500 ms",
                "settings file \"backup/1/1.bak\"",
                "remove \"src/out.txt\"",
                "restore \"backup/2/3.bak\" into \"../work/net/letters.txt\"",
                "verify node copy output 0 \"run-0.gc3v\"",
            ]
        );
    }

    #[test]
    fn a_rendered_line_reads_back_into_its_action_through_the_canonical_form() {
        // FR-519, FR-496: the canonical form of a rendered line, as the configuration renders
        // it, read by the core yields the action; the configuration itself is tested through
        // gc3 in 'tests/automation_configuration.rs'
        let folder = folder();
        let edit = Action::Edit {
            editor: EditorName::File {
                path: "/scripts/run/src/in.txt".to_owned(),
            },
            line: 2,
            column: 3,
            range: 4,
            text: "a\"b\\c\n".to_owned(),
        };
        let line = line_of(&edit, &folder);
        // the canonical form the generator would render: the fields separated by tabs
        let canonical = "edit\tfile\tsrc/in.txt\t\t\t2\t3\t4\ta\\\"b\\\\c\\n\n";
        let script = format!("automation script\n{line}\n");
        let steps = steps_of_canonical(&format!("\n{canonical}"), &script, &folder)
            .expect("the canonical form is read");
        assert_eq!(
            (
                steps.len(),
                &steps[0].action,
                steps[0].text.as_str(),
                unescaped(&escaped("x\n\t\"\\\r")),
                resolved(&folder, &relative(&folder, "/scripts/work/net"))
            ),
            (
                1,
                &edit,
                line.as_str(),
                "x\n\t\"\\\r".to_owned(),
                "/scripts/work/net".to_owned()
            )
        );
    }

    #[test]
    fn a_path_is_written_relative_to_the_folder_with_parent_segments() {
        // FR-519
        let folder = folder();
        assert_eq!(
            (
                relative(&folder, "/scripts/run/a/b.txt"),
                relative(&folder, "/scripts/other/c.txt"),
                relative(&folder, "/x/y"),
                relative(&folder, "/scripts/run"),
                relative(&folder, "relative/p"),
                relative(Path::new("relative"), "/a/b"),
            ),
            (
                "a/b.txt".to_owned(),
                "../other/c.txt".to_owned(),
                "../../x/y".to_owned(),
                ".".to_owned(),
                "relative/p".to_owned(),
                "/a/b".to_owned(),
            )
        );
    }

    #[test]
    fn a_position_is_a_line_and_a_column_from_an_offset_in_characters_or_units() {
        // FR-522, FR-525
        let text = "ab\ncd\u{1F600}e\nf";
        assert_eq!(
            (
                position_of_offset(text, 0),
                position_of_offset(text, 2),
                position_of_offset(text, 3),
                position_of_offset(text, 6),
                position_of_offset(text, 99),
                position_of_utf16_offset(text, 5),
                position_of_utf16_offset(text, 7),
                position_of_utf16_offset(text, 8),
            ),
            (
                (1, 1),
                (1, 3),
                (2, 1),
                (2, 4),
                (3, 2),
                (2, 3),
                (2, 4),
                (2, 5)
            )
        );
    }

    #[test]
    fn a_recording_writes_the_head_init_the_settings_file_and_every_action_as_it_arises() {
        // FR-519, FR-520, FR-521, FR-528, FR-537, IR-218
        let root = temp("head");
        let path = root.join("s").join("usage.gc3s");
        let settings = root.join("genc3wb.yaml");
        fs::create_dir_all(&root).expect("the root can be created");
        fs::write(&settings, "tab_size: 4\n").expect("written");
        let mut recorder = Recorder::default();
        let started = recorder.start(&path, "0.33.0.0", "2026-10-19_10-20-30", &settings);
        let running_before = recorder.is_running();
        recorder.record(&Action::Activate);
        let while_running = lines_of(&path);
        recorder.record(&Action::OpenNode("copy".to_owned()));
        recorder.stop();
        let after = lines_of(&path);
        let saved = fs::read_to_string(root.join("s").join("backup").join("1").join("1.bak")).ok();
        let empty = recorder.start(
            &root.join("s").join("other.gc3s"),
            "0.33.0.0",
            "t",
            Path::new(""),
        );
        let no_settings = lines_of(&root.join("s").join("other.gc3s"));
        recorder.stop();
        let _ = fs::remove_dir_all(&root);
        assert_eq!(
            (
                started,
                running_before,
                while_running,
                after,
                saved,
                recorder.is_running(),
                empty,
                no_settings
            ),
            (
                Ok(()),
                true,
                vec![
                    "// Recorded by genc³wb 0.33.0.0 on 2026-10-19 at 10:20:30".to_owned(),
                    "automation script".to_owned(),
                    "init".to_owned(),
                    "settings file \"backup/1/1.bak\"".to_owned(),
                    "activate".to_owned(),
                ],
                vec![
                    "// Recorded by genc³wb 0.33.0.0 on 2026-10-19 at 10:20:30".to_owned(),
                    "automation script".to_owned(),
                    "init".to_owned(),
                    "settings file \"backup/1/1.bak\"".to_owned(),
                    "activate".to_owned(),
                    "open node copy".to_owned(),
                ],
                Some("tab_size: 4\n".to_owned()),
                false,
                Ok(()),
                vec![
                    "// Recorded by genc³wb 0.33.0.0 on t at ".to_owned(),
                    "automation script".to_owned(),
                    "init".to_owned(),
                ],
            )
        );
    }

    #[test]
    fn the_typing_is_one_edit_before_the_next_line_and_at_the_idle() {
        // FR-522, FR-523
        let root = temp("edit");
        let path = root.join("s.gc3s");
        let mut recorder = Recorder::default();
        recorder
            .start(&path, "v", "t", Path::new(""))
            .expect("started");
        let editor = copy_input();
        recorder.text_changed(&editor, "hello world", "hello worldd");
        recorder.text_changed(&editor, "hello worldd", "hello worlddd");
        recorder.focus(&FocusTarget::Window(WindowName::Main));
        recorder.text_changed(&editor, "hello worlddd", "hello worldddX");
        recorder.idle(&editor, true);
        recorder.idle(&editor, false);
        recorder.idle(&EditorName::Network, true);
        recorder.text_changed(&EditorName::Network, "a\nb", "a\nbc");
        recorder.stop();
        let lines = lines_of(&path);
        let _ = fs::remove_dir_all(&root);
        assert_eq!(
            lines[3..],
            [
                "edit node copy input 0 1:12 0 \"dd\"".to_owned(),
                "focus main".to_owned(),
                "edit node copy input 0 1:14 0 \"X\"".to_owned(),
                "idle node copy input 0".to_owned(),
                "idle network".to_owned(),
                "edit network 2:2 0 \"c\"".to_owned(),
            ]
        );
    }

    #[test]
    fn cursor_and_selection_changes_coalesce_and_a_digit_edit_resets_the_text() {
        // FR-524, FR-525
        let root = temp("cursor");
        let path = root.join("s.gc3s");
        let mut recorder = Recorder::default();
        recorder
            .start(&path, "v", "t", Path::new(""))
            .expect("started");
        let editor = copy_input();
        recorder.cursor(&editor, 1, 2);
        recorder.cursor(&editor, 1, 3);
        recorder.select(&editor, (1, 1), (1, 3));
        recorder.text_changed(&editor, "hello", "hexlo");
        recorder.cursor(&editor, 1, 4);
        recorder.record_octet_edit(
            &Action::Digits {
                editor: editor.clone(),
                mode: EditorMode::Hex,
                start: 0,
                removed: 2,
                text: "6C".to_owned(),
            },
            "lexlo",
        );
        recorder.idle(&editor, true);
        recorder.long_idle(&editor, false, "lexlo");
        recorder.long_idle(&editor, true, "lexlo0");
        recorder.text_changed(&editor, "lexlo0", "lexlo01");
        recorder.stop();
        let lines = lines_of(&path);
        let _ = fs::remove_dir_all(&root);
        assert_eq!(
            lines[3..],
            [
                "select node copy input 0 1:1 1:3".to_owned(),
                "edit node copy input 0 1:3 1 \"x\"".to_owned(),
                "cursor node copy input 0 1:4".to_owned(),
                "digits node copy input 0 hex 0 2 \"6C\"".to_owned(),
                "idle node copy input 0".to_owned(),
                "long idle node copy input 0".to_owned(),
                "edit node copy input 0 1:7 0 \"1\"".to_owned(),
            ]
        );
    }

    #[test]
    fn the_focused_editor_is_stored_and_forgotten_with_its_window() {
        // FR-526, FR-531
        let root = temp("store");
        let path = root.join("r").join("demo.gc3s");
        let mut recorder = Recorder::default();
        recorder
            .start(&path, "v", "t", Path::new(""))
            .expect("started");
        let none_before = recorder.focused_editor();
        let editor = copy_input();
        recorder.focus(&FocusTarget::Editor(editor.clone()));
        let focused = recorder.focused_editor();
        recorder.text_changed(&editor, "a", "ab");
        let first = recorder.store_expected(&editor, b"ab");
        let second = recorder.store_expected(&EditorName::Network, b"n");
        recorder.record(&Action::CloseNode("copy".to_owned()));
        let forgotten = recorder.focused_editor();
        recorder.stop();
        let stored = fs::read(root.join("r").join("demo-0.gc3v")).ok();
        let lines = lines_of(&path);
        let _ = fs::remove_dir_all(&root);
        assert_eq!(
            (
                none_before,
                focused,
                first,
                second,
                forgotten,
                stored,
                &lines[3..]
            ),
            (
                None,
                Some(editor.clone()),
                Ok(root.join("r").join("demo-0.gc3v")),
                Ok(root.join("r").join("demo-1.gc3v")),
                None,
                Some(b"ab".to_vec()),
                &[
                    "focus node copy input 0".to_owned(),
                    "edit node copy input 0 1:2 0 \"b\"".to_owned(),
                    "verify node copy input 0 \"demo-0.gc3v\"".to_owned(),
                    "verify network \"demo-1.gc3v\"".to_owned(),
                    "close node copy".to_owned(),
                ][..]
            )
        );
    }

    #[test]
    fn the_network_line_is_held_back_until_the_backup_of_the_files_is_saved() {
        // FR-539, FR-540, IR-218
        let root = temp("backup");
        let path = root.join("r").join("s.gc3s");
        let work = root.join("work");
        fs::create_dir_all(work.join("src")).expect("the work folder can be created");
        fs::write(work.join("n.gc3n"), "net").expect("written");
        fs::write(work.join("src").join("in.txt"), "hello").expect("written");
        fs::write(work.join("n.gc3n.lock"), "held").expect("written");
        let network = work.join("n.gc3n").to_string_lossy().into_owned();
        let input = work
            .join("src")
            .join("in.txt")
            .to_string_lossy()
            .into_owned();
        let output = work
            .join("src")
            .join("out.txt")
            .to_string_lossy()
            .into_owned();
        let mut recorder = Recorder::default();
        recorder
            .start(&path, "v", "t", Path::new(""))
            .expect("started");
        recorder.hold_network_line(&Action::Network(network.clone()));
        recorder.record(&Action::Focus(FocusTarget::Window(WindowName::Main)));
        let before_build = lines_of(&path);
        recorder.back_up_network(&network, &[input.clone(), output.clone(), network.clone()]);
        // a build after an edit saves nothing anew, since no line is held back
        recorder.back_up_network(&network, &[input]);
        recorder.stop();
        let lines = lines_of(&path);
        let saved_network =
            fs::read_to_string(root.join("r").join("backup").join("1").join("1.bak")).ok();
        let saved_input =
            fs::read_to_string(root.join("r").join("backup").join("1").join("2.bak")).ok();
        let second_backup = root.join("r").join("backup").join("2").exists();
        let _ = fs::remove_dir_all(&root);
        assert_eq!(
            (
                before_build.len(),
                &lines[3..],
                saved_network,
                saved_input,
                second_backup
            ),
            (
                3,
                &[
                    "restore \"backup/1/1.bak\" into \"../work/n.gc3n\"".to_owned(),
                    "restore \"backup/1/2.bak\" into \"../work/src/in.txt\"".to_owned(),
                    "remove \"../work/src/out.txt\"".to_owned(),
                    "network \"../work/n.gc3n\"".to_owned(),
                    "focus main".to_owned(),
                ][..],
                Some("net".to_owned()),
                Some("hello".to_owned()),
                false
            )
        );
    }

    #[test]
    fn the_state_found_is_recorded_as_the_commands_that_recover_it() {
        // FR-540
        let root = temp("state");
        let path = root.join("r").join("s.gc3s");
        let work = root.join("work");
        fs::create_dir_all(&work).expect("the work folder can be created");
        fs::write(work.join("n.gc3n"), "net").expect("written");
        fs::write(work.join("in.txt"), "hello").expect("written");
        let network = work.join("n.gc3n").to_string_lossy().into_owned();
        let input = work.join("in.txt").to_string_lossy().into_owned();
        let mut recorder = Recorder::default();
        recorder
            .start(&root.join("first.gc3s"), "v", "t", Path::new(""))
            .expect("started");
        recorder.focus(&FocusTarget::Editor(copy_input()));
        recorder.cursor(&copy_input(), 1, 6);
        recorder.stop();
        recorder
            .start(&path, "v", "t", Path::new(""))
            .expect("started");
        recorder.record_state(&FoundState {
            build_system: root
                .join("bin")
                .join("genc3d")
                .to_string_lossy()
                .into_owned(),
            network: network.clone(),
            files: vec![input.clone()],
            active: Some(true),
            node_windows: vec!["copy".to_owned()],
            editor_windows: vec![input.clone()],
            edited: vec![(copy_input(), "hello".to_owned(), "hello world".to_owned())],
            insert_mode: false,
        });
        recorder.text_changed(&copy_input(), "hello world", "hello worlds");
        recorder.stop();
        let lines = lines_of(&path);
        let _ = fs::remove_dir_all(&root);
        assert_eq!(
            &lines[3..],
            &[
                "build system \"../bin/genc3d\"".to_owned(),
                "restore \"backup/1/1.bak\" into \"../work/n.gc3n\"".to_owned(),
                "restore \"backup/1/2.bak\" into \"../work/in.txt\"".to_owned(),
                "network \"../work/n.gc3n\"".to_owned(),
                "activate".to_owned(),
                "open node copy".to_owned(),
                "open file \"../work/in.txt\"".to_owned(),
                "edit node copy input 0 1:6 0 \" world\"".to_owned(),
                "typing mode overwrite".to_owned(),
                "focus node copy input 0".to_owned(),
                "cursor node copy input 0 1:6".to_owned(),
                "edit node copy input 0 1:12 0 \"s\"".to_owned(),
            ][..]
        );
    }

    #[test]
    fn nothing_is_recorded_while_suspended_or_stopped_and_a_failed_start_is_kept() {
        // FR-532, FR-533
        let root = temp("suspend");
        let path = root.join("s.gc3s");
        let mut recorder = Recorder::default();
        recorder.record(&Action::Activate);
        let stored_without = recorder.store_expected(&EditorName::Network, b"x").is_err();
        recorder
            .start(&path, "v", "t", Path::new(""))
            .expect("started");
        recorder.suspend(true);
        recorder.record(&Action::Activate);
        recorder.text_changed(&EditorName::Network, "a", "ab");
        recorder.cursor(&EditorName::Network, 1, 1);
        recorder.suspend(false);
        recorder.record(&Action::Deactivate);
        recorder.stop();
        let lines = lines_of(&path);
        let failed = recorder.start(
            &root.join("s.gc3s").join("x").join("y.gc3s"),
            "v",
            "t",
            Path::new(""),
        );
        let failure = recorder.failure().to_owned();
        let _ = fs::remove_dir_all(&root);
        assert_eq!(
            (
                stored_without,
                &lines[3..],
                failed.is_err(),
                failure.is_empty(),
                recorder.is_running()
            ),
            (true, &["deactivate".to_owned()][..], true, false, false)
        );
    }

    #[test]
    fn the_start_folder_and_the_comment_line_are_named_by_the_time() {
        // FR-519, FR-520, IR-216
        let name = local_time_name();
        assert_eq!(
            (
                start_folder(Path::new("/dist/bin"), "2026-10-19_10-20-30"),
                comment_line("0.33.0.0", "2026-10-19_10-20-30"),
                name.len(),
                name.chars().filter(|c| *c == '_').count(),
            ),
            (
                PathBuf::from("/dist/rec/rec_2026-10-19_10-20-30"),
                "// Recorded by genc³wb 0.33.0.0 on 2026-10-19 at 10:20:30".to_owned(),
                19,
                1
            )
        );
    }

    #[test]
    #[ignore]
    fn every_rendered_line_is_read_back_through_gc3() {
        // FR-519, FR-496: needs the basic executable gc3 beside the genc3d of GENC3D; run by
        // 'tests/automation_configuration.rs' as well, kept here for the round trip of the core
        let build_system = std::env::var("GENC3D").expect("GENC3D names genc3d");
        let gc3 = crate::core::automation::gc3_beside(&build_system);
        let configuration = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/automation.gc3");
        let root = temp("gc3");
        fs::create_dir_all(&root).expect("created");
        let path = root.join("r.gc3s");
        let action = Action::Edit {
            editor: copy_input(),
            line: 1,
            column: 2,
            range: 3,
            text: "x\"y\n".to_owned(),
        };
        fs::write(
            &path,
            format!("automation script\n{}\n", line_of(&action, &root)),
        )
        .expect("written");
        let script = read_script(&path, &gc3, &configuration);
        let _ = fs::remove_dir_all(&root);
        assert_eq!(
            script.map(|script| script.steps[0].action.clone()),
            Ok(action)
        );
    }
}
