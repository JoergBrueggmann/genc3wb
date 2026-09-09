//! One input file: its path, its text, its *file text*, loading, saving and the *processing state*.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use std::fmt;
use std::fs;
use std::path::Path;

// realises FR-001
/// Which of the two input files an *input group* edits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    /// the file that configures syntax and generators
    CompilerCompilerInput,
    /// the file that is parsed according to that configuration
    CompilerInput,
}

impl InputKind {
    /// Yields the index of the kind: 0 for the *compiler-compiler input file*, 1 for the other.
    pub fn index(self) -> usize {
        match self {
            InputKind::CompilerCompilerInput => 0,
            InputKind::CompilerInput => 1,
        }
    }

    /// Yields the kind of an index, `None` where the index names none.
    pub fn of_index(index: usize) -> Option<InputKind> {
        match index {
            0 => Some(InputKind::CompilerCompilerInput),
            1 => Some(InputKind::CompilerInput),
            _ => None,
        }
    }

    // realises FR-011
    /// Yields the caption of the *input group* and of its file selector.
    pub fn caption(self) -> &'static str {
        match self {
            InputKind::CompilerCompilerInput => "Compiler-compiler input file",
            InputKind::CompilerInput => "Compiler input file",
        }
    }

    // realises FR-011
    /// Yields the file filter of the file selector, as a file dialog names it.
    pub fn file_filter(self) -> &'static str {
        "All files (*)"
    }
}

// realises FR-018, FR-019, FR-020, FR-021
/// The *processing state* of an *input group*.
///
/// * The order of the variants is the order of the images the indicator shows.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ProcessingState {
    /// the named file exists and the text does not differ from it
    ValidFileTextUntouched,
    /// the named file exists and the text differs from it
    ValidFileTextChanged,
    /// it is not established that the file exists, and the text was not edited since
    #[default]
    UnknownFileTextUntouched,
    /// it is not established that the file exists, and the text was edited since
    UnknownFileTextChanged,
}

impl ProcessingState {
    /// Yields the index of the state in the order of the variants.
    pub fn index(self) -> usize {
        match self {
            ProcessingState::ValidFileTextUntouched => 0,
            ProcessingState::ValidFileTextChanged => 1,
            ProcessingState::UnknownFileTextUntouched => 2,
            ProcessingState::UnknownFileTextChanged => 3,
        }
    }

    /// Yields the state of an index, `None` where the index names none.
    pub fn of_index(index: usize) -> Option<ProcessingState> {
        match index {
            0 => Some(ProcessingState::ValidFileTextUntouched),
            1 => Some(ProcessingState::ValidFileTextChanged),
            2 => Some(ProcessingState::UnknownFileTextUntouched),
            3 => Some(ProcessingState::UnknownFileTextChanged),
            _ => None,
        }
    }
}

// realises IR-009, IR-010
/// Why an input file could not be loaded or saved.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InputFileError {
    /// the path is empty
    NoPath,
    /// the file could not be read or written; carries the path and the reason of the operating system
    Io { path: String, reason: String },
    /// the file exists but is not UTF-8
    Encoding { path: String },
}

impl fmt::Display for InputFileError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            InputFileError::NoPath => write!(formatter, "no path is named"),
            InputFileError::Io { path, reason } => write!(formatter, "{path}: {reason}"),
            InputFileError::Encoding { path } => write!(formatter, "{path}: not UTF-8"),
        }
    }
}

impl std::error::Error for InputFileError {}

// realises FR-007, FR-013, FR-015, FR-016, FR-017
/// One input file as an *input group* edits it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputFile {
    /// which input file this is
    kind: InputKind,
    /// the path named in the file name field, empty where none is named
    path: String,
    /// the text the code editor holds
    text: String,
    /// the *file text*, `None` where it is not established that the file exists
    file_text: Option<String>,
    /// whether the text was edited since the file became unknown
    edited_since_unknown: bool,
}

impl InputFile {
    /// Creates the input file of `kind` with an empty path and an empty text.
    pub fn new(kind: InputKind) -> InputFile {
        InputFile {
            kind,
            path: String::new(),
            text: String::new(),
            file_text: None,
            edited_since_unknown: false,
        }
    }

    /// Yields which input file this is.
    pub fn kind(&self) -> InputKind {
        self.kind
    }

    /// Yields the named path, empty where none is named.
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Yields the text the code editor holds.
    pub fn text(&self) -> &str {
        &self.text
    }

    // realises FR-013, FR-015, IR-009
    /// Names `path` and loads its content into the text where the file exists.
    ///
    /// * Where the file exists and is read, the text and the *file text* become its content, and
    ///   `true` is yielded.
    /// * Where the file does not exist, the text is left unchanged, the *file text* becomes absent,
    ///   and `false` is yielded.
    ///
    /// # Errors
    /// Returns [`InputFileError::Io`] where the file exists but cannot be read, and
    /// [`InputFileError::Encoding`] where its content is not UTF-8; the text is left unchanged.
    pub fn load(&mut self, path: &str) -> Result<bool, InputFileError> {
        self.path = path.to_owned();
        self.file_text = None;
        self.edited_since_unknown = false;
        if path.is_empty() || !Path::new(path).is_file() {
            return Ok(false);
        }
        let bytes = fs::read(path).map_err(|error| InputFileError::Io {
            path: path.to_owned(),
            reason: error.to_string(),
        })?;
        let content = String::from_utf8(bytes).map_err(|_| InputFileError::Encoding {
            path: path.to_owned(),
        })?;
        self.text = content.clone();
        self.file_text = Some(content);
        Ok(true)
    }

    // realises FR-014
    /// Yields whether the text differs from the *file text*, or was edited while the file is unknown.
    pub fn has_unsaved_changes(&self) -> bool {
        match &self.file_text {
            Some(file_text) => self.text != *file_text,
            None => self.edited_since_unknown,
        }
    }

    // realises FR-019, FR-021
    /// Replaces the text, as the user or *product* edited it.
    pub fn set_text(&mut self, text: &str) {
        if self.text == text {
            return;
        }
        self.text = text.to_owned();
        if self.file_text.is_none() {
            self.edited_since_unknown = true;
        }
    }

    // realises FR-016, IR-010
    /// Writes the text to the named file; the *file text* becomes the text.
    ///
    /// # Errors
    /// Returns [`InputFileError::NoPath`] where no path is named, and [`InputFileError::Io`]
    /// where the file cannot be written.
    pub fn save(&mut self) -> Result<(), InputFileError> {
        if self.path.is_empty() {
            return Err(InputFileError::NoPath);
        }
        fs::write(&self.path, self.text.as_bytes()).map_err(|error| InputFileError::Io {
            path: self.path.clone(),
            reason: error.to_string(),
        })?;
        self.file_text = Some(self.text.clone());
        self.edited_since_unknown = false;
        Ok(())
    }

    // realises FR-017, FR-018, FR-019, FR-020, FR-021
    /// Yields the *processing state*, from the *file text* and the text.
    pub fn state(&self) -> ProcessingState {
        match (&self.file_text, self.has_unsaved_changes()) {
            (Some(_), false) => ProcessingState::ValidFileTextUntouched,
            (Some(_), true) => ProcessingState::ValidFileTextChanged,
            (None, false) => ProcessingState::UnknownFileTextUntouched,
            (None, true) => ProcessingState::UnknownFileTextChanged,
        }
    }
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : InputGroup::set_path, InputGroup::set_text, InputGroup::answer_save, InputGroup::idle_expired */
#[cfg(test)]
mod tests {
    use super::*;

    /// A directory of this test case alone, removed when the case ends.
    struct CaseDir(std::path::PathBuf);

    impl CaseDir {
        fn new(name: &str) -> CaseDir {
            let dir = std::env::temp_dir()
                .join(format!("genc3wb-input_file-{name}-{}", std::process::id()));
            let _ = fs::remove_dir_all(&dir);
            fs::create_dir_all(&dir).expect("the temporary directory of the test can be created");
            CaseDir(dir)
        }
        fn path(&self, file: &str) -> String {
            self.0.join(file).to_string_lossy().into_owned()
        }
    }

    impl Drop for CaseDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    #[test]
    fn new_file_is_unknown_and_untouched() {
        // FR-020
        let file = InputFile::new(InputKind::CompilerInput);
        assert_eq!(file.kind(), InputKind::CompilerInput);
        assert_eq!(file.path(), "");
        assert_eq!(file.text(), "");
        assert_eq!(file.state(), ProcessingState::UnknownFileTextUntouched);
        assert!(!file.has_unsaved_changes());
    }

    #[test]
    fn existing_file_is_loaded_and_valid() {
        // FR-013, FR-018, IR-009
        let dir = CaseDir::new("existing");
        let path = dir.path("input.txt");
        fs::write(&path, "content\n").expect("the file can be written");
        let mut file = InputFile::new(InputKind::CompilerCompilerInput);
        assert_eq!(file.load(&path), Ok(true));
        assert_eq!(file.path(), path);
        assert_eq!(file.text(), "content\n");
        assert_eq!(file.state(), ProcessingState::ValidFileTextUntouched);
    }

    #[test]
    fn missing_file_leaves_the_text_unchanged() {
        // FR-015, FR-020
        let dir = CaseDir::new("missing");
        let mut file = InputFile::new(InputKind::CompilerInput);
        file.set_text("typed");
        assert_eq!(file.load(&dir.path("absent.txt")), Ok(false));
        assert_eq!(file.text(), "typed");
        assert_eq!(file.state(), ProcessingState::UnknownFileTextUntouched);
    }

    #[test]
    fn empty_path_names_no_file() {
        // FR-015
        let mut file = InputFile::new(InputKind::CompilerInput);
        assert_eq!(file.load(""), Ok(false));
        assert_eq!(file.save(), Err(InputFileError::NoPath));
    }

    #[test]
    fn file_that_is_not_utf8_is_reported() {
        // IR-009
        let dir = CaseDir::new("encoding");
        let path = dir.path("binary.txt");
        fs::write(&path, [0xff, 0xfe, 0x00]).expect("the file can be written");
        let mut file = InputFile::new(InputKind::CompilerInput);
        file.set_text("kept");
        assert_eq!(
            file.load(&path),
            Err(InputFileError::Encoding { path: path.clone() })
        );
        assert_eq!(file.text(), "kept");
        assert_eq!(file.state(), ProcessingState::UnknownFileTextUntouched);
    }

    #[test]
    fn edited_text_changes_the_state() {
        // FR-019, FR-021
        let dir = CaseDir::new("edited");
        let path = dir.path("input.txt");
        fs::write(&path, "content").expect("the file can be written");
        let mut file = InputFile::new(InputKind::CompilerInput);
        file.load(&path).expect("the file can be loaded");
        file.set_text("content changed");
        assert_eq!(file.state(), ProcessingState::ValidFileTextChanged);
        assert!(file.has_unsaved_changes());
        file.set_text("content");
        assert_eq!(file.state(), ProcessingState::ValidFileTextUntouched);
        let mut unknown = InputFile::new(InputKind::CompilerInput);
        unknown.set_text("typed");
        assert_eq!(unknown.state(), ProcessingState::UnknownFileTextChanged);
    }

    #[test]
    fn saved_text_becomes_the_file_text() {
        // FR-016, IR-010
        let dir = CaseDir::new("saved");
        let path = dir.path("new.txt");
        let mut file = InputFile::new(InputKind::CompilerInput);
        file.load(&path).expect("an absent file is no error");
        file.set_text("written");
        assert_eq!(file.state(), ProcessingState::UnknownFileTextChanged);
        assert_eq!(file.save(), Ok(()));
        assert_eq!(
            fs::read_to_string(&path).expect("the file was written"),
            "written"
        );
        assert_eq!(file.state(), ProcessingState::ValidFileTextUntouched);
    }

    #[test]
    fn file_that_cannot_be_written_is_reported() {
        // IR-010
        let dir = CaseDir::new("unwritable");
        let mut file = InputFile::new(InputKind::CompilerInput);
        file.load(&dir.path("no-such-dir/x.txt"))
            .expect("an absent file is no error");
        file.set_text("x");
        assert!(matches!(file.save(), Err(InputFileError::Io { .. })));
        assert_eq!(file.state(), ProcessingState::UnknownFileTextChanged);
    }

    #[test]
    fn indices_map_both_ways() {
        for state in [
            ProcessingState::ValidFileTextUntouched,
            ProcessingState::ValidFileTextChanged,
            ProcessingState::UnknownFileTextUntouched,
            ProcessingState::UnknownFileTextChanged,
        ] {
            assert_eq!(ProcessingState::of_index(state.index()), Some(state));
        }
        assert_eq!(ProcessingState::of_index(4), None);
        for kind in [InputKind::CompilerCompilerInput, InputKind::CompilerInput] {
            assert_eq!(InputKind::of_index(kind.index()), Some(kind));
        }
        assert_eq!(InputKind::of_index(2), None);
    }
}
