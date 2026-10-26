//! The *bridged type* `Automation`: the *player* of an *automation script*, stepped by a timer of
//! the *front end*.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::bridge::input_group::InputGroup;
use crate::bridge::network_editor::{Event, NetworkEditor};
use crate::core::automation::{
    ACTION_TIMEOUT, Action, DialogAnswer, EditorItem, EditorName, FocusTarget, MAX_DELAY_ACTION,
    Outcome, SETTLING, Script, ScriptError, Step, WindowName, configuration_path, difference,
    exit_code_of, gc3_beside, offset_after, offset_of_position, read_script, remove_file, restore,
};
use crate::core::key_stroke::KeyStroke;
use crate::core::launch;
use crate::core::recording::{Recorder, local_time_name, relative};
use crate::core::settings::{
    Settings, SettingsStore, constraint_violation, playback_delay_violation, tab_size_violation,
};
use crate::core::version::version_of_package;

use qtbridge::{QmlMethodInvoker, QmlObject, invoke_method, qobject};

use std::cell::RefCell;
use std::io::Write;
use std::path::Path;
use std::rc::Rc;
use std::time::{Duration, Instant};

// realises FR-501 to FR-504
/// The event of the *journal* an action waits for.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Wanted {
    /// none: the action is complete once *product* settled
    Nothing,
    NodeOpened(String),
    NodeClosed(String),
    FileOpened(String),
    FileClosed(String),
    Activated,
    Deactivated,
}

impl Wanted {
    /// Yields whether `events` satisfy the want: the event itself, or a dialog asked where the
    /// action opens or activates, which the next action of a recorded script answers.
    fn satisfied_by(&self, events: &[Event]) -> bool {
        let asked = events
            .iter()
            .any(|event| matches!(event, Event::CreationAsked | Event::CreationFailed(_)));
        match self {
            Wanted::Nothing => true,
            Wanted::NodeOpened(name) => {
                asked
                    || events
                        .iter()
                        .any(|e| matches!(e, Event::NodeOpened(n) if n == name))
            }
            Wanted::NodeClosed(name) => events
                .iter()
                .any(|e| matches!(e, Event::NodeClosed(n) if n == name)),
            Wanted::FileOpened(path) => {
                asked
                    || events
                        .iter()
                        .any(|e| matches!(e, Event::FileOpened(p) if p == path))
            }
            Wanted::FileClosed(path) => events
                .iter()
                .any(|e| matches!(e, Event::FileClosed(p) if p == path)),
            Wanted::Activated => asked || events.contains(&Event::Activated),
            Wanted::Deactivated => events.contains(&Event::Deactivated),
        }
    }
}

// realises FR-500 to FR-514
/// What an action performed waits for.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Waiting {
    /// the event of the *journal*
    wanted: Wanted,
    /// whether the *settling* is awaited after the event
    settle: bool,
    /// whether the *front end* was asked, so that `frontEndDone` is awaited
    front_end: bool,
    /// whether the question of FR-014 completes the action as well
    save_question: bool,
    /// the moment a `delay` action is complete, `None` for every other action (FR-547)
    until: Option<Instant>,
}

impl Waiting {
    /// What a queued invocation whose effect is a settled *product* waits for.
    fn settled() -> Waiting {
        Waiting {
            wanted: Wanted::Nothing,
            settle: true,
            front_end: false,
            save_question: false,
            until: None,
        }
    }

    /// What a `delay` action waits for: the moment `until`, nothing else (FR-547).
    fn until(until: Instant) -> Waiting {
        Waiting {
            wanted: Wanted::Nothing,
            settle: false,
            front_end: false,
            save_question: false,
            until: Some(until),
        }
    }

    /// What an event of the *journal* followed by the *settling* waits for.
    fn event(wanted: Wanted) -> Waiting {
        Waiting {
            wanted,
            settle: true,
            front_end: false,
            save_question: false,
            until: None,
        }
    }

    /// What a *front-end action* waits for: the answer of the *front end*, and the *settling*
    /// where `settle`.
    fn front_end(settle: bool) -> Waiting {
        Waiting {
            wanted: Wanted::Nothing,
            settle,
            front_end: true,
            save_question: false,
            until: None,
        }
    }
}

// realises FR-500 to FR-514
/// The step in progress.
struct Current {
    /// when the step began, or, once the *playback delay* before it passed, when it did, from
    /// which the timeout counts (FR-514)
    begun: Instant,
    /// whether the *playback delay* before the action passed, or none is waited, the action
    /// being the first (FR-500)
    delay_passed: bool,
    /// the time the timeout allows beyond `ACTION_TIMEOUT`: the entering of the characters of an
    /// `edit` with the *playback delay* between them (FR-505, FR-514)
    allowance: Duration,
    /// what the action performed waits for, `None` while it waits for its precondition or its
    /// delay
    waiting: Option<Waiting>,
    /// the length of the *journal* and the count of the save questions when the step began, so
    /// that the events since are its own
    journal_mark: usize,
    save_questions: u32,
    /// whether a *front-end action* was asked and not told done yet
    front_end_pending: bool,
    /// whether the *front end* found nothing to carry the action out on
    front_end_failed: bool,
    /// since when nothing is pending after the action, for the *settling*; `None` while
    /// something is
    quiet_since: Option<Instant>,
}

// realises FR-497, FR-500, FR-514 to FR-517
/// The *player*.
#[derive(Default)]
pub struct Automation {
    /// the settings, shared with the *workbench object*: the *playback delay* and the path of
    /// the *build system*, beside which `gc3` is found
    settings: Option<Rc<RefCell<SettingsStore>>>,
    /// the *playback delay* of the *playback* that runs, taken from the settings when it
    /// started and kept whatever the script applies (FR-543, FR-544)
    delay: Duration,
    /// the network editor, whose *journal*, state, *node editors* and *editor windows* the
    /// *player* reads
    network: Option<Rc<RefCell<NetworkEditor>>>,
    /// the *input group* of the *network file*
    network_input: Option<Rc<RefCell<InputGroup>>>,
    /// the invoker of the *workbench object*, by which the *player* sets the settings, the
    /// *insert key* and the *typing mode*; left by `started`
    workbench: Option<QmlMethodInvoker>,
    /// the *recorder*, which the *player* suspends while it plays, and which the menu actions of
    /// the *recording* reach through the *player*
    recorder: Option<Rc<RefCell<Recorder>>>,
    /// the gc3s-file of the switch, until the *front end* is up and the *playback* starts
    pending_verification: Option<String>,
    /// whether the *playback* is a *verification run*, under the switch
    verifying: bool,
    /// the script of the *playback*, `None` while none runs or the script was faulty
    script: Option<Script>,
    /// the index of the next step of the script
    next: usize,
    /// the step in progress, `None` between two steps
    current: Option<Current>,
    /// the outcome of every step begun, and the lines of the *playback report*
    outcomes: Vec<Outcome>,
    report: Vec<String>,
    /// whether a *playback* runs: the timer of the *front end* steps the *player* while it does
    running: bool,
    /// whether the user stopped the *playback*: it ends once the action in progress is complete
    /// (FR-546)
    stopping: bool,
}

// realises FR-497, FR-499 to FR-517
#[qobject(NoQmlElement)]
impl Automation {
    qproperty!("running", Read = running, Notify = running_changed);
    qproperty!("verifying", Read = verifying, Constant);
    qproperty!("report", Read = report, Notify = report_changed);
    // the line of the script the playback carries out, for the status bar (FR-545)
    qproperty!(
        "currentLineNumber",
        Read = current_line_number,
        Notify = current_line_changed
    );
    qproperty!(
        "currentLine",
        Read = current_line,
        Notify = current_line_changed
    );

    fn running(&self) -> bool {
        self.running
    }

    // realises FR-545
    // Yields the number of the line of the action in progress, 0 between two actions and while
    // no playback runs.
    fn current_line_number(&self) -> i32 {
        self.current
            .as_ref()
            .and_then(|_| self.current_step())
            .and_then(|step| i32::try_from(step.line).ok())
            .unwrap_or(0)
    }

    // realises FR-545
    // Yields the line of the action in progress as written, without its comment, empty between
    // two actions and while no playback runs.
    fn current_line(&self) -> String {
        self.current
            .as_ref()
            .and_then(|_| self.current_step())
            .map(|step| step.text)
            .unwrap_or_default()
    }

    // realises FR-516, FR-517
    fn verifying(&self) -> bool {
        self.verifying || self.pending_verification.is_some()
    }

    // realises FR-515
    fn report(&self) -> String {
        self.report.join("\n")
    }

    // signals
    #[qsignal(qml_name = "runningChanged")]
    fn running_changed(&mut self);

    #[qsignal(qml_name = "reportChanged")]
    fn report_changed(&mut self);

    #[qsignal(qml_name = "currentLineChanged")]
    fn current_line_changed(&mut self);

    // the front end replaces the text between the two offsets of the code editor by the text,
    // character by character with the delay in milliseconds (FR-505)
    #[qsignal(qml_name = "editAsked")]
    #[allow(clippy::too_many_arguments)] // the four fields of the code editor and the edit
    fn edit_asked(
        &mut self,
        kind: String,
        name: String,
        item: String,
        index: i32,
        start: i32,
        end: i32,
        text: String,
        delay: i32,
    );

    #[qsignal(qml_name = "cursorAsked")]
    fn cursor_asked(&mut self, kind: String, name: String, item: String, index: i32, offset: i32);

    #[qsignal(qml_name = "selectAsked")]
    fn select_asked(
        &mut self,
        kind: String,
        name: String,
        item: String,
        index: i32,
        start: i32,
        end: i32,
    );

    // the window named raised and activated, and the code editor focused where `window` is false
    // (FR-508)
    #[qsignal(qml_name = "focusAsked")]
    fn focus_asked(&mut self, kind: String, name: String, item: String, index: i32, window: bool);

    #[qsignal(qml_name = "modeAsked")]
    fn mode_asked(&mut self, kind: String, name: String, item: String, index: i32, mode: i32);

    // `dialog` is `save`, `creation` or `creation failure`, `choice` the accepting button,
    // `silent` the check box of FR-488 (FR-510)
    #[qsignal(qml_name = "answerAsked")]
    fn answer_asked(&mut self, dialog: String, choice: bool, silent: bool);

    // the playback ended with the exit code (FR-516, FR-517)
    #[qsignal(qml_name = "finished")]
    fn finished(&mut self, code: i32);

    // the recording (FR-519 to FR-533)
    qproperty!("recording", Read = recording, Notify = recording_changed);
    qproperty!(
        "recordingPath",
        Read = recording_path,
        Notify = recording_changed
    );
    qproperty!(
        "recordingFailure",
        Read = recording_failure,
        Notify = recording_changed
    );

    // realises FR-499, FR-528, FR-529, FR-532
    fn recording(&self) -> bool {
        self.recorder
            .as_ref()
            .is_some_and(|recorder| recorder.borrow().is_running())
    }

    // realises FR-529, FR-532
    fn recording_path(&self) -> String {
        self.recorder
            .as_ref()
            .and_then(|recorder| recorder.borrow().path().map(Path::to_path_buf))
            .map(|path| relative(&launch::executable_folder(), &path.to_string_lossy()))
            .unwrap_or_default()
    }

    // realises FR-532
    fn recording_failure(&self) -> String {
        self.recorder
            .as_ref()
            .map(|recorder| recorder.borrow().failure().to_owned())
            .unwrap_or_default()
    }

    #[qsignal(qml_name = "recordingChanged")]
    fn recording_changed(&mut self);

    // realises FR-528
    /// Starts an *individual recording* into the file at `path`, with the version of *product*,
    /// the local time and the path of the *network file*; a *recording* that runs is stopped
    /// first. Called by the *front end* from the file dialog.
    #[qslot(qml_name = "startRecording")]
    fn start_recording(&mut self, path: String) {
        let state = self
            .network
            .as_ref()
            .map(|network| network.borrow().found_state())
            .unwrap_or_default();
        if let Some(recorder) = &self.recorder {
            let mut recorder = recorder.borrow_mut();
            let started = recorder.start(
                Path::new(&launch::absolute(&path)),
                &version_of_package(env!("CARGO_PKG_VERSION")),
                &local_time_name(),
                &Settings::default_path(),
            );
            if started.is_ok() {
                recorder.record_state(&state);
            }
        }
        self.recording_changed();
    }

    // realises FR-529
    /// Stops the *recording* that runs; nothing where none runs. Called by the *front end* from
    /// the dialog that confirms the stop, and when the main window closes.
    #[qslot(qml_name = "stopRecording")]
    fn stop_recording(&mut self) {
        if let Some(recorder) = &self.recorder {
            recorder.borrow_mut().stop();
        }
        self.recording_changed();
    }

    // realises FR-531
    /// Yields whether a code editor holds or held the focus and is presented, so that 'Store to
    /// verify on replay' is enabled; called by the *front end* when the menu is about to show.
    #[qslot(qml_name = "canStore")]
    fn can_store(&self) -> bool {
        let Some(editor) = self.focused_editor() else {
            return false;
        };
        self.network
            .as_ref()
            .is_some_and(|network| self.presented(&network.borrow(), &editor).is_ok())
    }

    // realises FR-531
    /// Writes the *expected file* of the focused code editor, with the octets `verify` would
    /// compare, and records `verify`; a failure is kept by the *recorder* for the status bar.
    #[qslot(qml_name = "storeFocused")]
    fn store_focused(&mut self) {
        if let Some(editor) = self.focused_editor() {
            match self.octets_of(&editor) {
                Ok(octets) => {
                    if let Some(recorder) = &self.recorder {
                        let _ = recorder.borrow_mut().store_expected(&editor, &octets);
                    }
                }
                Err(outcome) => {
                    if let (Some(recorder), Outcome::Error(reason) | Outcome::Failed(reason)) =
                        (&self.recorder, outcome)
                    {
                        recorder.borrow_mut().note_failure(reason);
                    }
                }
            }
        }
        self.recording_changed();
    }

    // realises FR-526
    /// Records `focus main` for `main` and `focus node <name>` for `node`; called by the windows
    /// when they become active.
    #[qslot(qml_name = "recordWindowFocus")]
    fn record_window_focus(&mut self, kind: String, name: String) {
        let target = match kind.as_str() {
            "main" => FocusTarget::Window(WindowName::Main),
            "node" => FocusTarget::Window(WindowName::Node(name)),
            _ => return,
        };
        if let Some(recorder) = &self.recorder {
            recorder.borrow_mut().focus(&target);
        }
    }

    // slots
    // realises FR-497, FR-499, FR-500
    /// Reads the script at `path` through `read_script`, with `gc3` beside the *build system*
    /// of the settings and the configuration of `configuration_path`, and starts the *playback*;
    /// a faulty script is the whole report at once, `finished` emitted with 2. Called by the
    /// file dialog of the menu; does nothing while a *playback* runs.
    #[qslot(qml_name = "play")]
    fn play(&mut self, path: String) {
        if self.running {
            return;
        }
        self.verifying = false;
        self.start(&path);
    }

    // realises FR-517
    /// Plays the gc3s-file of the switch as `play` does, as a *verification run*; scheduled by
    /// `started` of the *workbench object*; does nothing where the command line named none.
    #[qslot(qml_name = "startVerification")]
    fn start_verification(&mut self) {
        let Some(path) = self.pending_verification.take() else {
            return;
        };
        self.verifying = true;
        self.start(&path);
    }

    // realises FR-500 to FR-514
    /// One step of the *player*: begins the next action where none is in progress, performs the
    /// one in progress once its precondition holds and its delay passed, and completes it once
    /// its effect is told; called by the timer of the *front end* every 50 milliseconds while
    /// `running`.
    #[qslot(qml_name = "step")]
    fn step(&mut self) {
        if !self.running {
            return;
        }
        if self.current.is_none() {
            let steps = self.script.as_ref().map_or(0, |script| script.steps.len());
            // after the last step, or where the user stopped the playback while no step was in
            // progress (FR-546)
            if self.next >= steps || self.stopping {
                self.finish();
                return;
            }
            let (journal_mark, save_questions) = {
                let network = self.network.as_ref().map(|network| network.borrow());
                let input = self.network_input.as_ref().map(|input| input.borrow());
                (
                    network.map_or(0, |network| network.journal_len()),
                    input.map_or(0, |input| input.save_questions()),
                )
            };
            self.current = Some(Current {
                begun: Instant::now(),
                // the first action waits no delay (FR-500)
                delay_passed: self.next == 0,
                allowance: Duration::ZERO,
                waiting: None,
                journal_mark,
                save_questions,
                front_end_pending: false,
                front_end_failed: false,
                quiet_since: None,
            });
            self.current_line_changed();
        }
        if let Some(outcome) = self.advance() {
            self.record(outcome);
        }
    }

    // realises FR-546
    /// Notes that the user stopped the *playback*: the step in progress is completed as every
    /// step is, its outcome turned into `Stopped`, which ends the *playback*; where none is in
    /// progress, the *playback* ends at the next step. Called by the menu item; nothing while no
    /// *playback* runs.
    #[qslot(qml_name = "stop")]
    fn stop(&mut self) {
        if self.running {
            self.stopping = true;
        }
    }

    // realises FR-505, FR-507, FR-508, FR-510, FR-514
    /// Takes that the *front end* carried out the *front-end action* asked, or found nothing to
    /// carry it out on where `done` is false, which fails the action.
    #[qslot(qml_name = "frontEndDone")]
    fn front_end_done(&mut self, done: bool) {
        if let Some(current) = self.current.as_mut() {
            current.front_end_pending = false;
            current.front_end_failed = !done;
        }
    }
}

impl Automation {
    /// Wires the *player*: the settings, the network editor and the *input group* of the
    /// *network file*, whose invokers it asks for when it performs an action, and the gc3s-file
    /// of the switch, where the command line named one; called by the *workbench object* in
    /// `Default`.
    pub fn configure(
        &mut self,
        settings: Rc<RefCell<SettingsStore>>,
        network: Rc<RefCell<NetworkEditor>>,
        network_input: Rc<RefCell<InputGroup>>,
        recorder: Rc<RefCell<Recorder>>,
        verify: Option<String>,
    ) {
        self.settings = Some(settings);
        self.network = Some(network);
        self.network_input = Some(network_input);
        self.recorder = Some(recorder);
        self.pending_verification = verify;
    }

    // realises FR-509
    /// Takes the invoker of the *workbench object*, by which the *player* sets the settings, the
    /// *insert key* and the *typing mode*; called by `started` of the *workbench object*.
    pub fn take_workbench_invoker(&mut self, invoker: QmlMethodInvoker) {
        self.workbench = Some(invoker);
    }

    // realises FR-497, FR-500
    /// Reads the script at `path` and starts the *playback*, or reports the faulty script and
    /// finishes with 2.
    fn start(&mut self, path: &str) {
        self.outcomes.clear();
        self.report.clear();
        self.next = 0;
        self.current = None;
        self.stopping = false;
        // the delay of the whole playback, as the settings hold it now: set in the playback
        // delay dialog or restored from the settings file (FR-543, FR-544)
        self.delay = Duration::from_millis(u64::from(self.delay_ms()));
        self.suspend_recorder(true);
        let build_system = self
            .settings
            .as_ref()
            .map(|settings| settings.borrow().build_system_path().to_owned())
            .unwrap_or_default();
        let gc3 = gc3_beside(&build_system);
        match read_script(Path::new(path), &gc3, &configuration_path()) {
            Ok(script) => {
                self.script = Some(script);
                self.running = true;
                self.set_playing(true);
                self.running_changed();
                self.report_changed();
            }
            Err(error) => {
                self.script = None;
                self.outcomes.push(Outcome::Error(error.to_string()));
                self.append_line(faulty_line(&error));
                self.suspend_recorder(false);
                self.finished(2);
            }
        }
    }

    // realises FR-533
    /// Suspends the *recorder* while the *playback* runs, so that nothing of it is recorded.
    fn suspend_recorder(&self, suspended: bool) {
        if let Some(recorder) = &self.recorder {
            recorder.borrow_mut().suspend(suspended);
        }
    }

    // realises FR-500, FR-516, FR-517
    /// Ends the *playback*: stops the timer by `running` and emits `finished` with the exit code.
    fn finish(&mut self) {
        self.current = None;
        self.current_line_changed();
        self.running = false;
        self.set_playing(false);
        if let Ok(workbench) = self.workbench() {
            // the settings file kept aside by `settings file` put back (FR-537)
            invoke_method!(workbench, "restoreSettingsFile");
        }
        self.running_changed();
        self.suspend_recorder(false);
        self.finished(exit_code_of(&self.outcomes));
    }

    // realises FR-534
    /// Tells the network editor whether a *playback* runs, so that it transmits no *activate
    /// request* of its own meanwhile.
    fn set_playing(&self, playing: bool) {
        if let Some(network) = &self.network {
            let invoker = network.borrow().get_qml_method_invoker();
            invoke_method!(&invoker, "setPlaying", playing);
        }
    }

    // realises FR-513 to FR-515
    /// Records the outcome of the step in progress: appends its line to the report, and goes on
    /// to the next step or ends the *playback*.
    fn record(&mut self, outcome: Outcome) {
        // the action completed before the stop ends the playback (FR-546)
        let outcome = if self.stopping && matches!(outcome, Outcome::Completed) {
            Outcome::Stopped
        } else {
            outcome
        };
        let line = self
            .script
            .as_ref()
            .and_then(|script| script.steps.get(self.next))
            .map(|step| crate::core::automation::report_line(step, &outcome))
            .unwrap_or_default();
        self.append_line(line);
        let continues = outcome.continues();
        self.outcomes.push(outcome);
        self.current = None;
        self.current_line_changed();
        self.next += 1;
        if !continues {
            self.finish();
        }
    }

    // realises FR-515, IR-215
    /// Appends a line to the report, written to standard output in a *verification run*.
    fn append_line(&mut self, line: String) {
        if self.verifying {
            println!("{line}");
            let _ = std::io::stdout().flush();
        }
        self.report.push(line);
        self.report_changed();
    }

    /// Yields the step in progress.
    fn current_step(&self) -> Option<Step> {
        self.script
            .as_ref()
            .and_then(|script| script.steps.get(self.next))
            .cloned()
    }

    /// Yields the *playback delay* of the settings in milliseconds.
    fn delay_ms(&self) -> u32 {
        self.settings
            .as_ref()
            .map_or(0, |settings| settings.borrow().playback_delay())
    }

    // realises FR-505, FR-514
    /// Yields the time the timeout of `action` allows beyond `ACTION_TIMEOUT`: the entering of
    /// the characters of an `edit`, one *playback delay* per character; nothing for every other
    /// action.
    fn allowance_of(&self, action: &Action) -> Duration {
        match action {
            Action::Edit { text, .. } => self
                .delay
                .saturating_mul(u32::try_from(text.chars().count()).unwrap_or(u32::MAX)),
            Action::Delay(milliseconds) => Duration::from_millis(u64::from(*milliseconds)),
            _ => Duration::ZERO,
        }
    }

    // realises FR-500 to FR-514
    /// Advances the step in progress by one step of the *player*: performs it once ready, or
    /// checks whether its effect is told; yields its outcome once it is complete or failed.
    fn advance(&mut self) -> Option<Outcome> {
        let step = self.current_step()?;
        if !self.current.as_ref()?.delay_passed {
            // the playback delay before every action but the first, counted from the
            // completion of the action before it; the timeout counts from its end (FR-500,
            // FR-514)
            if self.current.as_ref()?.begun.elapsed() < self.delay {
                return None;
            }
            let current = self.current.as_mut()?;
            current.delay_passed = true;
            current.begun = Instant::now();
        }
        let elapsed = self.current.as_ref()?.begun.elapsed();
        if self.current.as_ref()?.waiting.is_none() {
            return match self.ready(&step.action) {
                Readiness::Ready => match self.perform(&step.action) {
                    Ok(waiting) => {
                        let allowance = self.allowance_of(&step.action);
                        if let Some(current) = self.current.as_mut() {
                            current.front_end_pending = waiting.front_end;
                            current.waiting = Some(waiting);
                            current.allowance = allowance;
                        }
                        None
                    }
                    Err(outcome) => Some(outcome),
                },
                Readiness::Waiting(reason) => (elapsed >= ACTION_TIMEOUT)
                    .then(|| Outcome::Error(format!("not ready within 30 seconds: {reason}"))),
                Readiness::Failed(reason) => Some(Outcome::Error(reason)),
            };
        }
        let (mark, questions, waiting, pending, failed) = {
            let current = self.current.as_ref()?;
            (
                current.journal_mark,
                current.save_questions,
                current.waiting.clone()?,
                current.front_end_pending,
                current.front_end_failed,
            )
        };
        if failed {
            return Some(Outcome::Error(
                "the editor or the window is not presented".to_owned(),
            ));
        }
        if pending {
            let allowance = self.current.as_ref()?.allowance;
            return (elapsed >= ACTION_TIMEOUT + allowance).then(|| {
                Outcome::Error("the front end did not answer within 30 seconds".to_owned())
            });
        }
        let (events, quiet, asked) = {
            let network = self.network.as_ref()?.borrow();
            let asked = self
                .network_input
                .as_ref()
                .is_some_and(|input| input.borrow().save_questions() != questions);
            (
                network.journal_since(mark).to_vec(),
                network.is_quiet(),
                asked,
            )
        };
        if let Some(Event::Failed(message)) = events
            .iter()
            .find(|event| matches!(event, Event::Failed(_)))
        {
            return Some(Outcome::Error(message.clone()));
        }
        if waiting.save_question && asked {
            return Some(Outcome::Completed);
        }
        // a delay is complete once its moment is reached (FR-547)
        if let Some(until) = waiting.until {
            return (Instant::now() >= until).then_some(Outcome::Completed);
        }
        let wanted = waiting.wanted.satisfied_by(&events);
        let settled = if waiting.settle {
            let current = self.current.as_mut()?;
            if quiet && wanted {
                let since = *current.quiet_since.get_or_insert_with(Instant::now);
                since.elapsed() >= SETTLING
            } else {
                current.quiet_since = None;
                false
            }
        } else {
            true
        };
        if wanted && settled {
            return Some(Outcome::Completed);
        }
        (elapsed >= ACTION_TIMEOUT)
            .then(|| Outcome::Error("the effect was not told within 30 seconds".to_owned()))
    }

    // realises FR-501 to FR-514
    /// Yields whether the precondition of `action` holds, what it waits for, or why it cannot
    /// hold.
    fn ready(&self, action: &Action) -> Readiness {
        let network = match self.network.as_ref() {
            Some(network) => network.borrow(),
            None => return Readiness::Failed("the player is not wired".to_owned()),
        };
        match action {
            Action::OpenNode(name) => {
                if !network.has_node(name) {
                    return if network.kept_mode() == 0 {
                        Readiness::Waiting("no network graph is built yet".to_owned())
                    } else {
                        Readiness::Failed(format!("the network holds no node '{name}'"))
                    };
                }
                if network.kept_mode() < 2 {
                    Readiness::Waiting("the network serves no nodes".to_owned())
                } else {
                    Readiness::Ready
                }
            }
            Action::CloseNode(name) => {
                if network.node_editor_named(name).is_some() {
                    Readiness::Ready
                } else {
                    Readiness::Failed(format!("no node window of '{name}' is presented"))
                }
            }
            Action::OpenFile(path) => {
                if network.resolved_file_path(path).is_some() {
                    Readiness::Ready
                } else if network.kept_mode() == 0 && network.has_node("") {
                    Readiness::Waiting("no network graph is built yet".to_owned())
                } else {
                    Readiness::Failed(format!("the network holds no file '{path}'"))
                }
            }
            Action::CloseFile(path) => match network.resolved_file_path(path) {
                Some(resolved) if network.file_group_named(&resolved).is_some() => Readiness::Ready,
                _ => Readiness::Failed(format!("no editor window on '{path}' is presented")),
            },
            Action::Activate | Action::Deactivate => {
                if network.is_switchable() {
                    Readiness::Ready
                } else {
                    Readiness::Waiting("the activation switch is disabled".to_owned())
                }
            }
            Action::Edit { editor, .. }
            | Action::Digits { editor, .. }
            | Action::Undo(editor)
            | Action::Redo(editor) => match self.group_of(&network, editor) {
                Ok(_) => Readiness::Ready,
                Err(reason) => Readiness::Failed(reason),
            },
            // the expiry takes place after the playback delay of every action; no wait of its
            // own (FR-506)
            Action::Idle(editor) | Action::LongIdle(editor) => {
                match self.group_of(&network, editor) {
                    Ok(_) => Readiness::Ready,
                    Err(reason) => Readiness::Failed(reason),
                }
            }
            // a delay beyond the bound is a value the notation does not admit (FR-547)
            Action::Delay(milliseconds) => {
                if *milliseconds <= MAX_DELAY_ACTION {
                    Readiness::Ready
                } else {
                    Readiness::Failed(format!(
                        "the delay of {milliseconds} ms exceeds {} seconds",
                        MAX_DELAY_ACTION / 1000
                    ))
                }
            }
            Action::Cursor { editor, .. }
            | Action::Select { editor, .. }
            | Action::Mode { editor, .. }
            | Action::Verify { editor, .. }
            | Action::Focus(FocusTarget::Editor(editor)) => {
                match self.presented(&network, editor) {
                    Ok(()) => Readiness::Ready,
                    Err(reason) => Readiness::Failed(reason),
                }
            }
            Action::Focus(FocusTarget::Window(WindowName::Node(name))) => {
                if network.node_editor_named(name).is_some() {
                    Readiness::Ready
                } else {
                    Readiness::Failed(format!("no node window of '{name}' is presented"))
                }
            }
            Action::Settings(values) => {
                let violation = tab_size_violation(values.tab_size)
                    .or_else(|| {
                        (!values.automatic)
                            .then(|| constraint_violation(values.idle_time, values.long_idle_time))
                            .flatten()
                    })
                    .or_else(|| playback_delay_violation(values.playback_delay));
                match violation {
                    Some(message) => Readiness::Failed(message),
                    None => Readiness::Ready,
                }
            }
            Action::InsertKey(name) => {
                if KeyStroke::of_name(name).is_some() {
                    Readiness::Ready
                } else {
                    Readiness::Failed(format!("'{name}' names no key stroke"))
                }
            }
            Action::Restore { saved, .. } => {
                if Path::new(saved).is_file() {
                    Readiness::Ready
                } else {
                    Readiness::Failed(format!("the saved file '{saved}' does not exist"))
                }
            }
            Action::SettingsFile(path) => {
                if Path::new(path).is_file() {
                    Readiness::Ready
                } else {
                    Readiness::Failed(format!("the saved settings file '{path}' does not exist"))
                }
            }
            _ => Readiness::Ready,
        }
    }

    // realises FR-501 to FR-512
    /// Performs `action`: a queued invocation of a slot, a signal to the *front end*, or the
    /// restoring or the comparison through the *core*; yields what the action waits for, or its
    /// outcome where it is complete or failed at once.
    fn perform(&mut self, action: &Action) -> Result<Waiting, Outcome> {
        let not_wired = || Outcome::Error("the player is not wired".to_owned());
        let network_invoker = self
            .network
            .as_ref()
            .ok_or_else(not_wired)?
            .borrow()
            .get_qml_method_invoker();
        let input_invoker = self
            .network_input
            .as_ref()
            .ok_or_else(not_wired)?
            .borrow()
            .get_qml_method_invoker();
        let delay = i32::try_from(self.delay.as_millis()).unwrap_or(i32::MAX);
        match action {
            Action::Network(path) => {
                invoke_method!(&input_invoker, "namePath", path.clone());
                Ok(Waiting {
                    save_question: true,
                    ..Waiting::settled()
                })
            }
            Action::BuildSystem(path) => {
                invoke_method!(&network_invoker, "nameBuildSystem", path.clone());
                Ok(Waiting::settled())
            }
            Action::NewNetwork(path) => {
                invoke_method!(&input_invoker, "createFile", path.clone());
                Ok(Waiting::settled())
            }
            Action::OpenNode(name) => {
                invoke_method!(&network_invoker, "openNodeNamed", name.clone());
                Ok(Waiting::event(Wanted::NodeOpened(name.clone())))
            }
            Action::CloseNode(name) => {
                invoke_method!(&network_invoker, "closeNode", name.clone());
                Ok(Waiting::event(Wanted::NodeClosed(name.clone())))
            }
            Action::OpenFile(path) => {
                let resolved = self.slot_path(path)?;
                invoke_method!(&network_invoker, "openFileNamed", resolved.clone());
                Ok(Waiting::event(Wanted::FileOpened(resolved)))
            }
            Action::CloseFile(path) => {
                let resolved = self.slot_path(path)?;
                invoke_method!(&network_invoker, "closeFile", resolved.clone());
                Ok(Waiting::event(Wanted::FileClosed(resolved)))
            }
            Action::Activate => {
                if self.network_mode() == 3 {
                    return Err(Outcome::Completed);
                }
                invoke_method!(&network_invoker, "activate");
                Ok(Waiting::event(Wanted::Activated))
            }
            Action::Deactivate => {
                if self.network_mode() != 3 {
                    return Err(Outcome::Completed);
                }
                invoke_method!(&network_invoker, "deactivate");
                Ok(Waiting::event(Wanted::Deactivated))
            }
            Action::Edit {
                editor,
                line,
                column,
                range,
                text,
            } => {
                let group = self.group(editor)?;
                let current = group.borrow().document_text();
                let start = offset_of_position(&current, *line, *column).ok_or_else(|| {
                    Outcome::Error(format!("the position {line}:{column} is beyond the text"))
                })?;
                let end = offset_after(&current, start, *range);
                let (kind, name, item, index) = editor.fields();
                self.edit_asked(
                    kind,
                    name,
                    item,
                    index,
                    int_of(start),
                    int_of(end),
                    text.clone(),
                    delay,
                );
                Ok(Waiting::front_end(false))
            }
            Action::Digits {
                editor,
                mode,
                start,
                removed,
                text,
            } => {
                let group = self.group(editor)?;
                let invoker = group.borrow().get_qml_method_invoker();
                invoke_method!(
                    &invoker,
                    "applyDigits",
                    int_of(mode.index()),
                    int_of(*start),
                    int_of(*removed),
                    text.clone()
                );
                Err(Outcome::Completed)
            }
            Action::Idle(editor) | Action::LongIdle(editor) => {
                let group = self.group(editor)?;
                let invoker = group.borrow().get_qml_method_invoker();
                if matches!(action, Action::Idle(_)) {
                    invoke_method!(&invoker, "idleExpired");
                } else {
                    invoke_method!(&invoker, "longIdleExpired");
                }
                Ok(Waiting::settled())
            }
            Action::Cursor {
                editor,
                line,
                column,
            } => {
                let offset = self.offset_in(editor, *line, *column)?;
                let (kind, name, item, index) = editor.fields();
                self.cursor_asked(kind, name, item, index, int_of(offset));
                Ok(Waiting::front_end(false))
            }
            Action::Select { editor, from, to } => {
                let start = self.offset_in(editor, from.0, from.1)?;
                let end = self.offset_in(editor, to.0, to.1)?;
                let (kind, name, item, index) = editor.fields();
                self.select_asked(kind, name, item, index, int_of(start), int_of(end));
                Ok(Waiting::front_end(false))
            }
            Action::Focus(target) => {
                let (kind, name, item, index, window) = match target {
                    FocusTarget::Editor(editor) => {
                        let (kind, name, item, index) = editor.fields();
                        (kind, name, item, index, false)
                    }
                    FocusTarget::Window(WindowName::Main) => {
                        ("main".to_owned(), String::new(), String::new(), 0, true)
                    }
                    FocusTarget::Window(WindowName::Node(name)) => {
                        ("node".to_owned(), name.clone(), String::new(), 0, true)
                    }
                };
                self.focus_asked(kind, name, item, index, window);
                Ok(Waiting::front_end(false))
            }
            Action::Mode { editor, mode } => {
                let (kind, name, item, index) = editor.fields();
                self.mode_asked(kind, name, item, index, int_of(mode.index()));
                Ok(Waiting::front_end(false))
            }
            Action::Undo(editor) | Action::Redo(editor) => {
                let group = self.group(editor)?;
                let invoker = group.borrow().get_qml_method_invoker();
                if matches!(action, Action::Undo(_)) {
                    invoke_method!(&invoker, "applyUndo");
                } else {
                    invoke_method!(&invoker, "applyRedo");
                }
                Err(Outcome::Completed)
            }
            // the playback delay of the line is not applied: the delay the playback was started
            // with holds for the whole playback (FR-509, FR-543)
            Action::Settings(values) => {
                let workbench = self.workbench()?;
                invoke_method!(
                    workbench,
                    "applySettings",
                    int_of(values.idle_time as usize),
                    int_of(values.long_idle_time as usize),
                    values.automatic,
                    int_of(values.tab_size as usize),
                    values.tabs_to_spaces,
                    values.silent_creation
                );
                Err(Outcome::Completed)
            }
            Action::InsertKey(name) => {
                let workbench = self.workbench()?;
                invoke_method!(workbench, "setInsertKeyNamed", name.clone());
                Err(Outcome::Completed)
            }
            Action::TypingMode(insert) => {
                let workbench = self.workbench()?;
                invoke_method!(workbench, "setTypingMode", *insert);
                Err(Outcome::Completed)
            }
            Action::Answer(answer) => {
                let (dialog, choice, silent) = match answer {
                    DialogAnswer::Save(save) => ("save", *save, false),
                    DialogAnswer::Creation { create, silent } => ("creation", *create, *silent),
                    DialogAnswer::CreationFailure(proceed) => ("creation failure", *proceed, false),
                };
                self.answer_asked(dialog.to_owned(), choice, silent);
                Ok(Waiting::front_end(true))
            }
            Action::Restore { saved, file } => {
                restore(Path::new(saved), Path::new(file)).map_err(Outcome::Error)?;
                self.reload_documents_of(file);
                Ok(Waiting::settled())
            }
            Action::Remove(file) => {
                remove_file(Path::new(file)).map_err(Outcome::Error)?;
                self.reload_documents_of(file);
                Ok(Waiting::settled())
            }
            Action::Init => {
                invoke_method!(&network_invoker, "init");
                Ok(Waiting::settled())
            }
            Action::Delay(milliseconds) => Ok(Waiting::until(
                Instant::now() + Duration::from_millis(u64::from(*milliseconds)),
            )),
            Action::CloseNetwork => {
                invoke_method!(&network_invoker, "closeNetwork");
                Ok(Waiting::settled())
            }
            Action::SettingsFile(path) => {
                Settings::load(Path::new(path)).map_err(|error| {
                    Outcome::Error(format!(
                        "the saved settings file '{path}' is not applied: {error}"
                    ))
                })?;
                let workbench = self.workbench()?;
                invoke_method!(workbench, "applySettingsFile", path.clone());
                Ok(Waiting::settled())
            }
            Action::Verify { editor, file } => {
                let actual = self.octets_of(editor)?;
                let expected = std::fs::read(file).map_err(|error| {
                    Outcome::Error(format!(
                        "the expected file '{file}' cannot be read: {error}"
                    ))
                })?;
                Err(match difference(&actual, &expected) {
                    None => Outcome::Completed,
                    Some(difference) => Outcome::Failed(difference),
                })
            }
        }
    }

    /// Yields the *network mode* kept, 0 where none is.
    fn network_mode(&self) -> i32 {
        self.network
            .as_ref()
            .map_or(0, |network| network.borrow().kept_mode())
    }

    // realises FR-511
    /// Schedules `reload` of every *input group* that presents a file of the folder `source`,
    /// which `restore` replaced: the *input group* of the *network file*, those of every named
    /// *node editor* and those of the *editor windows*, so that the code editors show the
    /// restored content.
    fn reload_documents_of(&self, file: &str) {
        let Some(network) = self.network.as_ref() else {
            return;
        };
        let file = launch::absolute(file);
        let mut groups: Vec<Rc<RefCell<InputGroup>>> = Vec::new();
        if let Some(input) = &self.network_input {
            groups.push(Rc::clone(input));
        }
        {
            let network = network.borrow();
            for editor in network.named_node_editors() {
                groups.extend(editor.borrow().input_groups());
            }
            groups.extend(network.file_groups());
        }
        for group in groups {
            let invoker = {
                let group = group.borrow();
                if launch::absolute(&group.document_path()) != file {
                    continue;
                }
                group.get_qml_method_invoker()
            };
            invoke_method!(&invoker, "reload");
        }
    }

    // realises FR-531
    /// Yields the code editor the *recorder* holds as focused.
    fn focused_editor(&self) -> Option<EditorName> {
        self.recorder
            .as_ref()
            .and_then(|recorder| recorder.borrow().focused_editor())
    }

    /// Yields the invoker of the *workbench object*.
    fn workbench(&self) -> Result<&QmlMethodInvoker, Outcome> {
        self.workbench
            .as_ref()
            .ok_or_else(|| Outcome::Error("the front end is not up".to_owned()))
    }

    // realises FR-503
    /// Yields the path of the file of the *network graph* at `path`, in the form the *editor
    /// windows* are keyed by.
    fn slot_path(&self, path: &str) -> Result<String, Outcome> {
        self.network
            .as_ref()
            .and_then(|network| network.borrow().resolved_file_path(path))
            .ok_or_else(|| Outcome::Error(format!("the network holds no file '{path}'")))
    }

    // realises FR-505 to FR-508
    /// Yields the *input group* of the code editor `editor` whose document is edited: the one of
    /// the *network file*, of the *meta compiler DSL* or an *input* of a *node window*, or of an
    /// *editor window*.
    fn group(&self, editor: &EditorName) -> Result<Rc<RefCell<InputGroup>>, Outcome> {
        let network = self
            .network
            .as_ref()
            .ok_or_else(|| Outcome::Error("the player is not wired".to_owned()))?
            .borrow();
        self.group_of(&network, editor).map_err(Outcome::Error)
    }

    /// Yields the *input group* of `editor`, or why there is none.
    fn group_of(
        &self,
        network: &NetworkEditor,
        editor: &EditorName,
    ) -> Result<Rc<RefCell<InputGroup>>, String> {
        match editor {
            EditorName::Network => self
                .network_input
                .clone()
                .ok_or_else(|| "the player is not wired".to_owned()),
            EditorName::Node { name, item } => {
                let node = network
                    .node_editor_named(name)
                    .ok_or_else(|| format!("no node window of '{name}' is presented"))?;
                let node = node.borrow();
                match item {
                    EditorItem::MetaDsl => Ok(node.meta_dsl_group()),
                    EditorItem::Input(index) => node
                        .input_group(*index)
                        .ok_or_else(|| format!("the node '{name}' has no input {index}")),
                    EditorItem::Output(_) | EditorItem::Diagnostics => {
                        Err("an output page and the diagnostics page are not edited".to_owned())
                    }
                }
            }
            EditorName::File { path } => {
                let resolved = network
                    .resolved_file_path(path)
                    .ok_or_else(|| format!("the network holds no file '{path}'"))?;
                network
                    .file_group_named(&resolved)
                    .ok_or_else(|| format!("no editor window on '{path}' is presented"))
            }
        }
    }

    // realises FR-507, FR-508, FR-512
    /// Yields whether the code editor `editor` is presented, or why it is not.
    fn presented(&self, network: &NetworkEditor, editor: &EditorName) -> Result<(), String> {
        match editor {
            EditorName::Node {
                name,
                item: EditorItem::Output(index),
            } => {
                let node = network
                    .node_editor_named(name)
                    .ok_or_else(|| format!("no node window of '{name}' is presented"))?;
                let has_output = node.borrow().output_octets(*index).is_some();
                has_output
                    .then_some(())
                    .ok_or_else(|| format!("the node '{name}' has no output {index}"))
            }
            EditorName::Node {
                name,
                item: EditorItem::Diagnostics,
            } => network
                .node_editor_named(name)
                .map(|_| ())
                .ok_or_else(|| format!("no node window of '{name}' is presented")),
            _ => self.group_of(network, editor).map(|_| ()),
        }
    }

    // realises FR-508
    /// Yields the offset of the position in the text of the code editor `editor`.
    fn offset_in(&self, editor: &EditorName, line: usize, column: usize) -> Result<usize, Outcome> {
        let text = match editor {
            EditorName::Node {
                name,
                item: EditorItem::Diagnostics,
            } => {
                let network = self
                    .network
                    .as_ref()
                    .ok_or_else(|| Outcome::Error("the player is not wired".to_owned()))?
                    .borrow();
                network
                    .node_editor_named(name)
                    .map(|node| node.borrow().diagnostics_text())
                    .ok_or_else(|| {
                        Outcome::Error(format!("no node window of '{name}' is presented"))
                    })?
            }
            EditorName::Node {
                item: EditorItem::Output(_),
                ..
            } => String::from_utf8_lossy(&self.octets_of(editor)?).into_owned(),
            _ => self.group(editor)?.borrow().document_text(),
        };
        offset_of_position(&text, line, column).ok_or_else(|| {
            Outcome::Error(format!("the position {line}:{column} is beyond the text"))
        })
    }

    // realises FR-512
    /// Yields the octets of the document the code editor `editor` presents.
    fn octets_of(&self, editor: &EditorName) -> Result<Vec<u8>, Outcome> {
        let network = self
            .network
            .as_ref()
            .ok_or_else(|| Outcome::Error("the player is not wired".to_owned()))?
            .borrow();
        match editor {
            EditorName::Node {
                name,
                item: EditorItem::Output(index),
            } => {
                let node = network.node_editor_named(name).ok_or_else(|| {
                    Outcome::Error(format!("no node window of '{name}' is presented"))
                })?;
                let octets = node.borrow().output_octets(*index);
                octets.ok_or_else(|| {
                    Outcome::Error(format!("the node '{name}' has no output {index}"))
                })
            }
            EditorName::Node {
                name,
                item: EditorItem::Diagnostics,
            } => network
                .node_editor_named(name)
                .map(|node| node.borrow().diagnostics_text().into_bytes())
                .ok_or_else(|| Outcome::Error(format!("no node window of '{name}' is presented"))),
            _ => self
                .group_of(&network, editor)
                .map(|group| group.borrow().document_octets())
                .map_err(Outcome::Error),
        }
    }
}

// realises FR-501 to FR-514
/// Whether the precondition of an action holds.
enum Readiness {
    Ready,
    /// not yet; carries what is waited for
    Waiting(String),
    /// it cannot; carries the reason
    Failed(String),
}

/// Yields the report of a script that was not read: the rendering of the error, which carries
/// the diagnosis of `gc3` for a faulty script (FR-497, FR-515).
fn faulty_line(error: &ScriptError) -> String {
    error.to_string()
}

/// Yields `value` as the bridge carries a count, `i32::MAX` where it does not fit.
fn int_of(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}
