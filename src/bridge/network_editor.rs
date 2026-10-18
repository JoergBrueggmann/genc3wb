//! The *bridged type* `NetworkEditor`: the group for the *network graph* as the *front end* binds to it.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::bridge::document_registry::DocumentRegistry;
use crate::bridge::input_group::InputGroup;
use crate::bridge::node_editor::NodeEditor;
use crate::core::api_message::{NetworkMode, NodeDescription, NodeState, marks_of_diagnostics};
use crate::core::build_system::{BuildError, BuildSession, Watched};
use crate::core::executable;
use crate::core::input_file::{InputKind, MissingFile, create_files, missing_files};
use crate::core::network_builder::{BuildOutcome, BuiltGraph, NetworkBuilder, StateSinks};
use crate::core::network_graph::{
    FileRole, GraphFile, GraphLayout, file_index_of_vertex, files_of_node, node_index_of_vertex,
    nodes_of_file, path_of_identifier,
};
use crate::core::node_state::{NodeStates, StoreGate};
use crate::core::output::file_octets;
use crate::core::settings::SettingsStore;
use crate::core::text_increment::TextIncrement;

use qtbridge::{QmlMethodInvoker, QmlObject, invoke_method, qobject};

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
    /// transmit a *store request* to the *build system* (FR-109) for the *node editors* named
    /// by `requesters`; `deferred` says that it was held back first (FR-241)
    Store {
        deferred: bool,
        requesters: Vec<String>,
    },
    /// transmit an *activate request* to the *build system* (FR-260)
    Activate,
    /// transmit a *deactivate request* to the *build system* (FR-267)
    Deactivate,
    /// transmit a *hold request* naming the *node* to the *build system* (FR-281)
    Hold { name: String },
    /// transmit a *release request* naming the *node* to the *build system* (FR-282)
    Release { name: String },
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
    /// the outcome of a *store request*, the text of FR-112 where it failed, for the
    /// *node editors* named by `requesters`
    Stored {
        outcome: Result<(), String>,
        deferred: bool,
        requesters: Vec<String>,
    },
    /// the outcome of an *activate request*, the *error message* of FR-261 where it failed
    Activated(Result<(), String>),
    /// the outcome of a *deactivate request*, the *error message* of FR-268 where it failed
    Deactivated(Result<(), String>),
    /// the outcome of the *hold request* naming the *node*, the *error message* of FR-286
    /// where it failed
    Held {
        name: String,
        outcome: Result<(), String>,
    },
    /// the outcome of a *release request*, the *error message* of FR-287 where it failed
    Released(Result<(), String>),
    /// the *node states* and the *network mode* of a *state response* of the *build system*,
    /// as the reader thread received it (FR-229)
    Watched(Vec<NodeState>, Option<NetworkMode>),
    /// the connection to the *build system* was lost
    Lost,
    /// the build thread ended
    Stopped,
}

// realises FR-001, FR-085, FR-296, FR-297
/// One *node editor* of the network editor with its invoker, the name of its *node*, empty while
/// the editor is free, and whether a *node window* is presented for it; the editor is named
/// while a *node window* or an *editor window* holds the *node*.
struct NodeSlot {
    /// the name of the *node* the editor serves, empty where the editor is free
    name: String,
    /// whether a *node window* is bound to the editor; false for a *node* that *editor windows*
    /// alone hold
    window: bool,
    /// the *node editor*
    editor: Rc<RefCell<NodeEditor>>,
    /// its invoker, by which the network editor reaches it
    invoker: QmlMethodInvoker,
}

// realises FR-290 to FR-298
/// One *input group* of an *editor window* with its invoker, the path of its file, empty while no
/// *editor window* is bound to it, the name of the *producer* of a file a *node* produces, empty
/// for a *source input*, and the names of the *nodes* the window holds.
struct FileSlot {
    /// the path of the file the group presents, resolved; empty where the group is free
    path: String,
    /// the name of the *producer* of a *sink output* or of the file of a *connection*, empty for
    /// a *source input*
    producer: String,
    /// the names of the *nodes* the *editor window* holds, empty while the network serves none
    nodes: Vec<String>,
    /// the *input group*
    group: Rc<RefCell<InputGroup>>,
    /// its invoker, by which the network editor reaches it
    invoker: QmlMethodInvoker,
}

// realises FR-281, FR-285, FR-294, FR-295
/// A *hold request* that awaits its *response*: the name of the *node* and whether a
/// *node window* is wanted for it, which a double-click on the *node* sets while the request of
/// an *editor window* is pending.
struct PendingHold {
    /// the name of the *node*
    name: String,
    /// whether a *node window* is to be presented once the hold is acknowledged
    window: bool,
}

// realises FR-487 to FR-491
/// An action of the user that waits for the answer of a dialog on the files of its *nodes*: the
/// opening of a *node* for a *node window*, the opening of a file for an *editor window*, or the
/// activation of the network.
#[derive(Debug, Clone, PartialEq, Eq)]
enum PendingAction {
    /// a *node window* of the *node* of that name
    Node { name: String },
    /// an *editor window* on the file of that *document identifier*
    File { identifier: String },
    /// the *activate request* of the *activation switch*
    Activation,
}

// realises FR-487 to FR-491
/// What an action may do about the files of its *nodes* that do not exist.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CreationPolicy {
    /// ask the user before the files are created, unless the *silent creation setting* is on
    Ask,
    /// create the files, the user having confirmed it
    Create,
    /// create nothing: the user chose to continue without the files
    Skip,
}

// realises FR-001, FR-064, FR-085, FR-109, FR-229, FR-241, FR-252, FR-258, FR-259, FR-269,
// FR-270, FR-273, FR-274, FR-281, FR-285, FR-290 to FR-293, FR-487 to FR-491
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
    /// the URL of the image of the *network graph* of a network that serves *nodes*, a
    /// *valid network* or an *active network*, empty before the first successful *build* (FR-274)
    served_graph_source: String,
    /// the URL of the image of the *network graph* of an *invalid network*, empty before the
    /// first successful *build* (FR-273)
    invalid_graph_source: String,
    /// the layout of that image
    layout: GraphLayout,
    /// the *nodes* of that image
    nodes: Vec<NodeDescription>,
    /// the files of that image, in the order of the vertices `f<index>` (FR-289 to FR-291)
    files: Vec<GraphFile>,
    /// the *input groups* of the *editor windows*: one per *editor window* presented, named by
    /// the path of its file and holding the *nodes* that have the file, and those whose window
    /// closed, unnamed, kept for the next file opened (FR-290 to FR-297)
    file_groups: Vec<FileSlot>,
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
    /// the *nodes* whose *hold request* awaits its *response*, in the order of the requests,
    /// each with whether a *node window* is wanted for it (FR-281, FR-285, FR-294, FR-295)
    holds_pending: Vec<PendingHold>,
    /// the action that waits for the answer of the *input creation confirmation dialog* or of
    /// the dialog of a failed creation, with the files to be created; `None` where none does
    /// (FR-488 to FR-491)
    creation_pending: Option<(PendingAction, Vec<MissingFile>)>,
    /// the *node editors*: one per *node* held, by a *node window* or by *editor windows*,
    /// named by its *node*, and those whose *node* was released, unnamed, kept for the next
    /// *node* held (FR-001, FR-085, FR-296)
    editors: Vec<NodeSlot>,
    /// the commands to the build thread, `None` before the first one and after `shut_down`
    commands: Option<Sender<Command>>,
    /// the reports of the build thread
    reports: Option<Receiver<Report>>,
    /// the settings, to store the path; wired by the *workbench object*
    settings: Option<Rc<RefCell<SettingsStore>>>,
    /// the *document registry* handed to every *node editor*; wired by the *workbench object*
    documents: Option<Rc<RefCell<DocumentRegistry>>>,
    /// the invoker of the *input group* of the *network file*, which receives the marks of the
    /// *diagnostics* of a *build*
    network_input: Option<QmlMethodInvoker>,
}

// realises FR-064 to FR-069, FR-078 to FR-083, FR-085 to FR-089, FR-091, FR-102 to FR-105,
// FR-109, FR-128, FR-228, FR-229, FR-235, FR-241, FR-245, FR-252, FR-258 to FR-263, FR-267 to
// FR-270, FR-273 to FR-275, FR-281 to FR-287, FR-289 to FR-299
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
    qproperty!("opening", Read = opening, Notify = mode_changed);

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
    // The image of the network mode kept: the served one while the network is a valid network
    // or an active network, the invalid one otherwise.
    fn graph_source(&self) -> String {
        if self.states.serves_nodes() {
            self.served_graph_source.clone()
        } else {
            self.invalid_graph_source.clone()
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
        self.showing_error && !self.served_graph_source.is_empty()
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

    // realises FR-285
    // The names of the nodes whose hold request awaits its response, joined by ", ".
    fn opening(&self) -> String {
        self.holds_pending
            .iter()
            .map(|hold| hold.name.as_str())
            .collect::<Vec<&str>>()
            .join(", ")
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
            drop(settings);
            crate::bridge::workbench::settings_stored();
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

    #[qsignal(qml_name = "nodeOpened")]
    fn node_opened(&mut self, name: String);

    #[qsignal(qml_name = "nodeClosed")]
    fn node_closed(&mut self, name: String);

    #[qsignal(qml_name = "fileOpened")]
    fn file_opened(&mut self, path: String);

    #[qsignal(qml_name = "fileClosed")]
    fn file_closed(&mut self, path: String);

    #[qsignal(qml_name = "processingMeasured")]
    fn processing_measured(&mut self, processing_time: i32);

    // files of the nodes about to be opened or activated do not exist: the front end presents
    // the input creation confirmation dialog naming them, the meta compiler DSLs and the inputs
    // each joined by line feeds, and answers by `answerCreation` (FR-487, FR-488)
    #[qsignal(qml_name = "creationAsked")]
    fn creation_asked(&mut self, meta_dsls: String, inputs: String);

    // a folder or a file could not be created: the front end tells the user and answers by
    // `answerCreationFailure` (FR-491)
    #[qsignal(qml_name = "creationFailed")]
    fn creation_failed(&mut self, message: String);

    // slots
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

    // realises FR-085, FR-086, FR-087, FR-088, FR-102, FR-103, FR-104, FR-290, FR-291, FR-298,
    // IR-024
    /// Opens what the vertex at a point of the image names: the *node*, as `open_node` does, or
    /// the file, as `open_file` does; does nothing beside every vertex.
    ///
    /// # Arguments
    /// * `horizontal` - the distance from the left edge, as a fraction of the width of the image
    /// * `vertical` - the distance from the top edge, as a fraction of the height of the image
    #[qslot(qml_name = "openNodeAt")]
    fn open_node_at(&mut self, horizontal: f64, vertical: f64) {
        let Some(vertex) = self
            .layout
            .vertex_at(horizontal, vertical)
            .map(str::to_owned)
        else {
            return;
        };
        if let Some(index) = node_index_of_vertex(&vertex) {
            self.open_node(index);
        } else if let Some(index) = file_index_of_vertex(&vertex) {
            self.open_file(index);
        }
    }

    // realises FR-045, FR-292, FR-299
    /// Opens the file whose resolved path is `path` in an *editor window*, as `open_file` does
    /// for the file of a vertex, so that the window is presented or raised; does nothing for a
    /// path that is no file of the *network graph*. Called by the *front end* from the detach
    /// button of an *input group* and of an *output page*.
    #[qslot(qml_name = "openFileNamed")]
    fn open_file_named(&mut self, path: String) {
        let directory = self.network_directory();
        if let Some(index) = self
            .files
            .iter()
            .position(|file| path_of_identifier(Path::new(&directory), &file.identifier) == path)
        {
            self.open_file(index);
        }
    }

    // realises FR-290, FR-291
    /// Yields the *input group* named `path`, the one of the *editor window* of that file; a free
    /// one for a path none is named by, so that no binding fails. Called by the *front end* when
    /// it creates the *editor window* of the path.
    #[qslot(qml_name = "fileNamed")]
    fn file_named(&self, path: String) -> Rc<RefCell<InputGroup>> {
        let slot = self
            .file_groups
            .iter()
            .find(|slot| slot.path == path)
            .or_else(|| self.file_groups.iter().find(|slot| slot.path.is_empty()))
            .or_else(|| self.file_groups.first())
            .expect("the network editor is configured before the front end reaches it");
        Rc::clone(&slot.group)
    }

    // realises FR-293, FR-297
    /// Takes that the user closed the *editor window* of the file `path`: releases the
    /// *shared document* of its *input group*, unnames the group, so that the next file opened
    /// reuses it, releases every *node* the window held that no *node window* and no other
    /// *editor window* holds, and emits `file_closed`; does nothing where no *input group* is
    /// named `path`. Called by the *front end*.
    #[qslot(qml_name = "closeFile")]
    fn close_file(&mut self, path: String) {
        let Some(slot) = self.file_groups.iter_mut().find(|slot| slot.path == path) else {
            return;
        };
        slot.path.clear();
        slot.producer.clear();
        let held = std::mem::take(&mut slot.nodes);
        slot.group.borrow_mut().release_document();
        self.release_unheld(held);
        self.file_closed(path);
    }

    // realises FR-291
    /// Takes the content a *node editor* holds of its *output* at `path`: schedules `setContent`
    /// of the *input group* named `path`, where one is, so that the *editor window* of the
    /// *sink output* shows what the *producer* holds; scheduled by a *node editor*.
    #[qslot(qml_name = "outputServed")]
    fn output_served(&mut self, path: String, octets: Vec<u8>) {
        if let Some(slot) = self.file_groups.iter().find(|slot| slot.path == path) {
            invoke_method!(&slot.invoker, "setContent", octets);
        }
    }

    // realises FR-296
    /// Takes the marks of the *diagnostics* a *node editor* set in its *input group* of the
    /// document at `path`, in its text: schedules `setDiagnostics` of the *input group* named
    /// `path`, where one is, so that the *editor window* on the file shows them; scheduled by
    /// a *node editor*.
    #[qslot(qml_name = "markFile")]
    fn mark_file(
        &mut self,
        path: String,
        starts: Vec<i32>,
        ends: Vec<i32>,
        severities: Vec<i32>,
        texts: Vec<String>,
    ) {
        if let Some(slot) = self.file_groups.iter().find(|slot| slot.path == path) {
            invoke_method!(
                &slot.invoker,
                "setDiagnostics",
                starts,
                ends,
                severities,
                texts
            );
        }
    }

    // realises FR-296
    /// Takes the marks of the *diagnostics* a *node editor* set in its *input group* of the
    /// document at `path`, in its bits: schedules `setOctetMarks` of the *input group* named
    /// `path`, where one is; scheduled by a *node editor*.
    #[qslot(qml_name = "markFileOctets")]
    fn mark_file_octets(
        &mut self,
        path: String,
        starts: Vec<i32>,
        ends: Vec<i32>,
        severities: Vec<i32>,
        texts: Vec<String>,
    ) {
        if let Some(slot) = self.file_groups.iter().find(|slot| slot.path == path) {
            invoke_method!(
                &slot.invoker,
                "setOctetMarks",
                starts,
                ends,
                severities,
                texts
            );
        }
    }

    // realises FR-291
    /// Takes that a *node editor* ceased to serve its *output* at `path`: schedules `setContent`
    /// of the *input group* named `path`, where one is, with the octets of the file; scheduled
    /// by a *node editor*.
    #[qslot(qml_name = "outputUnserved")]
    fn output_unserved(&mut self, path: String) {
        if let Some(slot) = self.file_groups.iter().find(|slot| slot.path == path) {
            invoke_method!(&slot.invoker, "setContent", file_octets(&path));
        }
    }

    // realises FR-489
    /// Answers the *input creation confirmation dialog* of `creation_asked`: where `create`,
    /// switches the *silent creation setting* on and stores it where `silent`, creates the files
    /// and carries out the pending action; drops the pending action otherwise, so that no
    /// *hold request* and no *activate request* is transmitted and no window presented.
    #[qslot(qml_name = "answerCreation")]
    fn answer_creation(&mut self, create: bool, silent: bool) {
        let Some((action, _)) = self.creation_pending.take() else {
            return;
        };
        if !create {
            return;
        }
        if let (true, Some(settings)) = (silent, &self.settings) {
            let mut settings = settings.borrow_mut();
            settings.set_silent_creation(true);
            let _ = settings.store();
            drop(settings);
            crate::bridge::workbench::settings_stored();
        }
        self.resume(action, CreationPolicy::Create);
    }

    // realises FR-491
    /// Answers the dialog of `creation_failed`: where `proceed`, carries out the pending action
    /// without the files not created, as for files that do not exist; drops the pending action
    /// otherwise.
    #[qslot(qml_name = "answerCreationFailure")]
    fn answer_creation_failure(&mut self, proceed: bool) {
        let Some((action, _)) = self.creation_pending.take() else {
            return;
        };
        if proceed {
            self.resume(action, CreationPolicy::Skip);
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

    // realises FR-001, FR-085
    /// Yields the *node editor* named `name`; a free one for a name none is named by, so that
    /// no binding fails. Called by the *front end* when it creates the *node window* of the
    /// name.
    #[qslot(qml_name = "nodeNamed")]
    fn node_named(&self, name: String) -> Rc<RefCell<NodeEditor>> {
        let slot = self
            .editors
            .iter()
            .find(|slot| slot.name == name)
            .or_else(|| self.editors.iter().find(|slot| slot.name.is_empty()))
            .or_else(|| self.editors.first())
            .expect("the network editor is configured before the front end reaches it");
        Rc::clone(&slot.editor)
    }

    // realises FR-089, FR-105, FR-282, FR-297
    /// Takes that the user closed the *node window* of the *node* `name`: marks its
    /// *node editor* as having no window and emits `node_closed`; where no *editor window*
    /// holds the *node*, schedules `close` of the *node editor*, which closes the connection to
    /// the *node* and releases its *shared documents*, transmits a *release request* where the
    /// network serves *nodes*, and unnames the *node editor*, so that the next *node* held
    /// reuses it; where an *editor window* holds the *node*, the editor serves on without a
    /// window. Does nothing where no *node editor* is named `name`. Called by the *front end*.
    #[qslot(qml_name = "closeNode")]
    fn close_node(&mut self, name: String) {
        let Some(slot) = self.editors.iter_mut().find(|slot| slot.name == name) else {
            return;
        };
        slot.window = false;
        self.release_unheld(vec![name.clone()]);
        self.node_closed(name);
    }

    // realises FR-259, FR-260, FR-487
    /// Transmits an *activate request* to the *build system*: hands it to the build thread while
    /// a *network mode* is kept, the network is not an *active network* and no *request* of the
    /// switch awaits its *response*, the files of the *nodes* of the network prepared first
    /// (`activate_preparing`); does nothing otherwise, the switch being disabled or `act`
    /// highlighted then. In an *invalid network* the *build system* refuses the *request*, and
    /// its error response becomes the *error message*.
    #[qslot(qml_name = "activate")]
    fn activate(&mut self) {
        self.activate_preparing(CreationPolicy::Ask);
    }

    // realises FR-260, FR-487 to FR-491
    /// Transmits the *activate request* as `activate` does, the files of every *node* of the
    /// network prepared first (`prepared`) under `policy`: where the preparation waits for a
    /// dialog, nothing else happens until its answer resumes the activation.
    fn activate_preparing(&mut self, policy: CreationPolicy) {
        if !self.switchable() || self.states.is_active() {
            return;
        }
        let names: Vec<String> = self.nodes.iter().map(|node| node.name.clone()).collect();
        if !self.prepared(&names, PendingAction::Activation, policy) {
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
    /// Asks for the *store request* that follows a change the *node* of the *node editor*
    /// `requester` answered without an error: hands it to the build thread where the
    /// *node states* count no error and the network serves *nodes*, and holds it back
    /// otherwise, which `storeHeld` of that *node editor* is told; scheduled by the
    /// *node editor*.
    #[qslot(qml_name = "store")]
    fn store(&mut self, requester: String) {
        if self.gate.request(&requester, &self.states) {
            self.send(Command::Store {
                deferred: false,
                requesters: vec![requester],
            });
        } else if let Some(slot) = self.editors.iter().find(|slot| slot.name == requester) {
            invoke_method!(&slot.invoker, "storeHeld");
        }
    }

    // realises FR-109, FR-241, FR-252
    /// Gives up the *store request* held back for `requester`, a later change being answered
    /// with an error; scheduled by the *node editor*.
    #[qslot(qml_name = "cancelStore")]
    fn cancel_store(&mut self, requester: String) {
        self.gate.cancel(&requester);
    }

    // realises FR-068, FR-105
    /// Shuts every *node editor* down, then the *build system*, and ends the build thread;
    /// called when *product* terminates.
    #[qslot(qml_name = "shutDown")]
    fn shut_down(&mut self) {
        for slot in &self.editors {
            slot.editor.borrow_mut().end();
        }
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

    // realises FR-078, FR-079, FR-085, FR-109, FR-128, FR-229, FR-241, FR-245, FR-252, FR-258,
    // FR-259, FR-261, FR-263, FR-268, FR-283, FR-286, FR-287
    /// Takes the reports of the build thread and of the reader thread of the connection to the
    /// *build system*; scheduled by those threads on the main thread.
    ///
    /// * The *diagnostics* of a *build* are marked in the *input group* of the *network file*,
    ///   at their offsets in the *provided text*.
    /// * The *node states* and the *network mode* of a *state response* are kept in place of
    ///   those kept before, and the *store requests* held back are handed to the build thread
    ///   where they count no error and the network serves *nodes*; where the network comes to
    ///   be an *invalid network*, every *node editor* is closed, on which the *front end*
    ///   destroys the *node windows*; a *network deactivation* towards a *valid network* closes
    ///   nothing.
    /// * The error response to an *activate request*, a *deactivate request*, a *hold request*
    ///   or a *release request*, or the failure of its connection, becomes the *error message*.
    /// * The acknowledged response to a *hold request* names a *node editor* for the *node*,
    ///   opens it and, where a *node window* is wanted, emits `node_opened`, on which the
    ///   *front end* presents its *node window*; a *node* held for *editor windows* alone is
    ///   served without a window.
    /// * Where the network comes to serve *nodes* from not serving them, the *nodes* of every
    ///   *editor window* are held anew.
    /// * The answer to a *store request* is scheduled as `storeAnswered` of the *node editor*
    ///   of each requester.
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
                Report::Stored {
                    outcome,
                    deferred,
                    requesters,
                } => {
                    let message = outcome.clone().err().unwrap_or_default();
                    for slot in self
                        .editors
                        .iter()
                        .filter(|slot| requesters.contains(&slot.name))
                    {
                        invoke_method!(
                            &slot.invoker,
                            "storeAnswered",
                            outcome.is_ok(),
                            message.clone(),
                            deferred
                        );
                    }
                }
                Report::Held { name, outcome } => {
                    let window = self
                        .holds_pending
                        .iter()
                        .position(|pending| pending.name == name)
                        .map(|index| self.holds_pending.remove(index).window)
                        .unwrap_or(false);
                    self.mode_changed();
                    match outcome {
                        Ok(()) => self.present_node(&name, window),
                        Err(message) => self.take_outcome(Err(message)),
                    }
                }
                Report::Released(outcome) => {
                    if let Err(message) = outcome {
                        self.take_outcome(Err(message));
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
                    let was_served = self.states.serves_nodes();
                    self.states.take(states, mode);
                    let requesters = self.gate.states_changed(&self.states);
                    if !requesters.is_empty() {
                        self.send(Command::Store {
                            deferred: true,
                            requesters,
                        });
                    }
                    self.mode_changed();
                    if was_served != self.states.serves_nodes() {
                        // the image of the other mode is presented (FR-275)
                        self.view_changed();
                    }
                    if !self.states.serves_nodes() {
                        // an invalid network: the build system ended the held nodes (FR-263)
                        self.close_all_nodes();
                    } else if !was_served {
                        // the editor windows presented hold their nodes anew (FR-294, FR-295)
                        for index in 0..self.file_groups.len() {
                            self.hold_nodes_of_file(index);
                        }
                    }
                }
                Report::Lost => self.forget_states(),
                Report::Stopped => {}
            }
        }
    }
}

impl NetworkEditor {
    /// Wires the group: the settings, the *document registry*, and the invoker of the
    /// *input group* of the *network file*; creates the first *node editor*, so that
    /// `nodeNamed` has one to yield.
    ///
    /// * Called by the *workbench object* once, before the *front end* binds to the group.
    pub fn configure(
        &mut self,
        settings: Rc<RefCell<SettingsStore>>,
        documents: Rc<RefCell<DocumentRegistry>>,
        network_input: QmlMethodInvoker,
    ) {
        self.settings = Some(settings);
        self.documents = Some(documents);
        self.network_input = Some(network_input);
        self.create_editor();
        self.create_file_group();
    }

    // realises FR-290, FR-291
    /// Creates an *input group* for an *editor window*, of the kind `Input`, unnamed, and yields
    /// its index among the groups; it is configured anew with its kind when a file is opened in
    /// it.
    fn create_file_group(&mut self) -> usize {
        let group = InputGroup::default_with_attached_qobject();
        let invoker = group.borrow().get_qml_method_invoker();
        group
            .borrow_mut()
            .configure(InputKind::Input, None, None, self.documents.clone());
        self.file_groups.push(FileSlot {
            path: String::new(),
            producer: String::new(),
            nodes: Vec::new(),
            group,
            invoker,
        });
        self.file_groups.len() - 1
    }

    // realises FR-290, FR-291, FR-292, FR-294, FR-295, FR-298
    /// Opens the file of index `index` of the *network graph* in an *editor window*: emits
    /// `file_opened` where an *input group* is named by its path, so that the window is raised;
    /// names a free group otherwise, configured with the kind `MetaDsl` or `Input` for a
    /// *source input*, which edits the *shared document* of the file, and `Output` for a
    /// *sink output* and for the file of a *connection*, which shows the file and, where a
    /// *node editor* serves the *producer*, what it holds, holds the *nodes* that have the file
    /// where the network serves *nodes*, and emits `file_opened`; does nothing for an index that
    /// names no file.
    fn open_file(&mut self, index: usize) {
        self.open_file_preparing(index, CreationPolicy::Ask);
    }

    // realises FR-290, FR-294, FR-295, FR-487 to FR-491
    /// Opens the file of index `index` as `open_file` does, the files of the *nodes* the window
    /// would hold prepared first (`prepared`) under `policy`, where the network serves *nodes*:
    /// where the preparation waits for a dialog, nothing else happens until its answer resumes
    /// the opening.
    fn open_file_preparing(&mut self, index: usize, policy: CreationPolicy) {
        let Some(file) = self.files.get(index).cloned() else {
            return;
        };
        let directory = self.network_directory();
        let path = path_of_identifier(Path::new(&directory), &file.identifier);
        if self.file_groups.iter().any(|slot| slot.path == path) {
            self.file_opened(path);
            return;
        }
        if self.states.serves_nodes() {
            let holders = nodes_of_file(&file, &self.nodes);
            let action = PendingAction::File {
                identifier: file.identifier.clone(),
            };
            if !self.prepared(&holders, action, policy) {
                return;
            }
        }
        let (kind, producer) = match &file.role {
            FileRole::SourceInput { meta_dsl: true } => (InputKind::MetaDsl, String::new()),
            FileRole::SourceInput { meta_dsl: false } => (InputKind::Input, String::new()),
            FileRole::SinkOutput { producer } => (InputKind::Output, producer.clone()),
            FileRole::Connection => (
                InputKind::Output,
                nodes_of_file(&file, &self.nodes)
                    .first()
                    .cloned()
                    .unwrap_or_default(),
            ),
        };
        let index = self
            .file_groups
            .iter()
            .position(|slot| slot.path.is_empty())
            .unwrap_or_else(|| self.create_file_group());
        let produced = kind == InputKind::Output;
        {
            let slot = &mut self.file_groups[index];
            slot.path.clone_from(&path);
            slot.producer = producer;
            slot.group
                .borrow_mut()
                .configure(kind, None, None, self.documents.clone());
            invoke_method!(
                &slot.invoker,
                "nameDocument",
                file.identifier.clone(),
                path.clone(),
                produced
            );
            if produced {
                invoke_method!(&slot.invoker, "setContent", file_octets(&path));
            }
        }
        if produced {
            let producer = self.file_groups[index].producer.clone();
            if let Some(editor) = self.editors.iter().find(|slot| slot.name == producer) {
                invoke_method!(&editor.invoker, "resendOutputs");
            }
        } else {
            // the marks of a node served already, before its next diagnostics (FR-296)
            for holder in nodes_of_file(&file, &self.nodes) {
                if let Some(editor) = self.editors.iter().find(|slot| slot.name == holder) {
                    invoke_method!(&editor.invoker, "resendMarks");
                }
            }
        }
        self.hold_nodes_of_file(index);
        self.file_opened(path);
    }

    // realises FR-294, FR-295, FR-298
    /// Holds the *nodes* that have the file of the *input group* of index `index` for its
    /// *editor window*: where the group is named and the network serves *nodes*, records their
    /// names, from the graph of this moment, and holds each without a window, which transmits
    /// a *hold request* for one that is neither held nor being held; does nothing otherwise.
    fn hold_nodes_of_file(&mut self, index: usize) {
        if !self.states.serves_nodes() {
            return;
        }
        let Some(path) = self
            .file_groups
            .get(index)
            .map(|slot| slot.path.clone())
            .filter(|path| !path.is_empty())
        else {
            return;
        };
        let directory = self.network_directory();
        let Some(file) = self
            .files
            .iter()
            .find(|file| path_of_identifier(Path::new(&directory), &file.identifier) == path)
        else {
            return;
        };
        let holders = nodes_of_file(file, &self.nodes);
        self.file_groups[index].nodes.clone_from(&holders);
        for name in holders {
            self.hold(name, false);
        }
    }

    // realises FR-105, FR-282, FR-297
    /// Releases every *node* of `names` that no *editor window* holds and no *node window* is
    /// bound to: schedules `close` of its *node editor*, unnames it, and transmits a
    /// *release request* where the network serves *nodes*.
    fn release_unheld(&mut self, names: Vec<String>) {
        for name in names {
            let held_by_file = self
                .file_groups
                .iter()
                .any(|slot| !slot.path.is_empty() && slot.nodes.contains(&name));
            let held_by_window = self
                .editors
                .iter()
                .any(|slot| slot.name == name && slot.window);
            if held_by_file || held_by_window || !self.unname_editor(&name) {
                continue;
            }
            if self.states.serves_nodes() {
                self.send(Command::Release { name });
            }
        }
    }

    /// Yields the directory of the *network file*, against which the paths of the files are
    /// resolved; empty where none is named.
    fn network_directory(&self) -> String {
        Path::new(&self.network_path)
            .parent()
            .unwrap_or_else(|| Path::new(""))
            .to_string_lossy()
            .into_owned()
    }

    // realises FR-001, FR-085
    /// Creates a *node editor*, wired with the invoker of this group and the *document registry*,
    /// unnamed, and yields its index among the editors.
    fn create_editor(&mut self) -> usize {
        let editor = NodeEditor::default_with_attached_qobject();
        let invoker = editor.borrow().get_qml_method_invoker();
        if let Some(documents) = &self.documents {
            editor
                .borrow_mut()
                .configure(self.get_qml_method_invoker(), Rc::clone(documents));
        }
        self.editors.push(NodeSlot {
            name: String::new(),
            window: false,
            editor,
            invoker,
        });
        self.editors.len() - 1
    }

    // realises FR-085, FR-104, FR-296
    /// Names a free *node editor* for the *node* `name`, whose *hold request* was acknowledged,
    /// creating one where none is free, schedules its `openNode` with the files of the *node*,
    /// and, where a *node window* is wanted, emits `node_opened`; a *node* the *network graph*
    /// no longer holds, or that no window wants any more, is released again and opens nothing.
    fn present_node(&mut self, name: &str, window: bool) {
        let wanted = window
            || self
                .file_groups
                .iter()
                .any(|slot| !slot.path.is_empty() && slot.nodes.iter().any(|held| held == name));
        let Some(node) = self
            .nodes
            .iter()
            .find(|node| node.name == name)
            .filter(|_| wanted)
        else {
            self.send(Command::Release {
                name: name.to_owned(),
            });
            return;
        };
        let Some(files) = files_of_node(node, &self.nodes) else {
            return;
        };
        let socket = node.socket.clone();
        let directory = self.network_directory();
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
        let index = self
            .editors
            .iter()
            .position(|slot| slot.name.is_empty())
            .unwrap_or_else(|| self.create_editor());
        let slot = &mut self.editors[index];
        slot.name = files.name.clone();
        slot.window = window;
        invoke_method!(
            &slot.invoker,
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
        if window {
            self.node_opened(files.name);
        }
    }

    // realises FR-089, FR-105, FR-263
    /// Schedules `close` of the *node editor* named `name` and unnames it; yields whether there
    /// was one.
    fn unname_editor(&mut self, name: &str) -> bool {
        let Some(slot) = self.editors.iter_mut().find(|slot| slot.name == name) else {
            return false;
        };
        slot.name.clear();
        slot.window = false;
        invoke_method!(&slot.invoker, "close");
        true
    }

    // realises FR-105, FR-263
    /// Closes every named *node editor* and emits `node_closed` for each that has a window,
    /// without a *release request*: the network is an *invalid network*, in which the
    /// *build system* ended the *held nodes*, or the *build system* runs no more; forgets the
    /// *hold requests* pending and the *nodes* the *editor windows* hold as well.
    fn close_all_nodes(&mut self) {
        self.holds_pending.clear();
        for slot in &mut self.file_groups {
            slot.nodes.clear();
            if !slot.path.is_empty() {
                // the marks of the nodes that ended are gone with them (FR-296)
                let none: Vec<i32> = Vec::new();
                invoke_method!(
                    &slot.invoker,
                    "setDiagnostics",
                    none.clone(),
                    none.clone(),
                    none.clone(),
                    Vec::<String>::new()
                );
                invoke_method!(
                    &slot.invoker,
                    "setOctetMarks",
                    none.clone(),
                    none.clone(),
                    none,
                    Vec::<String>::new()
                );
            }
        }
        let named: Vec<(String, bool)> = self
            .editors
            .iter()
            .filter(|slot| !slot.name.is_empty())
            .map(|slot| (slot.name.clone(), slot.window))
            .collect();
        for (name, window) in named {
            self.unname_editor(&name);
            if window {
                self.node_closed(name);
            }
        }
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

    // realises FR-088, FR-262, FR-281, FR-284, FR-285, FR-296
    /// Opens the *node* of index `index` of the *network graph* for a *node window*, as `hold`
    /// with a window does: emits `node_opened` where a *node editor* of its name is named, so
    /// that its window is presented or raised; hands a *hold request* naming it to the build
    /// thread otherwise, counting the name among those opening, which the status bar tells;
    /// does nothing for a *proxy node*, for an index that names no *node*, and while the
    /// network serves no *nodes*, in an *invalid network* or without a *network mode*, the
    /// status bar showing the *network mode*.
    fn open_node(&mut self, index: usize) {
        self.open_node_preparing(index, CreationPolicy::Ask);
    }

    // realises FR-281, FR-487 to FR-491
    /// Opens the *node* of index `index` as `open_node` does, its files prepared first
    /// (`prepared`) under `policy`: where the preparation waits for a dialog, nothing else
    /// happens until its answer resumes the opening.
    fn open_node_preparing(&mut self, index: usize, policy: CreationPolicy) {
        if !self.states.serves_nodes() {
            return;
        }
        let Some(node) = self.nodes.get(index) else {
            return;
        };
        if files_of_node(node, &self.nodes).is_none() {
            return;
        }
        let name = node.name.clone();
        let action = PendingAction::Node { name: name.clone() };
        if self.prepared(std::slice::from_ref(&name), action, policy) {
            self.hold(name, true);
        }
    }

    // realises FR-487
    /// Yields the files of the *nodes* `names` that are to be created: the file of the
    /// *meta compiler DSL* and of each *input* that no *node* produces, resolved against the
    /// directory of the *network file*, that does not exist, each path once; a *node* that is
    /// held or being held contributes none, and a file an *input group* presents, in an
    /// *editor window* or in a *node editor* that is named, is left out.
    fn files_to_create(&self, names: &[String]) -> Vec<MissingFile> {
        let directory = self.network_directory();
        let resolved = |identifier: &str| path_of_identifier(Path::new(&directory), identifier);
        let files_of = |name: &str| {
            self.nodes
                .iter()
                .find(|node| node.name == name)
                .and_then(|node| files_of_node(node, &self.nodes))
        };
        let presented: Vec<String> = self
            .file_groups
            .iter()
            .filter(|slot| !slot.path.is_empty())
            .map(|slot| slot.path.clone())
            .chain(
                self.editors
                    .iter()
                    .filter(|slot| !slot.name.is_empty())
                    .filter_map(|slot| files_of(&slot.name))
                    .flat_map(|files| {
                        std::iter::once(files.meta_dsl)
                            .chain(files.inputs)
                            .map(|file| resolved(&file.identifier))
                            .collect::<Vec<String>>()
                    }),
            )
            .collect();
        let mut candidates: Vec<(String, bool)> = Vec::new();
        for name in names {
            let held = self.editors.iter().any(|slot| &slot.name == name)
                || self
                    .holds_pending
                    .iter()
                    .any(|pending| &pending.name == name);
            let Some(files) = files_of(name).filter(|_| !held) else {
                continue;
            };
            let own = std::iter::once((files.meta_dsl, true))
                .chain(files.inputs.into_iter().map(|file| (file, false)));
            for (file, meta_dsl) in own {
                if file.producer.is_empty() {
                    let path = resolved(&file.identifier);
                    if !presented.contains(&path) {
                        candidates.push((path, meta_dsl));
                    }
                }
            }
        }
        missing_files(&candidates)
    }

    // realises FR-487 to FR-491, IR-210
    /// Prepares the files of the *nodes* `names` for the action `action`: yields `true` where
    /// the action proceeds, since no file is to be created (`files_to_create`), the files were
    /// created (`create_files`), under `Create` or while the *silent creation setting* is on, or
    /// the user chose to continue without them (`Skip`); yields `false` where the action waits
    /// for the answer of a dialog, kept as `creation_pending`: `creation_asked` with the files
    /// under `Ask`, and `creation_failed` where a folder or a file could not be created.
    fn prepared(
        &mut self,
        names: &[String],
        action: PendingAction,
        policy: CreationPolicy,
    ) -> bool {
        if policy == CreationPolicy::Skip {
            return true;
        }
        let files = self.files_to_create(names);
        if files.is_empty() {
            return true;
        }
        let silent = self
            .settings
            .as_ref()
            .is_some_and(|settings| settings.borrow().silent_creation());
        if policy == CreationPolicy::Ask && !silent {
            let joined = |meta_dsl: bool| {
                files
                    .iter()
                    .filter(|file| file.meta_dsl == meta_dsl)
                    .map(|file| file.path.as_str())
                    .collect::<Vec<&str>>()
                    .join("\n")
            };
            let (meta_dsls, inputs) = (joined(true), joined(false));
            self.creation_pending = Some((action, files));
            self.creation_asked(meta_dsls, inputs);
            return false;
        }
        match create_files(&files) {
            Ok(()) => true,
            Err(failure) => {
                self.creation_pending = Some((action, files));
                self.creation_failed(format!("{}: {}", failure.path, failure.reason));
                false
            }
        }
    }

    // realises FR-489, FR-491
    /// Resumes the action `action` under `policy`: the *node* of the name or the file of the
    /// *document identifier* as the *network graph* holds it now, or the activation; nothing
    /// where the graph holds the *node* or the file no more.
    fn resume(&mut self, action: PendingAction, policy: CreationPolicy) {
        match action {
            PendingAction::Node { name } => {
                if let Some(index) = self.nodes.iter().position(|node| node.name == name) {
                    self.open_node_preparing(index, policy);
                }
            }
            PendingAction::File { identifier } => {
                if let Some(index) = self
                    .files
                    .iter()
                    .position(|file| file.identifier == identifier)
                {
                    self.open_file_preparing(index, policy);
                }
            }
            PendingAction::Activation => self.activate_preparing(policy),
        }
    }

    // realises FR-281, FR-284, FR-285, FR-294, FR-295, FR-296
    /// Holds the *node* `name` for a *node window* where `window`, and for an *editor window*
    /// otherwise: where its *node editor* is named, marks the window as wanted and emits
    /// `node_opened` for a window, and transmits nothing; where a *hold request* of the name is
    /// pending, marks the window as wanted on it; hands a *hold request* to the build thread
    /// otherwise, counting it among those pending, which the status bar tells.
    fn hold(&mut self, name: String, window: bool) {
        if let Some(slot) = self.editors.iter_mut().find(|slot| slot.name == name) {
            if window {
                slot.window = true;
                self.node_opened(name);
            }
            return;
        }
        if let Some(pending) = self
            .holds_pending
            .iter_mut()
            .find(|pending| pending.name == name)
        {
            pending.window |= window;
            return;
        }
        self.holds_pending.push(PendingHold {
            name: name.clone(),
            window,
        });
        self.send(Command::Hold { name });
        self.mode_changed();
    }

    // realises FR-078, FR-079, FR-245
    /// Presents the *network graph* of a successful *build* and hides both buttons, or keeps what
    /// is presented and holds the *error message* of a failed one.
    fn take_outcome(&mut self, outcome: Result<BuiltGraph, String>) {
        match outcome {
            Ok(graph) => {
                self.served_graph_source = format!("file://{}", graph.served_image_path.display());
                self.invalid_graph_source =
                    format!("file://{}", graph.invalid_image_path.display());
                self.layout = graph.layout;
                self.nodes = graph.nodes;
                self.files = graph.files;
                self.error_message.clear();
                self.showing_error = false;
            }
            Err(message) => self.error_message = message,
        }
        self.view_changed();
    }

    // realises FR-067, FR-229, FR-263, FR-275
    /// Forgets the *node states* and the *network mode*, the *build system* that told them
    /// running no more, gives up every *store request* held back, and closes every
    /// *node editor*, their *nodes* having run under that *build system*; emits `mode_changed`,
    /// and, where the network served *nodes*, `view_changed`, the image of an *invalid network*
    /// being presented.
    fn forget_states(&mut self) {
        let was_served = self.states.serves_nodes();
        self.states.clear();
        self.gate.clear();
        self.mode_changed();
        if was_served {
            self.view_changed();
        }
        self.close_all_nodes();
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
            Command::Store {
                deferred,
                requesters,
            } => Some(Report::Stored {
                outcome: builder.store(),
                deferred,
                requesters,
            }),
            Command::Activate => Some(Report::Activated(builder.activate())),
            Command::Deactivate => Some(Report::Deactivated(builder.deactivate())),
            Command::Hold { name } => Some(Report::Held {
                outcome: builder.hold(&name),
                name,
            }),
            Command::Release { name } => Some(Report::Released(builder.release(&name))),
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
