//! The *bridged type* `Workbench`: the *workbench object* that owns the settings and the other *bridged types*.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::bridge::input_group::InputGroup;
use crate::bridge::network_editor::NetworkEditor;
use crate::bridge::node_editor::NodeEditor;
use crate::bridge::output_group::OutputGroup;
use crate::core::input_file::InputKind;
use crate::core::settings::{Settings, times_of_processing};

use qtbridge::{QObjectHolder, qobject};

use std::cell::RefCell;
use std::rc::Rc;

// realises FR-001, FR-063, FR-090, FR-095, FR-099
/// The *workbench object*.
pub struct Workbench {
    /// the settings, shared with the groups
    settings: Rc<RefCell<Settings>>,
    /// the output group
    output: Rc<RefCell<OutputGroup>>,
    /// the *node editor*
    node: Rc<RefCell<NodeEditor>>,
    /// the *input group* of the *network file*
    network_input: Rc<RefCell<InputGroup>>,
    /// the group for the *network graph*
    network: Rc<RefCell<NetworkEditor>>,
}

impl Default for Workbench {
    /// Restores the settings, creates the groups, and wires them: the *node editor* with the
    /// output group, the network editor with the *node editor* and the *input group* of the
    /// *network file*, and that *input group* with the network editor, the two last with their
    /// paths.
    fn default() -> Self {
        let settings = Rc::new(RefCell::new(
            Settings::load(&Settings::default_path()).unwrap_or_default(),
        ));
        let output = OutputGroup::default_with_attached_qobject();
        let node = NodeEditor::default_with_attached_qobject();
        let network = NetworkEditor::default_with_attached_qobject();
        let network_input = InputGroup::default_with_attached_qobject();
        node.borrow_mut()
            .configure(output.borrow().get_qml_method_invoker());
        network.borrow_mut().configure(
            Rc::clone(&settings),
            node.borrow().get_qml_method_invoker(),
            network_input.borrow().get_qml_method_invoker(),
        );
        network_input.borrow_mut().configure(
            InputKind::Network,
            Some(Rc::clone(&settings)),
            Some(network.borrow().get_qml_method_invoker()),
        );
        network.borrow_mut().load_initial();
        network_input.borrow_mut().load_initial();
        Workbench {
            settings,
            output,
            node,
            network_input,
            network,
        }
    }
}

// realises FR-001, FR-063, FR-095, FR-099, FR-130 to FR-135
#[qobject(Singleton)]
impl Workbench {
    qproperty!("output", Read = output, Constant);
    qproperty!("node", Read = node, Constant);
    qproperty!("networkInput", Read = network_input, Constant);
    qproperty!("network", Read = network, Constant);
    qproperty!("idleTime", Read = idle_time, Notify = times_changed);
    qproperty!(
        "longIdleTime",
        Read = long_idle_time,
        Notify = times_changed
    );
    qproperty!("automatic", Read = automatic, Notify = times_changed);

    fn output(&self) -> Rc<RefCell<OutputGroup>> {
        Rc::clone(&self.output)
    }

    fn node(&self) -> Rc<RefCell<NodeEditor>> {
        Rc::clone(&self.node)
    }

    fn network_input(&self) -> Rc<RefCell<InputGroup>> {
        Rc::clone(&self.network_input)
    }

    fn network(&self) -> Rc<RefCell<NetworkEditor>> {
        Rc::clone(&self.network)
    }

    // realises FR-093, FR-095
    fn idle_time(&self) -> u32 {
        self.settings.borrow().idle_time()
    }

    // realises FR-093, FR-095
    fn long_idle_time(&self) -> u32 {
        self.settings.borrow().long_idle_time()
    }

    // realises FR-093, FR-095, FR-131
    fn automatic(&self) -> bool {
        self.settings.borrow().automatic()
    }

    // signals
    #[qsignal(qml_name = "timesChanged")]
    fn times_changed(&mut self);

    #[qsignal(qml_name = "timesRejected")]
    fn times_rejected(&mut self, message: String);

    // slots
    // realises FR-099, FR-100, FR-132, FR-133, FR-134, FR-135
    /// Sets the two times of every code editor, in milliseconds, and the *automatic setting*,
    /// and stores them; yields whether the times satisfy the constraints, and emits
    /// `times_rejected` with the message naming the violated one where they do not, nothing
    /// being set then.
    ///
    /// * With the *automatic setting* on, the times are left as they are, since the next
    ///   *processing time* sets them (FR-132).
    #[qslot(qml_name = "trySetTimes")]
    fn try_set_times(&mut self, idle_time: i32, long_idle_time: i32, automatic: bool) -> bool {
        let idle_time = u32::try_from(idle_time).unwrap_or(0);
        let long_idle_time = u32::try_from(long_idle_time).unwrap_or(0);
        if !automatic {
            let result = self
                .settings
                .borrow_mut()
                .set_times(idle_time, long_idle_time);
            if let Err(error) = result {
                self.times_rejected(error.to_string());
                return false;
            }
        }
        self.settings.borrow_mut().set_automatic(automatic);
        self.store();
        self.times_changed();
        true
    }

    // realises FR-129, FR-130
    /// Takes a measured *processing time* in milliseconds: with the *automatic setting* on, sets
    /// the two times to those of `times_of_processing`, stores them and emits `times_changed`;
    /// does nothing otherwise. Called by the *front end* on `processingMeasured` of the *node
    /// editor* and of the network editor.
    #[qslot(qml_name = "reportProcessingTime")]
    fn report_processing_time(&mut self, processing_time: i32) {
        if !self.automatic() {
            return;
        }
        let (idle_time, long_idle_time) =
            times_of_processing(u32::try_from(processing_time).unwrap_or(0));
        let unchanged = {
            let settings = self.settings.borrow();
            settings.idle_time() == idle_time && settings.long_idle_time() == long_idle_time
        };
        if unchanged {
            return;
        }
        let _ = self
            .settings
            .borrow_mut()
            .set_times(idle_time, long_idle_time);
        self.store();
        self.times_changed();
    }
}

impl Workbench {
    // realises IR-012
    /// Writes the *settings file*; a failure leaves the settings of the session as they are.
    fn store(&self) {
        let _ = self.settings.borrow().save(&Settings::default_path());
    }

    /// Yields the settings the groups share.
    pub fn settings(&self) -> Rc<RefCell<Settings>> {
        Rc::clone(&self.settings)
    }
}
