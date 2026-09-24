//! The *outputs* of the opened *node* and its *diagnostics*, as pages with their navigation.
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

// realises FR-150, FR-151, FR-152
/// Which page of the output group is presented, and to which *output page* the output group
/// returns when the *diagnostics* are gone.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Presentation {
    /// the index of the presented page
    pub page: usize,
    /// the index of the *output page* presented before the *diagnostics page* was presented for
    /// *diagnostics* that appeared, `None` where the output group did not switch
    pub return_page: Option<usize>,
}

// realises FR-027, FR-032, FR-033, FR-034, FR-035, FR-036, FR-122, FR-125, FR-150, FR-151,
// FR-152
/// The pages of the output group: the *output pages* of the opened *node*, then the
/// *diagnostics page*, and which of them is presented.
///
/// * The *output pages* are the *outputs* in the order of the *node description*; there is none
///   before a *node* is opened, and the *diagnostics page* is the one page then.
/// * The presented page follows the *diagnostics* as [`presentation_after`] yields it, and is
///   changed by hand through [`OutputPages::next`] and [`OutputPages::previous`] at any time.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OutputPages {
    /// the *outputs*, in the order of the *node description*
    files: Vec<OutputFile>,
    /// the text of the *diagnostics page*: the *diagnostics* of every document, one per line
    diagnostics: String,
    /// the index of the presented page
    current: usize,
    /// the *output page* to return to when the *diagnostics* are gone (FR-151)
    return_page: Option<usize>,
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
            return_page: None,
        }
    }

    // realises FR-107, FR-108, FR-125, FR-150, FR-151, FR-152
    /// Replaces the text of the *diagnostics page*, and presents the page that
    /// [`presentation_after`] yields for the change; yields whether the presented page changed.
    ///
    /// * An empty text holds no *diagnostics*; any other text holds at least one.
    pub fn set_diagnostics(&mut self, text: &str) -> bool {
        let had_diagnostics = !self.diagnostics.is_empty();
        let has_diagnostics = !text.is_empty();
        self.diagnostics = text.to_owned();
        let presentation = presentation_after(
            Presentation {
                page: self.current,
                return_page: self.return_page,
            },
            self.files.len(),
            had_diagnostics,
            has_diagnostics,
        );
        let changed = presentation.page != self.current;
        self.current = presentation.page;
        self.return_page = presentation.return_page;
        changed
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

// realises FR-150, FR-151, FR-152
/// Yields the presentation of the output group after its *diagnostics* changed from
/// `had_diagnostics` to `has_diagnostics`.
///
/// * Where *diagnostics* appear, the *diagnostics page* is presented, and the *output page*
///   presented before is kept to return to (FR-150); where the *diagnostics page* was presented
///   already, there is none to return to.
/// * Where the *diagnostics* are gone, the *output page* kept is presented again (FR-151); where
///   none is kept, the presented page stays.
/// * Where the *diagnostics* neither appear nor are gone, the presentation stays as it is, so
///   that a page selected by hand stays presented (FR-152).
///
/// # Arguments
/// * `presentation` - the presentation before the change
/// * `diagnostics_page` - the index of the *diagnostics page*: the number of *output pages*
/// * `had_diagnostics` - whether the *diagnostics page* held *diagnostics* before the change
/// * `has_diagnostics` - whether it holds *diagnostics* after the change
pub fn presentation_after(
    presentation: Presentation,
    diagnostics_page: usize,
    had_diagnostics: bool,
    has_diagnostics: bool,
) -> Presentation {
    match (had_diagnostics, has_diagnostics) {
        (false, true) => Presentation {
            page: diagnostics_page,
            return_page: Some(presentation.page).filter(|page| *page < diagnostics_page),
        },
        (true, false) => Presentation {
            page: presentation
                .return_page
                .filter(|page| *page < diagnostics_page)
                .unwrap_or(presentation.page),
            return_page: None,
        },
        (false, false) | (true, true) => presentation,
    }
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : OutputGroup::next, OutputGroup::previous, OutputGroup::set_paths,
 *                    OutputGroup::reload, OutputGroup::set_diagnostics */
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

    fn presentation(page: usize, return_page: Option<usize>) -> Presentation {
        Presentation { page, return_page }
    }

    #[test]
    fn diagnostics_that_appear_present_the_diagnostics_page_and_keep_the_output_page() {
        // FR-150
        assert_eq!(
            presentation_after(presentation(1, None), 3, false, true),
            presentation(3, Some(1))
        );
    }

    #[test]
    fn diagnostics_that_appear_on_the_presented_diagnostics_page_keep_no_output_page() {
        // FR-150
        assert_eq!(
            presentation_after(presentation(2, None), 2, false, true),
            presentation(2, None)
        );
    }

    #[test]
    fn diagnostics_that_are_gone_present_the_output_page_kept() {
        // FR-151
        assert_eq!(
            presentation_after(presentation(3, Some(1)), 3, true, false),
            presentation(1, None)
        );
    }

    #[test]
    fn diagnostics_that_are_gone_without_a_page_kept_leave_the_page_presented() {
        // FR-151, FR-152
        assert_eq!(
            presentation_after(presentation(2, None), 3, true, false),
            presentation(2, None)
        );
    }

    #[test]
    fn a_kept_page_beyond_the_output_pages_is_not_returned_to() {
        // FR-151: the output pages may have been replaced meanwhile
        assert_eq!(
            presentation_after(presentation(1, Some(5)), 1, true, false),
            presentation(1, None)
        );
    }

    #[test]
    fn diagnostics_that_neither_appear_nor_go_leave_the_presentation() {
        // FR-152
        assert_eq!(
            [
                presentation_after(presentation(0, Some(1)), 3, true, true),
                presentation_after(presentation(2, None), 3, false, false)
            ],
            [presentation(0, Some(1)), presentation(2, None)]
        );
    }

    #[test]
    fn the_pages_switch_to_the_diagnostics_and_back_to_the_page_selected_before() {
        // FR-150, FR-151
        let mut pages = OutputPages::of_paths(&paths(&["a", "b"]));
        pages.next();
        let appeared = (pages.set_diagnostics("a.gc3: fault"), pages.current());
        let gone = (pages.set_diagnostics(""), pages.current());
        assert_eq!((appeared, gone), ((true, 2), (true, 1)));
    }

    #[test]
    fn a_page_selected_by_hand_stays_until_the_diagnostics_change() {
        // FR-152
        let mut pages = OutputPages::of_paths(&paths(&["a", "b"]));
        pages.set_diagnostics("a.gc3: fault");
        pages.previous();
        let changed_text = (pages.set_diagnostics("a.gc3: other fault"), pages.current());
        let gone = (pages.set_diagnostics(""), pages.current());
        assert_eq!((changed_text, gone), ((false, 1), (true, 0)));
    }
}
