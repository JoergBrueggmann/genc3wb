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

// realises FR-007
/// One *input group* as the *front end* binds to it.
pub struct InputGroup {
    /// the input file
    file: InputFile,
    /// the *provided text* of the code editor of this group
    tracker: IncrementTracker,
    /// the path waiting to be loaded while the user is asked whether to save
    pending_path: Option<String>,
    /// the *processing state* last pushed to the runner group
    pushed_state: ProcessingState,
    /// the settings, to store the path; wired by the *workbench object*
    settings: Option<Rc<RefCell<Settings>>>,
    /// the invoker of the runner group, to push the *processing state* to; wired by the
    /// *workbench object* for an *input group* of the *node window*
    runner: Option<QmlMethodInvoker>,
    /// the invoker of the network editor, which receives the path, the *text increments* and the
    /// expiry of the *long idle time*; wired by the *workbench object* for the *network file*
    receiver: Option<QmlMethodInvoker>,
}

impl Default for InputGroup {
    fn default() -> Self {
        InputGroup {
            file: InputFile::new(InputKind::CompilerCompilerInput),
            tracker: IncrementTracker::default(),
            pending_path: None,
            pushed_state: ProcessingState::default(),
            settings: None,
            runner: None,
            receiver: None,
        }
    }
}

// realises FR-007, FR-011, FR-012, FR-013, FR-014, FR-015, FR-016, FR-017, FR-056, FR-059, FR-069,
// FR-083, FR-086, FR-087
#[qobject(NoQmlElement)]
impl InputGroup {
    qproperty!("caption", Read = caption, Constant);
    qproperty!("fileFilter", Read = file_filter, Constant);
    qproperty!("path", Read = path, Write = set_path, Notify = path_changed);
    qproperty!("text", Read = text, Write = set_text, Notify = text_changed);
    qproperty!("state", Read = state, Notify = state_changed);

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

    // setters
    // realises FR-012, FR-013, FR-014, FR-015
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

    // signals
    #[qsignal(qml_name = "pathChanged")]
    fn path_changed(&mut self);

    #[qsignal(qml_name = "textChanged")]
    fn text_changed(&mut self);

    #[qsignal(qml_name = "stateChanged")]
    fn state_changed(&mut self);

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

    // realises FR-016, FR-056, FR-057, FR-059, FR-069, IR-015, IR-016
    /// Provides the pending *text increment*, writes its rendering to standard output followed by
    /// a line separator, emits `increment_provided`, schedules `applyIncrement` of the network
    /// editor where one is wired, and saves the input file where it holds unsaved changes and a
    /// path is named.
    #[qslot(qml_name = "idleExpired")]
    fn idle_expired(&mut self) {
        if let Some(increment) = self.tracker.provide(self.file.text()) {
            let rendering = increment.rendering();
            let mut stdout = std::io::stdout().lock();
            let _ = writeln!(stdout, "{rendering}");
            let _ = stdout.flush();
            self.increment_provided(rendering);
            if let Some(receiver) = &self.receiver {
                invoke_method!(
                    receiver,
                    "applyIncrement",
                    i32::try_from(increment.position).unwrap_or(i32::MAX),
                    i32::try_from(increment.range).unwrap_or(i32::MAX),
                    increment.text
                );
            }
        }
        if self.file.has_unsaved_changes() && !self.file.path().is_empty() {
            let _ = self.file.save();
            self.push_state();
        }
    }

    // realises FR-083
    /// Schedules `longIdleExpired` of the network editor where one is wired: the text was not
    /// modified for the *long idle time*.
    #[qslot(qml_name = "longIdleExpired")]
    fn long_idle_expired(&mut self) {
        if let Some(receiver) = &self.receiver {
            invoke_method!(receiver, "longIdleExpired");
        }
    }

    // realises FR-086, FR-087
    /// Names `path`, as the file name field does; scheduled by the network editor when a *node*
    /// is opened.
    #[qslot(qml_name = "namePath")]
    fn name_path(&mut self, path: String) {
        self.set_path(path);
    }
}

impl InputGroup {
    /// Wires the group: which input file it edits, the settings, the invoker of the runner group
    /// for an *input group* of the *node window*, and the invoker of the network editor for the
    /// *input group* of the *network file*.
    ///
    /// * Called by the *workbench object* once, before the *front end* binds to the group.
    pub fn configure(
        &mut self,
        kind: InputKind,
        settings: Rc<RefCell<Settings>>,
        runner: Option<QmlMethodInvoker>,
        receiver: Option<QmlMethodInvoker>,
    ) {
        self.file = InputFile::new(kind);
        self.settings = Some(settings);
        self.runner = runner;
        self.receiver = receiver;
    }

    // realises FR-090
    /// Loads the file at the path the settings hold for this group, and pushes the state.
    pub fn load_initial(&mut self) {
        let path = self
            .settings
            .as_ref()
            .map(|settings| settings.borrow().input_path(self.file.kind()).to_owned())
            .unwrap_or_default();
        let _ = self.file.load(&path);
        self.push_state();
        self.announce_path();
    }

    // realises FR-067
    /// Schedules `setNetworkPath` of the network editor where one is wired.
    fn announce_path(&self) {
        if let Some(receiver) = &self.receiver {
            invoke_method!(receiver, "setNetworkPath", self.file.path().to_owned());
        }
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
        self.announce_path();
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
