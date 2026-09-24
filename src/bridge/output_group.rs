//! The *bridged type* `OutputGroup`: the *output pages* as the *front end* binds to them.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::output::OutputPages;

use qtbridge::qobject;

// realises FR-027
/// The output group as the *front end* binds to it.
#[derive(Default)]
pub struct OutputGroup {
    /// the *output pages*
    pages: OutputPages,
}

// realises FR-027, FR-030, FR-032 to FR-036, FR-042, FR-044, FR-102, FR-107, FR-108, FR-110,
// FR-111, FR-122, FR-125, FR-150 to FR-152
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
    qproperty!("diagnostics", Read = diagnostics, Notify = content_changed);

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

    // realises FR-125
    fn diagnostics(&self) -> String {
        self.pages.diagnostics().to_owned()
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

    // signals
    #[qsignal(qml_name = "pageChanged")]
    fn page_changed(&mut self);

    #[qsignal(qml_name = "contentChanged")]
    fn content_changed(&mut self);

    // slots
    // realises FR-032, FR-034
    #[qslot(qml_name = "next")]
    fn next(&mut self) {
        if self.pages.next() {
            self.page_changed();
            self.content_changed();
        }
    }

    // realises FR-033, FR-035
    #[qslot(qml_name = "previous")]
    fn previous(&mut self) {
        if self.pages.previous() {
            self.page_changed();
            self.content_changed();
        }
    }

    // realises FR-102, FR-110, FR-122
    /// Replaces the pages by those of the *outputs* at `paths`, and emits both signals;
    /// scheduled by the *node editor*.
    #[qslot(qml_name = "setPaths")]
    fn set_paths(&mut self, paths: Vec<String>) {
        self.pages = OutputPages::of_paths(&paths);
        self.page_changed();
        self.content_changed();
    }

    // realises FR-111
    /// Reads every *output* again and emits `content_changed`; scheduled by the *node editor*.
    #[qslot(qml_name = "reload")]
    fn reload(&mut self) {
        self.pages.reload();
        self.content_changed();
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
            self.page_changed();
        }
        self.content_changed();
    }
}
