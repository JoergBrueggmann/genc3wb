//! One input file: its path, its octets, its text, its *file octets*, loading, saving and the
//! *processing state*.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::editing::text_area_form;
use crate::core::octet_view::{is_utf8, text_of_octets};

use std::fmt;
use std::fs;
use std::path::Path;

// realises FR-001, FR-063
/// Which file an *input group* edits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    /// the *meta compiler DSL* of the opened *node*
    MetaDsl,
    /// one *input* of the opened *node*
    Input,
    /// the *network file* of the *compiler network editor*
    Network,
}

impl InputKind {
    // realises FR-011
    /// Yields the caption of the *input group* and of its file selector.
    pub fn caption(self) -> &'static str {
        match self {
            InputKind::MetaDsl => "Meta compiler DSL",
            InputKind::Input => "Input",
            InputKind::Network => "Compiler network file",
        }
    }

    // realises FR-011
    /// Yields the file filter of the file selector, as a file dialog names it.
    pub fn file_filter(self) -> &'static str {
        match self {
            InputKind::MetaDsl => "Meta compiler DSL files (*.gc3)",
            InputKind::Input => "All files (*)",
            InputKind::Network => "Compiler network files (*.gc3n)",
        }
    }

    // realises FR-007, FR-116
    /// Yields whether the user names the file of this kind, by the file name field and the file
    /// selector: for the *network file* alone; the files of a *node* are named from the
    /// *node description*.
    pub fn is_selectable(self) -> bool {
        match self {
            InputKind::MetaDsl | InputKind::Input => false,
            InputKind::Network => true,
        }
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
    /// the file exists but is not UTF-8, and is the *meta compiler DSL* or the *network file*
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

// realises FR-007, FR-013, FR-015, FR-016, FR-017, FR-163
/// One input file as an *input group* edits it.
///
/// * The file is held as its octets; the text is the octets read as UTF-8, as txt presents them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InputFile {
    /// which input file this is
    kind: InputKind,
    /// the path named in the file name field, empty where none is named
    path: String,
    /// the octets of the document
    octets: Vec<u8>,
    /// the octets read as UTF-8 (FR-157)
    text: String,
    /// the *file octets*, `None` where it is not established that the file exists
    file_octets: Option<Vec<u8>>,
    /// whether the text was edited since the file became unknown
    edited_since_unknown: bool,
}

impl InputFile {
    /// Creates the input file of `kind` with an empty path and an empty text.
    pub fn new(kind: InputKind) -> InputFile {
        InputFile {
            kind,
            path: String::new(),
            octets: Vec::new(),
            text: String::new(),
            file_octets: None,
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

    // realises FR-157
    /// Yields the text the code editor holds in txt: the octets read as UTF-8.
    pub fn text(&self) -> &str {
        &self.text
    }

    // realises FR-163
    /// Yields the octets of the document.
    pub fn octets(&self) -> &[u8] {
        &self.octets
    }

    // realises FR-156, FR-167
    /// Yields whether the octets of the document are valid UTF-8.
    pub fn is_text(&self) -> bool {
        is_utf8(&self.octets)
    }

    // realises FR-013, FR-015, FR-163, FR-164, IR-009
    /// Names `path` and loads the octets of its file where the file exists.
    ///
    /// * Where the file exists and is read, the octets and the *file octets* become its content,
    ///   and `true` is yielded.
    /// * Where the file does not exist, the octets are left unchanged, the *file octets* become
    ///   absent, and `false` is yielded.
    /// * The file of an *input* is loaded whatever its octets are; the *meta compiler DSL* and
    ///   the *network file* are texts.
    ///
    /// # Errors
    /// Returns [`InputFileError::Io`] where the file exists but cannot be read, and
    /// [`InputFileError::Encoding`] where the file of the *meta compiler DSL* or of the
    /// *network file* is not UTF-8; the octets are left unchanged.
    pub fn load(&mut self, path: &str) -> Result<bool, InputFileError> {
        self.path = path.to_owned();
        self.file_octets = None;
        self.edited_since_unknown = false;
        if path.is_empty() || !Path::new(path).is_file() {
            return Ok(false);
        }
        let octets = fs::read(path).map_err(|error| InputFileError::Io {
            path: path.to_owned(),
            reason: error.to_string(),
        })?;
        let binary_allowed = match self.kind {
            InputKind::Input => true,
            InputKind::MetaDsl | InputKind::Network => false,
        };
        if !binary_allowed && !is_utf8(&octets) {
            return Err(InputFileError::Encoding {
                path: path.to_owned(),
            });
        }
        self.text = text_of_octets(&octets);
        self.file_octets = Some(octets.clone());
        self.octets = octets;
        Ok(true)
    }

    // realises FR-014
    /// Yields whether the octets differ from the *file octets*, or were edited while the file is
    /// unknown.
    pub fn has_unsaved_changes(&self) -> bool {
        match &self.file_octets {
            Some(file_octets) => self.octets != *file_octets,
            None => self.edited_since_unknown,
        }
    }

    // realises FR-019, FR-021, FR-165, FR-166
    /// Replaces the octets by the UTF-8 encoding of `text`, as the user edited it in txt; yields
    /// whether the octets were replaced.
    ///
    /// * A text that equals the text of the document, or the form in which the text area of a
    ///   code editor holds it ([`text_area_form`]), is no edit: the octets stay as they are.
    pub fn set_text(&mut self, text: &str) -> bool {
        if self.text == text || text_area_form(&self.text) == text {
            return false;
        }
        self.text = text.to_owned();
        self.octets = text.as_bytes().to_vec();
        if self.file_octets.is_none() {
            self.edited_since_unknown = true;
        }
        true
    }

    // realises FR-016, IR-010
    /// Writes the octets to the named file; the *file octets* become the octets.
    ///
    /// # Errors
    /// Returns [`InputFileError::NoPath`] where no path is named, and [`InputFileError::Io`]
    /// where the file cannot be written.
    pub fn save(&mut self) -> Result<(), InputFileError> {
        if self.path.is_empty() {
            return Err(InputFileError::NoPath);
        }
        fs::write(&self.path, &self.octets).map_err(|error| InputFileError::Io {
            path: self.path.clone(),
            reason: error.to_string(),
        })?;
        self.file_octets = Some(self.octets.clone());
        self.edited_since_unknown = false;
        Ok(())
    }

    // realises FR-017, FR-018, FR-019, FR-020, FR-021
    /// Yields the *processing state*, from the *file octets* and the octets.
    pub fn state(&self) -> ProcessingState {
        match (&self.file_octets, self.has_unsaved_changes()) {
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
 * covers bridge    : InputGroup::set_path, InputGroup::name_document, InputGroup::set_text, InputGroup::answer_save,
 *                    InputGroup::idle_expired, InputGroup::is_text */
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
        let file = InputFile::new(InputKind::Input);
        assert_eq!(file.kind(), InputKind::Input);
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
        let mut file = InputFile::new(InputKind::MetaDsl);
        assert_eq!(file.load(&path), Ok(true));
        assert_eq!(file.path(), path);
        assert_eq!(file.text(), "content\n");
        assert_eq!(file.state(), ProcessingState::ValidFileTextUntouched);
    }

    #[test]
    fn missing_file_leaves_the_text_unchanged() {
        // FR-015, FR-020
        let dir = CaseDir::new("missing");
        let mut file = InputFile::new(InputKind::Input);
        file.set_text("typed");
        assert_eq!(file.load(&dir.path("absent.txt")), Ok(false));
        assert_eq!(file.text(), "typed");
        assert_eq!(file.state(), ProcessingState::UnknownFileTextUntouched);
    }

    #[test]
    fn empty_path_names_no_file() {
        // FR-015
        let mut file = InputFile::new(InputKind::Input);
        assert_eq!(file.load(""), Ok(false));
        assert_eq!(file.save(), Err(InputFileError::NoPath));
    }

    #[test]
    fn meta_dsl_and_network_file_that_are_not_utf8_are_reported() {
        // FR-164, IR-009
        let dir = CaseDir::new("encoding");
        let path = dir.path("binary.txt");
        fs::write(&path, [0xff, 0xfe, 0x00]).expect("the file can be written");
        let outcomes: Vec<_> = [InputKind::MetaDsl, InputKind::Network]
            .into_iter()
            .map(|kind| {
                let mut file = InputFile::new(kind);
                file.set_text("kept");
                let loaded = file.load(&path);
                (loaded, file.text().to_owned(), file.state())
            })
            .collect();
        let expected = (
            Err(InputFileError::Encoding { path: path.clone() }),
            "kept".to_owned(),
            ProcessingState::UnknownFileTextUntouched,
        );
        assert_eq!(outcomes, vec![expected.clone(), expected]);
    }

    #[test]
    fn input_that_is_not_utf8_is_loaded_as_its_octets() {
        // FR-013, FR-157, FR-163
        let dir = CaseDir::new("binary");
        let path = dir.path("binary.bin");
        fs::write(&path, [b'a', 0xff, 0xfe, 0x00]).expect("the file can be written");
        let mut file = InputFile::new(InputKind::Input);
        let loaded = file.load(&path);
        assert_eq!(
            (
                loaded,
                file.octets(),
                file.text(),
                file.is_text(),
                file.state()
            ),
            (
                Ok(true),
                &[b'a', 0xff, 0xfe, 0x00][..],
                "a\u{FFFD}\u{FFFD}\u{0}",
                false,
                ProcessingState::ValidFileTextUntouched
            )
        );
    }

    #[test]
    fn octets_of_a_text_are_valid_utf8() {
        // FR-156
        let mut file = InputFile::new(InputKind::Input);
        file.set_text("ä");
        assert_eq!((file.is_text(), file.octets()), (true, "ä".as_bytes()));
    }

    #[test]
    fn edit_in_txt_replaces_the_octets_by_the_encoding_of_the_text() {
        // FR-165
        let dir = CaseDir::new("replaced");
        let path = dir.path("binary.bin");
        fs::write(&path, [b'a', 0xff]).expect("the file can be written");
        let mut file = InputFile::new(InputKind::Input);
        file.load(&path).expect("the file can be loaded");
        let replaced = file.set_text("a\u{FFFD}b");
        assert_eq!(
            (replaced, file.octets(), file.is_text(), file.state()),
            (
                true,
                "a\u{FFFD}b".as_bytes(),
                true,
                ProcessingState::ValidFileTextChanged
            )
        );
    }

    #[test]
    fn text_as_the_text_area_holds_it_is_no_edit() {
        // FR-166
        let dir = CaseDir::new("unmodified");
        let path = dir.path("crlf.txt");
        fs::write(&path, "a\r\nb\u{00A0}c").expect("the file can be written");
        let mut file = InputFile::new(InputKind::Input);
        file.load(&path).expect("the file can be loaded");
        let replaced = (file.set_text("a\nb c"), file.set_text("a\r\nb\u{00A0}c"));
        assert_eq!(
            (replaced, file.octets(), file.state()),
            (
                (false, false),
                "a\r\nb\u{00A0}c".as_bytes(),
                ProcessingState::ValidFileTextUntouched
            )
        );
    }

    #[test]
    fn saving_writes_the_octets_as_they_are() {
        // FR-016, FR-166, IR-010
        let dir = CaseDir::new("octets-saved");
        let source = dir.path("source.bin");
        let target = dir.path("target.bin");
        fs::write(&source, [0x00, 0xff, 0x0d, 0x0a]).expect("the file can be written");
        let mut file = InputFile::new(InputKind::Input);
        file.load(&source).expect("the file can be loaded");
        let held = text_area_form(file.text());
        file.set_text(&held);
        file.load(&target).expect("an absent file is no error");
        let saved = file.save();
        assert_eq!(
            (saved, fs::read(&target).ok()),
            (Ok(()), Some(vec![0x00, 0xff, 0x0d, 0x0a]))
        );
    }

    #[test]
    fn edited_text_changes_the_state() {
        // FR-019, FR-021
        let dir = CaseDir::new("edited");
        let path = dir.path("input.txt");
        fs::write(&path, "content").expect("the file can be written");
        let mut file = InputFile::new(InputKind::Input);
        file.load(&path).expect("the file can be loaded");
        file.set_text("content changed");
        assert_eq!(file.state(), ProcessingState::ValidFileTextChanged);
        assert!(file.has_unsaved_changes());
        file.set_text("content");
        assert_eq!(file.state(), ProcessingState::ValidFileTextUntouched);
        let mut unknown = InputFile::new(InputKind::Input);
        unknown.set_text("typed");
        assert_eq!(unknown.state(), ProcessingState::UnknownFileTextChanged);
    }

    #[test]
    fn saved_text_becomes_the_file_octets() {
        // FR-016, IR-010
        let dir = CaseDir::new("saved");
        let path = dir.path("new.txt");
        let mut file = InputFile::new(InputKind::Input);
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
        let mut file = InputFile::new(InputKind::Input);
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
    }

    #[test]
    fn the_network_file_alone_is_selectable() {
        // FR-007, FR-116
        assert_eq!(
            [
                InputKind::MetaDsl.is_selectable(),
                InputKind::Input.is_selectable(),
                InputKind::Network.is_selectable()
            ],
            [false, false, true]
        );
    }
}
