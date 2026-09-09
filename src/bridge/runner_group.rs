//! The *bridged type* `RunnerGroup`: the group of the *compiler-compiler*, its readiness, and the run.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::input_file::{InputKind, ProcessingState};
use crate::core::runner;
use crate::core::settings::Settings;

use qtbridge::{QObjectHolder, QmlMethodInvoker, invoke_method, qobject};

use std::cell::RefCell;
use std::rc::Rc;

// realises FR-022
/// The group of the *compiler-compiler* as the *front end* binds to it.
#[derive(Default)]
pub struct RunnerGroup {
    /// the path of the *compiler-compiler*
    path: String,
    /// whether the named file is executable
    executable: bool,
    /// whether the *compiler-compiler* can be run
    ready: bool,
    /// the *processing state* of each *input group*, indexed by `InputKind::index`
    input_states: [ProcessingState; 2],
    /// whether a run is in progress
    running: bool,
    /// the settings, to store the path; wired by the *workbench object*
    settings: Option<Rc<RefCell<Settings>>>,
    /// the invoker of the output group, to clear it and to hand the result to; wired by the *workbench object*
    output: Option<QmlMethodInvoker>,
}

// realises FR-022, FR-023, FR-024, FR-025, FR-026, FR-040, FR-041
#[qobject(NoQmlElement)]
impl RunnerGroup {
    qproperty!("path", Read = path, Write = set_path, Notify = path_changed);
    qproperty!("executable", Read = executable, Notify = executable_changed);
    qproperty!("ready", Read = ready, Notify = ready_changed);
    qproperty!("running", Read = running, Notify = running_changed);

    // getters
    fn path(&self) -> String {
        self.path.clone()
    }

    fn executable(&self) -> bool {
        self.executable
    }

    fn ready(&self) -> bool {
        self.ready
    }

    fn running(&self) -> bool {
        self.running
    }

    // setters
    // realises FR-024, FR-025, FR-049
    // Names the path, stores it, determines whether it is executable, and updates the readiness.
    fn set_path(&mut self, path: String) {
        if path == self.path {
            return;
        }
        self.path = path;
        if let Some(settings) = &self.settings {
            let mut settings = settings.borrow_mut();
            settings.set_compiler_compiler_path(&self.path);
            let _ = settings.save(&Settings::default_path());
        }
        self.path_changed();
        let executable = runner::is_executable(&self.path);
        if executable != self.executable {
            self.executable = executable;
            self.executable_changed();
        }
        self.update_readiness();
    }

    // signals
    #[qsignal(qml_name = "pathChanged")]
    fn path_changed(&mut self);

    #[qsignal(qml_name = "executableChanged")]
    fn executable_changed(&mut self);

    #[qsignal(qml_name = "readyChanged")]
    fn ready_changed(&mut self);

    #[qsignal(qml_name = "runningChanged")]
    fn running_changed(&mut self);

    #[qsignal(qml_name = "finished")]
    fn finished(&mut self, exit_code: i32);

    // slots
    // realises FR-040
    /// Takes the result of the run from the run thread, hands it to the output group, and emits
    /// `finished`.
    #[qslot(qml_name = "completed")]
    fn completed(&mut self, stdout: String, stderr: String, exit_code: i32) {
        self.running = false;
        self.running_changed();
        if let Some(output) = &self.output {
            invoke_method!(output, "setResult", stdout, stderr, exit_code);
        }
        self.finished(exit_code);
    }

    // realises FR-026, FR-040, FR-041
    /// Records the *processing state* of one *input group*, updates the readiness, and starts a
    /// run where the group became ready and none is in progress.
    #[qslot(qml_name = "setInputState")]
    fn set_input_state(&mut self, kind: i32, state: i32) {
        let (Some(kind), Some(state)) = (
            usize::try_from(kind).ok().and_then(InputKind::of_index),
            usize::try_from(state)
                .ok()
                .and_then(ProcessingState::of_index),
        ) else {
            return;
        };
        self.input_states[kind.index()] = state;
        self.update_readiness();
    }
}

impl RunnerGroup {
    /// Wires the group: the settings and the invoker of the output group.
    ///
    /// * Called by the *workbench object* once, before the *front end* binds to the group.
    pub fn configure(&mut self, settings: Rc<RefCell<Settings>>, output: QmlMethodInvoker) {
        self.settings = Some(settings);
        self.output = Some(output);
    }

    // realises FR-048
    /// Names the path the settings hold for the *compiler-compiler*.
    pub fn load_initial(&mut self) {
        let path = self
            .settings
            .as_ref()
            .map(|settings| settings.borrow().compiler_compiler_path().to_owned())
            .unwrap_or_default();
        self.set_path(path);
    }

    // realises FR-026, FR-041
    /// Determines the readiness, emits `ready_changed` where it changed, and starts a run where
    /// the group became ready and none is in progress.
    fn update_readiness(&mut self) {
        let ready = runner::is_ready(self.executable, self.input_states);
        if ready == self.ready {
            return;
        }
        self.ready = ready;
        self.ready_changed();
        if ready && !self.running {
            self.start_run();
        }
    }

    // realises FR-040, FR-041, IR-013
    /// Clears the output group and runs the *compiler-compiler* on a thread of its own, which
    /// schedules `completed` on the main thread with the result.
    fn start_run(&mut self) {
        self.running = true;
        self.running_changed();
        if let Some(output) = &self.output {
            invoke_method!(output, "clear");
        }
        let invoker = self.get_qml_method_invoker();
        let path = self.path.clone();
        std::thread::spawn(move || {
            let (stdout, stderr, exit_code) = match runner::run(&path, &[]) {
                Ok(result) => (result.stdout, result.stderr, result.exit_code),
                Err(error) => (String::new(), error.to_string(), -1),
            };
            invoke_method!(invoker, "completed", stdout, stderr, exit_code);
        });
    }
}
