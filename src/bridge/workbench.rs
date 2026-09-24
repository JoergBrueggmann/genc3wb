//! The *bridged type* `Workbench`: the *workbench object*, owner of the settings and the groups.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::bridge::input_group::InputGroup;
use crate::bridge::network_editor::NetworkEditor;
use crate::bridge::node_editor::NodeEditor;
use crate::bridge::output_group::OutputGroup;
use crate::core::input_file::InputKind;
use crate::core::settings::{Settings, tab_size_violation, times_of_processing};

use qtbridge::{QObjectHolder, qobject};

use std::cell::RefCell;
use std::rc::Rc;

// realises FR-001, FR-063, FR-090, FR-095, FR-099, FR-140, FR-141, FR-149
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

// realises FR-001, FR-063, FR-095, FR-099, FR-130 to FR-135, FR-140, FR-141, FR-146 to FR-149
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
    // the tab size in characters (FR-140, FR-142)
    qproperty!("tabSize", Read = tab_size, Notify = tab_size_changed);
    // the positions of the splitters in pixels: the size of the left or upper part (FR-149)
    qproperty!(
        "nodeLeftWidth",
        Read = node_left_width,
        Notify = splitters_changed
    );
    qproperty!(
        "nodeUpperHeight",
        Read = node_upper_height,
        Notify = splitters_changed
    );
    qproperty!(
        "networkLeftWidth",
        Read = network_left_width,
        Notify = splitters_changed
    );

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

    // realises FR-140, FR-142
    fn tab_size(&self) -> u32 {
        self.settings.borrow().tab_size()
    }

    // realises FR-146, FR-149
    fn node_left_width(&self) -> u32 {
        self.settings.borrow().node_left_width()
    }

    // realises FR-147, FR-149
    fn node_upper_height(&self) -> u32 {
        self.settings.borrow().node_upper_height()
    }

    // realises FR-148, FR-149
    fn network_left_width(&self) -> u32 {
        self.settings.borrow().network_left_width()
    }

    // signals
    #[qsignal(qml_name = "timesChanged")]
    fn times_changed(&mut self);

    #[qsignal(qml_name = "tabSizeChanged")]
    fn tab_size_changed(&mut self);

    #[qsignal(qml_name = "splittersChanged")]
    fn splitters_changed(&mut self);

    #[qsignal(qml_name = "settingsRejected")]
    fn settings_rejected(&mut self, message: String);

    // slots
    // realises FR-099, FR-100, FR-132, FR-133, FR-134, FR-135, FR-140, FR-141
    /// Sets the two times of every code editor, in milliseconds, the *automatic setting* and the
    /// *tab size*, in characters, and stores them; yields whether the values satisfy the
    /// constraints, and emits `settings_rejected` with the message naming the violated one where
    /// they do not, nothing being set then.
    ///
    /// * The *tab size* is checked first, then the two times.
    /// * With the *automatic setting* on, the times are left as they are, since the next
    ///   *processing time* sets them (FR-132).
    #[qslot(qml_name = "trySetSettings")]
    fn try_set_settings(
        &mut self,
        idle_time: i32,
        long_idle_time: i32,
        automatic: bool,
        tab_size: i32,
    ) -> bool {
        let idle_time = u32::try_from(idle_time).unwrap_or(0);
        let long_idle_time = u32::try_from(long_idle_time).unwrap_or(0);
        let tab_size = u32::try_from(tab_size).unwrap_or(0);
        if let Some(message) = tab_size_violation(tab_size) {
            self.settings_rejected(message);
            return false;
        }
        if !automatic {
            let result = self
                .settings
                .borrow_mut()
                .set_times(idle_time, long_idle_time);
            if let Err(error) = result {
                self.settings_rejected(error.to_string());
                return false;
            }
        }
        let changed_tab_size = self.tab_size() != tab_size;
        {
            let mut settings = self.settings.borrow_mut();
            settings.set_automatic(automatic);
            let _ = settings.set_tab_size(tab_size);
        }
        self.store();
        self.times_changed();
        if changed_tab_size {
            self.tab_size_changed();
        }
        true
    }

    // realises FR-146, FR-149
    /// Sets the width of the left part of the *node window*, in pixels, and stores it; called
    /// by the *front end* when the splitter is released. A negative width is taken as 0.
    #[qslot(qml_name = "setNodeLeftWidth")]
    fn set_node_left_width(&mut self, width: i32) {
        let width = u32::try_from(width).unwrap_or(0);
        if width == self.node_left_width() {
            return;
        }
        self.settings.borrow_mut().set_node_left_width(width);
        self.store();
        self.splitters_changed();
    }

    // realises FR-147, FR-149
    /// Sets the height of the *input group* in the left part of the *node window*, in pixels,
    /// and stores it; called by the *front end* when the splitter is released. A negative height
    /// is taken as 0.
    #[qslot(qml_name = "setNodeUpperHeight")]
    fn set_node_upper_height(&mut self, height: i32) {
        let height = u32::try_from(height).unwrap_or(0);
        if height == self.node_upper_height() {
            return;
        }
        self.settings.borrow_mut().set_node_upper_height(height);
        self.store();
        self.splitters_changed();
    }

    // realises FR-148, FR-149
    /// Sets the width of the *input group* of the *network file*, in pixels, and stores it;
    /// called by the *front end* when the splitter is released. A negative width is taken as 0.
    #[qslot(qml_name = "setNetworkLeftWidth")]
    fn set_network_left_width(&mut self, width: i32) {
        let width = u32::try_from(width).unwrap_or(0);
        if width == self.network_left_width() {
            return;
        }
        self.settings.borrow_mut().set_network_left_width(width);
        self.store();
        self.splitters_changed();
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
