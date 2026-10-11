//! The node runner: the *node* it is attached to, its changes, its contents and its states.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::api_message::DiagnosticGroup;
use crate::core::build_system::{
    BuildError, DiagnosticAnswer, DocumentContent, NodeSession, StateSink,
};
use crate::core::text_increment::TextIncrement;

use std::collections::BTreeMap;
use std::path::PathBuf;

/// The text of FR-112 where a change or a content query is handed over while no *node* is served.
const NOT_SERVED: &str = "No node is served.";

// realises FR-104, IR-026, IR-038
/// What a *node* is attached to by: the path of its *service socket*, and its *inputs*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeAttach {
    /// the path of the *service socket* of the *node*, as the *node description* carries it,
    /// resolved against the directory of the *network file*
    pub socket: PathBuf,
    /// the *document identifiers* of the *inputs*, in the order of the *node description*
    pub inputs: Vec<String>,
}

// realises FR-226
/// Yields `answer` with those of its *diagnostic groups* alone that name one of `inputs`, the
/// *inputs* of the served *node*; a group naming another document is left out.
pub fn answer_of_inputs(mut answer: DiagnosticAnswer, inputs: &[String]) -> DiagnosticAnswer {
    answer
        .groups
        .retain(|group| inputs.contains(&group.document));
    answer
}

// realises FR-231, FR-232, FR-233
/// Yields those of `groups`, the *diagnostic groups* of a *state response* of the *node*, that
/// are taken, in their order.
///
/// * A group naming one of `produced`, the *inputs* that have a *producer*, is taken at
///   whatever *document version* it states: *product* does not edit that document, and marks
///   the *diagnostics* in the content it reads at that *document version* (FR-233).
/// * A group naming a document of `held`, the documents *product* edits with the
///   *document version* the *node* answered last for each, is taken where it states that
///   *document version* (FR-231); one that states another was computed from a state *product*
///   has changed since, and the next one tells its *diagnostics* (FR-232).
/// * A group naming any other document is left out: a document *product* has transmitted no
///   open request for, and a document that is none of the *node window* (FR-232).
pub fn groups_of_state(
    groups: Vec<DiagnosticGroup>,
    held: &BTreeMap<String, u64>,
    produced: &[String],
) -> Vec<DiagnosticGroup> {
    groups
        .into_iter()
        .filter(|group| {
            produced.contains(&group.document) || held.get(&group.document) == Some(&group.version)
        })
        .collect()
}

// realises FR-110, FR-111, FR-239
/// The documents of the *node* *product* shows and does not edit, the *inputs* that have a
/// *producer* and the *outputs*: the *document version* of the content held of each, and the
/// one a content query request is under way for.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct WatchedDocuments {
    /// the *document identifiers* of the documents
    documents: Vec<String>,
    /// the *document version* of the content held of a document; none before the first content
    held: BTreeMap<String, u64>,
    /// the *document version* a content query request not yet answered was transmitted for
    asked: BTreeMap<String, u64>,
}

impl WatchedDocuments {
    // realises FR-110, FR-239
    /// Creates the documents `documents`, of which no content is held.
    pub fn of(documents: Vec<String>) -> WatchedDocuments {
        WatchedDocuments {
            documents,
            held: BTreeMap::new(),
            asked: BTreeMap::new(),
        }
    }

    // realises FR-110, FR-111, FR-239
    /// Yields the documents whose content is to be read, in the order of `versions`, the
    /// *document versions* a *state response* states: each of the documents that is stated at
    /// another *document version* than the one of the content held, or of which no content is
    /// held; and records that a content query request is under way for it.
    ///
    /// * A document a content query request is under way for at the stated *document version*
    ///   is not yielded again, and neither is a document that is none of these.
    pub fn outdated(&mut self, versions: &[(String, u64)]) -> Vec<String> {
        let mut outdated = Vec::new();
        for (document, version) in versions {
            if !self.documents.contains(document)
                || self.held.get(document) == Some(version)
                || self.asked.get(document) == Some(version)
            {
                continue;
            }
            self.asked.insert(document.clone(), *version);
            outdated.push(document.clone());
        }
        outdated
    }

    // realises FR-111, FR-239
    /// Takes the content of `document` at `version` as held, the content query request being
    /// answered.
    pub fn take(&mut self, document: &str, version: u64) {
        self.asked.remove(document);
        if self.documents.iter().any(|known| known == document) {
            self.held.insert(document.to_owned(), version);
        }
    }

    // realises FR-112, FR-264
    /// Takes the content query request of `document` as failed or cancelled, so that the next
    /// *state response* has the content read again.
    pub fn fail(&mut self, document: &str) {
        self.asked.remove(document);
    }

    // realises FR-233
    /// Yields the *document version* of the content held of `document`, `None` where none is.
    pub fn version_of(&self, document: &str) -> Option<u64> {
        self.held.get(document).copied()
    }
}

// realises FR-104, FR-105, FR-106, FR-112, FR-226, FR-230, FR-239
/// The node runner: the *node* it is attached to.
///
/// * The runner is owned by one thread; every function of it blocks until the *node* answered.
/// * The *state responses* of the *node* do not pass through the runner: the sink of
///   [`NodeRunner::attach`] receives them on the reader thread of the connection.
#[derive(Debug, Default)]
pub struct NodeRunner {
    /// the *node* attached to, `None` where none is served
    session: Option<NodeSession>,
    /// the *inputs* of the *node* attached to last, by which a *diagnostic group* is taken
    /// (FR-226)
    inputs: Vec<String>,
}

impl NodeRunner {
    /// Creates the runner, which is attached to no *node*.
    pub fn new() -> NodeRunner {
        NodeRunner::default()
    }

    // realises FR-104, FR-105, FR-230
    /// Closes the connection to the *node* attached to and attaches to the one of `attach`,
    /// whose *state responses* `sink` receives.
    ///
    /// # Errors
    /// Returns what [`NodeSession::attach`] returns; no *node* is served then.
    pub fn attach(&mut self, attach: &NodeAttach, sink: StateSink) -> Result<(), BuildError> {
        self.detach();
        self.inputs.clone_from(&attach.inputs);
        self.session = Some(NodeSession::attach(&attach.socket, sink)?);
        Ok(())
    }

    // realises FR-106, FR-107, FR-112, FR-225, FR-226
    /// Transmits the change of a document to the served *node*, as [`NodeSession::transmit`].
    ///
    /// * Of the *diagnostic groups* answered, those naming an *input* of the *node* are yielded
    ///   ([`answer_of_inputs`]).
    /// * Where the connection fails, the *node* is given up, so that every later change is
    ///   reported as not served until a *node* is opened again.
    ///
    /// # Errors
    /// Returns the text of FR-112: that no *node* is served, or the rendering of the
    /// [`BuildError`].
    pub fn transmit(
        &mut self,
        document: &str,
        provided_before: &str,
        increment: &TextIncrement,
        provided_after: &str,
    ) -> Result<DiagnosticAnswer, String> {
        let Some(session) = self.session.as_mut() else {
            return Err(NOT_SERVED.to_owned());
        };
        let outcome = session.transmit(document, provided_before, increment, provided_after);
        self.give_up_on(&outcome);
        self.answered(outcome)
    }

    // realises FR-167, FR-107, FR-112, FR-225, FR-226
    /// Transmits the octets of a document to the served *node*, as
    /// [`NodeSession::open_octets`].
    ///
    /// # Errors
    /// Returns the text of FR-112, as [`NodeRunner::transmit`] does.
    pub fn open_octets(
        &mut self,
        document: &str,
        octets: &[u8],
    ) -> Result<DiagnosticAnswer, String> {
        let Some(session) = self.session.as_mut() else {
            return Err(NOT_SERVED.to_owned());
        };
        let outcome = session.open_octets(document, octets);
        self.give_up_on(&outcome);
        self.answered(outcome)
    }

    // realises FR-210, FR-107, FR-112, FR-225, FR-226
    /// Transmits the change of a *binary document* to the served *node*, as
    /// [`NodeSession::edit_octets`].
    ///
    /// # Errors
    /// Returns the text of FR-112, as [`NodeRunner::transmit`] does.
    pub fn edit_octets(
        &mut self,
        document: &str,
        octets_before: &[u8],
        octets_after: &[u8],
    ) -> Result<DiagnosticAnswer, String> {
        let Some(session) = self.session.as_mut() else {
            return Err(NOT_SERVED.to_owned());
        };
        let outcome = session.edit_octets(document, octets_before, octets_after);
        self.give_up_on(&outcome);
        self.answered(outcome)
    }

    // realises FR-110, FR-111, FR-112, FR-239, FR-264
    /// Reads the content the served *node* holds of a document, as
    /// [`NodeSession::query_content`]: `None` where the *node* cancelled the query, an edit
    /// request of the *producer* having superseded it.
    ///
    /// # Errors
    /// Returns the text of FR-112, as [`NodeRunner::transmit`] does.
    pub fn query_content(&mut self, document: &str) -> Result<Option<DocumentContent>, String> {
        let Some(session) = self.session.as_mut() else {
            return Err(NOT_SERVED.to_owned());
        };
        let outcome = session.query_content(document);
        self.give_up_on(&outcome);
        outcome.map_err(|error| error.to_string())
    }

    // realises FR-105
    /// Closes the connection to the *node* attached to, where there is one; the *node* keeps
    /// running, no *shutdown request* being transmitted.
    pub fn detach(&mut self) {
        self.session = None;
    }

    // realises FR-104, FR-123
    /// Yields whether a *node* is served.
    pub fn is_attached(&self) -> bool {
        self.session.is_some()
    }

    /// Yields the outcome of a change as the runner reports it: the answer with the
    /// *diagnostic groups* of the *inputs* alone, or the text of FR-112.
    fn answered(
        &self,
        outcome: Result<DiagnosticAnswer, BuildError>,
    ) -> Result<DiagnosticAnswer, String> {
        outcome
            .map(|answer| answer_of_inputs(answer, &self.inputs))
            .map_err(|error| error.to_string())
    }

    /// Gives the *node* up where `outcome` says that it is not reached any more.
    fn give_up_on<T>(&mut self, outcome: &Result<T, BuildError>) {
        if let Err(BuildError::Connection(_) | BuildError::Process(_) | BuildError::Protocol(_)) =
            outcome
        {
            self.session = None;
        }
    }
}

/*  * validated        : ✅
 * completeness     : ❌ a served node needs a build system that runs it; it is covered by the
 *                    ignored integration test 'tests/build_system.rs'
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : NodeEditor::open_node, NodeEditor::apply_increment, NodeEditor::apply_octets,
 *                    NodeEditor::report_arrived, NodeEditor::close, NodeEditor::shut_down */
#[cfg(test)]
mod tests {
    use super::*;

    fn increment() -> TextIncrement {
        TextIncrement {
            position: 0,
            range: 0,
            text: "a".to_owned(),
        }
    }

    fn group(document: &str, version: u64) -> DiagnosticGroup {
        DiagnosticGroup {
            document: document.to_owned(),
            version,
            diagnostics: vec![],
        }
    }

    fn held(of: &[(&str, u64)]) -> BTreeMap<String, u64> {
        of.iter()
            .map(|(document, version)| ((*document).to_owned(), *version))
            .collect()
    }

    fn versions(of: &[(&str, u64)]) -> Vec<(String, u64)> {
        of.iter()
            .map(|(document, version)| ((*document).to_owned(), *version))
            .collect()
    }

    fn watched() -> WatchedDocuments {
        WatchedDocuments::of(vec!["x.txt".to_owned(), "y.txt".to_owned()])
    }

    #[test]
    fn change_without_a_served_node_is_reported_as_not_served() {
        // FR-112
        let mut runner = NodeRunner::new();
        let transmitted = runner.transmit("a.gc3", "", &increment(), "a");
        let opened =
            runner
                .open_octets("a.bin", &[0xff])
                .and(runner.edit_octets("a.bin", &[0xff], &[0x00]));
        let content = runner.query_content("x.txt");
        assert_eq!(
            (transmitted, opened, content),
            (
                Err(NOT_SERVED.to_owned()),
                Err(NOT_SERVED.to_owned()),
                Err(NOT_SERVED.to_owned())
            )
        );
    }

    #[test]
    fn node_whose_service_socket_does_not_exist_is_not_served() {
        // FR-104, FR-112
        let mut runner = NodeRunner::new();
        let attached = runner.attach(
            &NodeAttach {
                socket: PathBuf::from("/nonexistent/genc3wb/a.gc3.sock"),
                inputs: vec![],
            },
            Box::new(|_| {}),
        );
        assert!(
            matches!(
                attached,
                Err(BuildError::Connection(_) | BuildError::Unsupported)
            ) && !runner.is_attached()
        );
    }

    #[test]
    fn diagnostic_group_naming_no_input_of_the_node_is_left_out() {
        // FR-226: 'b.in' is an input, the meta compiler DSL 'a.gc3' and 'x.in' are none
        let answer = DiagnosticAnswer {
            version: 0,
            diagnostics: vec![],
            groups: vec![group("a.gc3", 0), group("b.in", 0), group("x.in", 0)],
            erroneous: false,
        };
        assert_eq!(
            answer_of_inputs(answer, &["a.in".to_owned(), "b.in".to_owned()]),
            DiagnosticAnswer {
                version: 0,
                diagnostics: vec![],
                groups: vec![group("b.in", 0)],
                erroneous: false,
            }
        );
    }

    #[test]
    fn detach_without_a_served_node_does_nothing() {
        // FR-105
        let mut runner = NodeRunner::new();
        runner.detach();
        assert!(!runner.is_attached());
    }

    #[test]
    fn group_of_a_state_is_taken_at_the_version_held_of_an_edited_document() {
        // FR-231: the meta compiler DSL 'a.gc3' and the input 'a.in' are edited by the product
        assert_eq!(
            groups_of_state(
                vec![group("a.gc3", 2), group("a.in", 5)],
                &held(&[("a.gc3", 2), ("a.in", 5)]),
                &[]
            ),
            vec![group("a.gc3", 2), group("a.in", 5)]
        );
    }

    #[test]
    fn group_of_a_state_at_another_version_or_of_a_document_not_opened_is_left_out() {
        // FR-232: 'a.in' is held at 5, 'b.in' was not opened, 'z.in' is no document of the node
        assert_eq!(
            groups_of_state(
                vec![group("a.in", 4), group("b.in", 0), group("z.in", 0)],
                &held(&[("a.in", 5)]),
                &[]
            ),
            vec![]
        );
    }

    #[test]
    fn group_of_a_state_naming_an_input_of_a_producer_is_taken_at_any_version() {
        // FR-233
        assert_eq!(
            groups_of_state(
                vec![group("x.txt", 7), group("a.in", 4)],
                &held(&[("a.in", 5)]),
                &["x.txt".to_owned()]
            ),
            vec![group("x.txt", 7)]
        );
    }

    #[test]
    fn content_of_every_document_is_read_at_the_first_state() {
        // FR-110, FR-239: no content is held; 'a.in' is edited and not read
        let mut documents = watched();
        assert_eq!(
            documents.outdated(&versions(&[("a.in", 3), ("x.txt", 0), ("y.txt", 2)])),
            vec!["x.txt".to_owned(), "y.txt".to_owned()]
        );
    }

    #[test]
    fn content_is_read_again_where_the_state_reports_another_version() {
        // FR-111, FR-239
        let mut documents = watched();
        let _ = documents.outdated(&versions(&[("x.txt", 0), ("y.txt", 2)]));
        documents.take("x.txt", 0);
        documents.take("y.txt", 2);
        let unchanged = documents.outdated(&versions(&[("x.txt", 0), ("y.txt", 2)]));
        let changed = documents.outdated(&versions(&[("x.txt", 0), ("y.txt", 3)]));
        assert_eq!(
            (unchanged, changed, documents.version_of("y.txt")),
            (vec![], vec!["y.txt".to_owned()], Some(2))
        );
    }

    #[test]
    fn content_under_way_at_the_stated_version_is_not_asked_for_again() {
        // FR-239: two state responses before the content response
        let mut documents = watched();
        let first = documents.outdated(&versions(&[("x.txt", 1)]));
        let second = documents.outdated(&versions(&[("x.txt", 1)]));
        let third = documents.outdated(&versions(&[("x.txt", 2)]));
        assert_eq!(
            (first, second, third),
            (vec!["x.txt".to_owned()], vec![], vec!["x.txt".to_owned()])
        );
    }

    #[test]
    fn content_answered_at_a_later_version_is_the_one_held() {
        // FR-239: the node changed the document between the state and the content response
        let mut documents = watched();
        let _ = documents.outdated(&versions(&[("x.txt", 1)]));
        documents.take("x.txt", 2);
        assert_eq!(
            (
                documents.version_of("x.txt"),
                documents.outdated(&versions(&[("x.txt", 2)]))
            ),
            (Some(2), vec![])
        );
    }

    #[test]
    fn failed_content_query_is_asked_for_again_by_the_next_state() {
        // FR-112, FR-239, FR-264: a failed and a cancelled query alike
        let mut documents = watched();
        let _ = documents.outdated(&versions(&[("x.txt", 1)]));
        documents.fail("x.txt");
        assert_eq!(
            (
                documents.version_of("x.txt"),
                documents.outdated(&versions(&[("x.txt", 1)]))
            ),
            (None, vec!["x.txt".to_owned()])
        );
    }

    #[test]
    fn content_of_a_document_that_is_none_of_these_is_not_held() {
        let mut documents = watched();
        documents.take("a.in", 1);
        assert_eq!(documents.version_of("a.in"), None);
    }
}
