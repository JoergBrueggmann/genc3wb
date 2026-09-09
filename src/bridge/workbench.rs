//! The *bridged type* `Workbench`: the *workbench object* that owns the settings and the other *bridged types*.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::bridge::input_group::InputGroup;
use crate::bridge::output_group::OutputGroup;
use crate::bridge::runner_group::RunnerGroup;
use crate::core::input_file::InputKind;
use crate::core::settings::Settings;

use qtbridge::{QObjectHolder, qobject};

use std::cell::RefCell;
use std::rc::Rc;

// realises FR-001, FR-048
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
}

impl Default for Workbench {
    /// Restores the settings, creates the groups, and wires them: the output group first, the
    /// runner group with it, the two *input groups* with the runner group, each with its path.
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
            runner.borrow().get_qml_method_invoker(),
        );
        let c_input = InputGroup::default_with_attached_qobject();
        c_input.borrow_mut().configure(
            InputKind::CompilerInput,
            Rc::clone(&settings),
            runner.borrow().get_qml_method_invoker(),
        );
        runner.borrow_mut().load_initial();
        cc_input.borrow_mut().load_initial();
        c_input.borrow_mut().load_initial();
        Workbench {
            settings,
            cc_input,
            c_input,
            runner,
            output,
        }
    }
}

// realises FR-001, FR-048
#[qobject(Singleton)]
impl Workbench {
    qproperty!("ccInput", Read = cc_input, Constant);
    qproperty!("cInput", Read = c_input, Constant);
    qproperty!("runner", Read = runner, Constant);
    qproperty!("output", Read = output, Constant);

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
}

impl Workbench {
    /// Yields the settings the groups share.
    pub fn settings(&self) -> Rc<RefCell<Settings>> {
        Rc::clone(&self.settings)
    }
}
