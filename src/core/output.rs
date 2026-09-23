//! The *outputs* of the opened *node* and its *diagnostics*, arranged as pages with their navigation.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use std::fs;

// realises FR-030, FR-102, FR-110, FR-111
/// One *output* as its *output page* presents it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputFile {
    /// the path of the *output*, resolved against the directory of the *network file*
    pub path: String,
    /// the content of the file, empty where it could not be read
    pub content: String,
}

// realises FR-027, FR-032, FR-033, FR-034, FR-035, FR-036, FR-122, FR-125
/// The pages of the output group: the *output pages* of the opened *node*, then the
/// *diagnostics page*, and which of them is presented.
///
/// * The *output pages* are the *outputs* in the order of the *node description*; there is none
///   before a *node* is opened, and the *diagnostics page* is the one page then.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OutputPages {
    /// the *outputs*, in the order of the *node description*
    files: Vec<OutputFile>,
    /// the text of the *diagnostics page*: the *diagnostics* of every document, one per line
    diagnostics: String,
    /// the index of the presented page
    current: usize,
}

/// Yields the content of the file at `path`, empty where it cannot be read.
///
/// * The *node* may not have stored the file yet, so that an absent file is no error.
/// * A byte sequence that is not UTF-8 is replaced by the replacement character.
fn content_of(path: &str) -> String {
    fs::read(path)
        .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        .unwrap_or_default()
}

impl OutputPages {
    // realises FR-102, FR-110, FR-122, IR-009
    /// Creates the pages of the *outputs* at `paths`, in that order, each read; the first is
    /// presented.
    ///
    /// * The content of a file that cannot be read is empty.
    pub fn of_paths(paths: &[String]) -> OutputPages {
        OutputPages {
            files: paths
                .iter()
                .map(|path| OutputFile {
                    path: path.clone(),
                    content: content_of(path),
                })
                .collect(),
            diagnostics: String::new(),
            current: 0,
        }
    }

    // realises FR-107, FR-108, FR-125
    /// Replaces the text of the *diagnostics page*.
    pub fn set_diagnostics(&mut self, text: &str) {
        self.diagnostics = text.to_owned();
    }

    // realises FR-125
    /// Yields the text of the *diagnostics page*.
    pub fn diagnostics(&self) -> &str {
        &self.diagnostics
    }

    // realises FR-027, FR-036
    /// Yields whether the presented page is the *diagnostics page*.
    pub fn is_diagnostics_page(&self) -> bool {
        self.current >= self.files.len()
    }

    // realises FR-111, IR-009
    /// Reads the file of every *output* again.
    pub fn reload(&mut self) {
        for file in &mut self.files {
            file.content = content_of(&file.path);
        }
    }

    // realises FR-027, FR-036
    /// Yields the number of pages: the number of *outputs*, plus the *diagnostics page*.
    pub fn page_count(&self) -> usize {
        self.files.len() + 1
    }

    /// Yields the index of the presented page.
    pub fn current(&self) -> usize {
        self.current
    }

    /// Yields the *output* of the presented page, `None` on the *diagnostics page*.
    pub fn current_file(&self) -> Option<&OutputFile> {
        self.files.get(self.current)
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
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : OutputGroup::next, OutputGroup::previous, OutputGroup::set_paths, OutputGroup::reload,
 *                    OutputGroup::set_diagnostics */
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

    fn paths(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_owned()).collect()
    }

    #[test]
    fn pages_of_no_node_are_the_diagnostics_page_alone() {
        // FR-027, FR-036, FR-125
        let pages = OutputPages::default();
        assert_eq!(
            (
                pages.page_count(),
                pages.current(),
                pages.current_file(),
                pages.is_diagnostics_page(),
                pages.has_next(),
                pages.has_previous()
            ),
            (1, 0, None, true, false, false)
        );
    }

    #[test]
    fn pages_are_the_outputs_in_their_order_then_the_diagnostics_and_the_first_is_presented() {
        // FR-027, FR-102, FR-122, FR-125
        let pages = OutputPages::of_paths(&paths(&["b.txt", "a.txt"]));
        assert_eq!(
            (
                pages.page_count(),
                pages.current(),
                pages.current_file().map(|file| file.path.as_str()),
                pages.is_diagnostics_page()
            ),
            (3, 0, Some("b.txt"), false)
        );
    }

    #[test]
    fn navigation_stops_at_both_ends_and_ends_on_the_diagnostics_page() {
        // FR-032, FR-033, FR-034, FR-035, FR-125
        let mut pages = OutputPages::of_paths(&paths(&["a", "b"]));
        let at_start = (pages.has_previous(), pages.previous());
        let forward = (pages.next(), pages.next(), pages.current());
        let at_end = (
            pages.has_next(),
            pages.next(),
            pages.is_diagnostics_page(),
            pages.current_file().is_none(),
        );
        let back = (
            pages.previous(),
            pages.current(),
            pages.is_diagnostics_page(),
        );
        assert_eq!(
            (at_start, forward, at_end, back),
            (
                (false, false),
                (true, true, 2),
                (false, false, true, true),
                (true, 1, false)
            )
        );
    }

    #[test]
    fn the_diagnostics_page_holds_the_text_set_for_it() {
        // FR-107, FR-125
        let mut pages = OutputPages::default();
        let before = pages.diagnostics().to_owned();
        pages.set_diagnostics("a.gc3: error 1:1-1:1: fault");
        assert_eq!(
            (before, pages.diagnostics()),
            (String::new(), "a.gc3: error 1:1-1:1: fault")
        );
    }

    #[test]
    fn a_file_that_cannot_be_read_has_an_empty_content() {
        // FR-110, IR-009
        let dir = case_dir("absent");
        let path = dir.join("absent.txt").to_string_lossy().into_owned();
        let pages = OutputPages::of_paths(&[path]);
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(
            pages.current_file().map(|file| file.content.as_str()),
            Some("")
        );
    }

    #[test]
    fn reloading_reads_what_the_node_wrote() {
        // FR-111, IR-009
        let dir = case_dir("reload");
        let path = dir.join("out.txt");
        let path_text = path.to_string_lossy().into_owned();
        let mut pages = OutputPages::of_paths(&[path_text]);
        let before = pages.current_file().map(|file| file.content.clone());
        fs::write(&path, "generated").expect("the file can be written");
        pages.reload();
        let after = pages.current_file().map(|file| file.content.clone());
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(
            (before, after),
            (Some(String::new()), Some("generated".to_owned()))
        );
    }

    #[test]
    fn a_file_that_is_not_utf8_is_read_with_replacement_characters() {
        // IR-009
        let dir = case_dir("encoding");
        let path = dir.join("bin.txt");
        fs::write(&path, [b'a', 0xff, b'b']).expect("the file can be written");
        let pages = OutputPages::of_paths(&[path.to_string_lossy().into_owned()]);
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(
            pages.current_file().map(|file| file.content.as_str()),
            Some("a\u{fffd}b")
        );
    }
}
