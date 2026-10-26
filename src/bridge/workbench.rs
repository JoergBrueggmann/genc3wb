//! The *bridged type* `Workbench`: the *workbench object*, owner of the settings and the groups;
//! and the *handed file* and the file-open event of the operating system, as functions of the
//! module.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::bridge::automation::Automation;
use crate::bridge::document_registry::DocumentRegistry;
use crate::bridge::input_group::InputGroup;
use crate::bridge::network_editor::NetworkEditor;
use crate::core::automation::{Action, EditorName, SettingsValues};
use crate::core::input_file::InputKind;
use crate::core::key_stroke::KeyStroke;
use crate::core::launch;
use crate::core::recording::{AUTOMATIC_FILE_NAME, Recorder, local_time_name, start_folder};
use crate::core::settings::{
    PLAYBACK_DELAYS, Settings, SettingsStore, tab_size_violation, times_of_processing,
};
use crate::core::version::version_of_package;

use qtbridge::{QmlMethodInvoker, QmlObject, invoke_method, qobject};

use std::cell::{Cell, RefCell};
use std::fs;
use std::path::{Path, PathBuf};
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
    /// the gc3s-file of the switch `--verify` not taken by the *workbench object* yet (FR-517)
    verify: Option<String>,
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

// realises FR-517, IR-206
/// Leaves `path` as the gc3s-file of a *verification run* for the *workbench object*, which plays
/// it once the main window is up; called by 'main.rs' before the *front end* is loaded. No
/// file-open event of the launch is expected then.
pub fn verify_script(path: String) {
    LAUNCH.with(|launch| {
        let mut launch = launch.borrow_mut();
        launch.verify = Some(path);
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

// realises FR-480, FR-489
/// Schedules `announceSettingsStored` of the *workbench object*, where the *front end* is up,
/// so that the status bar and the settings dialog follow a store of the settings made by another
/// *bridged type*.
// realises FR-540
/// Yields whether the *typing mode* is the insert mode, as the *workbench object* holds it; read
/// by the network editor for the state an *individual recording* finds.
pub fn insert_mode() -> bool {
    INSERT_MODE.with(Cell::get)
}

thread_local! {
    /// the *typing mode* of the *workbench object*, readable without it (FR-540)
    static INSERT_MODE: Cell<bool> = const { Cell::new(true) };
}

// realises IR-219
/// Yields the path at which the *settings file* at `settings` is kept aside during a *playback*.
fn aside_path(settings: &Path) -> PathBuf {
    let mut path = settings.as_os_str().to_owned();
    path.push(".aside");
    PathBuf::from(path)
}

pub fn settings_stored() {
    LAUNCH.with(|launch| {
        if let Some(invoker) = &launch.borrow().invoker {
            invoke_method!(invoker, "announceSettingsStored");
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
    /// the *player* (FR-500)
    automation: Rc<RefCell<Automation>>,
    /// the *typing mode*: whether typing inserts, and overwrites otherwise (FR-192)
    insert_mode: bool,
    /// the failure of the last write of the *settings file* as announced last, so that the
    /// signal is emitted where it changed (FR-480)
    announced_failure: String,
    /// the *recorder*, shared with every *bridged type* that takes an action of the user
    recorder: Rc<RefCell<Recorder>>,
}

impl Default for Workbench {
    /// Restores the settings and takes the *handed file* left by `hand_file`, starts the
    /// *automatic recording* before any *bridged type* is created, so that nothing one of them
    /// names at its construction escapes the *recorder* (FR-520), creates the
    /// *document registry*, the network editor and the *input group* of the *network file*, and
    /// wires them: the network editor with the settings, the *document registry* and that
    /// *input group*, and that *input group* with the network editor, both with their paths, the
    /// *handed file* in place of the stored path of the *network file* where there is one, and
    /// no *network file* at all for a *verification run* (FR-517); the network editor creates the
    /// *node editors*; the *player* with the settings, the network editor, that *input group* and
    /// the gc3s-file of the switch.
    fn default() -> Self {
        let settings = Rc::new(RefCell::new(SettingsStore::open(&Settings::default_path())));
        let (handed, verify) = LAUNCH.with(|launch| {
            let mut launch = launch.borrow_mut();
            (launch.handed.take(), launch.verify.take())
        });
        let handed = if verify.is_some() {
            Some(String::new())
        } else {
            handed
        };
        let documents = Rc::new(RefCell::new(DocumentRegistry::new()));
        let recorder = Rc::new(RefCell::new(Recorder::default()));
        // the automatic recording of every start but a verification run, started before any
        // bridged type is created, so that nothing one of them names at its construction
        // escapes it (FR-520)
        if verify.is_none() {
            let time = local_time_name();
            let path = start_folder(&launch::executable_folder(), &time).join(AUTOMATIC_FILE_NAME);
            let _ = recorder.borrow_mut().start(
                &path,
                &version_of_package(env!("CARGO_PKG_VERSION")),
                &time,
                &Settings::default_path(),
            );
        }
        let network = NetworkEditor::default_with_attached_qobject();
        let network_input = InputGroup::default_with_attached_qobject();
        network.borrow_mut().configure(
            Rc::clone(&settings),
            Rc::clone(&documents),
            network_input.borrow().get_qml_method_invoker(),
            Rc::clone(&recorder),
        );
        network_input.borrow_mut().configure(
            InputKind::Network,
            Some(Rc::clone(&settings)),
            Some(network.borrow().get_qml_method_invoker()),
            None,
            Some(Rc::clone(&recorder)),
        );
        network_input
            .borrow_mut()
            .set_editor_name(Some(EditorName::Network));
        network
            .borrow_mut()
            .set_network_group(Rc::clone(&network_input));
        network.borrow_mut().load_initial();
        network_input.borrow_mut().load_initial(handed.as_deref());
        let automation = Automation::default_with_attached_qobject();
        automation.borrow_mut().configure(
            Rc::clone(&settings),
            Rc::clone(&network),
            Rc::clone(&network_input),
            Rc::clone(&recorder),
            verify.clone(),
        );
        Workbench {
            settings,
            documents,
            network_input,
            network,
            automation,
            insert_mode: true,
            announced_failure: String::new(),
            recorder,
        }
    }
}

// realises FR-001, FR-063, FR-095, FR-099, FR-130 to FR-135, FR-140, FR-141, FR-146 to FR-149,
// FR-192 to FR-194, FR-198 to FR-200, FR-214, FR-221, FR-224, FR-255 to FR-257, FR-476, FR-477,
// FR-480, FR-489, FR-490, FR-492
#[qobject(Singleton)]
impl Workbench {
    qproperty!("networkInput", Read = network_input, Constant);
    qproperty!("network", Read = network, Constant);
    // the player (FR-500)
    qproperty!("automation", Read = automation, Constant);
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
    // the spaces setting: whether the Tab key inserts spaces (FR-492)
    qproperty!(
        "tabsToSpaces",
        Read = tabs_to_spaces,
        Notify = tabs_to_spaces_changed
    );
    // the silent creation setting: whether the files of a node are created without the dialog
    // (FR-490)
    qproperty!(
        "silentCreation",
        Read = silent_creation,
        Notify = silent_creation_changed
    );
    // the playback delay in milliseconds, 0 while none is waited, whether one is waited, and the
    // index of the value chosen last among the thirteen (FR-518)
    qproperty!(
        "playbackDelay",
        Read = playback_delay,
        Notify = playback_delay_changed
    );
    qproperty!(
        "playbackDelayOn",
        Read = playback_delay_on,
        Notify = playback_delay_changed
    );
    qproperty!(
        "playbackDelayIndex",
        Read = playback_delay_index,
        Notify = playback_delay_changed
    );
    // the playback delay dialog setting: whether no playback delay dialog is presented (FR-544)
    qproperty!(
        "playbackDelayDialogDisabled",
        Read = playback_delay_dialog_disabled,
        Notify = playback_delay_dialog_disabled_changed
    );
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

    // realises FR-500
    fn automation(&self) -> Rc<RefCell<Automation>> {
        Rc::clone(&self.automation)
    }

    // realises FR-095, FR-518
    fn playback_delay(&self) -> u32 {
        self.settings.borrow().playback_delay()
    }

    // realises FR-095, FR-518, FR-542
    fn playback_delay_on(&self) -> bool {
        self.settings.borrow().playback_delay_on()
    }

    fn playback_delay_index(&self) -> i32 {
        i32::try_from(self.settings.borrow().playback_delay_index()).unwrap_or(0)
    }

    // realises FR-095, FR-544
    fn playback_delay_dialog_disabled(&self) -> bool {
        self.settings.borrow().playback_delay_dialog_disabled()
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
        INSERT_MODE.with(|mode| mode.set(insert_mode));
        self.recorder
            .borrow_mut()
            .record(&Action::TypingMode(insert_mode));
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

    // realises FR-095, FR-492
    fn tabs_to_spaces(&self) -> bool {
        self.settings.borrow().tabs_to_spaces()
    }

    // realises FR-095, FR-490
    fn silent_creation(&self) -> bool {
        self.settings.borrow().silent_creation()
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

    #[qsignal(qml_name = "tabsToSpacesChanged")]
    fn tabs_to_spaces_changed(&mut self);

    #[qsignal(qml_name = "silentCreationChanged")]
    fn silent_creation_changed(&mut self);

    #[qsignal(qml_name = "playbackDelayChanged")]
    fn playback_delay_changed(&mut self);

    #[qsignal(qml_name = "playbackDelayDialogDisabledChanged")]
    fn playback_delay_dialog_disabled_changed(&mut self);

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

    // realises FR-509
    /// Takes the key stroke named `name` as the insert key, as `set_insert_key` does; a name that
    /// names none sets nothing, the *player* having checked the name before. Scheduled by the
    /// *player*; it yields nothing, since a slot scheduled by an invoker yields nothing.
    #[qslot(qml_name = "setInsertKeyNamed")]
    fn set_insert_key_named(&mut self, name: String) {
        if let Some(insert_key) = KeyStroke::of_name(&name) {
            self.take_insert_key(insert_key);
        }
    }

    // realises FR-509
    /// Sets the *typing mode*, as `set_insert_mode` does; scheduled by the *player*.
    #[qslot(qml_name = "setTypingMode")]
    fn set_typing_mode(&mut self, insert: bool) {
        self.set_insert_mode(insert);
    }

    // realises FR-509, FR-543
    /// Sets the settings as `try_set_settings` does, the *player* having checked the constraints
    /// before, the *playback delay* and the *playback delay dialog setting* left as they are,
    /// since the delay the *playback* was started with holds for the whole *playback*;
    /// scheduled by the *player*, which is why it yields nothing.
    #[qslot(qml_name = "applySettings")]
    fn apply_settings(
        &mut self,
        idle_time: i32,
        long_idle_time: i32,
        automatic: bool,
        tab_size: i32,
        tabs_to_spaces: bool,
        silent_creation: bool,
    ) {
        let _ = self.try_set_settings(
            idle_time,
            long_idle_time,
            automatic,
            tab_size,
            tabs_to_spaces,
            silent_creation,
            self.playback_delay_on(),
            self.playback_delay_index(),
            self.playback_delay_dialog_disabled(),
        );
    }

    // realises FR-098, FR-518, FR-542
    /// Yields the value of the *playback delay* with the index `index` among the thirteen, in
    /// milliseconds, 0 for an index that names none; called by the fields of the two dialogs
    /// to show the value of their position.
    #[qslot(qml_name = "playbackDelayOf")]
    fn playback_delay_of(&self, index: i32) -> i32 {
        usize::try_from(index)
            .ok()
            .and_then(|index| PLAYBACK_DELAYS.get(index))
            .and_then(|value| i32::try_from(*value).ok())
            .unwrap_or(0)
    }

    // realises FR-543, IR-012, IR-220
    /// Sets the *playback delay*, whether one is waited and the index of its value, as the
    /// *playback delay dialog* accepts it, switches the *playback delay dialog setting* on where
    /// `disable_dialog`, and stores them; called by the *front end* when the dialog is accepted.
    /// An index that names no value sets nothing, the field of the dialog admitting none.
    #[qslot(qml_name = "setPlaybackDelay")]
    fn set_playback_delay(&mut self, on: bool, index: i32, disable_dialog: bool) {
        let Ok(index) = usize::try_from(index) else {
            return;
        };
        let changed_playback_delay =
            self.playback_delay_on() != on || self.playback_delay_index() != index as i32;
        let changed_disabled = disable_dialog && !self.playback_delay_dialog_disabled();
        {
            let mut settings = self.settings.borrow_mut();
            if settings.set_playback_delay(on, index).is_err() {
                return;
            }
            if disable_dialog {
                settings.set_playback_delay_dialog_disabled(true);
            }
        }
        self.store();
        if changed_playback_delay {
            self.playback_delay_changed();
        }
        if changed_disabled {
            self.playback_delay_dialog_disabled_changed();
        }
    }

    // realises FR-537, IR-219
    /// Keeps the *settings file* aside, writes the saved file at `path` in its place and applies
    /// the settings it holds, the paths of FR-090 excepted; scheduled by the *player*, which
    /// checked that the file is read.
    #[qslot(qml_name = "applySettingsFile")]
    fn apply_settings_file(&mut self, path: String) {
        let target = Settings::default_path();
        let aside = aside_path(&target);
        if target.is_file() && !aside.exists() {
            let _ = fs::copy(&target, &aside);
        }
        if fs::copy(&path, &target).is_ok() {
            self.reload_settings();
        }
    }

    // realises FR-537, IR-219
    /// Writes the *settings file* kept aside back and applies it, so that the settings of the
    /// user who ran the script are as before; scheduled by the *player* when the *playback*
    /// ends.
    #[qslot(qml_name = "restoreSettingsFile")]
    fn restore_settings_file(&mut self) {
        let target = Settings::default_path();
        let aside = aside_path(&target);
        if aside.is_file() && fs::copy(&aside, &target).is_ok() {
            let _ = fs::remove_file(&aside);
            self.reload_settings();
        }
    }

    // realises FR-537, FR-543
    /// Applies the *settings file* as it stands: the times, the *automatic setting*, the
    /// *tab size*, the *spaces setting*, the *silent creation setting* and the *insert key*,
    /// each as the user's change would, the paths, the *playback delay* and the *playback delay
    /// dialog setting* left as they are, since the delay the *playback* was started with holds
    /// for the whole *playback*.
    fn reload_settings(&mut self) {
        let Ok(loaded) = Settings::load(&Settings::default_path()) else {
            return;
        };
        let idle_time = i32::try_from(loaded.idle_time()).unwrap_or(i32::MAX);
        let long_idle_time = i32::try_from(loaded.long_idle_time()).unwrap_or(i32::MAX);
        let tab_size = i32::try_from(loaded.tab_size()).unwrap_or(i32::MAX);
        let _ = self.try_set_settings(
            idle_time,
            long_idle_time,
            loaded.automatic(),
            tab_size,
            loaded.tabs_to_spaces(),
            loaded.silent_creation(),
            self.playback_delay_on(),
            self.playback_delay_index(),
            self.playback_delay_dialog_disabled(),
        );
        self.set_insert_key_named(loaded.insert_key().name());
    }

    // slots
    // realises FR-099, FR-100, FR-132, FR-133, FR-134, FR-135, FR-140, FR-141, FR-255, FR-257,
    // FR-490, FR-492, FR-518, FR-544
    /// Sets the two times of every code editor, in milliseconds, the *automatic setting*, the
    /// *tab size*, in characters, the *spaces setting*, the *silent creation setting*, the
    /// *playback delay*, whether one is waited and the index of its value among the thirteen,
    /// and the *playback delay dialog setting*, and stores them; yields whether the values
    /// satisfy the constraints, and emits `settings_rejected` with the message naming the
    /// violated one where they do not, nothing being set then.
    ///
    /// * The *tab size* is checked first, then the index of the delay, then the two times.
    /// * With the *automatic setting* on, the times are left as they are, since the next
    ///   *processing time* sets them (FR-132).
    #[qslot(qml_name = "trySetSettings")]
    #[allow(clippy::too_many_arguments)] // the values of the settings dialog, as the front end hands them
    fn try_set_settings(
        &mut self,
        idle_time: i32,
        long_idle_time: i32,
        automatic: bool,
        tab_size: i32,
        tabs_to_spaces: bool,
        silent_creation: bool,
        playback_delay_on: bool,
        playback_delay_index: i32,
        playback_delay_dialog_disabled: bool,
    ) -> bool {
        let idle_time = u32::try_from(idle_time).unwrap_or(0);
        let long_idle_time = u32::try_from(long_idle_time).unwrap_or(0);
        let tab_size = u32::try_from(tab_size).unwrap_or(0);
        let playback_delay_index = usize::try_from(playback_delay_index).unwrap_or(usize::MAX);
        if let Some(message) = tab_size_violation(tab_size) {
            self.settings_rejected(message);
            return false;
        }
        if let Err(error) =
            Settings::default().set_playback_delay(playback_delay_on, playback_delay_index)
        {
            self.settings_rejected(error.to_string());
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
        let changed_tabs_to_spaces = self.tabs_to_spaces() != tabs_to_spaces;
        let changed_silent_creation = self.silent_creation() != silent_creation;
        let changed_playback_delay = self.playback_delay_on() != playback_delay_on
            || self.playback_delay_index() != playback_delay_index as i32;
        let changed_dialog_disabled =
            self.playback_delay_dialog_disabled() != playback_delay_dialog_disabled;
        {
            let mut settings = self.settings.borrow_mut();
            settings.set_automatic(automatic);
            let _ = settings.set_tab_size(tab_size);
            settings.set_tabs_to_spaces(tabs_to_spaces);
            settings.set_silent_creation(silent_creation);
            let _ = settings.set_playback_delay(playback_delay_on, playback_delay_index);
            settings.set_playback_delay_dialog_disabled(playback_delay_dialog_disabled);
        }
        self.store();
        self.times_changed();
        if changed_tab_size {
            self.tab_size_changed();
        }
        if changed_tabs_to_spaces {
            self.tabs_to_spaces_changed();
        }
        if changed_silent_creation {
            self.silent_creation_changed();
        }
        if changed_playback_delay {
            self.playback_delay_changed();
        }
        if changed_dialog_disabled {
            self.playback_delay_dialog_disabled_changed();
        }
        self.recorder
            .borrow_mut()
            .record(&Action::Settings(SettingsValues {
                idle_time,
                long_idle_time,
                automatic,
                tab_size,
                tabs_to_spaces,
                silent_creation,
                playback_delay: playback_delay_on.then(|| PLAYBACK_DELAYS[playback_delay_index]),
            }));
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

    // realises FR-476, FR-477, FR-517, IR-207
    /// Keeps the invoker of the *workbench object* in the state of the module, so that
    /// `file_opened` reaches `open_handed_file` in the *start-up phase* and `settings_stored`
    /// reaches `announce_settings_stored`, and starts the *start-up phase*, which lasts
    /// `START_UP_GRACE` from now, or ends at once where the command line handed the file, since
    /// no file-open event of the launch follows then; called by the *front end* once the main
    /// window is completed. A file the operating system handed over before is named then. The
    /// *player* takes the invoker as well, and `startVerification` of it is scheduled, so that a
    /// *verification run* begins once the main window is up.
    #[qslot(qml_name = "started")]
    fn started(&mut self) {
        let invoker = self.get_qml_method_invoker();
        self.automation
            .borrow_mut()
            .take_workbench_invoker(self.get_qml_method_invoker());
        let automation_invoker = self.automation.borrow().get_qml_method_invoker();
        invoke_method!(&automation_invoker, "startVerification");
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

    // realises FR-480, FR-489
    /// Takes a store of the settings made by another *bridged type*: emits
    /// `silent_creation_changed`, since the network editor switches that setting on from the
    /// *input creation confirmation dialog* (FR-489), and announces the failure of the write
    /// (`announce_settings_failure`). Scheduled by `settings_stored`.
    #[qslot(qml_name = "announceSettingsStored")]
    fn announce_settings_stored(&mut self) {
        self.silent_creation_changed();
        self.announce_settings_failure();
    }

    // realises FR-480
    /// Emits `settings_failure_changed` where the failure of the last write of the *settings
    /// file* differs from the one announced last.
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
        self.recorder
            .borrow_mut()
            .record(&Action::InsertKey(insert_key.name()));
        self.settings.borrow_mut().set_insert_key(insert_key);
        self.store();
        self.insert_key_changed();
    }

    /// Yields the settings the groups share.
    pub fn settings(&self) -> Rc<RefCell<SettingsStore>> {
        Rc::clone(&self.settings)
    }

    /// Yields the *recorder*.
    pub fn recorder(&self) -> Rc<RefCell<Recorder>> {
        Rc::clone(&self.recorder)
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
