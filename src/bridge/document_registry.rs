//! The *document registry*: the store of the *shared documents* with the invoker of every *presenter*.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::document_store::{Acquired, DocumentStore};
use crate::core::input_file::InputKind;

use qtbridge::{QmlMethodInvoker, invoke_method};

use std::collections::BTreeMap;

// realises FR-277, FR-278, FR-279, FR-280
/// The *document registry*: the one place where the *shared documents* are held, by their paths,
/// with the *input groups* that present each.
///
/// * It is no *bridged type*: the *workbench object*, the network editor, every *node editor*
///   and every *input group* of a *node window* hold it in `Rc<RefCell<_>>`, borrow it for a
///   call and release it before anything is emitted.
/// * It reaches a *presenter* by its invoker alone, by a queued invocation of `followDocument`
///   or `provideDocument`.
#[derive(Default)]
pub struct DocumentRegistry {
    /// the *shared documents* and their *presenters*
    store: DocumentStore,
    /// the invoker of every *presenter*, by its number
    presenters: BTreeMap<u32, QmlMethodInvoker>,
    /// the number the next *presenter* registered receives
    next_presenter: u32,
}

impl DocumentRegistry {
    /// Creates the registry of no *shared document* and no *presenter*.
    pub fn new() -> DocumentRegistry {
        DocumentRegistry::default()
    }

    // realises FR-277
    /// Registers an *input group* as a *presenter* by its invoker and yields its number.
    pub fn register(&mut self, invoker: QmlMethodInvoker) -> u32 {
        let presenter = self.next_presenter;
        self.next_presenter = self.next_presenter.wrapping_add(1);
        self.presenters.insert(presenter, invoker);
        presenter
    }

    // realises FR-277, FR-280
    /// Acquires the *shared document* of `path` for `presenter`, as [`DocumentStore::acquire`].
    pub fn acquire(&mut self, path: &str, kind: InputKind, presenter: u32) -> Acquired {
        self.store.acquire(path, kind, presenter)
    }

    // realises FR-277, FR-280
    /// Releases the *shared document* of `path` for `presenter`, as [`DocumentStore::release`].
    pub fn release(&mut self, path: &str, presenter: u32) -> bool {
        self.store.release(path, presenter)
    }

    // realises FR-280
    /// Yields whether `presenter` is the one *presenter* of the *shared document* of `path`.
    pub fn is_sole(&self, path: &str, presenter: u32) -> bool {
        self.store.presenters(path) == 1 && self.store.peers(path, presenter).is_empty()
    }

    // realises FR-278
    /// Schedules `followDocument` of every other *presenter* of the *shared document* of `path`,
    /// which `presenter` edited or saved.
    pub fn follow_peers(&self, path: &str, presenter: u32) {
        for peer in self.store.peers(path, presenter) {
            if let Some(invoker) = self.presenters.get(&peer) {
                invoke_method!(invoker, "followDocument");
            }
        }
    }

    // realises FR-279
    /// Schedules `provideDocument` of every other *presenter* of the *shared document* of
    /// `path`, which `presenter` provided to its *node*.
    pub fn provide_peers(&self, path: &str, presenter: u32) {
        for peer in self.store.peers(path, presenter) {
            if let Some(invoker) = self.presenters.get(&peer) {
                invoke_method!(invoker, "provideDocument");
            }
        }
    }
}
