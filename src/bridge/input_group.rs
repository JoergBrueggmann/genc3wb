//! The *bridged type* `InputGroup`: one *input group* as the *front end* binds to it.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::api_message::Marks;
use crate::core::input_file::{InputFile, InputKind, ProcessingState};
use crate::core::octet_view::{EditorMode, row_count, row_segments, row_text};
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
    /// the marks of the *diagnostics* of the document in hex and bin, their ranges in bits
    octet_marks: Marks,
    /// the octets last provided to the *node* as a *binary document*; `None` where the document
    /// was provided as a text, or not at all
    provided_octets: Option<Vec<u8>>,
    /// the number of changes of the octets since the group was created
    revision: i32,
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
            octet_marks: Marks::default(),
            provided_octets: None,
            revision: 0,
            settings: None,
            receiver: None,
        }
    }
}

// realises FR-007, FR-011 to FR-017, FR-056, FR-059, FR-069, FR-083, FR-086, FR-087, FR-106,
// FR-107, FR-108, FR-113, FR-114, FR-116, FR-126, FR-127, FR-128, FR-154, FR-156 to FR-159,
// FR-163 to FR-173, FR-175
#[qobject(NoQmlElement)]
impl InputGroup {
    qproperty!("caption", Read = caption, Constant);
    qproperty!("fileFilter", Read = file_filter, Constant);
    qproperty!("selectable", Read = selectable, Constant);
    qproperty!("octetModes", Read = octet_modes, Constant);
    qproperty!("isText", Read = is_text, Notify = text_changed);
    qproperty!("revision", Read = revision, Notify = text_changed);
    qproperty!("octetCount", Read = octet_count, Notify = text_changed);
    qproperty!(
        "octetMarkSeverities",
        Read = octet_mark_severities,
        Notify = octet_marks_changed
    );
    qproperty!(
        "octetMarkTexts",
        Read = octet_mark_texts,
        Notify = octet_marks_changed
    );
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

    // realises FR-154
    fn octet_modes(&self) -> bool {
        match self.file.kind() {
            InputKind::Input => true,
            InputKind::MetaDsl | InputKind::Network => false,
        }
    }

    // realises FR-156
    fn is_text(&self) -> bool {
        self.file.is_text()
    }

    fn revision(&self) -> i32 {
        self.revision
    }

    // realises FR-173, FR-175
    fn octet_count(&self) -> i32 {
        int_of(self.file.octets().len())
    }

    // realises FR-169, FR-170
    fn octet_mark_severities(&self) -> Vec<i32> {
        self.octet_marks.severities.clone()
    }

    // realises FR-172
    fn octet_mark_texts(&self) -> Vec<String> {
        self.octet_marks.texts.clone()
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

    // realises FR-019, FR-021, FR-055, FR-113, FR-165, FR-166
    // Replaces the octets by the encoding of the text where the user modified it, records that
    // the document was edited, and announces the *processing state*; a text as the text area
    // holds the text of the document is no edit.
    fn set_text(&mut self, text: String) {
        if !self.file.set_text(&text) {
            return;
        }
        let temporary_before = self.temporary_edit();
        self.edited_since_named = true;
        self.announce_text();
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

    #[qsignal(qml_name = "octetMarksChanged")]
    fn octet_marks_changed(&mut self);

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

    // realises FR-016, FR-056, FR-057, FR-059, FR-069, FR-106, FR-167, FR-168, IR-015, IR-016
    /// Provides the pending *text increment*, writes its rendering to standard output followed by
    /// a line separator, emits `increment_provided`, schedules `applyIncrement` of the receiver
    /// with the *document identifier* where one is wired, and saves the input file where it
    /// holds unsaved changes and a path is named.
    ///
    /// * Where the octets are no valid UTF-8, no *text increment* is provided: the octets are
    ///   provided where they differ from those provided before, the *provided text* becomes
    ///   empty, and `applyOctets` of the receiver is scheduled with the *document identifier*.
    #[qslot(qml_name = "idleExpired")]
    fn idle_expired(&mut self) {
        if !self.file.is_text() {
            self.provide_octets();
        } else if let Some(increment) = self.tracker.provide(self.file.text()) {
            self.provided_octets = None;
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
    /// *provided text*, the octets provided and the *diagnostics*, so that the next
    /// *text increment* carries the whole text, and names `path` as the file name field does,
    /// loading the file even where the path is the one before, since the file may have changed;
    /// scheduled by the *node editor*.
    #[qslot(qml_name = "nameDocument")]
    fn name_document(&mut self, identifier: String, path: String, producer: String) {
        self.identifier = identifier;
        self.producer = producer;
        self.tracker = IncrementTracker::default();
        self.provided_octets = None;
        self.document_changed();
        self.set_diagnostics(vec![], vec![], vec![], vec![]);
        self.set_octet_marks(vec![], vec![], vec![], vec![]);
        self.name_path(path);
    }

    // realises FR-158, FR-159
    /// Yields the number of rows of `per_row` octets the octets take, as [`row_count`] does;
    /// 0 for a negative `per_row`.
    #[qslot(qml_name = "rowCount")]
    fn row_count(&self, per_row: i32) -> i32 {
        int_of(row_count(self.file.octets().len(), count_of(per_row)))
    }

    // realises FR-158, FR-159
    /// Yields the text of the row `row` of `per_row` octets in the *editor mode* of index
    /// `mode`, as [`row_text`] does; empty for a negative row and for an index that names no
    /// mode.
    #[qslot(qml_name = "rowText")]
    fn row_text(&self, mode: i32, per_row: i32, row: i32) -> String {
        match (mode_of(mode), usize::try_from(row)) {
            (Some(mode), Ok(row)) => row_text(self.file.octets(), row, mode, count_of(per_row)),
            (None, _) | (_, Err(_)) => String::new(),
        }
    }

    // realises FR-169, FR-170, FR-171
    /// Yields the segments of the row `row` of `per_row` octets below which the *diagnostics*
    /// are marked in the *editor mode* of index `mode`, as [`row_segments`] does, as a flat
    /// list: per segment its first character column, its number of character columns and the
    /// index of its mark.
    #[qslot(qml_name = "rowMarks")]
    fn row_marks(&self, mode: i32, per_row: i32, row: i32) -> Vec<i32> {
        let (Some(mode), Ok(row)) = (mode_of(mode), usize::try_from(row)) else {
            return Vec::new();
        };
        let bits = |offsets: &[i32]| -> Vec<u64> {
            offsets
                .iter()
                .map(|offset| u64::try_from(*offset).unwrap_or(0))
                .collect()
        };
        row_segments(
            &bits(&self.octet_marks.starts),
            &bits(&self.octet_marks.ends),
            self.file.octets().len(),
            row,
            mode,
            count_of(per_row),
        )
        .into_iter()
        .flat_map(|segment| {
            [
                int_of(segment.first_column),
                int_of(segment.columns),
                int_of(segment.mark),
            ]
        })
        .collect()
    }

    // realises FR-169 to FR-172
    /// Replaces the marks of the *diagnostics* in hex and bin: per *diagnostic* the *offset* of
    /// the start and of the end of its range in bits, the index of its severity and its message
    /// text; scheduled by the *node editor*.
    #[qslot(qml_name = "setOctetMarks")]
    fn set_octet_marks(
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
        if marks == self.octet_marks {
            return;
        }
        self.octet_marks = marks;
        self.octet_marks_changed();
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

    // realises FR-167
    /// Yields the octets last provided to the *node* as a *binary document*, none where the
    /// document was provided as a text or not at all; read by the *node editor*.
    pub fn octets_provided(&self) -> Vec<u8> {
        self.provided_octets.clone().unwrap_or_default()
    }

    // realises FR-167, FR-168
    /// Provides the octets where they differ from those provided before: the *provided text*
    /// becomes empty, and `applyOctets` of the receiver is scheduled with the
    /// *document identifier* where one is wired.
    fn provide_octets(&mut self) {
        if self.provided_octets.as_deref() == Some(self.file.octets()) {
            return;
        }
        self.provided_octets = Some(self.file.octets().to_vec());
        self.tracker = IncrementTracker::default();
        if let Some(receiver) = &self.receiver {
            invoke_method!(receiver, "applyOctets", self.identifier.clone());
        }
    }

    /// Counts the change of the octets and emits `text_changed`.
    fn announce_text(&mut self) {
        self.revision = self.revision.wrapping_add(1);
        self.text_changed();
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
        let octets_before = self.file.octets().to_vec();
        let _ = self.file.load(path);
        let temporary_before = self.temporary_edit();
        self.edited_since_named = false;
        self.path_changed();
        if self.file.octets() != octets_before {
            self.announce_text();
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

/// Yields the *editor mode* of the index `mode`, `None` where the index names none.
fn mode_of(mode: i32) -> Option<EditorMode> {
    usize::try_from(mode).ok().and_then(EditorMode::of_index)
}

/// Yields `value` as a count; 0 where it is negative.
fn count_of(value: i32) -> usize {
    usize::try_from(value).unwrap_or(0)
}

/// Yields `value` as the integer QML takes; the greatest one where it is larger.
fn int_of(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}
