//! The *bridged type* `Workbench`: the *workbench object* that owns the settings and the other *bridged types*.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::bridge::input_group::InputGroup;
use crate::bridge::network_editor::NetworkEditor;
use crate::bridge::output_group::OutputGroup;
use crate::bridge::runner_group::RunnerGroup;
use crate::core::input_file::InputKind;
use crate::core::settings::Settings;

use qtbridge::{QObjectHolder, qobject};

use std::cell::RefCell;
use std::rc::Rc;

// realises FR-001, FR-063, FR-090, FR-095, FR-099
/// The *workbench object*.
pub struct Workbench {
    /// the settings, shared with the groups
    settings: Rc<RefCell<Settings>>,
    /// the *input group* of the *compiler-compiler input file*
    cc_input: Rc<RefCell<InputGroup>>,
    /// the *input group* of the *compiler input file*
    c_input: Rc<RefCell<InputGroup>>,
    /// the group of the *compiler-compiler*
    runner: Rc<RefCell<RunnerGroup>>,
    /// the output group
    output: Rc<RefCell<OutputGroup>>,
    /// the *input group* of the *network file*
    network_input: Rc<RefCell<InputGroup>>,
    /// the group for the *network graph*
    network: Rc<RefCell<NetworkEditor>>,
}

impl Default for Workbench {
    /// Restores the settings, creates the groups, and wires them: the output group first, the
    /// runner group with it, the two *input groups* of the *node window* with the runner group,
    /// the network editor with those two, and the *input group* of the *network file* with the
    /// network editor, each with its path.
    fn default() -> Self {
        let settings = Rc::new(RefCell::new(
            Settings::load(&Settings::default_path()).unwrap_or_default(),
        ));
        let output = OutputGroup::default_with_attached_qobject();
        output.borrow_mut().configure(Rc::clone(&settings));
        let runner = RunnerGroup::default_with_attached_qobject();
        runner.borrow_mut().configure(
            Rc::clone(&settings),
            output.borrow().get_qml_method_invoker(),
        );
        let cc_input = InputGroup::default_with_attached_qobject();
        cc_input.borrow_mut().configure(
            InputKind::CompilerCompilerInput,
            Rc::clone(&settings),
            Some(runner.borrow().get_qml_method_invoker()),
            None,
        );
        let c_input = InputGroup::default_with_attached_qobject();
        c_input.borrow_mut().configure(
            InputKind::CompilerInput,
            Rc::clone(&settings),
            Some(runner.borrow().get_qml_method_invoker()),
            None,
        );
        let network = NetworkEditor::default_with_attached_qobject();
        network.borrow_mut().configure(
            Rc::clone(&settings),
            cc_input.borrow().get_qml_method_invoker(),
            c_input.borrow().get_qml_method_invoker(),
        );
        let network_input = InputGroup::default_with_attached_qobject();
        network_input.borrow_mut().configure(
            InputKind::Network,
            Rc::clone(&settings),
            None,
            Some(network.borrow().get_qml_method_invoker()),
        );
        runner.borrow_mut().load_initial();
        cc_input.borrow_mut().load_initial();
        c_input.borrow_mut().load_initial();
        network.borrow_mut().load_initial();
        network_input.borrow_mut().load_initial();
        Workbench {
            settings,
            cc_input,
            c_input,
            runner,
            output,
            network_input,
            network,
        }
    }
}

// realises FR-001, FR-063
#[qobject(Singleton)]
impl Workbench {
    qproperty!("ccInput", Read = cc_input, Constant);
    qproperty!("cInput", Read = c_input, Constant);
    qproperty!("runner", Read = runner, Constant);
    qproperty!("output", Read = output, Constant);
    qproperty!("networkInput", Read = network_input, Constant);
    qproperty!("network", Read = network, Constant);
    qproperty!(
        "idleTime",
        Read = idle_time,
        Write = set_idle_time,
        Notify = times_changed
    );
    qproperty!(
        "longIdleTime",
        Read = long_idle_time,
        Write = set_long_idle_time,
        Notify = times_changed
    );

    fn cc_input(&self) -> Rc<RefCell<InputGroup>> {
        Rc::clone(&self.cc_input)
    }

    fn c_input(&self) -> Rc<RefCell<InputGroup>> {
        Rc::clone(&self.c_input)
    }

    fn runner(&self) -> Rc<RefCell<RunnerGroup>> {
        Rc::clone(&self.runner)
    }

    fn output(&self) -> Rc<RefCell<OutputGroup>> {
        Rc::clone(&self.output)
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

    // setters
    // realises FR-099
    // Sets the *idle time* of every code editor, in seconds, and stores it.
    fn set_idle_time(&mut self, idle_time: u32) {
        if idle_time.max(1) == self.settings.borrow().idle_time() {
            return;
        }
        self.settings.borrow_mut().set_idle_time(idle_time);
        self.store();
        self.times_changed();
    }

    // realises FR-099
    // Sets the *long idle time* of every code editor, in seconds, and stores it.
    fn set_long_idle_time(&mut self, long_idle_time: u32) {
        if long_idle_time.max(1) == self.settings.borrow().long_idle_time() {
            return;
        }
        self.settings
            .borrow_mut()
            .set_long_idle_time(long_idle_time);
        self.store();
        self.times_changed();
    }

    // signals
    #[qsignal(qml_name = "timesChanged")]
    fn times_changed(&mut self);
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
