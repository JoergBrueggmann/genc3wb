//! The *bridged type* `InputGroup`: one *input group* as the *front end* binds to it.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::bridge::document_registry::DocumentRegistry;
use crate::core::api_message::Marks;
use crate::core::file_lock::{FileLock, Refusal, Taking};
use crate::core::input_file::{InputFile, InputKind, ProcessingState};
use crate::core::octet_edit::{digits_of_text, text_of_digits};
use crate::core::octet_view::{EditorMode, row_count, row_segments, row_text_of_bits};
use crate::core::settings::SettingsStore;
use crate::core::text_increment::{IncrementTracker, TextIncrement};

use qtbridge::{QObjectHolder, QmlMethodInvoker, invoke_method, qobject};

use std::cell::RefCell;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;

// realises FR-007, FR-214, FR-216, FR-238, FR-239, FR-277
/// One *input group* as the *front end* binds to it.
pub struct InputGroup {
    /// the input file: the *shared document* acquired from the *document registry* for a
    /// document of a *node* the user edits, shared with its other *presenters*; a file of its
    /// own for the *network file* and for an *input* of a *producer* (FR-277)
    file: Rc<RefCell<InputFile>>,
    /// the *document registry*; wired for an *input group* of a *node window*
    documents: Option<Rc<RefCell<DocumentRegistry>>>,
    /// the number of this group as a *presenter* in the *document registry*
    presenter: u32,
    /// the path of the *shared document* acquired, `None` where none is
    shared_path: Option<String>,
    /// the *provided text* of the code editor of this group
    tracker: IncrementTracker,
    /// the path waiting to be loaded while the user is asked whether to save
    pending_path: Option<String>,
    /// the *processing state* last announced to the *front end*
    announced_state: ProcessingState,
    /// the *document identifier* of the file at the *node*; empty for the *network file*
    identifier: String,
    /// whether the document is an *input* that has a *producer*, which the user does not edit
    /// and whose content is what the *node* holds (FR-238, FR-239)
    produced: bool,
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
    settings: Option<Rc<RefCell<SettingsStore>>>,
    /// the *lock file* of the *network file*, held while it is named (FR-214); `None` for every
    /// other file, and where it could not be created (FR-222)
    network_lock: Option<FileLock>,
    /// the *refusal* of the *network file* last named, until a *network file* is named
    /// (FR-216, FR-217, FR-220)
    refusal: Option<Refusal>,
    /// the invoker of the receiver of the *text increments*: the network editor for the
    /// *network file*, which receives the path and the expiry of the *long idle time* as well,
    /// and the *node editor* for an *input group* of a *node window*
    receiver: Option<QmlMethodInvoker>,
}

impl Default for InputGroup {
    fn default() -> Self {
        InputGroup {
            file: Rc::new(RefCell::new(InputFile::new(InputKind::MetaDsl))),
            documents: None,
            presenter: 0,
            shared_path: None,
            tracker: IncrementTracker::default(),
            pending_path: None,
            announced_state: ProcessingState::default(),
            identifier: String::new(),
            produced: false,
            marks: Marks::default(),
            octet_marks: Marks::default(),
            provided_octets: None,
            revision: 0,
            settings: None,
            network_lock: None,
            refusal: None,
            receiver: None,
        }
    }
}

// realises FR-007, FR-011 to FR-017, FR-056, FR-059, FR-069, FR-083, FR-086, FR-087, FR-106,
// FR-116, FR-126, FR-127, FR-128, FR-154, FR-156 to FR-159,
// FR-163 to FR-173, FR-175, FR-177, FR-179 to FR-187, FR-202, FR-203, FR-205, FR-207, FR-208,
// FR-214, FR-216, FR-217, FR-219, FR-220, FR-238, FR-239, FR-240, FR-246, FR-247, FR-253,
// FR-277 to FR-280
#[qobject(NoQmlElement)]
impl InputGroup {
    qproperty!("caption", Read = caption, Constant);
    qproperty!("fileFilter", Read = file_filter, Constant);
    qproperty!("selectable", Read = selectable, Constant);
    qproperty!("octetModes", Read = octet_modes, Constant);
    qproperty!("isText", Read = is_text, Notify = text_changed);
    qproperty!("revision", Read = revision, Notify = text_changed);
    qproperty!("octetCount", Read = octet_count, Notify = text_changed);
    qproperty!("bitCount", Read = bit_count, Notify = text_changed);
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
    // whether the document is an input that has a producer, which the user does not edit (FR-238)
    qproperty!("produced", Read = produced, Notify = document_changed);
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
    // the refusal of the network file and its holder, empty and 0 where there is none (FR-220)
    qproperty!("refusedFile", Read = refused_file, Notify = refusal_changed);
    qproperty!("holderPid", Read = holder_pid, Notify = refusal_changed);
    qproperty!("holderHost", Read = holder_host, Notify = refusal_changed);
    qproperty!("holderSince", Read = holder_since, Notify = refusal_changed);

    // getters
    // realises FR-219, FR-220
    fn refused_file(&self) -> String {
        self.refusal
            .as_ref()
            .map(|refusal| refusal.file.clone())
            .unwrap_or_default()
    }

    // realises FR-219, FR-220
    fn holder_pid(&self) -> i32 {
        self.refusal
            .as_ref()
            .and_then(|refusal| refusal.holder.as_ref())
            .map_or(0, |holder| i32::try_from(holder.pid).unwrap_or(i32::MAX))
    }

    // realises FR-219, FR-220
    fn holder_host(&self) -> String {
        self.refusal
            .as_ref()
            .and_then(|refusal| refusal.holder.as_ref())
            .map(|holder| holder.host.clone())
            .unwrap_or_default()
    }

    // realises FR-219, FR-220
    fn holder_since(&self) -> String {
        self.refusal
            .as_ref()
            .and_then(|refusal| refusal.holder.as_ref())
            .map(|holder| holder.since.clone())
            .unwrap_or_default()
    }

    fn caption(&self) -> String {
        self.kind().caption().to_owned()
    }

    fn file_filter(&self) -> String {
        self.kind().file_filter().to_owned()
    }

    // realises FR-007, FR-116
    fn selectable(&self) -> bool {
        self.kind().is_selectable()
    }

    // realises FR-154
    fn octet_modes(&self) -> bool {
        match self.kind() {
            InputKind::Input | InputKind::Output => true,
            InputKind::MetaDsl | InputKind::Network => false,
        }
    }

    // realises FR-156
    fn is_text(&self) -> bool {
        self.file.borrow().is_text()
    }

    fn revision(&self) -> i32 {
        self.revision
    }

    // realises FR-173, FR-175
    fn octet_count(&self) -> i32 {
        int_of(self.file.borrow().octets().len())
    }

    // realises FR-179
    fn bit_count(&self) -> i32 {
        int_of(self.file.borrow().bit_count())
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
        self.file.borrow().path().to_owned()
    }

    fn text(&self) -> String {
        self.file.borrow().text().to_owned()
    }

    fn state(&self) -> i32 {
        self.file.borrow().state().index() as i32
    }

    // realises FR-238
    fn produced(&self) -> bool {
        self.produced
    }

    // realises FR-126, FR-246, FR-247
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
        if path == self.path() {
            return;
        }
        self.name_path(path);
    }

    // realises FR-019, FR-021, FR-055, FR-165, FR-166, FR-238, FR-278
    // Replaces the octets by the encoding of the text where the user modified it, announces the
    // text and the *processing state*, and has the other presenters of a shared document follow;
    // a text as the text area holds the text of the document is no edit, and neither is a text
    // set for an input that has a producer.
    fn set_text(&mut self, text: String) {
        if self.produced || !self.file.borrow_mut().set_text(&text) {
            return;
        }
        self.notify_peers(false);
        self.announce_text();
        self.announce_state();
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

    #[qsignal(qml_name = "refusalChanged")]
    fn refusal_changed(&mut self);

    #[qsignal(qml_name = "refused")]
    fn refused(&mut self);

    #[qsignal(qml_name = "incrementProvided")]
    fn increment_provided(&mut self, rendering: String);

    // slots
    // realises FR-014
    /// Answers the question of `ask_to_save`: saves where `save`, then loads the pending path.
    #[qslot(qml_name = "answerSave")]
    fn answer_save(&mut self, save: bool) {
        if save {
            let _ = self.file.borrow_mut().save();
        }
        if let Some(path) = self.pending_path.take() {
            self.apply_path(&path);
        }
    }

    // realises FR-016, FR-056, FR-057, FR-059, FR-069, FR-106, FR-167, FR-168, FR-278, FR-279,
    // IR-015, IR-016
    /// Provides the pending *text increment*, writes its rendering to standard output followed by
    /// a line separator, emits `increment_provided`, schedules `applyIncrement` of the receiver
    /// with the *document identifier* where one is wired, saves the input file where it holds
    /// unsaved changes and a path is named, and, for a *shared document*, has every other
    /// *presenter* follow it and provide it to its *node*.
    ///
    /// * Where the octets are no valid UTF-8, no *text increment* is provided: the octets are
    ///   provided where they differ from those provided before, the *provided text* becomes
    ///   empty, and `applyOctets` of the receiver is scheduled with the *document identifier*.
    /// * The document of an *input* that has a *producer* provides nothing and is not saved:
    ///   the *producer* transmits it to the *node* (FR-240).
    #[qslot(qml_name = "idleExpired")]
    fn idle_expired(&mut self) {
        if self.produced {
            return;
        }
        self.provide(false);
        let saved = {
            let mut file = self.file.borrow_mut();
            if file.has_unsaved_changes() && !file.path().is_empty() {
                let _ = file.save();
                true
            } else {
                false
            }
        };
        self.notify_peers(true);
        if saved {
            self.announce_state();
        }
    }

    // realises FR-083, FR-185, FR-278
    /// Schedules `longIdleExpired` of the network editor, for the *network file* alone: the text
    /// was not modified for the *long idle time*; and takes every placeholder of the document
    /// as a digit 0, emitting `text_changed` where there was one, which the other *presenters*
    /// of a *shared document* follow.
    #[qslot(qml_name = "longIdleExpired")]
    fn long_idle_expired(&mut self) {
        if let (Some(receiver), InputKind::Network) = (&self.receiver, self.kind()) {
            invoke_method!(receiver, "longIdleExpired");
        }
        if self.file.borrow_mut().complete_placeholders() {
            self.notify_peers(false);
            self.announce_text();
        }
    }

    // realises FR-177, FR-180 to FR-184, FR-186, FR-187, FR-203, FR-205, FR-278
    /// Replaces `removed` digits from the digit `start` on by the digits `text` holds in the
    /// *editor mode* of index `mode`, as `InputFile::replace_digits` does, records that the
    /// document was edited, and announces the text and the *processing state*; yields the digit
    /// behind the digits inserted, and -1 where nothing was replaced: for a text that holds
    /// another character than digits of the mode, for nothing replaced by nothing, for an index
    /// that names no mode, and for a group whose code editor has no *mode switch*.
    #[qslot(qml_name = "replaceDigits")]
    fn replace_digits(&mut self, mode: i32, start: i32, removed: i32, text: String) -> i32 {
        let replaced = mode_of(mode)
            .filter(|_| self.octet_modes() && !self.produced)
            .and_then(|mode| digits_of_text(&text, mode).map(|digits| (mode, digits)))
            .and_then(|(mode, digits)| {
                self.file.borrow_mut().replace_digits(
                    mode,
                    count_of(start),
                    count_of(removed),
                    &digits,
                )
            });
        self.announce_edit(replaced.is_some());
        replaced.map_or(-1, int_of)
    }

    // realises FR-207, FR-278
    /// Takes the last edit made in hex or bin back, as `InputFile::undo` does; yields the digit
    /// of the *editor mode* of index `mode` behind what was restored, -1 where no edit is made.
    #[qslot(qml_name = "undo")]
    fn undo(&mut self, mode: i32) -> i32 {
        let cursor = if self.produced {
            None
        } else {
            self.file.borrow_mut().undo()
        };
        self.announce_edit(cursor.is_some());
        digit_of_bit(cursor, mode)
    }

    // realises FR-208, FR-278
    /// Makes the edit taken back last again, as `InputFile::redo` does; yields the digit of the
    /// *editor mode* of index `mode` behind what was inserted, -1 where no edit is taken back.
    #[qslot(qml_name = "redo")]
    fn redo(&mut self, mode: i32) -> i32 {
        let cursor = if self.produced {
            None
        } else {
            self.file.borrow_mut().redo()
        };
        self.announce_edit(cursor.is_some());
        digit_of_bit(cursor, mode)
    }

    // realises FR-202
    /// Yields the digits from the digit `start` to the digit `end`, exclusive, of the
    /// *editor mode* of index `mode` as text, as `text_of_digits` does; empty for an index that
    /// names no mode.
    #[qslot(qml_name = "digitsText")]
    fn digits_text(&self, mode: i32, start: i32, end: i32) -> String {
        mode_of(mode).map_or_else(String::new, |mode| {
            text_of_digits(
                &self
                    .file
                    .borrow()
                    .digits(mode, count_of(start), count_of(end)),
                mode,
            )
        })
    }

    // realises FR-086, FR-087, FR-238, FR-239, FR-277, FR-280
    /// Names the document of a *node*: records `identifier` and whether the document is an
    /// *input* that has a *producer*, forgets the *provided text*, the octets provided and the
    /// *diagnostics*, so that the next *text increment* carries the whole text, and names
    /// `path` as the file name field does, acquiring the *shared document* of `path` from the
    /// *document registry*: a file no other *presenter* holds is loaded anew, since it may have
    /// changed, and one another *presenter* holds is presented as it is; scheduled by the
    /// *node editor*.
    ///
    /// * The file of an *input* that has a *producer* is not loaded and not shared: its
    ///   document is empty until `setContent` brings what the *node* holds of it.
    /// * Whether to save is asked where the document presented before holds unsaved changes
    ///   and this group is its one *presenter*; a *shared document* another *presenter* holds
    ///   stays with its changes.
    #[qslot(qml_name = "nameDocument")]
    fn name_document(&mut self, identifier: String, path: String, produced: bool) {
        self.identifier = identifier;
        self.produced = produced;
        self.tracker = IncrementTracker::default();
        self.provided_octets = None;
        self.document_changed();
        self.set_diagnostics(vec![], vec![], vec![], vec![]);
        self.set_octet_marks(vec![], vec![], vec![], vec![]);
        self.name_path(path);
    }

    // realises FR-239
    /// Replaces the octets of the document of an *input* that has a *producer* by `octets`, the
    /// content the *node* holds of it, and announces the text and the *processing state*; does
    /// nothing for every other document, which the user edits; scheduled by the *node editor*.
    #[qslot(qml_name = "setContent")]
    fn set_content(&mut self, octets: Vec<u8>) {
        if !self.produced || octets == self.file.borrow().octets() {
            return;
        }
        {
            let mut file = self.file.borrow_mut();
            let path = file.path().to_owned();
            file.hold(&path, octets);
        }
        self.announce_text();
        self.announce_state();
    }

    // realises FR-278
    /// Takes that another *presenter* edited or saved the *shared document*: announces the text
    /// and the *processing state*, so that the code editor and the indicator of this group show
    /// the document as it is; scheduled by the *document registry* for the *presenter* that
    /// edited.
    #[qslot(qml_name = "followDocument")]
    fn follow_document(&mut self) {
        self.announce_text();
        self.announce_state();
    }

    // realises FR-279
    /// Takes that another *presenter* provided the *shared document* to its *node*: provides it
    /// to the receiver of this group as `idleExpired` does, without saving it, which the other
    /// *presenter* did; scheduled by the *document registry* for the *presenter* that provided.
    #[qslot(qml_name = "provideDocument")]
    fn provide_document(&mut self) {
        if self.produced {
            return;
        }
        self.provide(false);
    }

    // realises FR-158, FR-159
    /// Yields the number of rows of `per_row` octets the octets take, as [`row_count`] does;
    /// 0 for a negative `per_row`.
    #[qslot(qml_name = "rowCount")]
    fn row_count(&self, per_row: i32) -> i32 {
        int_of(row_count(
            self.file.borrow().octets().len(),
            count_of(per_row),
        ))
    }

    // realises FR-158, FR-159
    /// Yields the text of the row `row` of `per_row` octets in the *editor mode* of index
    /// `mode`, as [`row_text_of_bits`] does; empty for a negative row and for an index that
    /// names no mode.
    #[qslot(qml_name = "rowText")]
    fn row_text(&self, mode: i32, per_row: i32, row: i32) -> String {
        match (mode_of(mode), usize::try_from(row)) {
            (Some(mode), Ok(row)) => {
                let file = self.file.borrow();
                row_text_of_bits(
                    file.octets(),
                    file.bit_count(),
                    row,
                    mode,
                    count_of(per_row),
                )
            }
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
            self.file.borrow().octets().len(),
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

    // realises FR-126, FR-127, FR-128, FR-246, FR-247
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
    /// file*, the invoker of the receiver of its *text increments*: the network editor for the
    /// *network file*, the *node editor* for an *input group* of a *node window*, and, for the
    /// latter, the *document registry*, in which the group is registered as a *presenter*.
    ///
    /// * Called by the *workbench object* or by a *node editor* once, before the *front end*
    ///   binds to the group, and by the network editor whenever it names a group for an
    ///   *editor window*, which changes the kind; the group is registered as a *presenter* once.
    pub fn configure(
        &mut self,
        kind: InputKind,
        settings: Option<Rc<RefCell<SettingsStore>>>,
        receiver: Option<QmlMethodInvoker>,
        documents: Option<Rc<RefCell<DocumentRegistry>>>,
    ) {
        self.file = Rc::new(RefCell::new(InputFile::new(kind)));
        self.settings = settings;
        self.receiver = receiver;
        if let (Some(documents), None) = (&documents, &self.documents) {
            self.presenter = documents
                .borrow_mut()
                .register(self.get_qml_method_invoker());
        }
        if documents.is_some() {
            self.documents = documents;
        }
    }

    // realises FR-214, FR-221
    /// Releases and removes the *lock file* of the *network file*, where it is held; called by
    /// the *workbench object* when *product* terminates.
    pub fn release_lock(&mut self) {
        self.network_lock = None;
    }

    // realises FR-277, FR-280
    /// Releases the *shared document* the group presents, where it presents one, so that the
    /// *document registry* drops it once no *presenter* holds it; called by the *node editor*
    /// when its *node window* closes, and before another document is acquired.
    pub fn release_document(&mut self) {
        if let (Some(documents), Some(path)) = (&self.documents, self.shared_path.take()) {
            documents.borrow_mut().release(&path, self.presenter);
        }
    }

    // realises FR-167
    /// Yields the octets last provided to the *node* as a *binary document*, none where the
    /// document was provided as a text or not at all; read by the *node editor*.
    pub fn octets_provided(&self) -> Vec<u8> {
        self.provided_octets.clone().unwrap_or_default()
    }

    /// Yields which file this group edits.
    fn kind(&self) -> InputKind {
        self.file.borrow().kind()
    }

    // realises FR-056, FR-057, FR-059, FR-106, FR-167, FR-242, IR-015, IR-016
    /// Provides what the document holds to the receiver: the pending *text increment*, written
    /// to standard output and announced by `increment_provided`, as `applyIncrement`; or, where
    /// the octets are no valid UTF-8, the octets, as `applyOctets`.
    ///
    /// * Where `registering`, the document is provided although nothing is pending, by a
    ///   *text increment* that changes nothing, so that the receiver transmits the open request
    ///   of a document that is empty as well (FR-242); that one is no *text increment* of the
    ///   code editor, and is neither written nor announced.
    fn provide(&mut self, registering: bool) {
        if !self.file.borrow().is_text() {
            self.provide_octets();
            return;
        }
        let text = self.file.borrow().text().to_owned();
        let increment = match self.tracker.provide(&text) {
            Some(increment) => {
                let rendering = increment.rendering();
                let mut stdout = std::io::stdout().lock();
                let _ = writeln!(stdout, "{rendering}");
                let _ = stdout.flush();
                drop(stdout);
                self.increment_provided(rendering);
                increment
            }
            None if registering => TextIncrement::default(),
            None => return,
        };
        self.provided_octets = None;
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

    // realises FR-167, FR-168
    /// Provides the octets where they differ from those provided before: the *provided text*
    /// becomes empty, and `applyOctets` of the receiver is scheduled with the
    /// *document identifier* where one is wired.
    fn provide_octets(&mut self) {
        let octets = self.file.borrow().octets().to_vec();
        if self.provided_octets.as_deref() == Some(octets.as_slice()) {
            return;
        }
        self.provided_octets = Some(octets);
        self.tracker = IncrementTracker::default();
        if let Some(receiver) = &self.receiver {
            invoke_method!(receiver, "applyOctets", self.identifier.clone());
        }
    }

    // realises FR-278, FR-279
    /// Has every other *presenter* of the *shared document* follow it, and, where `provide`,
    /// provide it to its *node*; nothing for a document that is no *shared document*.
    fn notify_peers(&self, provide: bool) {
        if let (Some(documents), Some(path)) = (&self.documents, &self.shared_path) {
            let documents = documents.borrow();
            documents.follow_peers(path, self.presenter);
            if provide {
                documents.provide_peers(path, self.presenter);
            }
        }
    }

    // realises FR-019, FR-021, FR-278
    /// Announces an edit made in hex or bin, where `edited`: announces the text and the
    /// *processing state*, and has the other *presenters* of a *shared document* follow.
    fn announce_edit(&mut self, edited: bool) {
        if !edited {
            return;
        }
        self.notify_peers(false);
        self.announce_text();
        self.announce_state();
    }

    /// Counts the change of the octets and emits `text_changed`.
    fn announce_text(&mut self) {
        self.revision = self.revision.wrapping_add(1);
        self.text_changed();
    }

    // realises FR-090, FR-214, FR-217
    /// Loads the file at the path the settings hold for the *network file*, announces the state,
    /// and announces the path to the network editor where one is wired.
    ///
    /// * Where another genc³wb holds the *lock file* of that *network file*, no *network file*
    ///   is named, and the path stays in the settings as it is.
    pub fn load_initial(&mut self) {
        let path = self
            .settings
            .as_ref()
            .filter(|_| self.kind() == InputKind::Network)
            .map(|settings| settings.borrow().network_path().to_owned())
            .unwrap_or_default();
        let path = if self.lock_network(&path) {
            path
        } else {
            String::new()
        };
        let _ = self.file.borrow_mut().load(&path);
        self.announce_state();
        self.announce_path();
    }

    // realises FR-013, FR-014, FR-015, FR-280
    /// Names `path`: where the editor holds unsaved changes that no other *presenter* keeps,
    /// keeps the path pending and emits `ask_to_save`; otherwise applies it.
    fn name_path(&mut self, path: String) {
        if self.file.borrow().has_unsaved_changes() && self.is_sole_presenter() {
            self.pending_path = Some(path.clone());
            self.ask_to_save(path);
            return;
        }
        self.apply_path(&path);
    }

    // realises FR-280
    /// Yields whether no other *presenter* holds the document of this group: for a document that
    /// is no *shared document* always.
    fn is_sole_presenter(&self) -> bool {
        match (&self.documents, &self.shared_path) {
            (Some(documents), Some(path)) => documents.borrow().is_sole(path, self.presenter),
            _ => true,
        }
    }

    // realises FR-067
    /// Schedules `setNetworkPath` of the network editor, for the *network file* alone.
    fn announce_path(&self) {
        if let (Some(receiver), InputKind::Network) = (&self.receiver, self.kind()) {
            invoke_method!(receiver, "setNetworkPath", self.path());
        }
    }

    // realises FR-012, FR-013, FR-091, FR-214, FR-216, FR-239, FR-242, FR-253, FR-277, FR-280
    /// Stores `path` in the settings for the *network file*, takes the document of `path`, and
    /// notifies the *front end*.
    ///
    /// * The document of a *node* the user edits is the *shared document* of `path`, acquired
    ///   from the *document registry*: loaded where this group created it, presented as it is
    ///   where another *presenter* holds it (FR-280); the document presented before is released.
    /// * The document of a *node* the user edits is provided to the *node editor* at once, an
    ///   empty one included, so that the *node* is transmitted an open request for it without
    ///   an edit (FR-242).
    /// * Where another genc³wb holds the *lock file* of the *network file* at `path`, the
    ///   *network file* named before stays, and `path_changed` restores its path in the field.
    /// * The file of an *input* that has a *producer* is not loaded and not shared: its
    ///   document is named and empty.
    fn apply_path(&mut self, path: &str) {
        if !self.lock_network(path) {
            self.path_changed();
            return;
        }
        let kind = self.kind();
        if let (Some(settings), InputKind::Network) = (&self.settings, kind) {
            let mut settings = settings.borrow_mut();
            settings.set_network_path(path);
            let _ = settings.store();
        }
        let octets_before = self.file.borrow().octets().to_vec();
        if self.produced {
            self.release_document();
            let file = Rc::new(RefCell::new(InputFile::new(kind)));
            file.borrow_mut().hold(path, Vec::new());
            self.file = file;
        } else if let Some(documents) = self.documents.clone() {
            self.release_document();
            let acquired = documents.borrow_mut().acquire(path, kind, self.presenter);
            if acquired.created {
                let _ = acquired.file.borrow_mut().load(path);
            }
            self.file = acquired.file;
            self.shared_path = Some(path.to_owned());
        } else {
            let _ = self.file.borrow_mut().load(path);
        }
        self.path_changed();
        if self.file.borrow().octets() != octets_before.as_slice() {
            self.announce_text();
        }
        self.announce_state();
        self.announce_path();
        if !self.identifier.is_empty() && !self.produced {
            self.provide(true);
        }
    }

    // realises FR-214, FR-216, FR-217, FR-219, FR-220, FR-221, FR-222, FR-253
    /// Takes the *lock file* of the *network file* at `path`, releasing the one held before;
    /// yields whether `path` may be named, which every file but a *network file* may.
    ///
    /// * The empty path names no *network file* and needs no *lock file*; a *lock file* that
    ///   cannot be created is passed over (FR-222).
    /// * The *lock file* already held for the same file is kept.
    /// * Where another genc³wb holds it, the *refusal* is held and announced by
    ///   `refusal_changed` and `refused`, and the *lock file* held before is kept.
    fn lock_network(&mut self, path: &str) -> bool {
        if self.kind() != InputKind::Network {
            return true;
        }
        let held = self
            .network_lock
            .as_ref()
            .is_some_and(|lock| lock.guards(Path::new(path)));
        if !held && !path.is_empty() {
            match FileLock::take(Path::new(path)) {
                Taking::Held(lock) => self.network_lock = Some(lock),
                Taking::Unlockable(_) => self.network_lock = None,
                Taking::Refused(refusal) => {
                    self.refusal = Some(refusal);
                    self.refusal_changed();
                    self.refused();
                    return false;
                }
            }
        } else if path.is_empty() {
            self.network_lock = None;
        }
        if self.refusal.take().is_some() {
            self.refusal_changed();
        }
        true
    }

    // realises FR-017
    /// Emits `state_changed` where the *processing state* changed.
    fn announce_state(&mut self) {
        let state = self.file.borrow().state();
        if state == self.announced_state {
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

/// Yields the digit of the *editor mode* of index `mode` that begins at the bit `bit`; -1 where
/// there is no bit, and for an index that names no mode or names txt.
fn digit_of_bit(bit: Option<usize>, mode: i32) -> i32 {
    let unit = mode_of(mode).map_or(0, EditorMode::digit_bits);
    match (bit, unit) {
        (None, _) | (_, 0) => -1,
        (Some(bit), unit) => int_of(bit / unit),
    }
}

/// Yields `value` as a count; 0 where it is negative.
fn count_of(value: i32) -> usize {
    usize::try_from(value).unwrap_or(0)
}

/// Yields `value` as the integer QML takes; the greatest one where it is larger.
fn int_of(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}
