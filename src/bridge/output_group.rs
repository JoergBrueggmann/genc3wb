//! The *bridged type* `OutputGroup`: the *output pages* as the *front end* binds to them.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::octet_view::{EditorMode, is_utf8, row_count, row_text};
use crate::core::output::OutputPages;

use qtbridge::qobject;

// realises FR-027
/// The output group as the *front end* binds to it.
#[derive(Default)]
pub struct OutputGroup {
    /// the *output pages*
    pages: OutputPages,
    /// the number of changes of the content since the group was created
    revision: i32,
    /// the path of the *output* whose *output page* was presented last; empty before the first
    presented_path: String,
}

// realises FR-027, FR-030, FR-032 to FR-036, FR-042, FR-044, FR-102, FR-107, FR-108, FR-110,
// FR-111, FR-122, FR-125, FR-150 to FR-152, FR-154, FR-156 to FR-159, FR-163, FR-173, FR-175,
// FR-211 to FR-213
#[qobject(NoQmlElement)]
impl OutputGroup {
    qproperty!("pageIndex", Read = page_index, Notify = page_changed);
    qproperty!("pageCount", Read = page_count, Notify = page_changed);
    qproperty!("hasNext", Read = has_next, Notify = page_changed);
    qproperty!("hasPrevious", Read = has_previous, Notify = page_changed);
    qproperty!(
        "diagnosticsPage",
        Read = diagnostics_page,
        Notify = page_changed
    );
    qproperty!("filePath", Read = file_path, Notify = page_changed);
    qproperty!("fileContent", Read = file_content, Notify = content_changed);
    qproperty!("isText", Read = is_text, Notify = content_changed);
    qproperty!("revision", Read = revision, Notify = content_changed);
    qproperty!("octetCount", Read = octet_count, Notify = content_changed);
    qproperty!("diagnostics", Read = diagnostics, Notify = content_changed);
    qproperty!("outdated", Read = outdated, Notify = content_changed);
    qproperty!(
        "storeFailure",
        Read = store_failure,
        Notify = content_changed
    );

    // getters
    fn page_index(&self) -> i32 {
        self.pages.current() as i32
    }

    fn page_count(&self) -> i32 {
        self.pages.page_count() as i32
    }

    fn has_next(&self) -> bool {
        self.pages.has_next()
    }

    fn has_previous(&self) -> bool {
        self.pages.has_previous()
    }

    // realises FR-036, FR-125
    fn diagnostics_page(&self) -> bool {
        self.pages.is_diagnostics_page()
    }

    // realises FR-125, FR-213
    fn diagnostics(&self) -> String {
        self.pages.listing()
    }

    // realises FR-211, FR-212
    fn outdated(&self) -> bool {
        self.pages.is_outdated()
    }

    // realises FR-212
    fn store_failure(&self) -> String {
        self.pages.store_failure().to_owned()
    }

    fn file_path(&self) -> String {
        self.pages
            .current_file()
            .map(|file| file.path.clone())
            .unwrap_or_default()
    }

    fn file_content(&self) -> String {
        self.pages
            .current_file()
            .map(|file| file.content.clone())
            .unwrap_or_default()
    }

    // realises FR-156
    fn is_text(&self) -> bool {
        is_utf8(self.octets())
    }

    fn revision(&self) -> i32 {
        self.revision
    }

    // realises FR-173, FR-175
    fn octet_count(&self) -> i32 {
        i32::try_from(self.octets().len()).unwrap_or(i32::MAX)
    }

    // signals
    #[qsignal(qml_name = "pageChanged")]
    fn page_changed(&mut self);

    #[qsignal(qml_name = "contentChanged")]
    fn content_changed(&mut self);

    #[qsignal(qml_name = "documentChanged")]
    fn document_changed(&mut self);

    // slots
    // realises FR-032, FR-034
    #[qslot(qml_name = "next")]
    fn next(&mut self) {
        if self.pages.next() {
            self.announce_page();
            self.announce_content();
        }
    }

    // realises FR-033, FR-035
    #[qslot(qml_name = "previous")]
    fn previous(&mut self) {
        if self.pages.previous() {
            self.announce_page();
            self.announce_content();
        }
    }

    // realises FR-158, FR-159
    /// Yields the number of rows of `per_row` octets the octets of the presented *output* take,
    /// as [`row_count`] does; 0 for a negative `per_row`.
    #[qslot(qml_name = "rowCount")]
    fn row_count(&self, per_row: i32) -> i32 {
        let per_row = usize::try_from(per_row).unwrap_or(0);
        i32::try_from(row_count(self.octets().len(), per_row)).unwrap_or(i32::MAX)
    }

    // realises FR-158, FR-159
    /// Yields the text of the row `row` of `per_row` octets of the presented *output* in the
    /// *editor mode* of index `mode`, as [`row_text`] does; empty for a negative row and for an
    /// index that names no mode.
    #[qslot(qml_name = "rowText")]
    fn row_text(&self, mode: i32, per_row: i32, row: i32) -> String {
        let per_row = usize::try_from(per_row).unwrap_or(0);
        match (mode_of(mode), usize::try_from(row)) {
            (Some(mode), Ok(row)) => row_text(self.octets(), row, mode, per_row),
            (None, _) | (_, Err(_)) => String::new(),
        }
    }

    // realises FR-102, FR-110, FR-122
    /// Replaces the pages by those of the *outputs* at `paths`, and emits both signals;
    /// scheduled by the *node editor*.
    #[qslot(qml_name = "setPaths")]
    fn set_paths(&mut self, paths: Vec<String>) {
        self.pages = OutputPages::of_paths(&paths);
        self.presented_path.clear();
        self.announce_page();
        self.announce_content();
    }

    // realises FR-111, FR-151, FR-211
    /// Reads every *output* again, after which the *output pages* are not *outdated*, and emits
    /// `content_changed`, and `page_changed` first where the line of a failed *store request*
    /// that is gone switched the presented page; scheduled by the *node editor*.
    #[qslot(qml_name = "reload")]
    fn reload(&mut self) {
        if self.pages.reload() {
            self.announce_page();
        }
        self.announce_content();
    }

    // realises FR-150, FR-211, FR-212, FR-213
    /// Takes the *output pages* as *outdated* by a *store request* that failed with `message`
    /// and emits `content_changed`, and `page_changed` first where the line the
    /// *diagnostics page* lists for it switched the presented page; scheduled by the
    /// *node editor*.
    #[qslot(qml_name = "setStoreFailure")]
    fn set_store_failure(&mut self, message: String) {
        if self.pages.set_store_failure(&message) {
            self.announce_page();
        }
        self.announce_content();
    }

    // realises FR-107, FR-108, FR-125, FR-150, FR-151, FR-152
    /// Replaces the text of the *diagnostics page* and emits `content_changed`, and
    /// `page_changed` first where the *diagnostics* that appeared or are gone switched the
    /// presented page; scheduled by the *node editor*.
    #[qslot(qml_name = "setDiagnostics")]
    fn set_diagnostics(&mut self, text: String) {
        if text == self.pages.diagnostics() {
            return;
        }
        if self.pages.set_diagnostics(&text) {
            self.announce_page();
        }
        self.announce_content();
    }
}

impl OutputGroup {
    // realises FR-163
    /// Yields the octets of the presented *output*, none on the *diagnostics page*.
    fn octets(&self) -> &[u8] {
        self.pages
            .current_file()
            .map_or(&[], |file| file.octets.as_slice())
    }

    // realises FR-156
    /// Emits `page_changed`, and `document_changed` where the presented page is the
    /// *output page* of another *output* than the one presented last, so that the
    /// *diagnostics page* presented in between leaves the selection of the *mode switch*.
    fn announce_page(&mut self) {
        self.page_changed();
        let path = self.file_path();
        if !self.pages.is_diagnostics_page() && path != self.presented_path {
            self.presented_path = path;
            self.document_changed();
        }
    }

    /// Counts the change of the content and emits `content_changed`.
    fn announce_content(&mut self) {
        self.revision = self.revision.wrapping_add(1);
        self.content_changed();
    }
}

/// Yields the *editor mode* of the index `mode`, `None` where the index names none.
fn mode_of(mode: i32) -> Option<EditorMode> {
    usize::try_from(mode).ok().and_then(EditorMode::of_index)
}
