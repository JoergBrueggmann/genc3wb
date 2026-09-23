//! The *bridged type* `InputGroup`: one *input group* as the *front end* binds to it.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::api_message::Marks;
use crate::core::input_file::{InputFile, InputKind, ProcessingState};
use crate::core::settings::Settings;
use crate::core::text_increment::IncrementTracker;

use qtbridge::{QmlMethodInvoker, invoke_method, qobject};

use std::cell::RefCell;
use std::io::Write;
use std::rc::Rc;

// realises FR-007, FR-113
/// One *input group* as the *front end* binds to it.
pub struct InputGroup {
    /// the input file
    file: InputFile,
    /// the *provided text* of the code editor of this group
    tracker: IncrementTracker,
    /// the path waiting to be loaded while the user is asked whether to save
    pending_path: Option<String>,
    /// the *processing state* last announced to the *front end*
    announced_state: ProcessingState,
    /// the *document identifier* of the file at the *node*; empty for the *network file*
    identifier: String,
    /// the name of the *producer* of the *input*; empty where it has none
    producer: String,
    /// whether the text was edited since the document was named
    edited_since_named: bool,
    /// the marks of the *diagnostics* of the document; none where there is none
    marks: Marks,
    /// the settings, to store the path; wired by the *workbench object* for the *network file*
    settings: Option<Rc<RefCell<Settings>>>,
    /// the invoker of the receiver of the *text increments*: the network editor for the
    /// *network file*, which receives the path and the expiry of the *long idle time* as well,
    /// and the *node editor* for an *input group* of the *node window*
    receiver: Option<QmlMethodInvoker>,
}

impl Default for InputGroup {
    fn default() -> Self {
        InputGroup {
            file: InputFile::new(InputKind::MetaDsl),
            tracker: IncrementTracker::default(),
            pending_path: None,
            announced_state: ProcessingState::default(),
            identifier: String::new(),
            producer: String::new(),
            edited_since_named: false,
            marks: Marks::default(),
            settings: None,
            receiver: None,
        }
    }
}

// realises FR-007, FR-011 to FR-017, FR-056, FR-059, FR-069, FR-083, FR-086, FR-087, FR-106,
// FR-107, FR-108, FR-113, FR-114, FR-116, FR-126, FR-127, FR-128
#[qobject(NoQmlElement)]
impl InputGroup {
    qproperty!("caption", Read = caption, Constant);
    qproperty!("fileFilter", Read = file_filter, Constant);
    qproperty!("selectable", Read = selectable, Constant);
    qproperty!("path", Read = path, Write = set_path, Notify = path_changed);
    qproperty!("text", Read = text, Write = set_text, Notify = text_changed);
    qproperty!("state", Read = state, Notify = state_changed);
    qproperty!("producer", Read = producer, Notify = document_changed);
    qproperty!(
        "temporaryEdit",
        Read = temporary_edit,
        Notify = state_changed
    );
    qproperty!(
        "diagnosticStarts",
        Read = diagnostic_starts,
        Notify = diagnostics_changed
    );
    qproperty!(
        "diagnosticEnds",
        Read = diagnostic_ends,
        Notify = diagnostics_changed
    );
    qproperty!(
        "diagnosticSeverities",
        Read = diagnostic_severities,
        Notify = diagnostics_changed
    );
    qproperty!(
        "diagnosticTexts",
        Read = diagnostic_texts,
        Notify = diagnostics_changed
    );

    // getters
    fn caption(&self) -> String {
        self.file.kind().caption().to_owned()
    }

    fn file_filter(&self) -> String {
        self.file.kind().file_filter().to_owned()
    }

    // realises FR-007, FR-116
    fn selectable(&self) -> bool {
        self.file.kind().is_selectable()
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

    // realises FR-114
    fn producer(&self) -> String {
        self.producer.clone()
    }

    // realises FR-113
    fn temporary_edit(&self) -> bool {
        !self.producer.is_empty() && self.edited_since_named
    }

    // realises FR-107, FR-108, FR-126
    fn diagnostic_starts(&self) -> Vec<i32> {
        self.marks.starts.clone()
    }

    fn diagnostic_ends(&self) -> Vec<i32> {
        self.marks.ends.clone()
    }

    fn diagnostic_severities(&self) -> Vec<i32> {
        self.marks.severities.clone()
    }

    // realises FR-127
    fn diagnostic_texts(&self) -> Vec<String> {
        self.marks.texts.clone()
    }

    // setters
    // realises FR-012, FR-013, FR-014, FR-015
    // Names the path: where the editor holds unsaved changes, keeps the path pending and emits
    // `ask_to_save`; otherwise stores the path and loads the file.
    fn set_path(&mut self, path: String) {
        if path == self.file.path() {
            return;
        }
        self.name_path(path);
    }

    // realises FR-019, FR-021, FR-055, FR-113
    // Replaces the text where it differs, records that the document was edited, and announces
    // the *processing state*.
    fn set_text(&mut self, text: String) {
        if text == self.file.text() {
            return;
        }
        self.file.set_text(&text);
        let temporary_before = self.temporary_edit();
        self.edited_since_named = true;
        self.text_changed();
        self.announce_state(temporary_before != self.temporary_edit());
    }

    // signals
    #[qsignal(qml_name = "pathChanged")]
    fn path_changed(&mut self);

    #[qsignal(qml_name = "textChanged")]
    fn text_changed(&mut self);

    #[qsignal(qml_name = "stateChanged")]
    fn state_changed(&mut self);

    #[qsignal(qml_name = "documentChanged")]
    fn document_changed(&mut self);

    #[qsignal(qml_name = "diagnosticsChanged")]
    fn diagnostics_changed(&mut self);

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

    // realises FR-016, FR-056, FR-057, FR-059, FR-069, FR-106, IR-015, IR-016
    /// Provides the pending *text increment*, writes its rendering to standard output followed by
    /// a line separator, emits `increment_provided`, schedules `applyIncrement` of the receiver
    /// with the *document identifier* where one is wired, and saves the input file where it
    /// holds unsaved changes and a path is named.
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
                    self.identifier.clone(),
                    i32::try_from(increment.position).unwrap_or(i32::MAX),
                    i32::try_from(increment.range).unwrap_or(i32::MAX),
                    increment.text
                );
            }
        }
        if self.file.has_unsaved_changes() && !self.file.path().is_empty() {
            let _ = self.file.save();
            self.announce_state(false);
        }
    }

    // realises FR-083
    /// Schedules `longIdleExpired` of the network editor, for the *network file* alone: the text
    /// was not modified for the *long idle time*.
    #[qslot(qml_name = "longIdleExpired")]
    fn long_idle_expired(&mut self) {
        if let (Some(receiver), InputKind::Network) = (&self.receiver, self.file.kind()) {
            invoke_method!(receiver, "longIdleExpired");
        }
    }

    // realises FR-086, FR-087, FR-113, FR-114
    /// Names the document of a *node*: records `identifier` and `producer`, forgets the
    /// *provided text* and the *diagnostics*, so that the next *text increment* carries the
    /// whole text, and names `path` as the file name field does, loading the file even where the
    /// path is the one before, since the file may have changed; scheduled by the *node editor*.
    #[qslot(qml_name = "nameDocument")]
    fn name_document(&mut self, identifier: String, path: String, producer: String) {
        self.identifier = identifier;
        self.producer = producer;
        self.tracker = IncrementTracker::default();
        self.document_changed();
        self.set_diagnostics(vec![], vec![], vec![], vec![]);
        self.name_path(path);
    }

    // realises FR-107, FR-108, FR-126, FR-127, FR-128
    /// Replaces the marks of the *diagnostics*: per *diagnostic* the start and the end of its
    /// range as offsets in UTF-16 code units, the index of its severity and its message text;
    /// scheduled by the *node editor* or by the network editor.
    #[qslot(qml_name = "setDiagnostics")]
    fn set_diagnostics(
        &mut self,
        starts: Vec<i32>,
        ends: Vec<i32>,
        severities: Vec<i32>,
        texts: Vec<String>,
    ) {
        let marks = Marks {
            starts,
            ends,
            severities,
            texts,
        };
        if marks == self.marks {
            return;
        }
        self.marks = marks;
        self.diagnostics_changed();
    }
}

impl InputGroup {
    /// Wires the group: which file it edits, the settings for the *input group* of the *network
    /// file*, and the invoker of the receiver of its *text increments*: the network editor for
    /// the *network file*, the *node editor* for an *input group* of the *node window*.
    ///
    /// * Called by the *workbench object* or by the *node editor* once, before the *front end*
    ///   binds to the group.
    pub fn configure(
        &mut self,
        kind: InputKind,
        settings: Option<Rc<RefCell<Settings>>>,
        receiver: Option<QmlMethodInvoker>,
    ) {
        self.file = InputFile::new(kind);
        self.settings = settings;
        self.receiver = receiver;
    }

    // realises FR-090
    /// Loads the file at the path the settings hold for the *network file*, announces the state,
    /// and announces the path to the network editor where one is wired.
    pub fn load_initial(&mut self) {
        let path = self
            .settings
            .as_ref()
            .filter(|_| self.file.kind() == InputKind::Network)
            .map(|settings| settings.borrow().network_path().to_owned())
            .unwrap_or_default();
        let _ = self.file.load(&path);
        self.announce_state(false);
        self.announce_path();
    }

    // realises FR-013, FR-014, FR-015
    /// Names `path`: where the editor holds unsaved changes, keeps the path pending and emits
    /// `ask_to_save`; otherwise applies it.
    fn name_path(&mut self, path: String) {
        if self.file.has_unsaved_changes() {
            self.pending_path = Some(path.clone());
            self.ask_to_save(path);
            return;
        }
        self.apply_path(&path);
    }

    // realises FR-067
    /// Schedules `setNetworkPath` of the network editor, for the *network file* alone.
    fn announce_path(&self) {
        if let (Some(receiver), InputKind::Network) = (&self.receiver, self.file.kind()) {
            invoke_method!(receiver, "setNetworkPath", self.file.path().to_owned());
        }
    }

    // realises FR-012, FR-013, FR-091
    /// Stores `path` in the settings for the *network file*, loads the file, and notifies the
    /// *front end*.
    fn apply_path(&mut self, path: &str) {
        if let (Some(settings), InputKind::Network) = (&self.settings, self.file.kind()) {
            let mut settings = settings.borrow_mut();
            settings.set_network_path(path);
            let _ = settings.save(&Settings::default_path());
        }
        let text_before = self.file.text().to_owned();
        let _ = self.file.load(path);
        let temporary_before = self.temporary_edit();
        self.edited_since_named = false;
        self.path_changed();
        if self.file.text() != text_before {
            self.text_changed();
        }
        self.announce_state(temporary_before != self.temporary_edit());
        self.announce_path();
    }

    // realises FR-017, FR-113
    /// Emits `state_changed` where the *processing state* changed, or where `temporary_changed`.
    fn announce_state(&mut self, temporary_changed: bool) {
        let state = self.file.state();
        if state == self.announced_state && !temporary_changed {
            return;
        }
        self.announced_state = state;
        self.state_changed();
    }
}
