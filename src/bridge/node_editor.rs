//! The *bridged type* `NodeEditor`: the *node editor*, the *node window* as the *front end* binds to it.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::bridge::input_group::InputGroup;
use crate::core::api_message::{
    Diagnostic, NodeMode, bit_marks_of_diagnostics, marks_of_diagnostics, rendering_of_documents,
};
use crate::core::build_system::{
    BuildError, DiagnosticAnswer, DocumentContent, StateSink, Watched,
};
use crate::core::input_file::InputKind;
use crate::core::network_graph::path_of_identifier;
use crate::core::node_runner::{NodeAttach, NodeRunner, WatchedDocuments, groups_of_state};
use crate::core::octet_view::text_of_octets;
use crate::core::text_increment::TextIncrement;

use qtbridge::{QObjectHolder, QmlMethodInvoker, invoke_method, qobject};

use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// The time the main thread waits for the *node thread* to close the connection to the *node*.
const SHUTDOWN_WAIT: Duration = Duration::from_secs(3);

/// What the main thread asks of the *node thread*.
enum NodeCommand {
    /// close the connection to the served *node* and attach to the named one, as the
    /// attachment of number `session` (FR-104, FR-105, FR-230)
    Attach { session: u32, attach: NodeAttach },
    /// transmit the change of a document (FR-106)
    Transmit {
        document: String,
        provided_before: String,
        increment: TextIncrement,
        provided_after: String,
    },
    /// transmit the octets of a document, as a *binary document* (FR-167)
    Open { document: String, octets: Vec<u8> },
    /// transmit the change of a *binary document* (FR-210)
    EditOctets {
        document: String,
        octets_before: Vec<u8>,
        octets_after: Vec<u8>,
    },
    /// read the content the *node* holds of a document (FR-110, FR-111, FR-239)
    QueryContent { document: String },
    /// close the connection to the served *node* (FR-105); the thread ends where `end` holds
    Detach { end: bool },
}

/// What the *node thread* and the reader thread of the connection to the *node* report to the
/// main thread, each with the number of the attachment it belongs to.
enum NodeReport {
    /// whether the *node* was attached to; carries the text of FR-112 where it was not
    Attached(Result<(), String>),
    /// the outcome of a change: the *diagnostics* of the document and of the other *inputs*
    /// the response grouped them for, or the text of FR-112
    Transmitted {
        document: String,
        outcome: Result<DiagnosticAnswer, String>,
    },
    /// the outcome of a content query request: the content of the document, `None` where the
    /// *node* cancelled the query (FR-264), or the text of FR-112
    Content {
        document: String,
        outcome: Result<Option<DocumentContent>, String>,
    },
    /// a *state response* of the *node*, as the reader thread received it (FR-231 to FR-234)
    Watched(Watched),
    /// the connection to the *node* was lost; carries the text of FR-112
    Lost(String),
    /// the connection to the *node* was closed; the thread ended where `ended` holds
    Detached { ended: bool },
}

/// An *input group* of the *node window* with its invoker, by which the *node editor* reaches it.
type Group = (Rc<RefCell<InputGroup>>, QmlMethodInvoker);

/// The invoker of the *node editor* as the *node thread* and the reader threads share it.
type SharedInvoker = Arc<Mutex<QmlMethodInvoker>>;

// realises FR-001, FR-103, FR-104, FR-117, FR-123, FR-230 to FR-234, FR-239
/// The *node editor*.
pub struct NodeEditor {
    /// the name of the opened *node*, empty before the first one
    name: String,
    /// the *input group* of the *meta compiler DSL*, with its invoker
    meta_dsl: Group,
    /// the *input groups* of the *inputs*, with their invokers; those beyond the *inputs* of the
    /// opened *node* are kept for a later *node*
    inputs: Vec<Group>,
    /// the *document identifiers* of the *meta compiler DSL* and of the *inputs* of the opened
    /// *node*, in that order
    documents: Vec<String>,
    /// the *document identifiers* of the *inputs* that have a *producer*, which *product* does
    /// not edit (FR-238)
    produced: Vec<String>,
    /// the *document identifiers* of the *outputs*, in the order of their *output pages*
    outputs: Vec<String>,
    /// the index of the presented *input*
    input_page: usize,
    /// the *provided text* of each document, by its *document identifier*, as a receiver holds it
    provided: BTreeMap<String, String>,
    /// the octets of each document provided as a *binary document*, by its
    /// *document identifier*, as the *node* holds them
    provided_octets: BTreeMap<String, Vec<u8>>,
    /// the *document version* the *node* answered last for each document *product* edits, by
    /// its *document identifier* (FR-231, FR-232)
    versions: BTreeMap<String, u64>,
    /// the content held of each *input* that has a *producer*, by its *document identifier*,
    /// as the *node* answered it (FR-239)
    contents: BTreeMap<String, Vec<u8>>,
    /// the *document version* the *diagnostics* of each *input* that has a *producer* were
    /// computed from (FR-233)
    group_versions: BTreeMap<String, u64>,
    /// the *inputs* that have a *producer* and the *outputs*, with the *document version* of
    /// the content held of each (FR-111, FR-239)
    watched: WatchedDocuments,
    /// the *diagnostics* of each document, by its *document identifier*, as the *node* last
    /// told them
    diagnostics: BTreeMap<String, Vec<Diagnostic>>,
    /// the number of the attachment: of the *node* opened last, raised when the
    /// *node window* closes as well, so that a report of an attachment before is passed over
    session: u32,
    /// whether the *node* is served
    served: bool,
    /// the *node mode* of the *node state* the *node* told last, `None` while it told none or
    /// is not served (FR-123)
    mode: Option<NodeMode>,
    /// the text of FR-112, empty where there is none
    status: String,
    /// the number of *requests* handed to the *node thread* and not yet reported
    requests_pending: u32,
    /// when each change not yet answered was handed over, in their order (FR-129)
    changes_started: VecDeque<Instant>,
    /// when the change each *store request* not yet answered follows was handed over (FR-129)
    stores_started: VecDeque<Instant>,
    /// the commands to the *node thread*, `None` before the first one and after `shut_down`
    commands: Option<Sender<NodeCommand>>,
    /// the reports of the *node thread* and of the reader threads, each with the number of its
    /// attachment
    reports: Option<Receiver<(u32, NodeReport)>>,
    /// the invoker of the output group; wired by the *workbench object*
    output: Option<QmlMethodInvoker>,
    /// the invoker of the network editor, which transmits the *store request* to the
    /// *build system* (FR-109); wired by the *workbench object*
    network: Option<QmlMethodInvoker>,
}

impl Default for NodeEditor {
    /// Creates the *node editor* with the *input group* of the *meta compiler DSL*, which the
    /// *front end* binds to by a constant property; the group is wired by `configure`.
    fn default() -> Self {
        let meta_dsl = InputGroup::default_with_attached_qobject();
        let invoker = meta_dsl.borrow().get_qml_method_invoker();
        NodeEditor {
            name: String::new(),
            meta_dsl: (meta_dsl, invoker),
            inputs: Vec::new(),
            documents: Vec::new(),
            produced: Vec::new(),
            outputs: Vec::new(),
            input_page: 0,
            provided: BTreeMap::new(),
            provided_octets: BTreeMap::new(),
            versions: BTreeMap::new(),
            contents: BTreeMap::new(),
            group_versions: BTreeMap::new(),
            watched: WatchedDocuments::default(),
            diagnostics: BTreeMap::new(),
            session: 0,
            served: false,
            mode: None,
            status: String::new(),
            requests_pending: 0,
            changes_started: VecDeque::new(),
            stores_started: VecDeque::new(),
            commands: None,
            reports: None,
            output: None,
            network: None,
        }
    }
}

// realises FR-001, FR-086, FR-087, FR-102 to FR-112, FR-117 to FR-123, FR-125, FR-126, FR-225,
// FR-230 to FR-234, FR-238, FR-239, FR-246 to FR-250, FR-252
#[qobject(NoQmlElement)]
impl NodeEditor {
    qproperty!("name", Read = name, Notify = node_changed);
    qproperty!("metaDsl", Read = meta_dsl, Constant);
    qproperty!("inputCount", Read = input_count, Notify = node_changed);
    qproperty!("inputPage", Read = input_page, Notify = input_page_changed);
    qproperty!(
        "hasNextInput",
        Read = has_next_input,
        Notify = input_page_changed
    );
    qproperty!(
        "hasPreviousInput",
        Read = has_previous_input,
        Notify = input_page_changed
    );
    qproperty!("served", Read = served, Notify = status_changed);
    qproperty!("nodeMode", Read = node_mode, Notify = status_changed);
    qproperty!("busy", Read = busy, Notify = status_changed);
    qproperty!("status", Read = status, Notify = status_changed);

    // getters
    fn name(&self) -> String {
        self.name.clone()
    }

    fn meta_dsl(&self) -> Rc<RefCell<InputGroup>> {
        Rc::clone(&self.meta_dsl.0)
    }

    fn input_count(&self) -> i32 {
        self.input_count_of_node() as i32
    }

    fn input_page(&self) -> i32 {
        self.input_page as i32
    }

    // realises FR-120
    fn has_next_input(&self) -> bool {
        self.input_page + 1 < self.input_count_of_node()
    }

    // realises FR-121
    fn has_previous_input(&self) -> bool {
        self.input_page > 0
    }

    fn served(&self) -> bool {
        self.served
    }

    // realises FR-123
    // The node mode as the message schema numbers it, 1 to 3, and 0 where none is told.
    fn node_mode(&self) -> i32 {
        self.mode
            .map_or(0, |mode| i32::try_from(mode.number()).unwrap_or(0))
    }

    fn busy(&self) -> bool {
        self.requests_pending > 0
    }

    fn status(&self) -> String {
        self.status.clone()
    }

    // signals
    #[qsignal(qml_name = "nodeChanged")]
    fn node_changed(&mut self);

    #[qsignal(qml_name = "inputPageChanged")]
    fn input_page_changed(&mut self);

    #[qsignal(qml_name = "statusChanged")]
    fn status_changed(&mut self);

    #[qsignal(qml_name = "processingMeasured")]
    fn processing_measured(&mut self, processing_time: i32);

    // slots
    // realises FR-001, FR-087
    /// Yields the *input group* of the *input* `index`; the one of the *meta compiler DSL* for
    /// an index there is no *input* for, so that no binding fails.
    #[qslot(qml_name = "inputAt")]
    fn input_at(&self, index: i32) -> Rc<RefCell<InputGroup>> {
        usize::try_from(index)
            .ok()
            .filter(|index| *index < self.input_count_of_node())
            .and_then(|index| self.inputs.get(index))
            .map_or_else(|| Rc::clone(&self.meta_dsl.0), |group| Rc::clone(&group.0))
    }

    // realises FR-118, FR-120
    /// Presents the *input group* of the next *input*.
    #[qslot(qml_name = "nextInput")]
    fn next_input(&mut self) {
        if self.has_next_input() {
            self.input_page += 1;
            self.input_page_changed();
        }
    }

    // realises FR-119, FR-121
    /// Presents the *input group* of the previous *input*.
    #[qslot(qml_name = "previousInput")]
    fn previous_input(&mut self) {
        if self.has_previous_input() {
            self.input_page -= 1;
            self.input_page_changed();
        }
    }

    // realises FR-086, FR-087, FR-102, FR-103, FR-104, FR-105, FR-110, FR-122, FR-230, FR-238
    /// Opens a *node*: forgets what is held of the *node* before, names the documents in the
    /// *input groups* and the paths in the output group, presents the first *input*, and hands
    /// the attachment to the *node thread*, which closes the connection to the *node* before;
    /// scheduled by the network editor.
    ///
    /// # Arguments
    /// * `name` - the name of the *node*
    /// * `directory` - the directory of the *network file*
    /// * `socket` - the path of the *service socket*, as the *node description* carries it
    /// * `meta_dsl`, `inputs`, `outputs` - the paths as the *node description* carries them
    /// * `meta_dsl_produced` - whether the *meta compiler DSL* has a *producer*, on which it is
    ///   treated as an *input* of a *producer* (FR-238 to FR-240)
    /// * `produced` - whether each *input* has a *producer*
    #[qslot(qml_name = "openNode")]
    #[allow(clippy::too_many_arguments)] // the description of a node, as the bridge carries it
    fn open_node(
        &mut self,
        name: String,
        directory: String,
        socket: String,
        meta_dsl: String,
        meta_dsl_produced: bool,
        inputs: Vec<String>,
        produced: Vec<bool>,
        outputs: Vec<String>,
    ) {
        let directory = PathBuf::from(directory);
        self.provided.clear();
        self.provided_octets.clear();
        self.versions.clear();
        self.contents.clear();
        self.group_versions.clear();
        self.diagnostics.clear();
        self.changes_started.clear();
        self.stores_started.clear();
        self.name = name;
        self.documents = std::iter::once(meta_dsl.clone())
            .chain(inputs.iter().cloned())
            .collect();
        self.produced = std::iter::once(meta_dsl.clone())
            .filter(|_| meta_dsl_produced)
            .chain(produced_inputs(&inputs, &produced))
            .collect();
        self.watched = WatchedDocuments::of(
            self.produced
                .iter()
                .chain(outputs.iter())
                .cloned()
                .collect(),
        );
        self.input_page = 0;
        self.name_document(&self.meta_dsl.1, &meta_dsl, &directory, meta_dsl_produced);
        while self.inputs.len() < inputs.len() {
            let group = InputGroup::default_with_attached_qobject();
            let receiver = self.get_qml_method_invoker();
            group
                .borrow_mut()
                .configure(InputKind::Input, None, Some(receiver));
            let invoker = group.borrow().get_qml_method_invoker();
            self.inputs.push((group, invoker));
        }
        for (index, identifier) in inputs.iter().enumerate() {
            let produced = self.produced.contains(identifier);
            self.name_document(&self.inputs[index].1, identifier, &directory, produced);
        }
        if let Some(output) = &self.output {
            let paths: Vec<String> = outputs
                .iter()
                .map(|identifier| path_of_identifier(&directory, identifier))
                .collect();
            invoke_method!(output, "setPaths", paths);
            invoke_method!(output, "setDiagnostics", String::new());
            invoke_method!(output, "setOrigins", Vec::<String>::new());
        }
        self.outputs = outputs;
        self.served = false;
        self.mode = None;
        self.status.clear();
        self.session = self.session.wrapping_add(1);
        self.send(NodeCommand::Attach {
            session: self.session,
            attach: NodeAttach {
                socket: directory.join(socket),
                inputs,
            },
        });
        self.node_changed();
        self.input_page_changed();
        self.status_changed();
    }

    // realises FR-106
    /// Carries the *text increment* into the *provided text* of `document` and hands the change
    /// to the *node thread*; scheduled by an *input group* of the *node window*.
    #[qslot(qml_name = "applyIncrement")]
    fn apply_increment(&mut self, document: String, position: i32, range: i32, text: String) {
        let increment = TextIncrement {
            position: usize::try_from(position).unwrap_or(0),
            range: usize::try_from(range).unwrap_or(0),
            text,
        };
        self.provided_octets.remove(&document);
        let provided = self.provided.entry(document.clone()).or_default();
        let provided_after = increment.overwriting(provided);
        let provided_before = std::mem::replace(provided, provided_after.clone());
        self.send(NodeCommand::Transmit {
            document,
            provided_before,
            increment,
            provided_after,
        });
        self.changes_started.push_back(Instant::now());
        self.count_request();
    }

    // realises FR-167, FR-168, FR-210
    /// Reads the octets the *input group* of `document` provided, keeps them as the octets the
    /// *node* holds, by whose reading under UTF-8 its *diagnostics* are marked in txt, and hands
    /// them to the *node thread*: as an open request where no octets of `document` were
    /// provided before, and as the change from those otherwise; scheduled by an *input group*
    /// of the *node window*.
    ///
    /// * The *input group* then holds an empty *provided text*, so that the next
    ///   *text increment* of the document carries its whole text; the *node editor* overwrites
    ///   the empty text with it.
    #[qslot(qml_name = "applyOctets")]
    fn apply_octets(&mut self, document: String) {
        let Some(octets) = self
            .input_of(&document)
            .map(|group| group.borrow().octets_provided())
        else {
            return;
        };
        let before = self
            .provided_octets
            .insert(document.clone(), octets.clone());
        self.provided.remove(&document);
        self.send(match before {
            Some(octets_before) => NodeCommand::EditOctets {
                document,
                octets_before,
                octets_after: octets,
            },
            None => NodeCommand::Open { document, octets },
        });
        self.changes_started.push_back(Instant::now());
        self.count_request();
    }

    // realises FR-105
    /// Closes the connection to the *node* without ending the *node thread*, the *node*
    /// running on: hands the detachment over; called when the *node window* closes.
    #[qslot(qml_name = "close")]
    fn close(&mut self) {
        self.session = self.session.wrapping_add(1);
        if self.commands.is_some() {
            self.send(NodeCommand::Detach { end: false });
        }
        self.set_served(false, String::new());
    }

    // realises FR-105
    /// Hands the detachment to the *node thread* and waits for its end; called when *product*
    /// terminates.
    #[qslot(qml_name = "shutDown")]
    fn shut_down(&mut self) {
        let Some(commands) = self.commands.take() else {
            return;
        };
        let _ = commands.send(NodeCommand::Detach { end: true });
        if let Some(reports) = self.reports.take() {
            while let Ok((_, report)) = reports.recv_timeout(SHUTDOWN_WAIT) {
                if matches!(report, NodeReport::Detached { ended: true }) {
                    break;
                }
            }
        }
    }

    // realises FR-109, FR-129, FR-211
    /// Takes the answer of the *build system* to a *store request*: takes the *output pages*
    /// as stored where `stored`, and as *outdated* by `message`, which the status line shows as
    /// well, where not; scheduled by the network editor.
    ///
    /// * `deferred` says that the *store request* was held back first (FR-241): its
    ///   *processing time* was measured when it was held back.
    #[qslot(qml_name = "storeAnswered")]
    fn store_answered(&mut self, stored: bool, message: String, deferred: bool) {
        if !deferred {
            self.uncount_request();
            let started = self.stores_started.pop_front();
            self.measure(started);
        }
        if stored {
            if let Some(output) = &self.output {
                invoke_method!(output, "stored");
            }
        } else {
            if let Some(output) = &self.output {
                invoke_method!(output, "setStoreFailure", message.clone());
            }
            self.set_status(message);
        }
    }

    // realises FR-109, FR-129, FR-252
    /// Takes that the *store request* asked for is held back, a *node* counting an error: the
    /// *processing time* of its change ends here; scheduled by the network editor.
    #[qslot(qml_name = "storeHeld")]
    fn store_held(&mut self) {
        self.uncount_request();
        let started = self.stores_started.pop_front();
        self.measure(started);
    }

    // realises FR-107, FR-108, FR-109, FR-110, FR-111, FR-112, FR-125, FR-126, FR-225,
    // FR-231 to FR-234, FR-239, FR-246 to FR-250, FR-252
    /// Takes every report of the *node thread* and of the reader thread of the connection to
    /// the *node*; scheduled by those threads on the main thread.
    ///
    /// * A report of an attachment before the present one is passed over, but for the
    ///   counting of its *request*.
    #[qslot(qml_name = "reportArrived")]
    fn report_arrived(&mut self) {
        let reports: Vec<(u32, NodeReport)> = self
            .reports
            .as_ref()
            .map(|reports| reports.try_iter().collect())
            .unwrap_or_default();
        for (session, report) in reports {
            let present = session == self.session;
            match report {
                NodeReport::Attached(outcome) => match outcome {
                    Ok(()) if present => self.set_served(true, String::new()),
                    Err(message) if present => self.set_served(false, message),
                    Ok(()) | Err(_) => {}
                },
                NodeReport::Transmitted { document, outcome } => {
                    self.uncount_request();
                    if present {
                        let started = self.changes_started.pop_front();
                        self.take_answer(&document, outcome, started);
                    }
                }
                NodeReport::Content { document, outcome } => {
                    self.uncount_request();
                    if present {
                        self.take_content(&document, outcome);
                    }
                }
                NodeReport::Watched(watched) => {
                    if present {
                        self.take_state(watched);
                    }
                }
                NodeReport::Lost(message) => {
                    if present {
                        self.set_served(false, message);
                    }
                }
                NodeReport::Detached { .. } => {}
            }
        }
    }
}

impl NodeEditor {
    /// Wires the *node editor*: the invoker of the output group, the invoker of the network
    /// editor, and the receiver of the *input group* of the *meta compiler DSL*, which is the
    /// *node editor* itself.
    ///
    /// * Called by the *workbench object* once, before the *front end* binds to the editor.
    pub fn configure(&mut self, output: QmlMethodInvoker, network: QmlMethodInvoker) {
        self.output = Some(output);
        self.network = Some(network);
        let receiver = self.get_qml_method_invoker();
        self.meta_dsl
            .0
            .borrow_mut()
            .configure(InputKind::MetaDsl, None, Some(receiver));
    }

    // realises FR-086, FR-087, FR-238
    /// Schedules `nameDocument` of an *input group* with the path of `identifier`, and whether
    /// the document is an *input* that has a *producer*.
    fn name_document(
        &self,
        group: &QmlMethodInvoker,
        identifier: &str,
        directory: &Path,
        produced: bool,
    ) {
        invoke_method!(
            group,
            "nameDocument",
            identifier.to_owned(),
            path_of_identifier(directory, identifier),
            produced
        );
    }

    // realises FR-107, FR-108, FR-109, FR-112, FR-225, FR-241, FR-246, FR-247, FR-248, FR-252
    /// Takes the outcome of the change of `document`: records the *document version* answered,
    /// records and marks the *diagnostics* of the document and of the *diagnostic groups*
    /// taken, lists them, and asks the network editor for the *store request* where the answer
    /// carries no error, and to give up the one held back where it carries one; the text of
    /// FR-112 becomes the status where the change failed.
    fn take_answer(
        &mut self,
        document: &str,
        outcome: Result<DiagnosticAnswer, String>,
        started: Option<Instant>,
    ) {
        match outcome {
            Ok(answer) => {
                self.versions.insert(document.to_owned(), answer.version);
                self.mark_diagnostics(document, &answer.diagnostics);
                for group in &answer.groups {
                    self.mark_diagnostics(&group.document, &group.diagnostics);
                }
                self.list_diagnostics();
                if answer.erroneous {
                    self.measure(started);
                    if let Some(network) = &self.network {
                        invoke_method!(network, "cancelStore");
                    }
                } else {
                    if let Some(network) = &self.network {
                        invoke_method!(network, "store");
                    }
                    self.count_request();
                    if let Some(started) = started {
                        self.stores_started.push_back(started);
                    }
                }
            }
            Err(message) => {
                self.measure(started);
                // the reason for which the node is not served stays where a change, as the
                // registering of a document, is handed over while none is
                if self.served || self.status.is_empty() {
                    self.set_status(message);
                }
            }
        }
    }

    // realises FR-110, FR-111, FR-123, FR-231, FR-232, FR-233, FR-234, FR-239, FR-249, FR-250
    /// Takes a *state response* of the *node*: keeps its *node mode*, hands its *origins* to the
    /// output group, records and marks the *diagnostics* of the *diagnostic groups* that are
    /// taken ([`groups_of_state`]), lists them, and hands a content query to the *node thread*
    /// for every *input* that has a *producer* and every *output* the response states another
    /// *document version* of than the one of the content held.
    fn take_state(&mut self, watched: Watched) {
        let (origins, mode) = watched
            .states
            .into_iter()
            .next()
            .map(|state| (state.origins, state.mode))
            .unwrap_or_default();
        if mode != self.mode {
            self.mode = mode;
            self.status_changed();
        }
        if let Some(output) = &self.output {
            invoke_method!(output, "setOrigins", origins);
        }
        for group in groups_of_state(watched.groups, &self.versions, &self.produced) {
            if self.produced.contains(&group.document) {
                self.diagnostics
                    .insert(group.document.clone(), group.diagnostics);
                self.group_versions
                    .insert(group.document.clone(), group.version);
                self.mark_produced(&group.document);
            } else {
                self.mark_diagnostics(&group.document, &group.diagnostics);
            }
        }
        self.list_diagnostics();
        for document in self.watched.outdated(&watched.versions) {
            self.send(NodeCommand::QueryContent { document });
            self.count_request();
        }
    }

    // realises FR-110, FR-111, FR-112, FR-233, FR-239, FR-264
    /// Takes the outcome of a content query: hands the content of an *input* that has a
    /// *producer* to its *input group* and marks its *diagnostics* in it, and hands the content
    /// of an *output* to the output group; the text of FR-112 becomes the status where the
    /// query failed, and the next *state response* has the content read again, as it does where
    /// the *node* cancelled the query, which leaves the content shown as it is.
    fn take_content(&mut self, document: &str, outcome: Result<Option<DocumentContent>, String>) {
        let content = match outcome {
            Ok(Some(content)) => content,
            Ok(None) => {
                self.watched.fail(document);
                return;
            }
            Err(message) => {
                self.watched.fail(document);
                self.set_status(message);
                return;
            }
        };
        self.watched.take(document, content.version);
        if self.produced.iter().any(|produced| produced == document) {
            if let Some(group) = self.group_of(document) {
                invoke_method!(group, "setContent", content.octets.clone());
            }
            self.contents.insert(document.to_owned(), content.octets);
            self.mark_produced(document);
        } else if let Some(index) = self.outputs.iter().position(|output| output == document)
            && let Some(output) = &self.output
        {
            invoke_method!(
                output,
                "setContent",
                i32::try_from(index).unwrap_or(i32::MAX),
                content.octets
            );
        }
    }

    // realises FR-233, FR-250
    /// Schedules the marks of the *diagnostics* recorded of `document`, an *input* that has a
    /// *producer*, in the content held of it, where that content is of the *document version*
    /// the *diagnostics* were computed from, and no mark otherwise: the content of that
    /// *document version* is under way, or the *state response* of the content held is.
    fn mark_produced(&self, document: &str) {
        let matching = matches!(
            (
                self.group_versions.get(document),
                self.watched.version_of(document)
            ),
            (Some(computed), Some(held)) if *computed == held
        );
        let diagnostics = self
            .diagnostics
            .get(document)
            .filter(|_| matching)
            .cloned()
            .unwrap_or_default();
        let text = self
            .contents
            .get(document)
            .map(|octets| text_of_octets(octets))
            .unwrap_or_default();
        self.schedule_marks(document, &text, &diagnostics);
    }

    // realises FR-107, FR-108, FR-126, FR-127, FR-225, FR-231, FR-246 to FR-249
    /// Records the *diagnostics* of `document`, a document *product* edits, in place of those
    /// recorded before, and schedules `setDiagnostics` and `setOctetMarks` of its *input group*
    /// with their marks in the *provided text* of the document and in its bits.
    fn mark_diagnostics(&mut self, document: &str, diagnostics: &[Diagnostic]) {
        self.diagnostics
            .insert(document.to_owned(), diagnostics.to_vec());
        let text = self
            .provided_octets
            .get(document)
            .map(|octets| text_of_octets(octets))
            .or_else(|| self.provided.get(document).cloned())
            .unwrap_or_default();
        self.schedule_marks(document, &text, diagnostics);
    }

    // realises FR-126, FR-127, FR-169, FR-170
    /// Schedules `setDiagnostics` and `setOctetMarks` of the *input group* of `document` with
    /// the marks of `diagnostics` in `text` and in the bits of the document.
    fn schedule_marks(&self, document: &str, text: &str, diagnostics: &[Diagnostic]) {
        if let Some(group) = self.group_of(document) {
            let marks = marks_of_diagnostics(text, diagnostics);
            invoke_method!(
                group,
                "setDiagnostics",
                marks.starts,
                marks.ends,
                marks.severities,
                marks.texts
            );
            let marks = bit_marks_of_diagnostics(diagnostics);
            invoke_method!(
                group,
                "setOctetMarks",
                marks.starts,
                marks.ends,
                marks.severities,
                marks.texts
            );
        }
    }

    // realises FR-107, FR-108, FR-125, FR-225, FR-231, FR-233
    /// Schedules `setDiagnostics` of the output group with the rendering of the *diagnostics*
    /// recorded of every document, in the order of the documents; once per *diagnostic*
    /// response and per *state response*, after those of its documents are recorded, so that
    /// the *diagnostics page* changes once.
    fn list_diagnostics(&self) {
        if let Some(output) = &self.output {
            let documents: Vec<(String, Vec<Diagnostic>)> = self
                .documents
                .iter()
                .map(|identifier| {
                    (
                        identifier.clone(),
                        self.diagnostics
                            .get(identifier)
                            .cloned()
                            .unwrap_or_default(),
                    )
                })
                .collect();
            invoke_method!(output, "setDiagnostics", rendering_of_documents(&documents));
        }
    }

    /// Yields the number of *inputs* of the opened *node*.
    fn input_count_of_node(&self) -> usize {
        self.documents.len().saturating_sub(1)
    }

    /// Yields the invoker of the *input group* whose document is `identifier`, `None` where the
    /// opened *node* has no such document.
    fn group_of(&self, identifier: &str) -> Option<&QmlMethodInvoker> {
        let index = self
            .documents
            .iter()
            .position(|document| document == identifier)?;
        match index {
            0 => Some(&self.meta_dsl.1),
            _ => self.inputs.get(index - 1).map(|(_, invoker)| invoker),
        }
    }

    /// Yields the *input group* of the *input* whose document is `identifier`, `None` where the
    /// opened *node* has no such *input*.
    fn input_of(&self, identifier: &str) -> Option<&Rc<RefCell<InputGroup>>> {
        let index = self
            .documents
            .iter()
            .position(|document| document == identifier)?;
        self.inputs
            .get(index.checked_sub(1)?)
            .map(|(group, _)| group)
    }

    /// Records whether the *node* is served and the status, forgets the *node mode* where it is
    /// not served, and emits `status_changed`.
    fn set_served(&mut self, served: bool, status: String) {
        self.served = served;
        if !served {
            self.mode = None;
        }
        self.status = status;
        self.status_changed();
    }

    /// Records the status where it changed, and emits `status_changed`.
    fn set_status(&mut self, status: String) {
        if status == self.status {
            return;
        }
        self.status = status;
        self.status_changed();
    }

    // realises FR-129
    /// Emits `processing_measured` with the milliseconds since `started`, where it is known.
    fn measure(&mut self, started: Option<Instant>) {
        if let Some(started) = started {
            self.processing_measured(
                i32::try_from(started.elapsed().as_millis()).unwrap_or(i32::MAX),
            );
        }
    }

    /// Counts a *request* handed to the *node thread*, and emits `status_changed` for the first.
    fn count_request(&mut self) {
        self.requests_pending += 1;
        if self.requests_pending == 1 {
            self.status_changed();
        }
    }

    /// Counts a *request* reported by the *node thread*, and emits `status_changed` for the last.
    fn uncount_request(&mut self) {
        self.requests_pending = self.requests_pending.saturating_sub(1);
        if self.requests_pending == 0 {
            self.status_changed();
        }
    }

    /// Hands `command` to the *node thread*, which is started with the first command.
    fn send(&mut self, command: NodeCommand) {
        if self.commands.is_none() {
            let (commands, received_commands) = channel::<NodeCommand>();
            let (reports, received_reports) = channel::<(u32, NodeReport)>();
            let invoker = self.get_qml_method_invoker();
            std::thread::spawn(move || node_thread(received_commands, reports, invoker));
            self.commands = Some(commands);
            self.reports = Some(received_reports);
        }
        if let Some(commands) = &self.commands {
            let _ = commands.send(command);
        }
    }
}

// realises FR-238
/// Yields the *document identifiers* of those of `inputs` that have a *producer*, as `produced`
/// says for each.
fn produced_inputs(inputs: &[String], produced: &[bool]) -> Vec<String> {
    inputs
        .iter()
        .zip(produced)
        .filter(|(_, produced)| **produced)
        .map(|(identifier, _)| identifier.clone())
        .collect()
}

/// Schedules `reportArrived` of the *node editor* on the main thread.
fn schedule_report(invoker: &SharedInvoker) {
    if let Ok(invoker) = invoker.lock() {
        invoke_method!(invoker, "reportArrived");
    }
}

// realises FR-231, FR-234, FR-112
/// Yields the sink of the watch of the attachment `session`: it is called on the reader thread
/// of the connection to the *node*, reports every *state response* and the loss of the
/// connection through `reports`, and schedules `reportArrived` on the main thread.
fn sink_of(session: u32, reports: Sender<(u32, NodeReport)>, invoker: SharedInvoker) -> StateSink {
    Box::new(move |told: Result<Watched, BuildError>| {
        let report = match told {
            Ok(watched) => NodeReport::Watched(watched),
            Err(error) => NodeReport::Lost(error.to_string()),
        };
        if reports.send((session, report)).is_ok() {
            schedule_report(&invoker);
        }
    })
}

/// Runs the *node thread*: carries out every command with the [`NodeRunner`] it owns, reports
/// through `reports` with the number of the attachment, and schedules `reportArrived` on the
/// main thread after every report.
fn node_thread(
    commands: Receiver<NodeCommand>,
    reports: Sender<(u32, NodeReport)>,
    invoker: QmlMethodInvoker,
) {
    let invoker: SharedInvoker = Arc::new(Mutex::new(invoker));
    let mut runner = NodeRunner::new();
    let mut session = 0;
    while let Ok(command) = commands.recv() {
        let (report, end) = match command {
            NodeCommand::Attach {
                session: opened,
                attach,
            } => {
                session = opened;
                let sink = sink_of(session, reports.clone(), Arc::clone(&invoker));
                (
                    NodeReport::Attached(
                        runner
                            .attach(&attach, sink)
                            .map_err(|error| error.to_string()),
                    ),
                    false,
                )
            }
            NodeCommand::Transmit {
                document,
                provided_before,
                increment,
                provided_after,
            } => (
                NodeReport::Transmitted {
                    outcome: runner.transmit(
                        &document,
                        &provided_before,
                        &increment,
                        &provided_after,
                    ),
                    document,
                },
                false,
            ),
            NodeCommand::Open { document, octets } => (
                NodeReport::Transmitted {
                    outcome: runner.open_octets(&document, &octets),
                    document,
                },
                false,
            ),
            NodeCommand::EditOctets {
                document,
                octets_before,
                octets_after,
            } => (
                NodeReport::Transmitted {
                    outcome: runner.edit_octets(&document, &octets_before, &octets_after),
                    document,
                },
                false,
            ),
            NodeCommand::QueryContent { document } => (
                NodeReport::Content {
                    outcome: runner.query_content(&document),
                    document,
                },
                false,
            ),
            NodeCommand::Detach { end } => {
                runner.detach();
                (NodeReport::Detached { ended: end }, end)
            }
        };
        if reports.send((session, report)).is_err() {
            break;
        }
        schedule_report(&invoker);
        if end {
            break;
        }
    }
    drop(runner);
}
