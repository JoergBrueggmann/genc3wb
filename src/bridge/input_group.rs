//! The *bridged type* `InputGroup`: one *input group* as the *front end* binds to it.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::input_file::{InputFile, InputKind, ProcessingState};
use crate::core::settings::Settings;
use crate::core::text_increment::IncrementTracker;

use qtbridge::{QmlMethodInvoker, invoke_method, qobject};

use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

/// The *maximum idle time* until another one is set, in milliseconds.
const DEFAULT_MAX_IDLE_TIME: u32 = 1000;

// realises FR-007
/// One *input group* as the *front end* binds to it.
pub struct InputGroup {
    /// the input file
    file: InputFile,
    /// the *provided text* of the code editor of this group
    tracker: IncrementTracker,
    /// the *maximum idle time* in milliseconds
    max_idle_time: u32,
    /// the path waiting to be loaded while the user is asked whether to save
    pending_path: Option<String>,
    /// the *processing state* last pushed to the runner group
    pushed_state: ProcessingState,
    /// the settings, to store the path; wired by the *workbench object*
    settings: Option<Rc<RefCell<Settings>>>,
    /// the invoker of the runner group, to push the *processing state* to; wired by the *workbench object*
    runner: Option<QmlMethodInvoker>,
}

impl Default for InputGroup {
    fn default() -> Self {
        InputGroup {
            file: InputFile::new(InputKind::CompilerCompilerInput),
            tracker: IncrementTracker::default(),
            max_idle_time: DEFAULT_MAX_IDLE_TIME,
            pending_path: None,
            pushed_state: ProcessingState::default(),
            settings: None,
            runner: None,
        }
    }
}

// realises FR-007, FR-011, FR-012, FR-013, FR-014, FR-015, FR-016, FR-017, FR-056, FR-058, FR-059
#[qobject(NoQmlElement)]
impl InputGroup {
    qproperty!("caption", Read = caption, Constant);
    qproperty!("fileFilter", Read = file_filter, Constant);
    qproperty!("path", Read = path, Write = set_path, Notify = path_changed);
    qproperty!("text", Read = text, Write = set_text, Notify = text_changed);
    qproperty!("state", Read = state, Notify = state_changed);
    qproperty!(
        "maxIdleTime",
        Read = max_idle_time,
        Write = set_max_idle_time,
        Notify = max_idle_time_changed
    );

    // getters
    fn caption(&self) -> String {
        self.file.kind().caption().to_owned()
    }

    fn file_filter(&self) -> String {
        self.file.kind().file_filter().to_owned()
    }

    fn path(&self) -> String {
        self.file.path().to_owned()
    }

    fn text(&self) -> String {
        self.file.text().to_owned()
    }

    fn state(&self) -> i32 {
        self.file.state().index() as i32
    }

    fn max_idle_time(&self) -> u32 {
        self.max_idle_time
    }

    // setters
    // realises FR-012, FR-013, FR-014, FR-015, FR-049
    // Names the path: where the editor holds unsaved changes, keeps the path pending and emits
    // `ask_to_save`; otherwise stores the path and loads the file.
    fn set_path(&mut self, path: String) {
        if path == self.file.path() {
            return;
        }
        if self.file.has_unsaved_changes() {
            self.pending_path = Some(path.clone());
            self.ask_to_save(path);
            return;
        }
        self.apply_path(&path);
    }

    // realises FR-019, FR-021, FR-055
    // Replaces the text where it differs, and pushes the *processing state* to the runner group.
    fn set_text(&mut self, text: String) {
        if text == self.file.text() {
            return;
        }
        self.file.set_text(&text);
        self.text_changed();
        self.push_state();
    }

    // realises FR-058
    fn set_max_idle_time(&mut self, max_idle_time: u32) {
        let max_idle_time = max_idle_time.max(1);
        if max_idle_time == self.max_idle_time {
            return;
        }
        self.max_idle_time = max_idle_time;
        self.max_idle_time_changed();
    }

    // signals
    #[qsignal(qml_name = "pathChanged")]
    fn path_changed(&mut self);

    #[qsignal(qml_name = "textChanged")]
    fn text_changed(&mut self);

    #[qsignal(qml_name = "stateChanged")]
    fn state_changed(&mut self);

    #[qsignal(qml_name = "maxIdleTimeChanged")]
    fn max_idle_time_changed(&mut self);

    #[qsignal(qml_name = "askToSave")]
    fn ask_to_save(&mut self, path: String);

    #[qsignal(qml_name = "incrementProvided")]
    fn increment_provided(&mut self, rendering: String);

    // slots
    // realises FR-014
    /// Answers the question of `ask_to_save`: saves where `save`, then loads the pending path.
    #[qslot(qml_name = "answerSave")]
    fn answer_save(&mut self, save: bool) {
        if save {
            let _ = self.file.save();
        }
        if let Some(path) = self.pending_path.take() {
            self.apply_path(&path);
        }
    }

    // realises FR-016, FR-056, FR-057, FR-059, IR-015, IR-016
    /// Provides the pending *text increment*, writes its rendering to standard output followed by
    /// a line separator, emits `increment_provided`, and saves the input file where it holds
    /// unsaved changes and a path is named.
    #[qslot(qml_name = "idleExpired")]
    fn idle_expired(&mut self) {
        if let Some(increment) = self.tracker.provide(self.file.text()) {
            let rendering = increment.rendering();
            let mut stdout = std::io::stdout().lock();
            let _ = writeln!(stdout, "{rendering}");
            let _ = stdout.flush();
            self.increment_provided(rendering);
        }
        if self.file.has_unsaved_changes() && !self.file.path().is_empty() {
            let _ = self.file.save();
            self.push_state();
        }
    }
}

impl InputGroup {
    /// Wires the group: which input file it edits, the settings, and the invoker of the runner group.
    ///
    /// * Called by the *workbench object* once, before the *front end* binds to the group.
    pub fn configure(
        &mut self,
        kind: InputKind,
        settings: Rc<RefCell<Settings>>,
        runner: QmlMethodInvoker,
    ) {
        self.file = InputFile::new(kind);
        self.settings = Some(settings);
        self.runner = Some(runner);
    }

    // realises FR-048
    /// Loads the file at the path the settings hold for this group, and pushes the state.
    pub fn load_initial(&mut self) {
        let path = self
            .settings
            .as_ref()
            .map(|settings| settings.borrow().input_path(self.file.kind()).to_owned())
            .unwrap_or_default();
        let _ = self.file.load(&path);
        self.push_state();
    }

    /// Stores `path` in the settings, loads the file, and notifies the *front end*.
    fn apply_path(&mut self, path: &str) {
        if let Some(settings) = &self.settings {
            let mut settings = settings.borrow_mut();
            settings.set_input_path(self.file.kind(), path);
            let _ = settings.save(&Settings::default_path());
        }
        let text_before = self.file.text().to_owned();
        let _ = self.file.load(path);
        self.path_changed();
        if self.file.text() != text_before {
            self.text_changed();
        }
        self.push_state();
    }

    // realises FR-017, FR-026
    /// Emits `state_changed` and schedules `setInputState` of the runner group where the state changed.
    fn push_state(&mut self) {
        let state = self.file.state();
        if state == self.pushed_state {
            return;
        }
        self.pushed_state = state;
        self.state_changed();
        if let Some(runner) = &self.runner {
            invoke_method!(
                runner,
                "setInputState",
                self.file.kind().index() as i32,
                state.index() as i32
            );
        }
    }
}
