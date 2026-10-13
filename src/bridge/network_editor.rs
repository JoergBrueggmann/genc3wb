//! The *bridged type* `NetworkEditor`: the group for the *network graph* as the *front end* binds to it.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::api_message::{NetworkMode, NodeDescription, NodeState, marks_of_diagnostics};
use crate::core::build_system::{BuildError, BuildSession, Watched};
use crate::core::executable;
use crate::core::network_builder::{BuildOutcome, BuiltGraph, NetworkBuilder, StateSinks};
use crate::core::network_graph::{GraphLayout, files_of_node, node_index_of_vertex};
use crate::core::node_state::{NodeStates, StoreGate};
use crate::core::settings::SettingsStore;
use crate::core::text_increment::TextIncrement;

use qtbridge::{QObjectHolder, QmlMethodInvoker, invoke_method, qobject};

use std::cell::RefCell;
use std::collections::VecDeque;
use std::path::Path;
use std::rc::Rc;
use std::sync::mpsc::{Receiver, Sender, channel};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

/// The time the main thread waits for the build thread to shut the *build system* down.
const SHUTDOWN_WAIT: Duration = Duration::from_secs(3);

/// What the main thread asks of the build thread.
enum Command {
    /// name the *build system* and the *network file*, and start the *build system* (FR-067)
    Restart {
        executable: String,
        network_path: String,
    },
    /// carry out a *build* (FR-069)
    Build {
        provided_before: String,
        increment: TextIncrement,
        provided_after: String,
    },
    /// transmit a *store request* to the *build system* (FR-109); `deferred` says that it was
    /// held back first (FR-241)
    Store { deferred: bool },
    /// transmit an *activate request* to the *build system* (FR-260)
    Activate,
    /// transmit a *deactivate request* to the *build system* (FR-267)
    Deactivate,
    /// shut the *build system* down and end the thread (FR-068)
    Stop,
}

/// What the build thread and the reader thread of the connection to the *build system* report
/// to the main thread.
enum Report {
    /// the outcome of a *build*: the *diagnostics* of the change, and the graph or the
    /// *error message*
    Built(BuildOutcome),
    /// the *build system* could not be started; carries the *error message*
    NotStarted(String),
    /// the outcome of a *store request*, the text of FR-112 where it failed
    Stored {
        outcome: Result<(), String>,
        deferred: bool,
    },
    /// the outcome of an *activate request*, the *error message* of FR-261 where it failed
    Activated(Result<(), String>),
    /// the outcome of a *deactivate request*, the *error message* of FR-268 where it failed
    Deactivated(Result<(), String>),
    /// the *node states* and the *network mode* of a *state response* of the *build system*,
    /// as the reader thread received it (FR-229)
    Watched(Vec<NodeState>, Option<NetworkMode>),
    /// the connection to the *build system* was lost
    Lost,
    /// the build thread ended
    Stopped,
}

// realises FR-064, FR-109, FR-229, FR-241, FR-252, FR-258, FR-259, FR-269, FR-270, FR-273,
// FR-274
/// The group for the *network graph* as the *front end* binds to it.
#[derive(Default)]
pub struct NetworkEditor {
    /// the path of the *build system*
    build_system_path: String,
    /// whether the named *build system* is executable
    executable: bool,
    /// the path of the *network file*, as its *input group* announced it
    network_path: String,
    /// the *provided text* of the code editor of the *network file*, as a receiver holds it
    provided: String,
    /// the URL of the image of the *network graph* of an *active network*, empty before the
    /// first successful *build* (FR-274)
    active_graph_source: String,
    /// the URL of the image of the *network graph* of a network that is not active, empty before
    /// the first successful *build* (FR-273)
    inactive_graph_source: String,
    /// the layout of that image
    layout: GraphLayout,
    /// the *nodes* of that image
    nodes: Vec<NodeDescription>,
    /// the *error message* of the last *build*, empty where it was successful
    error_message: String,
    /// whether the *error message* is presented instead of the *network graph*
    showing_error: bool,
    /// the number of *builds* handed to the build thread and not yet reported
    builds_pending: u32,
    /// when each *build* not yet reported was handed over, in their order (FR-129)
    builds_started: VecDeque<Instant>,
    /// the *node state* of every *node* and the *network mode*, as the *build system* told them
    /// last (FR-229)
    states: NodeStates,
    /// whether a *store request* is held back until the *node states* count no error and the
    /// network is active (FR-109, FR-241, FR-252)
    gate: StoreGate,
    /// the number of *activate requests* handed to the build thread and not yet reported
    /// (FR-259, FR-269)
    activations_pending: u32,
    /// the number of *deactivate requests* handed to the build thread and not yet reported
    /// (FR-259, FR-270)
    deactivations_pending: u32,
    /// the commands to the build thread, `None` before the first one and after `shut_down`
    commands: Option<Sender<Command>>,
    /// the reports of the build thread
    reports: Option<Receiver<Report>>,
    /// the settings, to store the path; wired by the *workbench object*
    settings: Option<Rc<RefCell<SettingsStore>>>,
    /// the invoker of the *node editor*, which receives the opened *node*
    node: Option<QmlMethodInvoker>,
    /// the invoker of the *input group* of the *network file*, which receives the marks of the
    /// *diagnostics* of a *build*
    network_input: Option<QmlMethodInvoker>,
}

// realises FR-064 to FR-069, FR-078 to FR-083, FR-085 to FR-088, FR-091, FR-092, FR-102 to FR-104,
// FR-109, FR-128, FR-228, FR-229, FR-235, FR-241, FR-245, FR-252, FR-258 to FR-263, FR-267 to
// FR-270, FR-273 to FR-275
#[qobject(NoQmlElement)]
impl NetworkEditor {
    qproperty!(
        "buildSystemPath",
        Read = build_system_path,
        Write = set_build_system_path,
        Notify = build_system_path_changed
    );
    qproperty!(
        "executable",
        Read = executable,
        Notify = build_system_path_changed
    );
    qproperty!("available", Read = available, Constant);
    qproperty!("graphSource", Read = graph_source, Notify = view_changed);
    qproperty!("errorMessage", Read = error_message, Notify = view_changed);
    qproperty!("showingError", Read = showing_error, Notify = view_changed);
    qproperty!(
        "errorButtonVisible",
        Read = error_button_visible,
        Notify = view_changed
    );
    qproperty!(
        "graphButtonVisible",
        Read = graph_button_visible,
        Notify = view_changed
    );
    qproperty!("building", Read = building, Notify = building_changed);
    qproperty!("networkMode", Read = network_mode, Notify = mode_changed);
    qproperty!("switchable", Read = switchable, Notify = mode_changed);
    qproperty!("activating", Read = activating, Notify = mode_changed);
    qproperty!("deactivating", Read = deactivating, Notify = mode_changed);

    // getters
    fn build_system_path(&self) -> String {
        self.build_system_path.clone()
    }

    fn executable(&self) -> bool {
        self.executable
    }

    // realises C-005
    fn available(&self) -> bool {
        BuildSession::is_available()
    }

    // realises FR-078, FR-273, FR-274, FR-275
    // The image of the network mode kept: the one of an active network while the network is
    // active, the grey one otherwise.
    fn graph_source(&self) -> String {
        if self.states.is_active() {
            self.active_graph_source.clone()
        } else {
            self.inactive_graph_source.clone()
        }
    }

    fn error_message(&self) -> String {
        self.error_message.clone()
    }

    fn showing_error(&self) -> bool {
        self.showing_error
    }

    // realises FR-079, FR-081, FR-245
    fn error_button_visible(&self) -> bool {
        !self.error_message.is_empty() && !self.showing_error
    }

    // realises FR-081
    fn graph_button_visible(&self) -> bool {
        self.showing_error && !self.active_graph_source.is_empty()
    }

    fn building(&self) -> bool {
        self.builds_pending > 0
    }

    // realises FR-258
    // The network mode as the message schema numbers it, 1 to 3, and 0 where none is kept.
    fn network_mode(&self) -> i32 {
        self.states
            .mode()
            .map_or(0, |mode| i32::try_from(mode.number()).unwrap_or(0))
    }

    // realises FR-259
    // The activation switch is enabled while a network mode is kept and no activate request and
    // no deactivate request awaits its response.
    fn switchable(&self) -> bool {
        self.states.has_mode() && self.activations_pending == 0 && self.deactivations_pending == 0
    }

    // realises FR-269
    fn activating(&self) -> bool {
        self.activations_pending > 0
    }

    // realises FR-270
    fn deactivating(&self) -> bool {
        self.deactivations_pending > 0
    }

    // setters
    // realises FR-066, FR-067, FR-091
    // Names the path of the build system, stores it, determines whether it is executable, and
    // restarts the build system.
    fn set_build_system_path(&mut self, build_system_path: String) {
        if build_system_path == self.build_system_path {
            return;
        }
        self.build_system_path = build_system_path;
        if let Some(settings) = &self.settings {
            let mut settings = settings.borrow_mut();
            settings.set_build_system_path(&self.build_system_path);
            let _ = settings.store();
        }
        self.executable = executable::is_executable(&self.build_system_path);
        self.build_system_path_changed();
        self.restart();
    }

    // signals
    #[qsignal(qml_name = "buildSystemPathChanged")]
    fn build_system_path_changed(&mut self);

    #[qsignal(qml_name = "viewChanged")]
    fn view_changed(&mut self);

    #[qsignal(qml_name = "buildingChanged")]
    fn building_changed(&mut self);

    #[qsignal(qml_name = "modeChanged")]
    fn mode_changed(&mut self);

    #[qsignal(qml_name = "networkDeactivated")]
    fn network_deactivated(&mut self);

    #[qsignal(qml_name = "nodeOpened")]
    fn node_opened(&mut self, name: String);

    #[qsignal(qml_name = "processingMeasured")]
    fn processing_measured(&mut self, processing_time: i32);

    // slots
    // realises FR-092
    /// Names `path` as the path of the *build system* where it names an executable file; does
    /// nothing otherwise, so that the path is used as soon as the user has typed one that works.
    ///
    /// # Arguments
    /// * `path` - the text of the file name field of the *build system*, as the user typed it
    #[qslot(qml_name = "tryBuildSystemPath")]
    fn try_build_system_path(&mut self, path: String) {
        if executable::is_executable(&path) {
            self.set_build_system_path(path);
        }
    }

    // realises FR-080
    /// Presents the *error message* instead of the *network graph*.
    #[qslot(qml_name = "showError")]
    fn show_error(&mut self) {
        if !self.error_message.is_empty() && !self.showing_error {
            self.showing_error = true;
            self.view_changed();
        }
    }

    // realises FR-082
    /// Presents the *network graph* instead of the *error message*.
    #[qslot(qml_name = "showGraph")]
    fn show_graph(&mut self) {
        if self.showing_error {
            self.showing_error = false;
            self.view_changed();
        }
    }

    // realises FR-085, FR-086, FR-087, FR-088, FR-102, FR-103, FR-104, IR-024
    /// Opens the *node* whose vertex lies at a point of the image, as `open_node` does; does
    /// nothing for a file, and beside every vertex.
    ///
    /// # Arguments
    /// * `horizontal` - the distance from the left edge, as a fraction of the width of the image
    /// * `vertical` - the distance from the top edge, as a fraction of the height of the image
    #[qslot(qml_name = "openNodeAt")]
    fn open_node_at(&mut self, horizontal: f64, vertical: f64) {
        let index = self
            .layout
            .vertex_at(horizontal, vertical)
            .and_then(node_index_of_vertex);
        if let Some(index) = index {
            self.open_node(index);
        }
    }

    // realises FR-235, IR-036
    /// Opens the *node* of the name `name`, as `open_node` does: the *node* an *origin* names,
    /// whose icon the user activated; does nothing where the *network graph* holds no *node*
    /// of that name.
    #[qslot(qml_name = "openNodeNamed")]
    fn open_node_named(&mut self, name: String) {
        if let Some(index) = self.nodes.iter().position(|node| node.name == name) {
            self.open_node(index);
        }
    }

    // realises FR-259, FR-260
    /// Transmits an *activate request* to the *build system*: hands it to the build thread while
    /// a *network mode* is kept, the network is not an *active network* and no *request* of the
    /// switch awaits its *response*; does nothing otherwise, the switch being disabled or `act`
    /// highlighted then. In an *invalid network* the *build system* refuses the *request*, and
    /// its error response becomes the *error message*.
    #[qslot(qml_name = "activate")]
    fn activate(&mut self) {
        if !self.switchable() || self.states.is_active() {
            return;
        }
        self.activations_pending += 1;
        self.send(Command::Activate);
        self.mode_changed();
    }

    // realises FR-259, FR-267
    /// Transmits a *deactivate request* to the *build system*: hands it to the build thread while
    /// the network is an *active network* and no *request* of the switch awaits its *response*;
    /// does nothing otherwise, the switch being disabled or `ina` highlighted then.
    #[qslot(qml_name = "deactivate")]
    fn deactivate(&mut self) {
        if !self.switchable() || !self.states.is_active() {
            return;
        }
        self.deactivations_pending += 1;
        self.send(Command::Deactivate);
        self.mode_changed();
    }

    // realises FR-109, FR-241, FR-252
    /// Asks for the *store request* that follows a change the *node* answered without an error:
    /// hands it to the build thread where the *node states* count no error and the network is
    /// active, and holds it back otherwise, which `storeHeld` of the *node editor* is told;
    /// scheduled by the *node editor*.
    #[qslot(qml_name = "store")]
    fn store(&mut self) {
        if self.gate.request(&self.states) {
            self.send(Command::Store { deferred: false });
        } else if let Some(node) = &self.node {
            invoke_method!(node, "storeHeld");
        }
    }

    // realises FR-109, FR-241, FR-252
    /// Gives up the *store request* held back, a later change being answered with an error;
    /// scheduled by the *node editor*.
    #[qslot(qml_name = "cancelStore")]
    fn cancel_store(&mut self) {
        self.gate.cancel();
    }

    // realises FR-068
    /// Shuts the *build system* down and ends the build thread; called when *product* terminates.
    #[qslot(qml_name = "shutDown")]
    fn shut_down(&mut self) {
        let Some(commands) = self.commands.take() else {
            return;
        };
        let _ = commands.send(Command::Stop);
        if let Some(reports) = self.reports.take() {
            while let Ok(report) = reports.recv_timeout(SHUTDOWN_WAIT) {
                if matches!(report, Report::Stopped) {
                    break;
                }
            }
        }
    }

    // realises FR-067
    /// Takes the path of the *network file* from its *input group*, and restarts the
    /// *build system* on it; scheduled by that *input group*.
    #[qslot(qml_name = "setNetworkPath")]
    fn set_network_path(&mut self, network_path: String) {
        if network_path == self.network_path {
            return;
        }
        self.network_path = network_path;
        self.restart();
    }

    // realises FR-069, FR-070
    /// Takes a *text increment* of the code editor of the *network file*, carries it into the
    /// *provided text*, and hands the *build* to the build thread; scheduled by the *input group*.
    ///
    /// * `_document` is passed over: the *document identifier* of the *network file* is its file
    ///   name, which the build session names (IR-019).
    #[qslot(qml_name = "applyIncrement")]
    fn apply_increment(&mut self, _document: String, position: i32, range: i32, text: String) {
        let increment = TextIncrement {
            position: usize::try_from(position).unwrap_or(0),
            range: usize::try_from(range).unwrap_or(0),
            text,
        };
        let provided_after = increment.overwriting(&self.provided);
        let provided_before = std::mem::replace(&mut self.provided, provided_after.clone());
        self.send(Command::Build {
            provided_before,
            increment,
            provided_after,
        });
        self.count_build();
    }

    // realises FR-083
    /// Presents the *error message* where the last *build* failed; scheduled by the *input group*
    /// when the text was not modified for the *long idle time*.
    #[qslot(qml_name = "longIdleExpired")]
    fn long_idle_expired(&mut self) {
        self.show_error();
    }

    // realises FR-078, FR-079, FR-109, FR-128, FR-229, FR-241, FR-245, FR-252, FR-258, FR-259,
    // FR-261, FR-263, FR-268
    /// Takes the reports of the build thread and of the reader thread of the connection to the
    /// *build system*; scheduled by those threads on the main thread.
    ///
    /// * The *diagnostics* of a *build* are marked in the *input group* of the *network file*,
    ///   at their offsets in the *provided text*.
    /// * The *node states* and the *network mode* of a *state response* are kept in place of
    ///   those kept before, and the *store request* held back is handed to the build thread
    ///   where they count no error and the network is active; where the network ceases to be
    ///   active, `network_deactivated` is emitted, on which the *front end* closes the
    ///   *node window*.
    /// * The error response to an *activate request* or to a *deactivate request*, or the
    ///   failure of its connection, becomes the *error message*.
    /// * The answer to a *store request* is scheduled as `storeAnswered` of the *node editor*.
    #[qslot(qml_name = "reportArrived")]
    fn report_arrived(&mut self) {
        let reports: Vec<Report> = self
            .reports
            .as_ref()
            .map(|reports| reports.try_iter().collect())
            .unwrap_or_default();
        for report in reports {
            match report {
                Report::Built(outcome) => {
                    self.builds_pending = self.builds_pending.saturating_sub(1);
                    if self.builds_pending == 0 {
                        self.building_changed();
                    }
                    if let Some(started) = self.builds_started.pop_front() {
                        self.processing_measured(milliseconds_since(started));
                    }
                    if let Some(network_input) = &self.network_input {
                        let marks = marks_of_diagnostics(&self.provided, &outcome.diagnostics);
                        invoke_method!(
                            network_input,
                            "setDiagnostics",
                            marks.starts,
                            marks.ends,
                            marks.severities,
                            marks.texts
                        );
                    }
                    self.take_outcome(outcome.graph);
                }
                Report::NotStarted(message) => self.take_outcome(Err(message)),
                Report::Stored { outcome, deferred } => {
                    if let Some(node) = &self.node {
                        invoke_method!(
                            node,
                            "storeAnswered",
                            outcome.is_ok(),
                            outcome.err().unwrap_or_default(),
                            deferred
                        );
                    }
                }
                Report::Activated(outcome) => {
                    self.activations_pending = self.activations_pending.saturating_sub(1);
                    self.mode_changed();
                    if let Err(message) = outcome {
                        self.take_outcome(Err(message));
                    }
                }
                Report::Deactivated(outcome) => {
                    self.deactivations_pending = self.deactivations_pending.saturating_sub(1);
                    self.mode_changed();
                    if let Err(message) = outcome {
                        self.take_outcome(Err(message));
                    }
                }
                Report::Watched(states, mode) => {
                    let was_active = self.states.is_active();
                    self.states.take(states, mode);
                    if self.gate.states_changed(&self.states) {
                        self.send(Command::Store { deferred: true });
                    }
                    self.mode_changed();
                    if was_active != self.states.is_active() {
                        // the image of the other mode is presented (FR-275)
                        self.view_changed();
                    }
                    if was_active && !self.states.is_active() {
                        self.network_deactivated();
                    }
                }
                Report::Lost => self.forget_states(),
                Report::Stopped => {}
            }
        }
    }
}

impl NetworkEditor {
    /// Wires the group: the settings, the invoker of the *node editor*, and the invoker of the
    /// *input group* of the *network file*.
    ///
    /// * Called by the *workbench object* once, before the *front end* binds to the group.
    pub fn configure(
        &mut self,
        settings: Rc<RefCell<SettingsStore>>,
        node: QmlMethodInvoker,
        network_input: QmlMethodInvoker,
    ) {
        self.settings = Some(settings);
        self.node = Some(node);
        self.network_input = Some(network_input);
    }

    // realises FR-090
    /// Names the path the settings hold for the *build system*.
    pub fn load_initial(&mut self) {
        let path = self
            .settings
            .as_ref()
            .map(|settings| settings.borrow().build_system_path().to_owned())
            .unwrap_or_default();
        self.set_build_system_path(path);
    }

    // realises FR-085, FR-086, FR-087, FR-088, FR-102, FR-103, FR-104, FR-238, FR-262
    /// Opens the *node* of index `index` of the *network graph*: schedules `openNode` of the
    /// *node editor* with the name, the directory of the *network file*, the path of the
    /// *service socket*, the *document identifiers* of the files and whether each *input* has a
    /// *producer*, and emits `node_opened`; does nothing for a *proxy node*, for an index
    /// that names no *node*, and while the network is not an *active network*, since no *node*
    /// runs then, the status bar showing the *network mode*.
    fn open_node(&mut self, index: usize) {
        if !self.states.is_active() {
            return;
        }
        let Some(node) = self.nodes.get(index) else {
            return;
        };
        let Some(files) = files_of_node(node, &self.nodes) else {
            return;
        };
        let socket = node.socket.clone();
        let directory = Path::new(&self.network_path)
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .to_string_lossy()
            .into_owned();
        let identifiers = |files: &[crate::core::network_graph::NodeFile]| {
            files
                .iter()
                .map(|file| file.identifier.clone())
                .collect::<Vec<String>>()
        };
        let produced: Vec<bool> = files
            .inputs
            .iter()
            .map(|file| !file.producer.is_empty())
            .collect();
        if let Some(node) = &self.node {
            invoke_method!(
                node,
                "openNode",
                files.name.clone(),
                directory,
                socket,
                files.meta_dsl.identifier.clone(),
                !files.meta_dsl.producer.is_empty(),
                identifiers(&files.inputs),
                produced,
                identifiers(&files.outputs)
            );
        }
        self.node_opened(files.name);
    }

    // realises FR-078, FR-079, FR-245
    /// Presents the *network graph* of a successful *build* and hides both buttons, or keeps what
    /// is presented and holds the *error message* of a failed one.
    fn take_outcome(&mut self, outcome: Result<BuiltGraph, String>) {
        match outcome {
            Ok(graph) => {
                self.active_graph_source = format!("file://{}", graph.active_image_path.display());
                self.inactive_graph_source =
                    format!("file://{}", graph.inactive_image_path.display());
                self.layout = graph.layout;
                self.nodes = graph.nodes;
                self.error_message.clear();
                self.showing_error = false;
            }
            Err(message) => self.error_message = message,
        }
        self.view_changed();
    }

    // realises FR-067, FR-229, FR-263, FR-275
    /// Forgets the *node states* and the *network mode*, the *build system* that told them
    /// running no more, and gives up the *store request* held back; emits `mode_changed`, and,
    /// where the network was active, `view_changed`, the grey image being presented, and
    /// `network_deactivated`.
    fn forget_states(&mut self) {
        let was_active = self.states.is_active();
        self.states.clear();
        self.gate.cancel();
        self.mode_changed();
        if was_active {
            self.view_changed();
            self.network_deactivated();
        }
    }

    // realises FR-067, FR-070, FR-229, FR-263
    /// Hands the paths to the build thread, which restarts the *build system*, and, where a
    /// *provided text* is held and both paths are named, a *build* without a change, which opens
    /// the *network file* with the *provided text*; the *node states* and the *network mode* of
    /// the *build system* before are forgotten, and so is the *store request* held back.
    fn restart(&mut self) {
        self.forget_states();
        self.send(Command::Restart {
            executable: self.build_system_path.clone(),
            network_path: self.network_path.clone(),
        });
        if self.provided.is_empty()
            || self.build_system_path.is_empty()
            || self.network_path.is_empty()
        {
            return;
        }
        self.send(Command::Build {
            provided_before: self.provided.clone(),
            increment: TextIncrement::default(),
            provided_after: self.provided.clone(),
        });
        self.count_build();
    }

    /// Counts a *build* handed to the build thread, records when, and emits `building_changed`
    /// for the first.
    fn count_build(&mut self) {
        self.builds_started.push_back(Instant::now());
        self.builds_pending += 1;
        if self.builds_pending == 1 {
            self.building_changed();
        }
    }

    /// Hands `command` to the build thread, which is started with the first command.
    fn send(&mut self, command: Command) {
        if self.commands.is_none() {
            let (commands, received_commands) = channel::<Command>();
            let (reports, received_reports) = channel::<Report>();
            let invoker = self.get_qml_method_invoker();
            std::thread::spawn(move || build_thread(received_commands, reports, invoker));
            self.commands = Some(commands);
            self.reports = Some(received_reports);
        }
        if let Some(commands) = &self.commands {
            let _ = commands.send(command);
        }
    }
}

// realises FR-129
/// Yields the milliseconds since `started`, as the bridge carries them.
fn milliseconds_since(started: Instant) -> i32 {
    i32::try_from(started.elapsed().as_millis()).unwrap_or(i32::MAX)
}

/// Schedules `reportArrived` of the network editor on the main thread.
fn schedule_report(invoker: &Arc<Mutex<QmlMethodInvoker>>) {
    if let Ok(invoker) = invoker.lock() {
        invoke_method!(invoker, "reportArrived");
    }
}

// realises FR-229
/// Yields what yields the sink of the watch of each *build system* started: the sink is called
/// on the reader thread of the connection to the *build system*, reports the *node states* of
/// every *state response* and the loss of the connection through `reports`, and schedules
/// `reportArrived` on the main thread.
fn sinks_of(reports: Sender<Report>, invoker: Arc<Mutex<QmlMethodInvoker>>) -> StateSinks {
    Box::new(move || {
        let reports = reports.clone();
        let invoker = Arc::clone(&invoker);
        Box::new(move |told: Result<Watched, BuildError>| {
            let report = match told {
                Ok(watched) => Report::Watched(watched.states, watched.mode),
                Err(_) => Report::Lost,
            };
            if reports.send(report).is_ok() {
                schedule_report(&invoker);
            }
        })
    })
}

/// Runs the build thread: carries out every command with the [`NetworkBuilder`] it owns, reports
/// through `reports`, and schedules `reportArrived` on the main thread after every report.
fn build_thread(commands: Receiver<Command>, reports: Sender<Report>, invoker: QmlMethodInvoker) {
    let invoker = Arc::new(Mutex::new(invoker));
    let mut builder = NetworkBuilder::new(
        NetworkBuilder::default_directory(),
        sinks_of(reports.clone(), Arc::clone(&invoker)),
    );
    while let Ok(command) = commands.recv() {
        let report = match command {
            Command::Restart {
                executable,
                network_path,
            } => match builder.restart(&executable, &network_path) {
                Ok(()) => None,
                Err(error) => Some(Report::NotStarted(error.to_string())),
            },
            Command::Build {
                provided_before,
                increment,
                provided_after,
            } => Some(Report::Built(builder.build(
                &provided_before,
                &increment,
                &provided_after,
            ))),
            Command::Store { deferred } => Some(Report::Stored {
                outcome: builder.store(),
                deferred,
            }),
            Command::Activate => Some(Report::Activated(builder.activate())),
            Command::Deactivate => Some(Report::Deactivated(builder.deactivate())),
            Command::Stop => break,
        };
        if let Some(report) = report {
            if reports.send(report).is_err() {
                break;
            }
            schedule_report(&invoker);
        }
    }
    drop(builder);
    let _ = reports.send(Report::Stopped);
}
