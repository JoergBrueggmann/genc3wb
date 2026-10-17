//! The *bridged type* `Workbench`: the *workbench object*, owner of the settings and the groups;
//! and the *handed file* and the file-open event of the operating system, as functions of the
//! module.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::bridge::document_registry::DocumentRegistry;
use crate::bridge::input_group::InputGroup;
use crate::bridge::network_editor::NetworkEditor;
use crate::core::input_file::InputKind;
use crate::core::key_stroke::KeyStroke;
use crate::core::launch;
use crate::core::settings::{Settings, SettingsStore, tab_size_violation, times_of_processing};
use crate::core::version::version_of_package;

use qtbridge::{QmlMethodInvoker, QmlObject, invoke_method, qobject};

use std::cell::RefCell;
use std::rc::Rc;
use std::time::{Duration, Instant};

// realises FR-476, FR-477
/// How long the *start-up phase* lasts after the *front end* is up: the file the operating
/// system hands over at launch arrives within about a tenth of a second of it, a double-click of
/// the user later.
const START_UP_GRACE: Duration = Duration::from_secs(2);

// realises FR-476, FR-477, IR-206, IR-207
/// The state of the module: what reaches the *workbench object* from outside the *front end*.
///
/// The *workbench object* is created by the QML engine when the *front end* first names it, so
/// that 'main.rs' and the handler of the file-open event reach no instance of it: the *handed
/// file* of the command line is left here for `Default` to take, and the invoker of the
/// *workbench object*, left by `started`, carries a file the operating system hands over in the
/// *start-up phase* to `open_handed_file`.
#[derive(Default)]
struct Launch {
    /// the *handed file* not taken by the *workbench object* yet
    handed: Option<String>,
    /// whether the command line handed the file, after which no file-open event of the launch
    /// is to be expected
    by_command_line: bool,
    /// the invoker of the *workbench object*, once the *front end* is up
    invoker: Option<QmlMethodInvoker>,
    /// when the *start-up phase* ends, `None` while the *front end* is not up
    phase_end: Option<Instant>,
}

impl Launch {
    /// Yields whether the *start-up phase* lasts: the *front end* is up and its end is not
    /// reached yet.
    fn in_start_up_phase(&self) -> bool {
        self.phase_end.is_some_and(|end| Instant::now() < end)
    }
}

thread_local! {
    static LAUNCH: RefCell<Launch> = RefCell::new(Launch::default());
}

// realises FR-476, IR-206
/// Leaves `path` as the *handed file* for the *workbench object*, which names it as its
/// *network file* in place of the stored path when it is created; called by 'main.rs' before
/// the *front end* is loaded.
pub fn hand_file(path: String) {
    LAUNCH.with(|launch| {
        let mut launch = launch.borrow_mut();
        launch.handed = Some(path);
        launch.by_command_line = true;
    });
}

// realises FR-476, FR-477, IR-207
/// Takes the file-open event of the operating system carrying `path`; the handler installed by
/// 'main.rs' with `QApp::on_file_open`, called on the main thread from the event loop.
///
/// * Before the *front end* is up, the file is the *handed file* (`hand_file`).
/// * In the *start-up phase*, `openHandedFile` of the *workbench object* is scheduled with it.
/// * After the *start-up phase*, a second *product* is started on it (`launch::start_second`);
///   where that fails, the reason is written to standard error.
pub fn file_opened(path: String) {
    LAUNCH.with(|launch| {
        let mut launch = launch.borrow_mut();
        match &launch.invoker {
            None => launch.handed = Some(path),
            Some(invoker) if launch.in_start_up_phase() => {
                invoke_method!(invoker, "openHandedFile", path);
            }
            Some(_) => {
                if let Err(reason) = launch::start_second(&path) {
                    eprintln!(
                        "genc3wb: a second genc3wb could not be started on '{path}': {reason}"
                    );
                }
            }
        }
    });
}

// realises FR-480
/// Schedules `announceSettingsFailure` of the *workbench object*, where the *front end* is up,
/// so that the status bar follows a store of the settings made by another *bridged type*.
pub fn settings_stored() {
    LAUNCH.with(|launch| {
        if let Some(invoker) = &launch.borrow().invoker {
            invoke_method!(invoker, "announceSettingsFailure");
        }
    });
}

// realises FR-001, FR-063, FR-090, FR-095, FR-099, FR-140, FR-141, FR-149, FR-192, FR-277,
// FR-476, FR-480
/// The *workbench object*.
pub struct Workbench {
    /// the settings with the *settings file*, shared with the groups
    settings: Rc<RefCell<SettingsStore>>,
    /// the *document registry*, shared with every *input group* of a *node window* (FR-277)
    documents: Rc<RefCell<DocumentRegistry>>,
    /// the *input group* of the *network file*
    network_input: Rc<RefCell<InputGroup>>,
    /// the group for the *network graph*
    network: Rc<RefCell<NetworkEditor>>,
    /// the *typing mode*: whether typing inserts, and overwrites otherwise (FR-192)
    insert_mode: bool,
    /// the failure of the last write of the *settings file* as announced last, so that the
    /// signal is emitted where it changed (FR-480)
    announced_failure: String,
}

impl Default for Workbench {
    /// Restores the settings and takes the *handed file* left by `hand_file`, creates the
    /// *document registry*, the network editor and the *input group* of the *network file*, and
    /// wires them: the network editor with the settings, the *document registry* and that
    /// *input group*, and that *input group* with the network editor, both with their paths, the
    /// *handed file* in place of the stored path of the *network file* where there is one; the
    /// network editor creates the *node editors*.
    fn default() -> Self {
        let settings = Rc::new(RefCell::new(SettingsStore::open(&Settings::default_path())));
        let handed = LAUNCH.with(|launch| launch.borrow_mut().handed.take());
        let documents = Rc::new(RefCell::new(DocumentRegistry::new()));
        let network = NetworkEditor::default_with_attached_qobject();
        let network_input = InputGroup::default_with_attached_qobject();
        network.borrow_mut().configure(
            Rc::clone(&settings),
            Rc::clone(&documents),
            network_input.borrow().get_qml_method_invoker(),
        );
        network_input.borrow_mut().configure(
            InputKind::Network,
            Some(Rc::clone(&settings)),
            Some(network.borrow().get_qml_method_invoker()),
            None,
        );
        network.borrow_mut().load_initial();
        network_input.borrow_mut().load_initial(handed.as_deref());
        Workbench {
            settings,
            documents,
            network_input,
            network,
            insert_mode: true,
            announced_failure: String::new(),
        }
    }
}

// realises FR-001, FR-063, FR-095, FR-099, FR-130 to FR-135, FR-140, FR-141, FR-146 to FR-149,
// FR-192 to FR-194, FR-198 to FR-200, FR-214, FR-221, FR-224, FR-255 to FR-257, FR-476, FR-477,
// FR-480
#[qobject(Singleton)]
impl Workbench {
    qproperty!("networkInput", Read = network_input, Constant);
    qproperty!("network", Read = network, Constant);
    qproperty!("idleTime", Read = idle_time, Notify = times_changed);
    qproperty!(
        "longIdleTime",
        Read = long_idle_time,
        Notify = times_changed
    );
    qproperty!("automatic", Read = automatic, Notify = times_changed);
    // the typing mode: whether typing inserts, and overwrites otherwise (FR-192)
    qproperty!(
        "insertMode",
        Read = insert_mode,
        Write = set_insert_mode,
        Notify = insert_mode_changed
    );
    // the name of the insert key (FR-198, FR-199)
    qproperty!(
        "insertKeyName",
        Read = insert_key_name,
        Notify = insert_key_changed
    );
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
    // the reason of the last failed write of the settings file, empty where it succeeded (FR-480)
    qproperty!(
        "settingsFailure",
        Read = settings_failure,
        Notify = settings_failure_changed
    );
    // the version of the product in its four parts (FR-224)
    qproperty!("version", Read = version, Constant);

    fn network_input(&self) -> Rc<RefCell<InputGroup>> {
        Rc::clone(&self.network_input)
    }

    fn network(&self) -> Rc<RefCell<NetworkEditor>> {
        Rc::clone(&self.network)
    }

    // realises FR-192
    fn insert_mode(&self) -> bool {
        self.insert_mode
    }

    // realises FR-198, FR-199
    fn insert_key_name(&self) -> String {
        self.settings.borrow().insert_key().name()
    }

    // realises FR-192, FR-194
    // Sets the typing mode and emits `insert_mode_changed` where it changed.
    fn set_insert_mode(&mut self, insert_mode: bool) {
        if insert_mode == self.insert_mode {
            return;
        }
        self.insert_mode = insert_mode;
        self.insert_mode_changed();
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

    // realises FR-480
    fn settings_failure(&self) -> String {
        self.settings
            .borrow()
            .failure()
            .map(ToOwned::to_owned)
            .unwrap_or_default()
    }

    // realises FR-224: the version the package was compiled as
    fn version(&self) -> String {
        version_of_package(env!("CARGO_PKG_VERSION"))
    }

    // signals
    #[qsignal(qml_name = "timesChanged")]
    fn times_changed(&mut self);

    #[qsignal(qml_name = "tabSizeChanged")]
    fn tab_size_changed(&mut self);

    #[qsignal(qml_name = "splittersChanged")]
    fn splitters_changed(&mut self);

    #[qsignal(qml_name = "settingsFailureChanged")]
    fn settings_failure_changed(&mut self);

    #[qsignal(qml_name = "settingsRejected")]
    fn settings_rejected(&mut self, message: String);

    #[qsignal(qml_name = "insertModeChanged")]
    fn insert_mode_changed(&mut self);

    #[qsignal(qml_name = "insertKeyChanged")]
    fn insert_key_changed(&mut self);

    // realises FR-193
    /// Toggles the *typing mode* where the key event of `key` with `modifiers` is the insert
    /// key, and yields whether it is.
    #[qslot(qml_name = "toggledByKey")]
    fn toggled_by_key(&mut self, key: i32, modifiers: i32) -> bool {
        let toggled = self
            .settings
            .borrow()
            .insert_key()
            .matches(bits_of(key), bits_of(modifiers));
        if toggled {
            self.set_insert_mode(!self.insert_mode);
        }
        toggled
    }

    // realises FR-199
    /// Yields the name of the key stroke of `key` with `modifiers`, as `KeyStroke::name` does;
    /// empty where `key` is a modifier itself, which is no key stroke.
    #[qslot(qml_name = "keyName")]
    fn key_name(&self, key: i32, modifiers: i32) -> String {
        if KeyStroke::is_modifier_key(bits_of(key)) {
            return String::new();
        }
        KeyStroke::new(bits_of(key), bits_of(modifiers)).name()
    }

    // realises FR-199, FR-200
    /// Takes the key stroke of `key` with `modifiers` as the insert key, stores the settings,
    /// and emits `insert_key_changed`; a modifier itself is passed over.
    #[qslot(qml_name = "setInsertKey")]
    fn set_insert_key(&mut self, key: i32, modifiers: i32) {
        if KeyStroke::is_modifier_key(bits_of(key)) {
            return;
        }
        self.take_insert_key(KeyStroke::new(bits_of(key), bits_of(modifiers)));
    }

    // realises FR-198, FR-199
    /// Takes the default insert key as the insert key, stores the settings, and emits
    /// `insert_key_changed`.
    #[qslot(qml_name = "resetInsertKey")]
    fn reset_insert_key(&mut self) {
        self.take_insert_key(KeyStroke::default_insert_key());
    }

    // slots
    // realises FR-099, FR-100, FR-132, FR-133, FR-134, FR-135, FR-140, FR-141, FR-255, FR-257
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

    // realises FR-214, FR-221
    /// Releases and removes the *lock file* of the *network file*; called by the *front end*
    /// when the main window closes, since *product* terminates then.
    #[qslot(qml_name = "releaseLock")]
    fn release_lock(&mut self) {
        self.network_input.borrow_mut().release_lock();
    }

    // realises FR-476, FR-477, IR-207
    /// Keeps the invoker of the *workbench object* in the state of the module, so that
    /// `file_opened` reaches `open_handed_file` in the *start-up phase* and `settings_stored`
    /// reaches `announce_settings_failure`, and starts the *start-up phase*, which lasts
    /// `START_UP_GRACE` from now, or ends at once where the command line handed the file, since
    /// no file-open event of the launch follows then; called by the *front end* once the main
    /// window is completed. A file the operating system handed over before is named then.
    #[qslot(qml_name = "started")]
    fn started(&mut self) {
        let invoker = self.get_qml_method_invoker();
        let handed = LAUNCH.with(|launch| {
            let mut launch = launch.borrow_mut();
            launch.invoker = Some(invoker);
            let now = Instant::now();
            launch.phase_end = Some(if launch.by_command_line {
                now
            } else {
                now + START_UP_GRACE
            });
            launch.handed.take()
        });
        if let Some(path) = handed {
            self.open_handed_file(path);
        }
    }

    // realises FR-476
    /// Names `path` as the *handed file*: the *network file* of the session, its path not stored;
    /// schedules `nameHanded` of the *input group* of the *network file* by its invoker, since
    /// that group emits its signals in the call and the *front end* reads its properties then,
    /// which a borrow from here would refuse. Scheduled by `file_opened` in the *start-up phase*.
    #[qslot(qml_name = "openHandedFile")]
    fn open_handed_file(&mut self, path: String) {
        let invoker = self.network_input.borrow().get_qml_method_invoker();
        invoke_method!(invoker, "nameHanded", path);
    }

    // realises FR-480
    /// Emits `settings_failure_changed` where the failure of the last write of the *settings
    /// file* differs from the one announced last; scheduled by `settings_stored` after a store
    /// made by another *bridged type*.
    #[qslot(qml_name = "announceSettingsFailure")]
    fn announce_settings_failure(&mut self) {
        let failure = self.settings_failure();
        if failure != self.announced_failure {
            self.announced_failure = failure;
            self.settings_failure_changed();
        }
    }

    // realises FR-129, FR-130, FR-256
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
    // realises FR-479, FR-480, IR-012
    /// Writes the *settings file*; a failure leaves the settings of the session as they are and
    /// is announced where it differs from the one announced last.
    fn store(&mut self) {
        let _ = self.settings.borrow_mut().store();
        self.announce_settings_failure();
    }

    // realises FR-199, FR-200
    /// Sets the insert key where it is another, stores the settings, and emits
    /// `insert_key_changed`.
    fn take_insert_key(&mut self, insert_key: KeyStroke) {
        if insert_key == self.settings.borrow().insert_key() {
            return;
        }
        self.settings.borrow_mut().set_insert_key(insert_key);
        self.store();
        self.insert_key_changed();
    }

    /// Yields the settings the groups share.
    pub fn settings(&self) -> Rc<RefCell<SettingsStore>> {
        Rc::clone(&self.settings)
    }

    // realises FR-277
    /// Yields the *document registry* the *input groups* of the *node windows* share.
    pub fn documents(&self) -> Rc<RefCell<DocumentRegistry>> {
        Rc::clone(&self.documents)
    }
}

/// Yields `value`, a key code or the modifiers of a key event as QML carries them, as its bits.
fn bits_of(value: i32) -> u32 {
    u32::from_ne_bytes(value.to_ne_bytes())
}
