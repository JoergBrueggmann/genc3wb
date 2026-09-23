//! The *bridged type* `NodeEditor`: the *node editor*, the *node window* as the *front end* binds to it.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::bridge::input_group::InputGroup;
use crate::core::api_message::{
    Diagnostic, has_error, marks_of_diagnostics, rendering_of_documents,
};
use crate::core::input_file::InputKind;
use crate::core::network_builder::NetworkBuilder;
use crate::core::network_graph::path_of_identifier;
use crate::core::node_runner::{NodeRunner, NodeStart, Originals};
use crate::core::text_increment::TextIncrement;

use qtbridge::{QObjectHolder, QmlMethodInvoker, invoke_method, qobject};

use std::cell::RefCell;
use std::collections::{BTreeMap, VecDeque};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

/// The time the main thread waits for the *node thread* to shut the *node* down.
const SHUTDOWN_WAIT: Duration = Duration::from_secs(3);

/// What the main thread asks of the *node thread*.
enum NodeCommand {
    /// shut the served *node* down and start the named one (FR-104, FR-105)
    Start(NodeStart),
    /// transmit the change of a document (FR-106)
    Transmit {
        document: String,
        provided_before: String,
        increment: TextIncrement,
        provided_after: String,
    },
    /// transmit a *store request* (FR-109)
    Store,
    /// shut the served *node* down (FR-105); the thread ends where `end` holds
    Stop { end: bool },
}

/// What the *node thread* reports to the main thread.
enum NodeReport {
    /// whether the *node* was started; carries the text of FR-112 where it was not
    Started(Result<(), String>),
    /// the outcome of a change: the *diagnostics*, or the text of FR-112
    Transmitted {
        document: String,
        outcome: Result<Vec<Diagnostic>, String>,
    },
    /// the outcome of a *store request*
    Stored(Result<(), String>),
    /// the *node* was shut down; the thread ended where `ended` holds
    Stopped { ended: bool },
}

/// An *input group* of the *node window* with its invoker, by which the *node editor* reaches it.
type Group = (Rc<RefCell<InputGroup>>, QmlMethodInvoker);

// realises FR-001, FR-103, FR-104, FR-117, FR-123
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
    /// the index of the presented *input*
    input_page: usize,
    /// the *provided text* of each document, by its *document identifier*, as a receiver holds it
    provided: BTreeMap<String, String>,
    /// the *diagnostics* of each document, by its *document identifier*, as the *node* last
    /// answered them
    diagnostics: BTreeMap<String, Vec<Diagnostic>>,
    /// the *originals* of the served *node*
    originals: Originals,
    /// whether the *node* is served
    served: bool,
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
    /// the reports of the *node thread*
    reports: Option<Receiver<NodeReport>>,
    /// the invoker of the output group; wired by the *workbench object*
    output: Option<QmlMethodInvoker>,
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
            input_page: 0,
            provided: BTreeMap::new(),
            diagnostics: BTreeMap::new(),
            originals: Originals::default(),
            served: false,
            status: String::new(),
            requests_pending: 0,
            changes_started: VecDeque::new(),
            stores_started: VecDeque::new(),
            commands: None,
            reports: None,
            output: None,
        }
    }
}

// realises FR-001, FR-086, FR-087, FR-102 to FR-112, FR-115, FR-117 to FR-123, FR-125, FR-126
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

    // realises FR-086, FR-087, FR-102, FR-103, FR-104, FR-105, FR-110, FR-115, FR-122
    /// Opens a *node*: writes the *originals* of the *node* before back and captures those of
    /// this one, forgets the *provided texts*, names the documents in the *input groups* and the
    /// paths in the output group, presents the first *input*, and hands the start to the
    /// *node thread*; scheduled by the network editor.
    ///
    /// # Arguments
    /// * `name` - the name of the *node*
    /// * `build_system` - the path of the *build system*
    /// * `directory` - the directory of the *network file*
    /// * `meta_dsl`, `inputs`, `outputs` - the paths as the *node description* carries them
    /// * `producers` - the name of the *producer* of each *input*, empty where it has none
    #[qslot(qml_name = "openNode")]
    #[allow(clippy::too_many_arguments)] // the description of a node, as the bridge carries it
    fn open_node(
        &mut self,
        name: String,
        build_system: String,
        directory: String,
        meta_dsl: String,
        inputs: Vec<String>,
        producers: Vec<String>,
        outputs: Vec<String>,
    ) {
        let directory = PathBuf::from(directory);
        self.restore_originals();
        self.originals = Originals::capture(&produced_paths(&directory, &inputs, &producers));
        self.provided.clear();
        self.diagnostics.clear();
        self.changes_started.clear();
        self.stores_started.clear();
        self.name = name;
        self.documents = std::iter::once(meta_dsl.clone())
            .chain(inputs.iter().cloned())
            .collect();
        self.input_page = 0;
        self.name_document(&self.meta_dsl.1, &meta_dsl, &directory, "");
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
            let producer = producers.get(index).map(String::as_str).unwrap_or("");
            self.name_document(&self.inputs[index].1, identifier, &directory, producer);
        }
        if let Some(output) = &self.output {
            let paths: Vec<String> = outputs
                .iter()
                .map(|identifier| path_of_identifier(&directory, identifier))
                .collect();
            invoke_method!(output, "setPaths", paths);
            invoke_method!(output, "setDiagnostics", String::new());
        }
        self.served = false;
        self.status.clear();
        self.send(NodeCommand::Start(NodeStart {
            executable: build_system,
            directory,
            meta_dsl,
            inputs,
            outputs,
        }));
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

    // realises FR-105, FR-115
    /// Shuts the *node* down without ending the *node thread*: writes the *originals* back and
    /// hands the stop over; called when the *node window* closes.
    #[qslot(qml_name = "close")]
    fn close(&mut self) {
        self.restore_originals();
        if self.commands.is_some() {
            self.send(NodeCommand::Stop { end: false });
        }
        self.set_served(false, String::new());
    }

    // realises FR-105, FR-115
    /// Writes the *originals* back, hands the stop to the *node thread* and waits for its end;
    /// called when *product* terminates.
    #[qslot(qml_name = "shutDown")]
    fn shut_down(&mut self) {
        self.restore_originals();
        let Some(commands) = self.commands.take() else {
            return;
        };
        let _ = commands.send(NodeCommand::Stop { end: true });
        if let Some(reports) = self.reports.take() {
            while let Ok(report) = reports.recv_timeout(SHUTDOWN_WAIT) {
                if matches!(report, NodeReport::Stopped { ended: true }) {
                    break;
                }
            }
        }
    }

    // realises FR-107, FR-108, FR-109, FR-111, FR-112, FR-125, FR-126
    /// Takes every report of the *node thread*; scheduled by that thread on the main thread.
    #[qslot(qml_name = "reportArrived")]
    fn report_arrived(&mut self) {
        let reports: Vec<NodeReport> = self
            .reports
            .as_ref()
            .map(|reports| reports.try_iter().collect())
            .unwrap_or_default();
        for report in reports {
            match report {
                NodeReport::Started(outcome) => match outcome {
                    Ok(()) => self.set_served(true, String::new()),
                    Err(message) => self.set_served(false, message),
                },
                NodeReport::Transmitted { document, outcome } => {
                    self.uncount_request();
                    let started = self.changes_started.pop_front();
                    match outcome {
                        Ok(diagnostics) => {
                            self.present_diagnostics(&document, &diagnostics);
                            if has_error(&diagnostics) {
                                self.measure(started);
                            } else {
                                self.send(NodeCommand::Store);
                                self.count_request();
                                if let Some(started) = started {
                                    self.stores_started.push_back(started);
                                }
                            }
                        }
                        Err(message) => {
                            self.measure(started);
                            self.set_status(message);
                        }
                    }
                }
                NodeReport::Stored(outcome) => {
                    self.uncount_request();
                    let started = self.stores_started.pop_front();
                    self.measure(started);
                    match outcome {
                        Ok(()) => {
                            if let Some(output) = &self.output {
                                invoke_method!(output, "reload");
                            }
                        }
                        Err(message) => self.set_status(message),
                    }
                }
                NodeReport::Stopped { .. } => {}
            }
        }
    }
}

impl NodeEditor {
    /// Wires the *node editor*: the invoker of the output group, and the receiver of the
    /// *input group* of the *meta compiler DSL*, which is the *node editor* itself.
    ///
    /// * Called by the *workbench object* once, before the *front end* binds to the editor.
    pub fn configure(&mut self, output: QmlMethodInvoker) {
        self.output = Some(output);
        let receiver = self.get_qml_method_invoker();
        self.meta_dsl
            .0
            .borrow_mut()
            .configure(InputKind::MetaDsl, None, Some(receiver));
    }

    // realises FR-086, FR-087
    /// Schedules `nameDocument` of an *input group* with the path of `identifier`.
    fn name_document(
        &self,
        group: &QmlMethodInvoker,
        identifier: &str,
        directory: &Path,
        producer: &str,
    ) {
        invoke_method!(
            group,
            "nameDocument",
            identifier.to_owned(),
            path_of_identifier(directory, identifier),
            producer.to_owned()
        );
    }

    // realises FR-107, FR-108, FR-125, FR-126, FR-127
    /// Records the *diagnostics* of `document`, schedules `setDiagnostics` of its *input group*
    /// with their marks in the *provided text* of the document, and `setDiagnostics` of the
    /// output group with the rendering of those of every document, in the order of the
    /// documents.
    fn present_diagnostics(&mut self, document: &str, diagnostics: &[Diagnostic]) {
        self.diagnostics
            .insert(document.to_owned(), diagnostics.to_vec());
        let text = self.provided.get(document).cloned().unwrap_or_default();
        if let Some(group) = self.group_of(document) {
            let marks = marks_of_diagnostics(&text, diagnostics);
            invoke_method!(
                group,
                "setDiagnostics",
                marks.starts,
                marks.ends,
                marks.severities,
                marks.texts
            );
        }
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

    // realises FR-115
    /// Writes the *originals* back and forgets them.
    fn restore_originals(&mut self) {
        self.originals.restore();
        self.originals = Originals::default();
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

    /// Records whether the *node* is served and the status, and emits `status_changed`.
    fn set_served(&mut self, served: bool, status: String) {
        self.served = served;
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
            let (reports, received_reports) = channel::<NodeReport>();
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

// realises FR-115
/// Yields the paths of the *inputs* that have a *producer*, resolved against `directory`.
fn produced_paths(directory: &Path, inputs: &[String], producers: &[String]) -> Vec<String> {
    inputs
        .iter()
        .zip(producers)
        .filter(|(_, producer)| !producer.is_empty())
        .map(|(identifier, _)| path_of_identifier(directory, identifier))
        .collect()
}

/// Runs the *node thread*: carries out every command with the [`NodeRunner`] it owns, reports
/// through `reports`, and schedules `reportArrived` on the main thread after every report.
fn node_thread(
    commands: Receiver<NodeCommand>,
    reports: Sender<NodeReport>,
    invoker: QmlMethodInvoker,
) {
    let mut runner = NodeRunner::new(NetworkBuilder::default_directory());
    while let Ok(command) = commands.recv() {
        let (report, end) = match command {
            NodeCommand::Start(start) => (
                NodeReport::Started(runner.start(&start).map_err(|error| error.to_string())),
                false,
            ),
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
            NodeCommand::Store => (NodeReport::Stored(runner.store()), false),
            NodeCommand::Stop { end } => {
                runner.stop();
                (NodeReport::Stopped { ended: end }, end)
            }
        };
        if reports.send(report).is_err() {
            break;
        }
        invoke_method!(invoker, "reportArrived");
        if end {
            break;
        }
    }
    drop(runner);
}
