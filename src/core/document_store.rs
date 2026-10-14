//! The store of the *shared documents*: one input file per path, shared by the *presenters* that
//! acquired it.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::input_file::{InputFile, InputKind};

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

// realises FR-277, FR-280
/// One *shared document* as the store holds it: the input file the *presenters* share, and the
/// *presenters*, in the order in which they acquired it.
#[derive(Debug)]
struct SharedDocument {
    /// the input file, shared by every *presenter*
    file: Rc<RefCell<InputFile>>,
    /// the numbers of the *presenters* that hold the document
    presenters: Vec<u32>,
}

// realises FR-277, FR-280
/// What acquiring a *shared document* yields: the input file, and whether it was created by this
/// acquisition, on which the *presenter* loads the file.
#[derive(Debug)]
pub struct Acquired {
    /// the input file, shared with the other *presenters* of the path
    pub file: Rc<RefCell<InputFile>>,
    /// whether the input file was created now, no *presenter* having held the path before
    pub created: bool,
}

// realises FR-277, FR-278, FR-279, FR-280
/// The store of the *shared documents*, by their paths.
///
/// * The store creates an input file and hands it out, and loads and saves nothing: the
///   *presenter* that created it loads the file.
/// * A *shared document* is dropped once no *presenter* holds it, so that the next acquisition
///   creates it anew.
#[derive(Debug, Default)]
pub struct DocumentStore {
    /// the *shared documents*, by the path the *presenters* name
    documents: BTreeMap<String, SharedDocument>,
}

impl DocumentStore {
    // realises FR-277, FR-280
    /// Acquires the *shared document* of `path` for `presenter`: yields the input file held of
    /// it, created of `kind` where none is held, which `created` says, and records the
    /// *presenter*; a *presenter* that holds it already is recorded once.
    pub fn acquire(&mut self, path: &str, kind: InputKind, presenter: u32) -> Acquired {
        let created = !self.documents.contains_key(path);
        let document = self
            .documents
            .entry(path.to_owned())
            .or_insert_with(|| SharedDocument {
                file: Rc::new(RefCell::new(InputFile::new(kind))),
                presenters: Vec::new(),
            });
        if !document.presenters.contains(&presenter) {
            document.presenters.push(presenter);
        }
        Acquired {
            file: Rc::clone(&document.file),
            created,
        }
    }

    // realises FR-277, FR-280
    /// Releases the *shared document* of `path` for `presenter`: yields whether it was dropped,
    /// which it is once no *presenter* holds it; does nothing for a path not held or a
    /// *presenter* that does not hold it.
    pub fn release(&mut self, path: &str, presenter: u32) -> bool {
        let Some(document) = self.documents.get_mut(path) else {
            return false;
        };
        document.presenters.retain(|held| *held != presenter);
        if document.presenters.is_empty() {
            self.documents.remove(path);
            return true;
        }
        false
    }

    // realises FR-278, FR-279
    /// Yields the other *presenters* of the *shared document* of `path`, in the order in which
    /// they acquired it; none where it is not held.
    pub fn peers(&self, path: &str, presenter: u32) -> Vec<u32> {
        self.documents.get(path).map_or_else(Vec::new, |document| {
            document
                .presenters
                .iter()
                .copied()
                .filter(|held| *held != presenter)
                .collect()
        })
    }

    // realises FR-280
    /// Yields the number of *presenters* of the *shared document* of `path`, 0 where it is not
    /// held.
    pub fn presenters(&self, path: &str) -> usize {
        self.documents
            .get(path)
            .map_or(0, |document| document.presenters.len())
    }
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : DocumentRegistry::acquire, DocumentRegistry::release, DocumentRegistry::is_sole,
 *                    DocumentRegistry::follow_peers, DocumentRegistry::provide_peers,
 *                    InputGroup::name_document, InputGroup::release_document, NodeEditor::close */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn first_acquisition_creates_the_document_and_a_second_shares_it() {
        // FR-277, FR-280: the second presenter receives the same input file, not created anew
        let mut store = DocumentStore::default();
        let first = store.acquire("/n/a.gc3", InputKind::MetaDsl, 1);
        first.file.borrow_mut().set_text("edited");
        let second = store.acquire("/n/a.gc3", InputKind::MetaDsl, 2);
        assert_eq!(
            (
                first.created,
                second.created,
                Rc::ptr_eq(&first.file, &second.file),
                second.file.borrow().text().to_owned(),
                store.presenters("/n/a.gc3")
            ),
            (true, false, true, "edited".to_owned(), 2)
        );
    }

    #[test]
    fn documents_of_different_paths_are_different() {
        // FR-277: by the same path alone
        let mut store = DocumentStore::default();
        let a = store.acquire("/n/a.in", InputKind::Input, 1);
        let b = store.acquire("/n/b.in", InputKind::Input, 1);
        assert_eq!(
            (a.created, b.created, Rc::ptr_eq(&a.file, &b.file)),
            (true, true, false)
        );
    }

    #[test]
    fn presenter_acquiring_twice_is_recorded_once() {
        // FR-277
        let mut store = DocumentStore::default();
        let _ = store.acquire("/n/a.in", InputKind::Input, 1);
        let _ = store.acquire("/n/a.in", InputKind::Input, 1);
        assert_eq!(
            (store.presenters("/n/a.in"), store.peers("/n/a.in", 1)),
            (1, vec![])
        );
    }

    #[test]
    fn peers_are_the_other_presenters_in_the_order_of_acquisition() {
        // FR-278, FR-279
        let mut store = DocumentStore::default();
        for presenter in [3, 1, 2] {
            let _ = store.acquire("/n/a.in", InputKind::Input, presenter);
        }
        assert_eq!(
            (
                store.peers("/n/a.in", 1),
                store.peers("/n/a.in", 4),
                store.peers("/n/x.in", 1)
            ),
            (vec![3, 2], vec![3, 1, 2], vec![])
        );
    }

    #[test]
    fn document_is_dropped_by_the_last_release_and_created_anew_after() {
        // FR-280
        let mut store = DocumentStore::default();
        let first = store.acquire("/n/a.in", InputKind::Input, 1);
        let _ = store.acquire("/n/a.in", InputKind::Input, 2);
        let released_first = store.release("/n/a.in", 1);
        let still_held = store.presenters("/n/a.in");
        let released_last = store.release("/n/a.in", 2);
        let again = store.acquire("/n/a.in", InputKind::Input, 1);
        assert_eq!(
            (
                released_first,
                still_held,
                released_last,
                store.presenters("/n/a.in"),
                again.created,
                Rc::ptr_eq(&first.file, &again.file)
            ),
            (false, 1, true, 1, true, false)
        );
    }

    #[test]
    fn release_of_a_path_not_held_or_of_a_presenter_not_holding_does_nothing() {
        // FR-280
        let mut store = DocumentStore::default();
        let _ = store.acquire("/n/a.in", InputKind::Input, 1);
        assert_eq!(
            (
                store.release("/n/x.in", 1),
                store.release("/n/a.in", 2),
                store.presenters("/n/a.in")
            ),
            (false, false, 1)
        );
    }
}
