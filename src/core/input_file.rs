//! One input file: its path, its octets, its text, its *file octets*, loading, saving and the
//! *processing state*.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::editing::text_area_form;
use crate::core::octet_edit::{Bits, History, applied, replacement};
use crate::core::octet_view::{EditorMode, is_utf8, text_of_octets};

use std::fmt;
use std::fs;
use std::path::Path;

// realises FR-001, FR-063
/// Which file an *input group* edits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputKind {
    /// the *meta compiler DSL* of the opened *node*, or a *source input* a *node* has as such in
    /// an *editor window*
    MetaDsl,
    /// one *input* of the opened *node*, or a *source input* in an *editor window*
    Input,
    /// a *sink output* in an *editor window*, shown read-only (FR-291)
    Output,
    /// the *network file* of the *compiler network editor*
    Network,
}

impl InputKind {
    // realises FR-011, FR-290, FR-291
    /// Yields the caption of the *input group* and of its file selector.
    pub fn caption(self) -> &'static str {
        match self {
            InputKind::MetaDsl => "Meta compiler DSL",
            InputKind::Input => "Input",
            InputKind::Output => "Output",
            InputKind::Network => "Compiler network file",
        }
    }

    // realises FR-011
    /// Yields the file filter of the file selector, as a file dialog names it.
    pub fn file_filter(self) -> &'static str {
        match self {
            InputKind::MetaDsl => "Meta compiler DSL files (*.gc3)",
            InputKind::Input | InputKind::Output => "All files (*)",
            InputKind::Network => "Compiler network files (*.gc3n)",
        }
    }

    // realises FR-007
    /// Yields whether the user names the file of this kind, by the file selector and the new-file
    /// button: for the *network file* alone; the files of a *node* are named from the
    /// *node description*.
    pub fn is_selectable(self) -> bool {
        match self {
            InputKind::MetaDsl | InputKind::Input | InputKind::Output => false,
            InputKind::Network => true,
        }
    }
}

// realises FR-483
/// The *network template*: the text a new *network file* is created with, a network of two
/// *nodes* with a comment line, the *nodes* indented by a tab and their items by two and three,
/// UTF-8 with line feeds.
pub const NETWORK_TEMPLATE: &str = concat!(
    "// a basic template with two nodes\n",
    "meta compiler-compiler network\n",
    "\n",
    "\tnode A\n",
    "\t\tinputs\n",
    "\t\t\t../src/inputOfA.txt\n",
    "\t\toutputs\n",
    "\t\t\t../src/connectionAToB.txt\n",
    "\n",
    "\tnode B\n",
    "\t\tinputs\n",
    "\t\t\t../src/connectionAToB.txt\n",
    "\t\toutputs\n",
    "\t\t\t../src/outputOfB.txt\n",
);

// realises FR-484, FR-485, IR-209
/// Writes the *network template* to the file at `path`, creating the file or replacing its
/// content.
///
/// # Errors
/// Returns [`InputFileError::Io`] with the path and the reason of the operating system where
/// the file cannot be written.
pub fn write_network_template(path: &str) -> Result<(), InputFileError> {
    fs::write(path, NETWORK_TEMPLATE).map_err(|error| InputFileError::Io {
        path: path.to_owned(),
        reason: error.to_string(),
    })
}

// realises FR-486
/// The *meta DSL template*: the text the file of a *meta compiler DSL* is created with when its
/// *node* is opened or activated while the file does not exist; a syntax taking every
/// UTF-8 character and a generator rendering the symbol tree, the rules indented by a tab, UTF-8
/// with line feeds.
pub const META_DSL_TEMPLATE: &str = concat!(
    "syntax // takes all UTF-8 characters\n",
    "\tAll = {.}\n",
    "generator // creates a symbol tree from the syntax above\n",
    "\tAll -> str(symbol)\n",
);

// realises FR-487
/// A file of a *node* that does not exist and is to be created: the file of its
/// *meta compiler DSL*, created from the *meta DSL template*, or the file of an *input* that no
/// *node* produces, created empty.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MissingFile {
    /// the path of the file, resolved against the directory of the *network file*
    pub path: String,
    /// whether the file is a *meta compiler DSL*, and an *input* otherwise
    pub meta_dsl: bool,
}

// realises FR-489, FR-491
/// Why a folder or a file could not be created: its path and the reason of the operating system.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreationFailure {
    /// the path of the folder or the file
    pub path: String,
    /// the reason of the operating system
    pub reason: String,
}

// realises FR-487
/// Yields, of the files `candidates`, each a path with whether it is a *meta compiler DSL*, those
/// that do not exist, in their order, a path named twice taken once.
pub fn missing_files(candidates: &[(String, bool)]) -> Vec<MissingFile> {
    let mut missing: Vec<MissingFile> = Vec::new();
    for (path, meta_dsl) in candidates {
        if Path::new(path).exists() || missing.iter().any(|file| &file.path == path) {
            continue;
        }
        missing.push(MissingFile {
            path: path.clone(),
            meta_dsl: *meta_dsl,
        });
    }
    missing
}

// realises FR-489, FR-490, FR-491, IR-210
/// Creates every file of `files`, its missing folders first: the *meta DSL template* is written
/// into a *meta compiler DSL*, nothing into an *input*; a file that exists meanwhile is left as
/// it is.
///
/// # Errors
/// Returns [`CreationFailure`] with the path of the first folder or file that could not be
/// created and the reason of the operating system; the files before it are created.
pub fn create_files(files: &[MissingFile]) -> Result<(), CreationFailure> {
    for file in files {
        let path = Path::new(&file.path);
        if path.exists() {
            continue;
        }
        if let Some(folder) = path
            .parent()
            .filter(|folder| !folder.as_os_str().is_empty() && !folder.exists())
        {
            fs::create_dir_all(folder).map_err(|error| CreationFailure {
                path: folder.to_string_lossy().into_owned(),
                reason: error.to_string(),
            })?;
        }
        let content = if file.meta_dsl { META_DSL_TEMPLATE } else { "" };
        fs::write(path, content).map_err(|error| CreationFailure {
            path: file.path.clone(),
            reason: error.to_string(),
        })?;
    }
    Ok(())
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
    /// the octets of the document, and how many of their bits are digits of it (FR-179)
    bits: Bits,
    /// the *edit history* of the edits made in hex and bin (FR-207)
    history: History,
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
            bits: Bits::default(),
            history: History::default(),
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
        self.bits.octets()
    }

    // realises FR-179
    /// Yields the number of bits of the octets that are digits of the document; the bits of the
    /// last octet behind them are placeholders.
    pub fn bit_count(&self) -> usize {
        self.bits.len()
    }

    // realises FR-156, FR-167
    /// Yields whether the octets of the document are valid UTF-8.
    pub fn is_text(&self) -> bool {
        is_utf8(self.bits.octets())
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
            InputKind::Input | InputKind::Output => true,
            InputKind::MetaDsl | InputKind::Network => false,
        };
        if !binary_allowed && !is_utf8(&octets) {
            return Err(InputFileError::Encoding {
                path: path.to_owned(),
            });
        }
        self.text = text_of_octets(&octets);
        self.file_octets = Some(octets.clone());
        self.bits = Bits::of_octets(octets);
        self.history.clear();
        Ok(true)
    }

    // realises FR-239, FR-240
    /// Names `path` and takes `octets` as the octets of the document, as the *node* holds them:
    /// the document of an *input* that has a *producer*, which is read from no file and written
    /// to none.
    ///
    /// * The *file octets* become `octets` as well, so that the document holds no unsaved
    ///   change and its *processing state* is *valid file, text untouched*.
    pub fn hold(&mut self, path: &str, octets: Vec<u8>) {
        self.path = path.to_owned();
        self.edited_since_unknown = false;
        self.text = text_of_octets(&octets);
        self.file_octets = Some(octets.clone());
        self.bits = Bits::of_octets(octets);
        self.history.clear();
    }

    // realises FR-014
    /// Yields whether the octets differ from the *file octets*, or were edited while the file is
    /// unknown.
    pub fn has_unsaved_changes(&self) -> bool {
        match &self.file_octets {
            Some(file_octets) => self.bits.octets() != file_octets.as_slice(),
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
        self.bits = Bits::of_octets(text.as_bytes().to_vec());
        self.history.clear();
        if self.file_octets.is_none() {
            self.edited_since_unknown = true;
        }
        true
    }

    // realises FR-177, FR-180 to FR-184, FR-187, FR-205, FR-207
    /// Replaces `removed` digits of `mode` from the digit `start` on by `digits`, as
    /// [`replacement`] does, and records the step in the *edit history*; yields the digit behind
    /// the digits inserted, `None` where nothing is replaced by nothing, and for txt.
    pub fn replace_digits(
        &mut self,
        mode: EditorMode,
        start: usize,
        removed: usize,
        digits: &[u8],
    ) -> Option<usize> {
        let unit = mode.digit_bits();
        if unit == 0 {
            return None;
        }
        let step = replacement(&self.bits, unit, start, removed, digits);
        let cursor = step
            .last()
            .map(|edit| (edit.start + edit.inserted.len()) / unit)?;
        self.take_bits(applied(&self.bits, &step));
        self.history.record(step);
        Some(cursor)
    }

    // realises FR-207
    /// Takes the last step of the *edit history* back; yields the bit behind what it restored,
    /// `None` where no step is made.
    pub fn undo(&mut self) -> Option<usize> {
        let (bits, cursor) = self.history.undo(&self.bits)?;
        self.take_bits(bits);
        Some(cursor)
    }

    // realises FR-208
    /// Makes the step taken back last again; yields the bit behind what it inserted, `None`
    /// where no step is taken back.
    pub fn redo(&mut self) -> Option<usize> {
        let (bits, cursor) = self.history.redo(&self.bits)?;
        self.take_bits(bits);
        Some(cursor)
    }

    // realises FR-185
    /// Takes every placeholder as a digit 0; yields whether there was one.
    ///
    /// * The octets stay as they are, the placeholders being 0 already.
    pub fn complete_placeholders(&mut self) -> bool {
        if self.bits.padding() == 0 {
            return false;
        }
        self.bits = self.bits.completed();
        true
    }

    // realises FR-202
    /// Yields the digits of `mode` from the digit `start` to the digit `end`, exclusive, as
    /// their values; none for txt.
    pub fn digits(&self, mode: EditorMode, start: usize, end: usize) -> Vec<u8> {
        match mode.digit_bits() {
            0 => Vec::new(),
            unit => self.bits.digits(unit, start, end),
        }
    }

    /// Takes `bits` as the document after an edit in hex or bin: the text becomes their reading.
    fn take_bits(&mut self, bits: Bits) {
        self.text = text_of_octets(bits.octets());
        self.bits = bits;
        if self.file_octets.is_none() {
            self.edited_since_unknown = true;
        }
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
        fs::write(&self.path, self.bits.octets()).map_err(|error| InputFileError::Io {
            path: self.path.clone(),
            reason: error.to_string(),
        })?;
        self.file_octets = Some(self.bits.octets().to_vec());
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
 * covers bridge    : InputGroup::set_path, InputGroup::name_document, InputGroup::set_content,
 *                    InputGroup::set_text, InputGroup::answer_save,
 *                    InputGroup::idle_expired, InputGroup::is_text, InputGroup::replace_digits,
 *                    InputGroup::undo, InputGroup::redo, InputGroup::digits_text,
 *                    InputGroup::long_idle_expired, InputGroup::create_file,
 *                    NetworkEditor::files_to_create, NetworkEditor::prepared */
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
    fn content_held_of_an_input_of_a_producer_is_read_from_no_file_and_is_no_unsaved_change() {
        // FR-239, FR-240: the path names no file that exists
        let mut file = InputFile::new(InputKind::Input);
        file.hold("/nonexistent/genc3wb/x.txt", vec![b'a', 0xff]);
        assert_eq!(
            (
                file.path(),
                file.octets(),
                file.text(),
                file.is_text(),
                file.has_unsaved_changes(),
                file.state()
            ),
            (
                "/nonexistent/genc3wb/x.txt",
                &[b'a', 0xff][..],
                "a\u{fffd}",
                false,
                false,
                ProcessingState::ValidFileTextUntouched
            )
        );
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
    fn a_digit_typed_in_hex_changes_the_octets_the_text_and_the_state() {
        // FR-177, FR-180, FR-182
        let dir = CaseDir::new("digit");
        let path = dir.path("a.bin");
        fs::write(&path, [0x41, 0x42]).expect("the file can be written");
        let mut file = InputFile::new(InputKind::Input);
        file.load(&path).expect("the file can be loaded");
        let cursor = file.replace_digits(EditorMode::Hex, 4, 0, &[4]);
        assert_eq!(
            (
                cursor,
                file.octets(),
                file.bit_count(),
                file.text(),
                file.state()
            ),
            (
                Some(5),
                &[0x41, 0x42, 0x40][..],
                20,
                "AB@",
                ProcessingState::ValidFileTextChanged
            )
        );
    }

    #[test]
    fn an_edit_in_hex_is_taken_back_and_made_again() {
        // FR-207, FR-208
        let mut file = InputFile::new(InputKind::Input);
        file.set_text("AB");
        let _ = file.replace_digits(EditorMode::Hex, 1, 1, &[2]);
        let edited = file.octets().to_vec();
        let undone = (file.undo(), file.octets().to_vec());
        let redone = (file.redo(), file.octets().to_vec(), file.redo());
        assert_eq!(
            (edited, undone, redone),
            (
                vec![0x42, 0x42],
                (Some(8), vec![0x41, 0x42]),
                (Some(8), vec![0x42, 0x42], None)
            )
        );
    }

    #[test]
    fn an_edit_in_txt_and_a_load_forget_the_edit_history() {
        // FR-209
        let dir = CaseDir::new("forgotten");
        let path = dir.path("a.bin");
        fs::write(&path, [0x41]).expect("the file can be written");
        let mut file = InputFile::new(InputKind::Input);
        file.set_text("AB");
        let _ = file.replace_digits(EditorMode::Hex, 0, 1, &[5]);
        file.set_text("typed");
        let after_text = file.undo();
        let _ = file.replace_digits(EditorMode::Hex, 0, 1, &[5]);
        file.load(&path).expect("the file can be loaded");
        assert_eq!((after_text, file.undo()), (None, None));
    }

    #[test]
    fn placeholders_are_completed_once_and_leave_the_octets() {
        // FR-185
        let mut file = InputFile::new(InputKind::Input);
        let _ = file.replace_digits(EditorMode::Hex, 0, 0, &[0xa]);
        let before = (file.bit_count(), file.octets().to_vec());
        let completed = (file.complete_placeholders(), file.complete_placeholders());
        assert_eq!(
            (before, completed, file.bit_count(), file.octets()),
            ((4, vec![0xa0]), (true, false), 8, &[0xa0][..])
        );
    }

    #[test]
    fn digits_of_a_span_are_read_in_the_mode_and_txt_has_none() {
        // FR-202
        let mut file = InputFile::new(InputKind::Input);
        file.set_text("A");
        assert_eq!(
            (
                file.digits(EditorMode::Hex, 0, 2),
                file.digits(EditorMode::Bin, 0, 3),
                file.digits(EditorMode::Txt, 0, 2),
                file.replace_digits(EditorMode::Txt, 0, 0, &[1])
            ),
            (vec![4, 1], vec![0, 1, 0], vec![], None)
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
                InputKind::Output.is_selectable(),
                InputKind::Network.is_selectable()
            ],
            [false, false, false, true]
        );
    }

    #[test]
    fn an_output_is_captioned_output_and_loaded_whatever_its_octets() {
        // FR-291: the file of a sink output in an editor window
        let dir = CaseDir::new("output");
        let path = dir.path("out.bin");
        fs::write(&path, [0xff, 0x00]).expect("the file can be written");
        let mut file = InputFile::new(InputKind::Output);
        let loaded = file.load(&path);
        assert_eq!(
            (
                InputKind::Output.caption(),
                InputKind::Output.file_filter(),
                loaded,
                file.octets()
            ),
            ("Output", "All files (*)", Ok(true), &[0xff, 0x00][..])
        );
    }

    #[test]
    fn the_network_template_is_a_network_of_two_nodes_indented_by_tabs() {
        // FR-483: the comment line, the network line, the nodes indented by one tab, their items
        // by two and three, every line ended by a line feed
        let lines: Vec<&str> = NETWORK_TEMPLATE.split_inclusive('\n').collect();
        assert_eq!(
            (
                lines.len(),
                lines[0],
                lines[1],
                lines[3],
                lines[5],
                lines[9],
                lines[13],
                NETWORK_TEMPLATE.ends_with('\n'),
                NETWORK_TEMPLATE.contains("\r")
            ),
            (
                14,
                "// a basic template with two nodes\n",
                "meta compiler-compiler network\n",
                "\tnode A\n",
                "\t\t\t../src/inputOfA.txt\n",
                "\tnode B\n",
                "\t\t\t../src/outputOfB.txt\n",
                true,
                false
            )
        );
    }

    #[test]
    fn a_network_file_is_created_with_the_template_and_replaced_by_it() {
        // FR-484, IR-209: the file is written with the template, and a file that exists is
        // replaced by it
        let dir = CaseDir::new("create");
        let path = dir.path("new.gc3n");
        let created = write_network_template(&path);
        let content_created = fs::read(&path).expect("the created file is read");
        fs::write(&path, "edited").expect("the file can be written");
        let replaced = write_network_template(&path);
        let content_replaced = fs::read(&path).expect("the replaced file is read");
        let mut file = InputFile::new(InputKind::Network);
        let loaded = file.load(&path);
        assert_eq!(
            (
                created,
                content_created.as_slice(),
                replaced,
                content_replaced.as_slice(),
                loaded,
                file.text()
            ),
            (
                Ok(()),
                NETWORK_TEMPLATE.as_bytes(),
                Ok(()),
                NETWORK_TEMPLATE.as_bytes(),
                Ok(true),
                NETWORK_TEMPLATE
            )
        );
    }

    #[test]
    fn the_meta_dsl_template_takes_every_character_and_renders_the_tree() {
        // FR-486: the syntax line with its comment, the rule indented by a tab, the generator
        // line with its comment, the rendering rule
        assert_eq!(
            META_DSL_TEMPLATE,
            "syntax // takes all UTF-8 characters\n\tAll = {.}\n\
             generator // creates a symbol tree from the syntax above\n\tAll -> str(symbol)\n"
        );
    }

    #[test]
    fn the_files_that_do_not_exist_are_yielded_once_each_in_their_order() {
        // FR-487: a file that exists is passed over, a path named twice is taken once
        let dir = CaseDir::new("missing");
        fs::write(dir.path("a.gc3"), "syntax\n").expect("the file can be written");
        let candidates = [
            (dir.path("a.gc3"), true),
            (dir.path("b.gc3"), true),
            (dir.path("in.txt"), false),
            (dir.path("in.txt"), false),
        ];
        let missing = missing_files(&candidates);
        assert_eq!(
            missing,
            vec![
                MissingFile {
                    path: dir.path("b.gc3"),
                    meta_dsl: true
                },
                MissingFile {
                    path: dir.path("in.txt"),
                    meta_dsl: false
                }
            ]
        );
    }

    #[test]
    fn missing_files_are_created_with_their_folders_the_template_and_empty() {
        // FR-489, IR-210: the meta compiler DSL in a folder that does not exist gets the
        // template, the input is empty, a file that exists is left as it is
        let dir = CaseDir::new("create");
        fs::write(dir.path("kept.txt"), "kept").expect("the file can be written");
        let files = [
            MissingFile {
                path: dir.path("sub/deeper/a.gc3"),
                meta_dsl: true,
            },
            MissingFile {
                path: dir.path("in.txt"),
                meta_dsl: false,
            },
            MissingFile {
                path: dir.path("kept.txt"),
                meta_dsl: false,
            },
        ];
        let created = create_files(&files);
        let template = fs::read_to_string(dir.path("sub/deeper/a.gc3")).expect("created");
        let input = fs::read(dir.path("in.txt")).expect("created");
        let kept = fs::read_to_string(dir.path("kept.txt")).expect("kept");
        assert_eq!(
            (created, template.as_str(), input.len(), kept.as_str()),
            (Ok(()), META_DSL_TEMPLATE, 0, "kept")
        );
    }

    #[test]
    fn a_folder_that_cannot_be_created_is_reported_with_its_path() {
        // FR-491: the folder lies under a file, so that it cannot be created; the file before it
        // is created all the same
        let dir = CaseDir::new("create-failure");
        fs::write(dir.path("blocking"), "a file, not a folder").expect("the file can be written");
        let files = [
            MissingFile {
                path: dir.path("first.txt"),
                meta_dsl: false,
            },
            MissingFile {
                path: dir.path("blocking/sub/a.gc3"),
                meta_dsl: true,
            },
        ];
        let result = create_files(&files);
        let first = Path::new(&dir.path("first.txt")).exists();
        assert!(
            matches!(&result, Err(CreationFailure { path, reason })
                if path == &dir.path("blocking/sub") && !reason.is_empty())
                && first,
            "{result:?}"
        );
    }

    #[test]
    fn a_network_file_that_cannot_be_written_is_reported_with_its_path() {
        // FR-485: the path and the reason of the operating system
        let dir = CaseDir::new("create-failure");
        let path = dir.path("missing/new.gc3n");
        let result = write_network_template(&path);
        assert!(
            matches!(&result, Err(InputFileError::Io { path: reported, reason }) if reported == &path && !reason.is_empty()),
            "{result:?}"
        );
    }
}
