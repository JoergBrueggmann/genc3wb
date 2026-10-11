//! The *outputs* of the opened *node* and its *diagnostics*, as pages with their navigation.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::octet_view::text_of_octets;

// realises FR-030, FR-102, FR-110, FR-111, FR-163
/// One *output* as its *output page* presents it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OutputFile {
    /// the path of the *output*, resolved against the directory of the *network file*
    pub path: String,
    /// the octets the *node* holds of the *output*, none before its content was read
    pub octets: Vec<u8>,
    /// the octets read as UTF-8 (FR-157), as txt presents them
    pub content: String,
}

impl OutputFile {
    // realises FR-102, FR-110
    /// Creates the *output* at `path`, whose content is not read yet: it has no octets.
    pub fn of_path(path: &str) -> OutputFile {
        OutputFile {
            path: path.to_owned(),
            octets: Vec::new(),
            content: String::new(),
        }
    }

    // realises FR-110, FR-111, FR-163
    /// Replaces the content by `octets`, as the *node* holds them.
    pub fn set_octets(&mut self, octets: Vec<u8>) {
        self.content = text_of_octets(&octets);
        self.octets = octets;
    }
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
// FR-152, FR-211, FR-213, FR-234, FR-237
/// The pages of the output group: the *output pages* of the opened *node*, then the
/// *diagnostics page*, and which of them is presented.
///
/// * The *output pages* are the *outputs* in the order of the *node description*; there is none
///   before a *node* is opened, and the *diagnostics page* is the one page then.
/// * The presented page follows the *diagnostics* as [`presentation_after`] yields it, and is
///   changed by hand through [`OutputPages::next`] and [`OutputPages::previous`] at any time.
/// * The *output pages* are *outdated* from a failed *store request* until one is
///   acknowledged; the *diagnostics page* then lists the failure before the *diagnostics*, and
///   the failure counts as a *diagnostic* for the presented page.
/// * The *diagnostics page* carries the *origins* of the opened *node*, each of which counts
///   as a *diagnostic* for the presented page as well.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct OutputPages {
    /// the *outputs*, in the order of the *node description*
    files: Vec<OutputFile>,
    /// the text of the *diagnostics page*: the *diagnostics* of every document, one per line
    diagnostics: String,
    /// the text of FR-112 of the failed *store request* while the *output pages* are
    /// *outdated* (FR-211), `None` otherwise
    store_failure: Option<String>,
    /// the *origins* of the opened *node*, in the order of its *node state* (FR-234)
    origins: Vec<String>,
    /// the index of the presented page
    current: usize,
    /// the *output page* to return to when the *diagnostics* are gone (FR-151)
    return_page: Option<usize>,
}

impl OutputPages {
    // realises FR-102, FR-110, FR-122
    /// Creates the pages of the *outputs* at `paths`, in that order; the first is presented.
    ///
    /// * The content of every *output* is empty until [`OutputPages::set_content`] brings what
    ///   the *node* holds of it.
    pub fn of_paths(paths: &[String]) -> OutputPages {
        OutputPages {
            files: paths.iter().map(|path| OutputFile::of_path(path)).collect(),
            diagnostics: String::new(),
            store_failure: None,
            origins: Vec::new(),
            current: 0,
            return_page: None,
        }
    }

    // realises FR-110, FR-111, FR-163
    /// Replaces the content of the *output page* of index `index` by `octets`, as the *node*
    /// holds them; yields whether there is such a page.
    pub fn set_content(&mut self, index: usize, octets: Vec<u8>) -> bool {
        match self.files.get_mut(index) {
            Some(file) => {
                file.set_octets(octets);
                true
            }
            None => false,
        }
    }

    // realises FR-150, FR-151, FR-234, FR-237
    /// Replaces the *origins* the *diagnostics page* carries, and presents the page that
    /// [`presentation_after`] yields for the change; yields whether the presented page changed.
    pub fn set_origins(&mut self, origins: &[String]) -> bool {
        let listed = self.lists();
        self.origins = origins.to_vec();
        self.present_after(listed)
    }

    // realises FR-234
    /// Yields the *origins* the *diagnostics page* carries, in the order of the *node state*.
    pub fn origins(&self) -> &[String] {
        &self.origins
    }

    // realises FR-107, FR-108, FR-125, FR-150, FR-151, FR-152
    /// Replaces the *diagnostics* the *diagnostics page* lists, and presents the page that
    /// [`presentation_after`] yields for the change; yields whether the presented page changed.
    ///
    /// * An empty text holds no *diagnostics*; any other text holds at least one.
    pub fn set_diagnostics(&mut self, text: &str) -> bool {
        let listed = self.lists();
        self.diagnostics = text.to_owned();
        self.present_after(listed)
    }

    // realises FR-150, FR-151, FR-211, FR-213
    /// Takes the *output pages* as *outdated* by a *store request* that failed with `message`,
    /// and presents the page that [`presentation_after`] yields for the line the
    /// *diagnostics page* lists for it; yields whether the presented page changed.
    pub fn set_store_failure(&mut self, message: &str) -> bool {
        let listed = self.lists();
        self.store_failure = Some(message.to_owned());
        self.present_after(listed)
    }

    // realises FR-211
    /// Yields whether the *output pages* are *outdated*.
    pub fn is_outdated(&self) -> bool {
        self.store_failure.is_some()
    }

    // realises FR-212
    /// Yields the text of FR-112 of the failed *store request*, empty where the *output pages*
    /// are not *outdated*.
    pub fn store_failure(&self) -> &str {
        self.store_failure.as_deref().unwrap_or("")
    }

    // realises FR-125, FR-213
    /// Yields the text of the *diagnostics page*: the line of the failed *store request* where
    /// the *output pages* are *outdated*, then the *diagnostics*.
    pub fn listing(&self) -> String {
        match &self.store_failure {
            Some(message) if self.diagnostics.is_empty() => format!("store: error: {message}"),
            Some(message) => format!("store: error: {message}\n{}", self.diagnostics),
            None => self.diagnostics.clone(),
        }
    }

    /// Yields whether the *diagnostics page* lists a line or carries an *origin*.
    fn lists(&self) -> bool {
        self.store_failure.is_some() || !self.diagnostics.is_empty() || !self.origins.is_empty()
    }

    /// Presents the page that [`presentation_after`] yields where the *diagnostics page* listed
    /// a line before as `listed` says; yields whether the presented page changed.
    fn present_after(&mut self, listed: bool) -> bool {
        let presentation = presentation_after(
            Presentation {
                page: self.current,
                return_page: self.return_page,
            },
            self.files.len(),
            listed,
            self.lists(),
        );
        let changed = presentation.page != self.current;
        self.current = presentation.page;
        self.return_page = presentation.return_page;
        changed
    }

    // realises FR-125
    /// Yields the *diagnostics* the *diagnostics page* lists.
    pub fn diagnostics(&self) -> &str {
        &self.diagnostics
    }

    // realises FR-027, FR-036
    /// Yields whether the presented page is the *diagnostics page*.
    pub fn is_diagnostics_page(&self) -> bool {
        self.current >= self.files.len()
    }

    // realises FR-151, FR-211
    /// Takes the *outputs* as stored, a *store request* being acknowledged, after which the
    /// *output pages* are not *outdated*; yields whether the presented page changed, the line
    /// of a failed *store request* being gone.
    pub fn stored(&mut self) -> bool {
        let listed = self.lists();
        self.store_failure = None;
        self.present_after(listed)
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
 *                    OutputGroup::set_content, OutputGroup::stored, OutputGroup::set_diagnostics,
 *                    OutputGroup::set_origins, OutputGroup::set_store_failure */
#[cfg(test)]
mod tests {
    use super::*;

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
    fn an_output_whose_content_is_not_read_yet_has_an_empty_content() {
        // FR-110
        let pages = OutputPages::of_paths(&paths(&["out.txt"]));
        assert_eq!(
            pages
                .current_file()
                .map(|file| (file.octets.as_slice(), file.content.as_str())),
            Some((&[][..], ""))
        );
    }

    #[test]
    fn the_content_of_an_output_page_is_what_the_node_holds() {
        // FR-110, FR-111: the second output is replaced, the first stays
        let mut pages = OutputPages::of_paths(&paths(&["a.txt", "b.txt"]));
        let first = pages.set_content(1, b"generated".to_vec());
        let second = pages.set_content(1, b"generated anew".to_vec());
        let contents: Vec<String> = [0, 1]
            .iter()
            .map(|index| pages.files[*index].content.clone())
            .collect();
        assert_eq!(
            (first, second, contents),
            (true, true, vec![String::new(), "generated anew".to_owned()])
        );
    }

    #[test]
    fn the_content_of_a_page_that_does_not_exist_is_not_taken() {
        // FR-111
        let mut pages = OutputPages::of_paths(&paths(&["a.txt"]));
        assert!(!pages.set_content(1, b"x".to_vec()));
    }

    #[test]
    fn a_content_that_is_not_utf8_is_held_as_its_octets_and_read_with_replacement_characters() {
        // FR-157, FR-163
        let mut pages = OutputPages::of_paths(&paths(&["bin.txt"]));
        pages.set_content(0, vec![b'a', 0xff, b'b']);
        assert_eq!(
            pages
                .current_file()
                .map(|file| (file.octets.as_slice(), file.content.as_str())),
            Some((&[b'a', 0xff, b'b'][..], "a\u{fffd}b"))
        );
    }

    #[test]
    fn origins_that_appear_present_the_diagnostics_page_and_origins_that_go_return() {
        // FR-234, FR-237
        let mut pages = OutputPages::of_paths(&paths(&["a", "b"]));
        pages.next();
        let appeared = (
            pages.set_origins(&paths(&["join", "a"])),
            pages.current(),
            pages.origins().to_vec(),
            pages.listing(),
        );
        let gone = (pages.set_origins(&[]), pages.current());
        assert_eq!(
            (appeared, gone),
            ((true, 2, paths(&["join", "a"]), String::new()), (true, 1))
        );
    }

    #[test]
    fn origins_that_go_keep_the_diagnostics_page_while_diagnostics_are_listed() {
        // FR-151, FR-237
        let mut pages = OutputPages::of_paths(&paths(&["a"]));
        pages.set_origins(&paths(&["join"]));
        pages.set_diagnostics("a.gc3: error 1:1-1:2: fault");
        assert_eq!((pages.set_origins(&[]), pages.current()), (false, 1));
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

    #[test]
    fn pages_are_not_outdated_before_a_store_fails() {
        // FR-211
        let pages = OutputPages::of_paths(&paths(&["a"]));
        assert_eq!(
            (pages.is_outdated(), pages.store_failure(), pages.listing()),
            (false, "", String::new())
        );
    }

    #[test]
    fn a_failed_store_leaves_the_content_and_takes_the_pages_as_outdated() {
        // FR-211, FR-212
        let mut pages = OutputPages::of_paths(&paths(&["out.txt"]));
        pages.set_content(0, b"held".to_vec());
        pages.set_store_failure("out.txt: does not exist");
        pages.previous();
        let content = pages.current_file().map(|file| file.content.clone());
        assert_eq!(
            (pages.is_outdated(), pages.store_failure(), content),
            (true, "out.txt: does not exist", Some("held".to_owned()))
        );
    }

    #[test]
    fn a_failed_store_with_an_empty_message_takes_the_pages_as_outdated() {
        // FR-211
        let mut pages = OutputPages::of_paths(&paths(&["a"]));
        pages.set_store_failure("");
        assert_eq!(
            (pages.is_outdated(), pages.listing()),
            (true, "store: error: ".to_owned())
        );
    }

    #[test]
    fn the_diagnostics_page_lists_the_failed_store_before_the_diagnostics() {
        // FR-213
        let mut pages = OutputPages::of_paths(&paths(&["a"]));
        pages.set_store_failure("fault of the store");
        let alone = pages.listing();
        pages.set_diagnostics("a.gc3: warning 1:1-1:2: fault");
        assert_eq!(
            (
                alone.as_str(),
                pages.listing().as_str(),
                pages.diagnostics()
            ),
            (
                "store: error: fault of the store",
                "store: error: fault of the store\na.gc3: warning 1:1-1:2: fault",
                "a.gc3: warning 1:1-1:2: fault"
            )
        );
    }

    #[test]
    fn a_failed_store_presents_the_diagnostics_page_and_an_acknowledged_one_returns() {
        // FR-150, FR-151, FR-211, FR-213
        let mut pages = OutputPages::of_paths(&paths(&["a", "b"]));
        pages.next();
        let failed = (pages.set_store_failure("fault"), pages.current());
        let failed_again = (pages.set_store_failure("other fault"), pages.current());
        let stored = (pages.stored(), pages.current(), pages.is_outdated());
        assert_eq!(
            (failed, failed_again, stored),
            ((true, 2), (false, 2), (true, 1, false))
        );
    }

    #[test]
    fn an_acknowledged_store_keeps_the_diagnostics_page_while_diagnostics_are_listed() {
        // FR-151, FR-213
        let mut pages = OutputPages::of_paths(&paths(&["a"]));
        pages.set_diagnostics("a.gc3: warning 1:1-1:2: fault");
        pages.set_store_failure("fault");
        let stored = (pages.stored(), pages.current(), pages.listing());
        assert_eq!(
            stored,
            (false, 1, "a.gc3: warning 1:1-1:2: fault".to_owned())
        );
    }

    #[test]
    fn diagnostics_that_go_keep_the_diagnostics_page_while_the_pages_are_outdated() {
        // FR-151, FR-213
        let mut pages = OutputPages::of_paths(&paths(&["a"]));
        pages.set_store_failure("fault");
        pages.set_diagnostics("a.gc3: error 1:1-1:2: fault");
        let gone = (pages.set_diagnostics(""), pages.current());
        assert_eq!(gone, (false, 1));
    }

    #[test]
    fn the_pages_of_a_node_opened_anew_are_not_outdated() {
        // FR-211
        let mut pages = OutputPages::of_paths(&paths(&["a"]));
        pages.set_store_failure("fault");
        pages = OutputPages::of_paths(&paths(&["a"]));
        assert!(!pages.is_outdated());
    }
}
