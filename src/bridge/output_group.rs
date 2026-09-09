//! The *bridged type* `OutputGroup`: the *output pages* as the *front end* binds to them.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::output::OutputPages;
use crate::core::runner::RunResult;
use crate::core::settings::Settings;

use qtbridge::qobject;

use std::cell::RefCell;
use std::rc::Rc;

// realises FR-027
/// The output group as the *front end* binds to it.
#[derive(Default)]
pub struct OutputGroup {
    /// the *output pages*
    pages: OutputPages,
    /// the settings, to store and remove the paths of the *output files*; wired by the *workbench object*
    settings: Option<Rc<RefCell<Settings>>>,
}

// realises FR-027 to FR-042, FR-044
#[qobject(NoQmlElement)]
impl OutputGroup {
    qproperty!("stdOut", Read = std_out, Notify = results_changed);
    qproperty!("stdErr", Read = std_err, Notify = results_changed);
    qproperty!("exitCode", Read = exit_code, Notify = results_changed);
    qproperty!("hasRun", Read = has_run, Notify = results_changed);
    qproperty!("pageIndex", Read = page_index, Notify = page_changed);
    qproperty!("pageCount", Read = page_count, Notify = page_changed);
    qproperty!("pageKind", Read = page_kind, Notify = page_changed);
    qproperty!("hasNext", Read = has_next, Notify = page_changed);
    qproperty!("hasPrevious", Read = has_previous, Notify = page_changed);
    qproperty!("fileNumber", Read = file_number, Notify = page_changed);
    qproperty!(
        "fileEnabled",
        Read = file_enabled,
        Write = set_file_enabled,
        Notify = page_changed
    );
    qproperty!(
        "filePath",
        Read = file_path,
        Write = set_file_path,
        Notify = page_changed
    );
    qproperty!("fileContent", Read = file_content, Notify = results_changed);

    // getters
    fn std_out(&self) -> String {
        self.pages.stdout().to_owned()
    }

    fn std_err(&self) -> String {
        self.pages.stderr().to_owned()
    }

    fn exit_code(&self) -> i32 {
        self.pages.exit_code().unwrap_or(0)
    }

    fn has_run(&self) -> bool {
        self.pages.exit_code().is_some()
    }

    fn page_index(&self) -> i32 {
        self.pages.current() as i32
    }

    fn page_count(&self) -> i32 {
        self.pages.page_count() as i32
    }

    fn page_kind(&self) -> i32 {
        self.pages.current_kind().index() as i32
    }

    fn has_next(&self) -> bool {
        self.pages.has_next()
    }

    fn has_previous(&self) -> bool {
        self.pages.has_previous()
    }

    fn file_number(&self) -> i32 {
        i32::from(self.pages.current_number().unwrap_or(0))
    }

    fn file_enabled(&self) -> bool {
        self.pages.current_file().is_some()
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

    // setters
    // realises FR-037, FR-038, FR-049
    // Enables the *output file* of the presented page at its path, or disables it and removes its
    // path from the settings.
    fn set_file_enabled(&mut self, enabled: bool) {
        let Some(number) = self.pages.current_number() else {
            return;
        };
        if enabled == self.file_enabled() {
            return;
        }
        if enabled {
            let path = self.file_path();
            let _ = self.pages.enable_file(number, &path);
            self.store_output_path(number, Some(&path));
        } else {
            self.pages.disable_file(number);
            self.store_output_path(number, None);
        }
        self.page_changed();
        self.results_changed();
    }

    // realises FR-038, FR-039, FR-049
    // Names the path of the *output file* of the presented page, stores it, and reads the file.
    fn set_file_path(&mut self, path: String) {
        let Some(number) = self.pages.current_number() else {
            return;
        };
        if self.file_enabled() && path == self.file_path() {
            return;
        }
        let _ = self.pages.enable_file(number, &path);
        self.store_output_path(number, Some(&path));
        self.page_changed();
        self.results_changed();
    }

    // signals
    #[qsignal(qml_name = "resultsChanged")]
    fn results_changed(&mut self);

    #[qsignal(qml_name = "pageChanged")]
    fn page_changed(&mut self);

    // slots
    // realises FR-032, FR-034
    #[qslot(qml_name = "next")]
    fn next(&mut self) {
        if self.pages.next() {
            self.page_changed();
            self.results_changed();
        }
    }

    // realises FR-033, FR-035
    #[qslot(qml_name = "previous")]
    fn previous(&mut self) {
        if self.pages.previous() {
            self.page_changed();
            self.results_changed();
        }
    }

    // realises FR-041
    /// Clears every page and emits `results_changed`; scheduled by the runner group.
    #[qslot(qml_name = "clear")]
    fn clear(&mut self) {
        self.pages.clear();
        self.results_changed();
    }

    // realises FR-040
    /// Replaces the results and emits `results_changed`; scheduled by the runner group.
    #[qslot(qml_name = "setResult")]
    fn set_result(&mut self, stdout: String, stderr: String, exit_code: i32) {
        self.pages.set_result(&RunResult {
            stdout,
            stderr,
            exit_code,
        });
        self.results_changed();
    }
}

impl OutputGroup {
    /// Wires the group: the settings, whose *output files* become the pages.
    ///
    /// * Called by the *workbench object* once, before the *front end* binds to the group.
    pub fn configure(&mut self, settings: Rc<RefCell<Settings>>) {
        self.pages = OutputPages::from_settings(&settings.borrow());
        self.settings = Some(settings);
    }

    /// Stores the path of one *output file* in the settings, or removes it where `path` is `None`.
    fn store_output_path(&mut self, number: u8, path: Option<&str>) {
        let Some(settings) = &self.settings else {
            return;
        };
        let mut settings = settings.borrow_mut();
        match path {
            Some(path) => {
                let _ = settings.set_output_path(number, path);
            }
            None => settings.remove_output_path(number),
        }
        let _ = settings.save(&Settings::default_path());
    }
}
