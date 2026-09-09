//! The standard output, the standard error, the exit code and the *output files* of a run, arranged as *output pages*.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::runner::RunResult;
use crate::core::settings::{MAX_FILE_NUMBER, MIN_FILE_NUMBER, Settings};

use std::collections::BTreeMap;
use std::fmt;
use std::fs;

// realises FR-027
/// The kind of an *output page*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputPageKind {
    /// what the *compiler-compiler* wrote to standard output
    StandardOutput,
    /// what it wrote to standard error, with its exit code
    StandardError,
    /// the content of one *output file*, or the *new file page*
    OutputFile,
}

impl OutputPageKind {
    /// Yields the index of the kind in the order of the variants.
    pub fn index(self) -> usize {
        match self {
            OutputPageKind::StandardOutput => 0,
            OutputPageKind::StandardError => 1,
            OutputPageKind::OutputFile => 2,
        }
    }
}

// realises FR-030, FR-031, FR-037, FR-039
/// One *output file* as an *output file* page presents it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputFile {
    /// the number of the file, `MIN_FILE_NUMBER..=MAX_FILE_NUMBER`
    pub number: u8,
    /// the path named on the page, empty where none is named
    pub path: String,
    /// the content of the file, empty where it could not be read
    pub content: String,
}

// realises FR-038, FR-039, IR-009
/// Why an *output file* could not be enabled or read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OutputError {
    /// an *output file* number outside `MIN_FILE_NUMBER..=MAX_FILE_NUMBER`
    FileNumberOutOfRange(u8),
    /// the file could not be read; carries the path and the reason of the operating system
    Io { path: String, reason: String },
}

impl fmt::Display for OutputError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputError::FileNumberOutOfRange(number) => {
                write!(
                    formatter,
                    "output file number {number} is not within {MIN_FILE_NUMBER}..={MAX_FILE_NUMBER}"
                )
            }
            OutputError::Io { path, reason } => write!(formatter, "{path}: {reason}"),
        }
    }
}

impl std::error::Error for OutputError {}

/// Yields the content of the *output file* at `path`.
///
/// # Errors
/// Returns [`OutputError::Io`] where the file cannot be read; an empty path is such a file.
fn content_of(path: &str) -> Result<String, OutputError> {
    fs::read(path)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .map_err(|error| OutputError::Io {
            path: path.to_owned(),
            reason: error.to_string(),
        })
}

// realises FR-027, FR-031, FR-032, FR-033, FR-034, FR-035, FR-036, FR-040, FR-041
/// The *output pages* of the last run and which of them is presented.
///
/// * The pages are, in order: standard output, standard error, the enabled *output files* by
///   number, and the *new file page* while fewer than nine are enabled.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OutputPages {
    /// what the last run wrote to standard output
    stdout: String,
    /// what the last run wrote to standard error
    stderr: String,
    /// the exit code of the last run, `None` before the first run and after `clear`
    exit_code: Option<i32>,
    /// the enabled *output files*, by number
    files: BTreeMap<u8, OutputFile>,
    /// the index of the presented page
    current: usize,
}

impl OutputPages {
    /// Creates the pages of no run, with the *output files* of `settings` enabled and read.
    pub fn from_settings(settings: &Settings) -> OutputPages {
        let mut pages = OutputPages::default();
        for number in settings.output_numbers() {
            let path = settings.output_path(number).unwrap_or_default();
            let _ = pages.enable_file(number, path);
        }
        pages
    }

    // realises FR-040
    /// Replaces the standard output, the standard error and the exit code, and reads every enabled
    /// *output file* again.
    pub fn set_result(&mut self, result: &RunResult) {
        self.stdout = result.stdout.clone();
        self.stderr = result.stderr.clone();
        self.exit_code = Some(result.exit_code);
        for file in self.files.values_mut() {
            file.content = content_of(&file.path).unwrap_or_default();
        }
    }

    // realises FR-041
    /// Clears the content of every page: the texts become empty, the exit code absent.
    pub fn clear(&mut self) {
        self.stdout.clear();
        self.stderr.clear();
        self.exit_code = None;
        for file in self.files.values_mut() {
            file.content.clear();
        }
    }

    /// Yields what the last run wrote to standard output.
    pub fn stdout(&self) -> &str {
        &self.stdout
    }

    /// Yields what the last run wrote to standard error.
    pub fn stderr(&self) -> &str {
        &self.stderr
    }

    // realises FR-029
    /// Yields the exit code of the last run, `None` where none has run since the last clearing.
    pub fn exit_code(&self) -> Option<i32> {
        self.exit_code
    }

    /// Yields whether the *new file page* is presented after the enabled files.
    fn has_new_file_page(&self) -> bool {
        self.files.len() < usize::from(MAX_FILE_NUMBER - MIN_FILE_NUMBER + 1)
    }

    /// Yields the lowest number not yet enabled, `None` where every number is enabled.
    fn lowest_unused_number(&self) -> Option<u8> {
        (MIN_FILE_NUMBER..=MAX_FILE_NUMBER).find(|number| !self.files.contains_key(number))
    }

    // realises FR-027, FR-031, FR-036
    /// Yields the number of pages: two, plus the enabled *output files*, plus the *new file page*
    /// while fewer than nine are enabled.
    pub fn page_count(&self) -> usize {
        2 + self.files.len() + usize::from(self.has_new_file_page())
    }

    /// Yields the index of the presented page.
    pub fn current(&self) -> usize {
        self.current
    }

    /// Yields the kind of the presented page.
    pub fn current_kind(&self) -> OutputPageKind {
        match self.current {
            0 => OutputPageKind::StandardOutput,
            1 => OutputPageKind::StandardError,
            _ => OutputPageKind::OutputFile,
        }
    }

    /// Yields the number of the *output file* of the presented page: the file's number, or the
    /// number the *new file page* would enable; `None` on the first two pages.
    pub fn current_number(&self) -> Option<u8> {
        if self.current < 2 {
            return None;
        }
        self.files
            .keys()
            .nth(self.current - 2)
            .copied()
            .or_else(|| self.lowest_unused_number())
    }

    /// Yields the *output file* of the presented page, `None` on the first two pages and on the
    /// *new file page*.
    pub fn current_file(&self) -> Option<&OutputFile> {
        if self.current < 2 {
            return None;
        }
        self.files.values().nth(self.current - 2)
    }

    // realises FR-032, FR-034
    /// Presents the next page; yields whether there was one.
    // The name is the one the design gives the slot of the front end; the pages are no iterator.
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> bool {
        if !self.has_next() {
            return false;
        }
        self.current += 1;
        true
    }

    /// Yields whether a page follows the presented one.
    pub fn has_next(&self) -> bool {
        self.current + 1 < self.page_count()
    }

    // realises FR-033, FR-035
    /// Presents the previous page; yields whether there was one.
    pub fn previous(&mut self) -> bool {
        if !self.has_previous() {
            return false;
        }
        self.current -= 1;
        true
    }

    /// Yields whether a page precedes the presented one.
    pub fn has_previous(&self) -> bool {
        self.current > 0
    }

    // realises FR-030, FR-038, FR-039, IR-009
    /// Enables the *output file* `number` at `path` and reads it; the content is empty where the
    /// file cannot be read.
    ///
    /// * The presented page stays the page of that number where it was the *new file page* for it.
    ///
    /// # Errors
    /// Returns [`OutputError::FileNumberOutOfRange`] where `number` is not within
    /// `MIN_FILE_NUMBER..=MAX_FILE_NUMBER`.
    pub fn enable_file(&mut self, number: u8, path: &str) -> Result<(), OutputError> {
        if !(MIN_FILE_NUMBER..=MAX_FILE_NUMBER).contains(&number) {
            return Err(OutputError::FileNumberOutOfRange(number));
        }
        let content = content_of(path).unwrap_or_default();
        self.files.insert(
            number,
            OutputFile {
                number,
                path: path.to_owned(),
                content,
            },
        );
        self.current = self.current.min(self.page_count() - 1);
        Ok(())
    }

    // realises FR-037
    /// Disables the *output file* `number`, so that its page is no longer presented.
    pub fn disable_file(&mut self, number: u8) {
        self.files.remove(&number);
        self.current = self.current.min(self.page_count() - 1);
    }
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : OutputGroup::next, OutputGroup::previous, OutputGroup::set_file_enabled, OutputGroup::set_file_path, OutputGroup::clear, OutputGroup::set_result */
#[cfg(test)]
mod tests {
    use super::*;

    fn case_dir(name: &str) -> std::path::PathBuf {
        let dir =
            std::env::temp_dir().join(format!("genc3wb-output-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("the temporary directory of the test can be created");
        dir
    }

    #[test]
    fn pages_of_no_run_are_the_two_fixed_pages_and_the_new_file_page() {
        // FR-027, FR-031, FR-036
        let pages = OutputPages::default();
        assert_eq!(pages.page_count(), 3);
        assert_eq!(pages.current(), 0);
        assert_eq!(pages.current_kind(), OutputPageKind::StandardOutput);
        assert_eq!(pages.current_number(), None);
        assert_eq!(pages.exit_code(), None);
    }

    #[test]
    fn navigation_stops_at_both_ends() {
        // FR-032, FR-033, FR-034, FR-035
        let mut pages = OutputPages::default();
        assert!(!pages.has_previous());
        assert!(!pages.previous());
        assert!(pages.next());
        assert_eq!(pages.current_kind(), OutputPageKind::StandardError);
        assert!(pages.next());
        assert_eq!(pages.current_kind(), OutputPageKind::OutputFile);
        assert_eq!(pages.current_number(), Some(1));
        assert_eq!(pages.current_file(), None);
        assert!(!pages.has_next());
        assert!(!pages.next());
        assert!(pages.previous());
        assert_eq!(pages.current(), 1);
    }

    #[test]
    fn enabled_files_are_pages_in_the_order_of_their_numbers() {
        // FR-027, FR-030, FR-031, FR-038
        let mut pages = OutputPages::default();
        pages.enable_file(5, "five.txt").expect("5 is within range");
        pages.enable_file(2, "two.txt").expect("2 is within range");
        assert_eq!(pages.page_count(), 5);
        pages.next();
        pages.next();
        assert_eq!(pages.current_number(), Some(2));
        assert_eq!(
            pages.current_file().map(|file| file.path.as_str()),
            Some("two.txt")
        );
        pages.next();
        assert_eq!(pages.current_number(), Some(5));
        pages.next();
        assert_eq!(pages.current_number(), Some(1));
        assert_eq!(pages.current_file(), None);
    }

    #[test]
    fn nine_files_leave_no_new_file_page() {
        // FR-031
        let mut pages = OutputPages::default();
        for number in MIN_FILE_NUMBER..=MAX_FILE_NUMBER {
            pages.enable_file(number, "").expect("within range");
        }
        assert_eq!(pages.page_count(), 11);
        assert_eq!(
            pages.enable_file(0, ""),
            Err(OutputError::FileNumberOutOfRange(0))
        );
        assert_eq!(
            pages.enable_file(10, ""),
            Err(OutputError::FileNumberOutOfRange(10))
        );
    }

    #[test]
    fn disabled_file_is_no_longer_a_page() {
        // FR-037
        let mut pages = OutputPages::default();
        pages.enable_file(1, "").expect("within range");
        pages.enable_file(2, "").expect("within range");
        for _ in 0..4 {
            pages.next();
        }
        assert_eq!(pages.current(), 4);
        pages.disable_file(2);
        assert_eq!(pages.page_count(), 4);
        assert_eq!(pages.current(), 3);
        pages.disable_file(7);
        assert_eq!(pages.page_count(), 4);
    }

    #[test]
    fn result_replaces_the_pages_and_reads_the_files() {
        // FR-028, FR-029, FR-039, FR-040, IR-009
        let dir = case_dir("result");
        let path = dir.join("out1.txt");
        let path_text = path.to_string_lossy().into_owned();
        let mut pages = OutputPages::default();
        pages.enable_file(1, &path_text).expect("within range");
        assert_eq!(pages.files[&1].content, "");
        fs::write(&path, "generated").expect("the file can be written");
        pages.set_result(&RunResult {
            stdout: "out".into(),
            stderr: "err".into(),
            exit_code: 2,
        });
        assert_eq!(pages.stdout(), "out");
        assert_eq!(pages.stderr(), "err");
        assert_eq!(pages.exit_code(), Some(2));
        assert_eq!(pages.files[&1].content, "generated");
        let _ = fs::remove_dir_all(&dir);
    }

    #[test]
    fn clearing_empties_every_page() {
        // FR-041
        let mut pages = OutputPages::default();
        pages.enable_file(3, "").expect("within range");
        pages.set_result(&RunResult {
            stdout: "out".into(),
            stderr: "err".into(),
            exit_code: 0,
        });
        pages.clear();
        assert_eq!(pages.stdout(), "");
        assert_eq!(pages.stderr(), "");
        assert_eq!(pages.exit_code(), None);
        assert_eq!(pages.files[&3].content, "");
        assert_eq!(pages.page_count(), 4);
    }

    #[test]
    fn pages_of_the_settings_enable_the_stored_files() {
        // FR-048
        let mut settings = Settings::default();
        settings
            .set_output_path(4, "four.txt")
            .expect("within range");
        let pages = OutputPages::from_settings(&settings);
        assert_eq!(pages.page_count(), 4);
        assert_eq!(pages.files[&4].path, "four.txt");
    }
}
