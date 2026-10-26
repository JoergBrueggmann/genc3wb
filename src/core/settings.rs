//! The *settings file*: the two times, the *automatic setting*, the *tab size*, the *spaces
//! setting*, the *silent creation setting*, the positions of the splitters and the paths of
//! *product*, in YAML.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::key_stroke::KeyStroke;
use crate::core::launch::executable_folder;

use serde::{Deserialize, Serialize};

use std::fmt;
use std::fs;
use std::ops::{Deref, DerefMut};
use std::path::{Path, PathBuf};

// realises IR-011, IR-012
/// The name of the *settings file* in the working directory of *product*.
pub const SETTINGS_FILE_NAME: &str = "genc3wb.yaml";

// realises FR-094
/// The *idle time* in milliseconds where the *settings file* does not hold one.
pub const DEFAULT_IDLE_TIME: u32 = 2000;
/// The *long idle time* in milliseconds where the *settings file* does not hold one.
pub const DEFAULT_LONG_IDLE_TIME: u32 = 16000;
/// Whether the *automatic setting* is on where the *settings file* does not hold it.
pub const DEFAULT_AUTOMATIC: bool = true;

// realises FR-098, FR-130, FR-133
/// The step of the two times in the settings dialog and of the rounding of the *automatic
/// setting*, in milliseconds: 0.2 seconds.
pub const TIME_STEP: u32 = 200;
/// The least *idle time*, in milliseconds: 0.2 seconds.
pub const MIN_IDLE_TIME: u32 = TIME_STEP;
/// The name of the *build system* in the folder of the executable, which is its path where the
/// *settings file* does not hold one (`Settings::default_build_system_path`).
pub const DEFAULT_BUILD_SYSTEM_NAME: &str = "genc3d";

// realises FR-140
/// The *tab size* where the *settings file* does not hold one.
pub const DEFAULT_TAB_SIZE: u32 = 4;
/// The least *tab size*.
pub const MIN_TAB_SIZE: u32 = 1;
/// The greatest *tab size*.
pub const MAX_TAB_SIZE: u32 = 16;

// realises FR-094, FR-492
/// Whether the *spaces setting* is on where the *settings file* does not hold it.
pub const DEFAULT_TABS_TO_SPACES: bool = false;

// realises FR-094, FR-490
/// Whether the *silent creation setting* is on where the *settings file* does not hold it.
pub const DEFAULT_SILENT_CREATION: bool = false;

// realises FR-094, FR-518
/// The thirteen values a *playback delay* may have, in milliseconds: 0.05 · √2ⁿ seconds for n
/// from 0 to 12, each rounded to the millisecond; and whether a delay is waited and which, where
/// the *settings file* holds none.
pub const PLAYBACK_DELAYS: [u32; 13] = [
    50, 71, 100, 141, 200, 283, 400, 566, 800, 1131, 1600, 2263, 3200,
];
pub const DEFAULT_PLAYBACK_DELAY_ON: bool = false;
pub const DEFAULT_PLAYBACK_DELAY_INDEX: usize = 0;

// realises FR-094, FR-544
/// Whether the *playback delay dialog setting* is on where the *settings file* does not hold it.
pub const DEFAULT_PLAYBACK_DELAY_DIALOG_DISABLED: bool = false;

// realises FR-149
/// The width of the left part of the *node window*, in pixels, where the *settings file* does not
/// hold one (FR-146).
pub const DEFAULT_NODE_LEFT_WIDTH: u32 = 560;
/// The height of the *input group* in the left part of the *node window*, in pixels, where the
/// *settings file* does not hold one (FR-147).
pub const DEFAULT_NODE_UPPER_HEIGHT: u32 = 340;
/// The width of the *input group* of the *network file* in the *compiler network editor*, in
/// pixels, where the *settings file* does not hold one (FR-148).
pub const DEFAULT_NETWORK_LEFT_WIDTH: u32 = 480;

// realises IR-011, IR-012
/// Why the settings could not be restored or stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsError {
    /// the *settings file* could not be read or written; carries the reason of the operating system
    Io(String),
    /// the *settings file* is no YAML mapping of the keys of [`Settings`]; carries the reason
    Malformed(String),
    /// the two times violate FR-133 or FR-134, or the *tab size* FR-140; carries the message
    /// naming the constraint
    Constraint(String),
}

impl fmt::Display for SettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingsError::Io(reason) => write!(formatter, "settings file: {reason}"),
            SettingsError::Malformed(reason) => {
                write!(formatter, "settings file is not read: {reason}")
            }
            SettingsError::Constraint(message) => write!(formatter, "{message}"),
        }
    }
}

impl std::error::Error for SettingsError {}

// realises FR-090, FR-091, FR-093, FR-094, FR-133, FR-134, FR-140, FR-149, FR-490, FR-492
/// The *idle time*, the *long idle time*, the *automatic setting*, the *tab size*, the *spaces
/// setting*, the *silent creation setting*, the positions of the splitters and the paths
/// *product* restores between sessions.
///
/// * The two times are held in milliseconds, as the code editors take them; the *settings file*
///   and the settings dialog carry them in seconds with one decimal.
/// * The two times satisfy FR-133 and FR-134: the *idle time* is at least `MIN_IDLE_TIME`, and
///   the *long idle time* is greater than the *idle time*.
/// * The *tab size* lies in `MIN_TAB_SIZE..=MAX_TAB_SIZE` (FR-140).
/// * The position of a splitter is the size, in pixels, of the part before it: the left or the
///   upper one.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// the *idle time* in milliseconds
    idle_time: u32,
    /// the *long idle time* in milliseconds
    long_idle_time: u32,
    /// whether the *automatic setting* is on
    automatic: bool,
    /// the *tab size* of every code editor, in characters
    tab_size: u32,
    /// whether the *spaces setting* is on: the Tab key inserts spaces (FR-492)
    tabs_to_spaces: bool,
    /// whether the *silent creation setting* is on: the files of a *node* are created without
    /// the *input creation confirmation dialog* (FR-490)
    silent_creation: bool,
    /// whether a *playback delay* is waited, and the index of its value in `PLAYBACK_DELAYS`,
    /// kept while none is waited, so that the field shows the value last chosen (FR-518)
    playback_delay_on: bool,
    playback_delay_index: usize,
    /// whether the *playback delay dialog setting* is on: no *playback delay dialog* is presented
    /// (FR-544)
    playback_delay_dialog_disabled: bool,
    /// the width of the left part of the *node window* (FR-146)
    node_left_width: u32,
    /// the height of the *input group* in the left part of the *node window* (FR-147)
    node_upper_height: u32,
    /// the width of the *input group* of the *network file* (FR-148)
    network_left_width: u32,
    /// the path of the *network file*; `None` where none is named (FR-090, FR-535)
    network_path: Option<String>,
    /// the path of the *build system*
    build_system_path: String,
    /// the insert key, which toggles the *typing mode* (FR-193)
    insert_key: KeyStroke,
}

impl Default for Settings {
    // realises FR-094, FR-140, FR-149
    /// The settings where no *settings file* exists: the two default times, the *automatic
    /// setting* on, the default *tab size* and positions of the splitters, the *spaces setting*
    /// and the *silent creation setting* off, the *build system* of the distribution, and no
    /// *network file*.
    fn default() -> Self {
        Settings {
            idle_time: DEFAULT_IDLE_TIME,
            long_idle_time: DEFAULT_LONG_IDLE_TIME,
            automatic: DEFAULT_AUTOMATIC,
            tab_size: DEFAULT_TAB_SIZE,
            tabs_to_spaces: DEFAULT_TABS_TO_SPACES,
            silent_creation: DEFAULT_SILENT_CREATION,
            playback_delay_on: DEFAULT_PLAYBACK_DELAY_ON,
            playback_delay_index: DEFAULT_PLAYBACK_DELAY_INDEX,
            playback_delay_dialog_disabled: DEFAULT_PLAYBACK_DELAY_DIALOG_DISABLED,
            node_left_width: DEFAULT_NODE_LEFT_WIDTH,
            node_upper_height: DEFAULT_NODE_UPPER_HEIGHT,
            network_left_width: DEFAULT_NETWORK_LEFT_WIDTH,
            network_path: None,
            build_system_path: Settings::default_build_system_path(),
            insert_key: KeyStroke::default_insert_key(),
        }
    }
}

// realises FR-093, FR-140, FR-149, C-007
/// The *settings file* as YAML holds it: one key per value, each key absent where its default
/// holds.
///
/// * No path of the *node window* is among the keys: every path of the *node window* is named
///   from the *node description* when a *node* is opened (FR-086, FR-087, FR-102).
/// * The *tab size* and the positions of the splitters are read as any integer, so that a value
///   out of range is corrected rather than failing the whole file.
#[derive(Debug, Serialize, Deserialize)]
struct Stored {
    /// the *idle time* in seconds with one decimal
    #[serde(default = "default_idle_time")]
    idle_time: f64,
    /// the *long idle time* in seconds with one decimal
    #[serde(default = "default_long_idle_time")]
    long_idle_time: f64,
    #[serde(default = "default_automatic")]
    automatic: bool,
    #[serde(default = "default_tab_size")]
    tab_size: i64,
    /// the *spaces setting* and the *silent creation setting* (FR-490, FR-492)
    #[serde(default = "default_tabs_to_spaces")]
    tabs_to_spaces: bool,
    #[serde(default = "default_silent_creation")]
    silent_creation: bool,
    /// the *playback delay* in seconds, one of `PLAYBACK_DELAYS`, present while a delay is
    /// waited and absent while none is; a value read is brought to the nearest of the thirteen,
    /// an integer, as a file of a version before 0.38.0 holds it, read as tenths of a second,
    /// and 0, as a file before 0.39.0 holds it, read as no delay (FR-093, FR-095)
    #[serde(
        default,
        deserialize_with = "seconds_of_stored_delay",
        skip_serializing_if = "Option::is_none"
    )]
    playback_delay: Option<f64>,
    /// the *playback delay dialog setting* (FR-544)
    #[serde(default = "default_playback_delay_dialog_disabled")]
    playback_delay_dialog_disabled: bool,
    #[serde(default = "default_node_left_width")]
    node_left_width: i64,
    #[serde(default = "default_node_upper_height")]
    node_upper_height: i64,
    #[serde(default = "default_network_left_width")]
    network_left_width: i64,
    /// the path of the *network file*: absent, `null` or empty where none is named
    #[serde(
        default,
        deserialize_with = "path_or_none",
        skip_serializing_if = "Option::is_none"
    )]
    network_path: Option<String>,
    #[serde(default = "default_build_system_path")]
    build_system_path: String,
    /// the name of the insert key, as `KeyStroke::name` yields it
    #[serde(default = "default_insert_key")]
    insert_key: String,
}

/// Reads the key `network_path`: `None` where it holds `null` or the empty text, which a
/// *settings file* of a version before 0.34.0 holds where no *network file* was named.
fn path_or_none<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<String>, D::Error> {
    let value: Option<String> = Option::deserialize(deserializer)?;
    Ok(value.filter(|path| !path.is_empty()))
}

/// The default of the key `idle_time`, for a *settings file* that lacks it.
fn default_idle_time() -> f64 {
    seconds_of_milliseconds(DEFAULT_IDLE_TIME)
}

/// The default of the key `long_idle_time`, for a *settings file* that lacks it.
fn default_long_idle_time() -> f64 {
    seconds_of_milliseconds(DEFAULT_LONG_IDLE_TIME)
}

/// The default of the key `automatic`, for a *settings file* that lacks it.
fn default_automatic() -> bool {
    DEFAULT_AUTOMATIC
}

/// The default of the key `tab_size`, for a *settings file* that lacks it.
fn default_tab_size() -> i64 {
    i64::from(DEFAULT_TAB_SIZE)
}

/// The default of the key `tabs_to_spaces`, for a *settings file* that lacks it.
fn default_tabs_to_spaces() -> bool {
    DEFAULT_TABS_TO_SPACES
}

/// The default of the key `silent_creation`, for a *settings file* that lacks it.
fn default_silent_creation() -> bool {
    DEFAULT_SILENT_CREATION
}

/// Reads the key `playback_delay` in seconds: an integer value, as a *settings file* written
/// before 0.38.0 holds it, is a number of tenths of a second; a value that is no number is no
/// delay.
fn seconds_of_stored_delay<'de, D: serde::Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<f64>, D::Error> {
    let value = serde_norway::Value::deserialize(deserializer)?;
    Ok(match value {
        serde_norway::Value::Number(number) => match number.as_i64() {
            Some(tenths) => Some(tenths as f64 / 10.0),
            None => number.as_f64(),
        },
        _ => None,
    })
}

/// The default of the key `playback_delay_dialog_disabled`, for a *settings file* that lacks it.
fn default_playback_delay_dialog_disabled() -> bool {
    DEFAULT_PLAYBACK_DELAY_DIALOG_DISABLED
}

/// The default of the key `node_left_width`, for a *settings file* that lacks it.
fn default_node_left_width() -> i64 {
    i64::from(DEFAULT_NODE_LEFT_WIDTH)
}

/// The default of the key `node_upper_height`, for a *settings file* that lacks it.
fn default_node_upper_height() -> i64 {
    i64::from(DEFAULT_NODE_UPPER_HEIGHT)
}

/// The default of the key `network_left_width`, for a *settings file* that lacks it.
fn default_network_left_width() -> i64 {
    i64::from(DEFAULT_NETWORK_LEFT_WIDTH)
}

/// Yields `value` as a `u32` within `least..=greatest`, the nearest bound where it lies outside.
fn clamped(value: i64, least: u32, greatest: u32) -> u32 {
    let bounded = value.clamp(i64::from(least), i64::from(greatest));
    u32::try_from(bounded).unwrap_or(least)
}

/// Yields a time in seconds with one decimal, as the *settings file* holds it.
fn seconds_of_milliseconds(milliseconds: u32) -> f64 {
    (f64::from(milliseconds) / 100.0).round() / 10.0
}

/// Yields a time in milliseconds of a time in seconds, as the *settings file* holds it; a
/// value that is negative or not a number yields 0.
fn milliseconds_of_seconds(seconds: f64) -> u32 {
    if seconds.is_finite() && seconds > 0.0 {
        (seconds * 1000.0).round().min(f64::from(u32::MAX)) as u32
    } else {
        0
    }
}

// realises FR-133, FR-134, FR-135
/// Yields the message naming the constraint that `idle_time` and `long_idle_time`, in
/// milliseconds, violate, `None` where they satisfy both.
pub fn constraint_violation(idle_time: u32, long_idle_time: u32) -> Option<String> {
    if idle_time < MIN_IDLE_TIME {
        return Some("The idle time is at least 0.2 seconds.".to_owned());
    }
    if long_idle_time <= idle_time {
        return Some("The long idle time is greater than the idle time.".to_owned());
    }
    None
}

// realises FR-140, FR-141
/// Yields the message naming the constraint that `tab_size` violates, `None` where it lies in
/// `MIN_TAB_SIZE..=MAX_TAB_SIZE`.
pub fn tab_size_violation(tab_size: u32) -> Option<String> {
    if (MIN_TAB_SIZE..=MAX_TAB_SIZE).contains(&tab_size) {
        None
    } else {
        Some(format!(
            "The tab size is an integer from {MIN_TAB_SIZE} to {MAX_TAB_SIZE}."
        ))
    }
}

// realises FR-518
/// Yields the message naming FR-518 where `delay`, no delay or milliseconds, is a value that is
/// none of `PLAYBACK_DELAYS`, `None` otherwise.
pub fn playback_delay_violation(delay: Option<u32>) -> Option<String> {
    match delay {
        Some(milliseconds) if !PLAYBACK_DELAYS.contains(&milliseconds) => Some(format!(
            "The playback delay is off or one of {} seconds.",
            PLAYBACK_DELAYS
                .iter()
                .map(|value| playback_delay_text(*value))
                .collect::<Vec<_>>()
                .join(", ")
        )),
        _ => None,
    }
}

// realises FR-495, FR-518
/// Yields a value of `PLAYBACK_DELAYS`, in milliseconds, as seconds rounded to two decimals, as
/// the `settings` line writes it and the dialogs show it: 71 is `0.07`, 566 is `0.57`, 3200 is
/// `3.20`; the value read back is brought to the nearest of the thirteen.
pub fn playback_delay_text(milliseconds: u32) -> String {
    let hundredths = (milliseconds + 5) / 10;
    format!("{}.{:02}", hundredths / 100, hundredths % 100)
}

// realises FR-095, FR-518
/// Yields the index in `PLAYBACK_DELAYS` of the value nearest to `delay`, in milliseconds, the
/// smaller one where two are equally near, as a stored value is taken.
pub fn nearest_playback_delay_index(delay: u32) -> usize {
    PLAYBACK_DELAYS
        .iter()
        .enumerate()
        .min_by_key(|(_, value)| value.abs_diff(delay))
        .map_or(0, |(index, _)| index)
}

// realises FR-130
/// Yields the *idle time* and the *long idle time* the *automatic setting* derives from a
/// *processing time* in milliseconds: twice the *processing time*, rounded up to the next
/// multiple of `TIME_STEP` and at least `MIN_IDLE_TIME`, and 8 times that.
pub fn times_of_processing(processing_time: u32) -> (u32, u32) {
    let doubled = processing_time.saturating_mul(2);
    let idle_time = doubled
        .div_ceil(TIME_STEP)
        .saturating_mul(TIME_STEP)
        .max(MIN_IDLE_TIME);
    (idle_time, idle_time.saturating_mul(8))
}

/// The default of the key `insert_key`, for a *settings file* that lacks it.
fn default_insert_key() -> String {
    KeyStroke::default_insert_key().name()
}

/// The default of the key `build_system_path`, for a *settings file* that lacks it.
fn default_build_system_path() -> String {
    Settings::default_build_system_path()
}

impl Settings {
    // realises IR-011, IR-012
    /// Yields the path of the *settings file*: 'genc3wb.yaml' in the folder of the executable
    /// (`launch::executable_folder`).
    pub fn default_path() -> PathBuf {
        executable_folder().join(SETTINGS_FILE_NAME)
    }

    // realises FR-094
    /// Yields the default path of the *build system*: `DEFAULT_BUILD_SYSTEM_NAME` in the folder
    /// of the executable (`launch::executable_folder`).
    pub fn default_build_system_path() -> String {
        executable_folder()
            .join(DEFAULT_BUILD_SYSTEM_NAME)
            .to_string_lossy()
            .into_owned()
    }

    // realises FR-090, FR-094, FR-095, FR-140, FR-149, IR-011
    /// Restores the settings stored at the last termination.
    ///
    /// * Where the *settings file* does not exist, the default settings are yielded (FR-094).
    /// * A key the settings do not know is ignored, and a key that is absent takes its default,
    ///   so that a file of another version is read.
    /// * The two times are read in seconds and held in milliseconds; where they violate FR-133
    ///   or FR-134, the *idle time* is raised to the least one and the *long idle time* to the
    ///   *idle time* plus one step.
    /// * A *tab size* outside `MIN_TAB_SIZE..=MAX_TAB_SIZE` is brought to the nearest bound, a
    ///   *playback delay* to the nearest of `PLAYBACK_DELAYS`, an absent key and a value that is
    ///   no positive number read as no delay, and a negative position of a splitter to 0.
    ///
    /// # Errors
    /// Returns [`SettingsError::Io`] where the file exists but cannot be read, and
    /// [`SettingsError::Malformed`] where it is no YAML mapping of these keys.
    pub fn load(path: &Path) -> Result<Settings, SettingsError> {
        if !path.is_file() {
            return Ok(Settings::default());
        }
        let content =
            fs::read_to_string(path).map_err(|error| SettingsError::Io(error.to_string()))?;
        let stored: Stored = serde_norway::from_str(&content)
            .map_err(|error| SettingsError::Malformed(error.to_string()))?;
        let idle_time = milliseconds_of_seconds(stored.idle_time).max(MIN_IDLE_TIME);
        let long_idle_time =
            milliseconds_of_seconds(stored.long_idle_time).max(idle_time.saturating_add(TIME_STEP));
        let playback_delay = stored
            .playback_delay
            .filter(|seconds| seconds.is_finite() && *seconds > 0.0);
        Ok(Settings {
            idle_time,
            long_idle_time,
            automatic: stored.automatic,
            tab_size: clamped(stored.tab_size, MIN_TAB_SIZE, MAX_TAB_SIZE),
            tabs_to_spaces: stored.tabs_to_spaces,
            silent_creation: stored.silent_creation,
            playback_delay_on: playback_delay.is_some(),
            playback_delay_index: playback_delay.map_or(DEFAULT_PLAYBACK_DELAY_INDEX, |seconds| {
                nearest_playback_delay_index(milliseconds_of_seconds(seconds))
            }),
            playback_delay_dialog_disabled: stored.playback_delay_dialog_disabled,
            node_left_width: clamped(stored.node_left_width, 0, u32::MAX),
            node_upper_height: clamped(stored.node_upper_height, 0, u32::MAX),
            network_left_width: clamped(stored.network_left_width, 0, u32::MAX),
            network_path: stored.network_path,
            build_system_path: stored.build_system_path,
            insert_key: KeyStroke::of_name(&stored.insert_key)
                .unwrap_or_else(KeyStroke::default_insert_key),
        })
    }

    // realises FR-091, FR-099, FR-141, FR-149, FR-255, FR-479, IR-012
    /// Stores the settings, to be restored at the next start.
    ///
    /// * The directory of the file is created where it does not exist.
    /// * The content is written into a temporary file beside the *settings file*, named after
    ///   it with the process number appended, and that file is renamed over the *settings
    ///   file*, so that the file is replaced as a whole: a genc³wb reading it at the same time
    ///   reads the content before or after, never a part, and the last write wins (FR-479).
    ///
    /// # Errors
    /// Returns [`SettingsError::Io`] where the file cannot be written.
    pub fn save(&self, path: &Path) -> Result<(), SettingsError> {
        if let Some(directory) = path.parent().filter(|d| !d.as_os_str().is_empty()) {
            fs::create_dir_all(directory).map_err(|error| SettingsError::Io(error.to_string()))?;
        }
        let stored = Stored {
            idle_time: seconds_of_milliseconds(self.idle_time),
            long_idle_time: seconds_of_milliseconds(self.long_idle_time),
            automatic: self.automatic,
            tab_size: i64::from(self.tab_size),
            tabs_to_spaces: self.tabs_to_spaces,
            silent_creation: self.silent_creation,
            playback_delay: self
                .playback_delay_on
                .then(|| f64::from(self.playback_delay()) / 1000.0),
            playback_delay_dialog_disabled: self.playback_delay_dialog_disabled,
            node_left_width: i64::from(self.node_left_width),
            node_upper_height: i64::from(self.node_upper_height),
            network_left_width: i64::from(self.network_left_width),
            network_path: self.network_path.clone(),
            build_system_path: self.build_system_path.clone(),
            insert_key: self.insert_key.name(),
        };
        let content = serde_norway::to_string(&stored)
            .map_err(|error| SettingsError::Malformed(error.to_string()))?;
        let temporary = temporary_path_of(path);
        fs::write(&temporary, content).map_err(|error| SettingsError::Io(error.to_string()))?;
        fs::rename(&temporary, path).map_err(|error| {
            let _ = fs::remove_file(&temporary);
            SettingsError::Io(error.to_string())
        })
    }

    // realises FR-093, FR-095
    /// Yields the *idle time* in milliseconds.
    pub fn idle_time(&self) -> u32 {
        self.idle_time
    }

    // realises FR-093, FR-095
    /// Yields the *long idle time* in milliseconds.
    pub fn long_idle_time(&self) -> u32 {
        self.long_idle_time
    }

    // realises FR-099, FR-130, FR-133, FR-134
    /// Sets the two times, in milliseconds.
    ///
    /// # Errors
    /// Returns [`SettingsError::Constraint`] with the message of [`constraint_violation`] where
    /// the times violate FR-133 or FR-134; the settings are left as they are then.
    pub fn set_times(&mut self, idle_time: u32, long_idle_time: u32) -> Result<(), SettingsError> {
        if let Some(message) = constraint_violation(idle_time, long_idle_time) {
            return Err(SettingsError::Constraint(message));
        }
        self.idle_time = idle_time;
        self.long_idle_time = long_idle_time;
        Ok(())
    }

    // realises FR-093, FR-095
    /// Yields whether the *automatic setting* is on.
    pub fn automatic(&self) -> bool {
        self.automatic
    }

    // realises FR-099
    /// Switches the *automatic setting*.
    pub fn set_automatic(&mut self, automatic: bool) {
        self.automatic = automatic;
    }

    // realises FR-140, FR-142
    /// Yields the *tab size*, in characters.
    pub fn tab_size(&self) -> u32 {
        self.tab_size
    }

    // realises FR-140, FR-141
    /// Sets the *tab size*, in characters.
    ///
    /// # Errors
    /// Returns [`SettingsError::Constraint`] with the message of [`tab_size_violation`] where
    /// `tab_size` lies outside `MIN_TAB_SIZE..=MAX_TAB_SIZE`; the settings are left as they are
    /// then.
    pub fn set_tab_size(&mut self, tab_size: u32) -> Result<(), SettingsError> {
        if let Some(message) = tab_size_violation(tab_size) {
            return Err(SettingsError::Constraint(message));
        }
        self.tab_size = tab_size;
        Ok(())
    }

    // realises FR-492
    /// Yields whether the *spaces setting* is on.
    pub fn tabs_to_spaces(&self) -> bool {
        self.tabs_to_spaces
    }

    // realises FR-099, FR-492
    /// Switches the *spaces setting*.
    pub fn set_tabs_to_spaces(&mut self, tabs_to_spaces: bool) {
        self.tabs_to_spaces = tabs_to_spaces;
    }

    // realises FR-490
    /// Yields whether the *silent creation setting* is on.
    pub fn silent_creation(&self) -> bool {
        self.silent_creation
    }

    // realises FR-099, FR-489
    /// Switches the *silent creation setting*.
    pub fn set_silent_creation(&mut self, silent_creation: bool) {
        self.silent_creation = silent_creation;
    }

    // realises FR-093, FR-095, FR-518
    /// Yields the *playback delay* in milliseconds: the value chosen while a delay is waited,
    /// 0 while none is.
    pub fn playback_delay(&self) -> u32 {
        if self.playback_delay_on {
            PLAYBACK_DELAYS[self.playback_delay_index.min(PLAYBACK_DELAYS.len() - 1)]
        } else {
            0
        }
    }

    // realises FR-518, FR-542
    /// Yields whether a *playback delay* is waited, and the index in `PLAYBACK_DELAYS` of the
    /// value chosen last, kept while none is waited.
    pub fn playback_delay_on(&self) -> bool {
        self.playback_delay_on
    }

    pub fn playback_delay_index(&self) -> usize {
        self.playback_delay_index
    }

    // realises FR-099, FR-518, FR-543
    /// Sets whether a *playback delay* is waited and the index of its value in
    /// `PLAYBACK_DELAYS`.
    ///
    /// # Errors
    /// Returns [`SettingsError::Constraint`] with the message of `playback_delay_violation` where
    /// `index` names no value, nothing being set then.
    pub fn set_playback_delay(&mut self, on: bool, index: usize) -> Result<(), SettingsError> {
        if index >= PLAYBACK_DELAYS.len() {
            return Err(SettingsError::Constraint(
                playback_delay_violation(Some(u32::MAX)).unwrap_or_default(),
            ));
        }
        self.playback_delay_on = on;
        self.playback_delay_index = index;
        Ok(())
    }

    // realises FR-093, FR-095, FR-544
    /// Yields whether the *playback delay dialog setting* is on.
    pub fn playback_delay_dialog_disabled(&self) -> bool {
        self.playback_delay_dialog_disabled
    }

    // realises FR-099, FR-543
    /// Switches the *playback delay dialog setting*.
    pub fn set_playback_delay_dialog_disabled(&mut self, disabled: bool) {
        self.playback_delay_dialog_disabled = disabled;
    }

    // realises FR-146, FR-149
    /// Yields the width of the left part of the *node window*, in pixels.
    pub fn node_left_width(&self) -> u32 {
        self.node_left_width
    }

    // realises FR-146, FR-149
    /// Sets the width of the left part of the *node window*, in pixels.
    pub fn set_node_left_width(&mut self, width: u32) {
        self.node_left_width = width;
    }

    // realises FR-147, FR-149
    /// Yields the height of the *input group* in the left part of the *node window*, in pixels.
    pub fn node_upper_height(&self) -> u32 {
        self.node_upper_height
    }

    // realises FR-147, FR-149
    /// Sets the height of the *input group* in the left part of the *node window*, in pixels.
    pub fn set_node_upper_height(&mut self, height: u32) {
        self.node_upper_height = height;
    }

    // realises FR-148, FR-149
    /// Yields the width of the *input group* of the *network file*, in pixels.
    pub fn network_left_width(&self) -> u32 {
        self.network_left_width
    }

    // realises FR-148, FR-149
    /// Sets the width of the *input group* of the *network file*, in pixels.
    pub fn set_network_left_width(&mut self, width: u32) {
        self.network_left_width = width;
    }

    /// Yields the path of the *network file*, `None` where none is named.
    pub fn network_path(&self) -> Option<&str> {
        self.network_path.as_deref()
    }

    // realises FR-012, FR-091
    /// Sets the path of the *network file*.
    pub fn set_network_path(&mut self, path: Option<&str>) {
        self.network_path = path.filter(|path| !path.is_empty()).map(str::to_owned);
    }

    /// Yields the path of the *build system*, empty where none is stored.
    pub fn build_system_path(&self) -> &str {
        &self.build_system_path
    }

    // realises FR-066, FR-091
    /// Sets the path of the *build system*.
    pub fn set_build_system_path(&mut self, path: &str) {
        self.build_system_path = path.to_owned();
    }

    // realises FR-198, FR-200
    /// Yields the insert key.
    pub fn insert_key(&self) -> KeyStroke {
        self.insert_key
    }

    // realises FR-199, FR-200
    /// Sets the insert key.
    pub fn set_insert_key(&mut self, insert_key: KeyStroke) {
        self.insert_key = insert_key;
    }
}

// realises FR-479, FR-480, IR-011, IR-012
/// The settings of the session with the *settings file* they are stored in, and the reason of
/// the last failed write of it.
///
/// * It holds no lock: every store writes, and the last write wins (FR-479).
/// * Where a write fails, the reason is held until a write succeeds, so that the status bar
///   tells it (FR-480); the settings of the session change all the same.
/// * It dereferences to the [`Settings`] it holds, which are read and changed through it.
#[derive(Debug)]
pub struct SettingsStore {
    /// the settings of the session
    settings: Settings,
    /// the path of the *settings file*
    path: PathBuf,
    /// the reason of the operating system where the last write failed, `None` where it
    /// succeeded or none was made yet
    failure: Option<String>,
}

impl SettingsStore {
    // realises FR-094, FR-095, IR-011
    /// Restores the settings from the *settings file* at `path`, as [`Settings::load`] does;
    /// the default settings where it cannot be read.
    pub fn open(path: &Path) -> SettingsStore {
        SettingsStore {
            settings: Settings::load(path).unwrap_or_default(),
            path: path.to_owned(),
            failure: None,
        }
    }

    // realises FR-479, FR-480, IR-012
    /// Writes the settings to the *settings file*, and holds the reason where the write fails,
    /// or clears it where it succeeds.
    ///
    /// # Errors
    /// Returns what [`Settings::save`] returns.
    pub fn store(&mut self) -> Result<(), SettingsError> {
        let result = self.settings.save(&self.path);
        self.failure = result.as_ref().err().map(ToString::to_string);
        result
    }

    // realises FR-480
    /// Yields the reason of the last failed write of the *settings file*, `None` where the last
    /// write succeeded or none was made yet.
    pub fn failure(&self) -> Option<&str> {
        self.failure.as_deref()
    }
}

// realises FR-479
/// Yields the path of the temporary file a save of the *settings file* at `path` writes first:
/// the path with `.` and the process number appended.
fn temporary_path_of(path: &Path) -> PathBuf {
    let mut temporary = path.as_os_str().to_owned();
    temporary.push(format!(".{}", std::process::id()));
    PathBuf::from(temporary)
}

impl Deref for SettingsStore {
    type Target = Settings;

    fn deref(&self) -> &Settings {
        &self.settings
    }
}

impl DerefMut for SettingsStore {
    fn deref_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : Workbench::default, Workbench::try_set_settings, Workbench::set_playback_delay,
 *                    Workbench::report_processing_time, Workbench::set_node_left_width,
 *                    Workbench::set_node_upper_height, Workbench::set_network_left_width,
 *                    InputGroup::set_path, NetworkEditor::set_build_system_path */
#[cfg(test)]
mod tests {
    use super::*;

    fn case_file(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("genc3wb-settings-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir.join("sub").join(SETTINGS_FILE_NAME)
    }

    fn case_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("genc3wb-settings-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("the temporary directory of the test can be created");
        dir
    }

    fn remove_case(file: &Path) {
        let _ = fs::remove_dir_all(
            file.parent()
                .and_then(Path::parent)
                .expect("the case directory"),
        );
    }

    #[test]
    fn default_settings_hold_the_defaults_with_the_automatic_setting_on() {
        // FR-094
        let settings = Settings::default();
        assert_eq!(
            (
                settings.network_path(),
                settings.build_system_path(),
                settings.idle_time(),
                settings.long_idle_time(),
                settings.automatic()
            ),
            (
                None,
                Settings::default_build_system_path().as_str(),
                2000,
                16000,
                true
            )
        );
    }

    #[test]
    fn the_two_paths_are_held_apart() {
        // FR-091
        let mut settings = Settings::default();
        settings.set_network_path(Some("n.gc3n"));
        settings.set_build_system_path("/usr/bin/genc3d");
        assert_eq!(
            (settings.network_path(), settings.build_system_path()),
            (Some("n.gc3n"), "/usr/bin/genc3d")
        );
    }

    #[test]
    fn settings_survive_save_and_load_with_the_times_in_seconds() {
        // FR-090, FR-091, FR-093, IR-011, IR-012
        let file = case_file("roundtrip");
        let mut settings = Settings::default();
        settings.set_network_path(Some("/tmp/n.gc3n"));
        settings.set_build_system_path("/opt/genc3d");
        settings
            .set_times(1400, 9000)
            .expect("the times satisfy the constraints");
        settings.set_automatic(false);
        let saved = settings.save(&file);
        let content = fs::read_to_string(&file).unwrap_or_default();
        let loaded = Settings::load(&file);
        remove_case(&file);
        assert_eq!(
            (
                saved,
                content.contains("idle_time: 1.4\n"),
                content.contains("long_idle_time: 9.0\n"),
                content.contains("automatic: false\n"),
                loaded
            ),
            (Ok(()), true, true, true, Ok(settings))
        );
    }

    #[test]
    fn the_insert_key_is_stored_by_its_name_and_restored() {
        // FR-199, FR-200
        let dir = std::env::temp_dir().join(format!(
            "genc3wb-settings-insert-key-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        let path = dir.join(SETTINGS_FILE_NAME);
        let mut settings = Settings::default();
        settings.set_insert_key(KeyStroke::new(u32::from('I'), 0x0400_0000));
        let saved = settings.save(&path);
        let content = fs::read_to_string(&path).unwrap_or_default();
        let restored = Settings::load(&path).map(|settings| settings.insert_key().name());
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(
            (saved, content.contains("insert_key: Ctrl+I"), restored),
            (Ok(()), true, Ok("Ctrl+I".to_owned()))
        );
    }

    #[test]
    fn an_insert_key_that_is_no_name_or_absent_takes_the_default() {
        // FR-198, FR-200
        let dir = std::env::temp_dir().join(format!(
            "genc3wb-settings-insert-key-default-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("the directory can be created");
        let path = dir.join(SETTINGS_FILE_NAME);
        fs::write(&path, "insert_key: Hyper+Nothing\n").expect("the file can be written");
        let unknown = Settings::load(&path).map(|settings| settings.insert_key());
        fs::write(&path, "tab_size: 4\n").expect("the file can be written");
        let absent = Settings::load(&path).map(|settings| settings.insert_key());
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(
            (unknown, absent),
            (
                Ok(KeyStroke::default_insert_key()),
                Ok(KeyStroke::default_insert_key())
            )
        );
    }

    #[test]
    fn missing_settings_file_yields_the_defaults() {
        // FR-094
        let file = case_file("missing");
        assert_eq!(Settings::load(&file), Ok(Settings::default()));
    }

    #[test]
    fn unknown_key_is_ignored_and_an_absent_key_takes_its_default() {
        // FR-094, FR-095, IR-011
        let file = case_file("unknown");
        fs::create_dir_all(file.parent().expect("a parent")).expect("the directory can be created");
        fs::write(&file, "network_path: a\nfuture_key: b\n").expect("the file can be written");
        let settings = Settings::load(&file).expect("an unknown key is no error");
        remove_case(&file);
        assert_eq!(
            (
                settings.network_path(),
                settings.idle_time(),
                settings.long_idle_time(),
                settings.automatic(),
                settings.build_system_path()
            ),
            (
                Some("a"),
                DEFAULT_IDLE_TIME,
                DEFAULT_LONG_IDLE_TIME,
                DEFAULT_AUTOMATIC,
                Settings::default_build_system_path().as_str()
            )
        );
    }

    #[test]
    fn a_file_of_the_version_before_with_whole_seconds_is_read() {
        // FR-093, FR-095, IR-011: 0.9.0.0 wrote the times as whole seconds
        let file = case_file("whole");
        fs::create_dir_all(file.parent().expect("a parent")).expect("the directory can be created");
        fs::write(&file, "idle_time: 3\nlong_idle_time: 20\n").expect("the file can be written");
        let settings = Settings::load(&file).expect("whole seconds are read");
        remove_case(&file);
        assert_eq!(
            (settings.idle_time(), settings.long_idle_time()),
            (3000, 20000)
        );
    }

    #[test]
    fn times_of_a_file_that_violate_the_constraints_are_raised_to_satisfy_them() {
        // FR-133, FR-134
        let file = case_file("violated");
        fs::create_dir_all(file.parent().expect("a parent")).expect("the directory can be created");
        fs::write(&file, "idle_time: 0.1\nlong_idle_time: -4\n").expect("the file can be written");
        let settings = Settings::load(&file).expect("the file is read");
        remove_case(&file);
        assert_eq!(
            (settings.idle_time(), settings.long_idle_time()),
            (200, 400)
        );
    }

    #[test]
    fn a_file_that_is_no_yaml_mapping_is_reported() {
        // IR-011
        let file = case_file("notyaml");
        fs::create_dir_all(file.parent().expect("a parent")).expect("the directory can be created");
        fs::write(&file, "idle_time: [1, 2\n").expect("the file can be written");
        let loaded = Settings::load(&file);
        remove_case(&file);
        assert!(matches!(loaded, Err(SettingsError::Malformed(_))));
    }

    #[test]
    fn times_that_satisfy_the_constraints_are_set() {
        // FR-099, FR-133, FR-134
        let mut settings = Settings::default();
        let result = settings.set_times(200, 400);
        assert_eq!(
            (result, settings.idle_time(), settings.long_idle_time()),
            (Ok(()), 200, 400)
        );
    }

    #[test]
    fn an_idle_time_below_the_least_one_is_refused_with_its_message() {
        // FR-133, FR-135
        let mut settings = Settings::default();
        let result = settings.set_times(100, 1000);
        assert_eq!(
            (result, settings.idle_time()),
            (
                Err(SettingsError::Constraint(
                    "The idle time is at least 0.2 seconds.".to_owned()
                )),
                2000
            )
        );
    }

    #[test]
    fn a_long_idle_time_not_above_the_idle_time_is_refused_with_its_message() {
        // FR-134, FR-135
        let mut settings = Settings::default();
        let result = settings.set_times(1000, 1000);
        assert_eq!(
            (result, settings.long_idle_time()),
            (
                Err(SettingsError::Constraint(
                    "The long idle time is greater than the idle time.".to_owned()
                )),
                16000
            )
        );
    }

    #[test]
    fn the_automatic_setting_derives_the_times_from_the_processing_time() {
        // FR-130: twice the processing time, rounded up to 0.2 seconds, at least 0.2 seconds
        assert_eq!(
            [
                times_of_processing(0),
                times_of_processing(50),
                times_of_processing(100),
                times_of_processing(150),
                times_of_processing(1234),
            ],
            [
                (200, 1600),
                (200, 1600),
                (200, 1600),
                (400, 3200),
                (2600, 20800)
            ]
        );
    }

    #[test]
    fn the_automatic_setting_is_switched() {
        // FR-099
        let mut settings = Settings::default();
        settings.set_automatic(false);
        assert!(!settings.automatic());
    }

    #[test]
    fn default_settings_hold_the_default_tab_size_and_positions_of_the_splitters() {
        // FR-140, FR-149
        let settings = Settings::default();
        assert_eq!(
            (
                settings.tab_size(),
                settings.node_left_width(),
                settings.node_upper_height(),
                settings.network_left_width()
            ),
            (4, 560, 340, 480)
        );
    }

    #[test]
    fn tab_size_and_positions_of_the_splitters_survive_save_and_load() {
        // FR-140, FR-141, FR-149
        let file = case_file("layout");
        let mut settings = Settings::default();
        settings
            .set_tab_size(8)
            .expect("the tab size satisfies the constraint");
        settings.set_node_left_width(700);
        settings.set_node_upper_height(250);
        settings.set_network_left_width(333);
        let saved = settings.save(&file);
        let content = fs::read_to_string(&file).unwrap_or_default();
        let loaded = Settings::load(&file);
        remove_case(&file);
        assert_eq!(
            (
                saved,
                content.contains("tab_size: 8\n"),
                content.contains("node_left_width: 700\n"),
                content.contains("node_upper_height: 250\n"),
                content.contains("network_left_width: 333\n"),
                loaded
            ),
            (Ok(()), true, true, true, true, Ok(settings))
        );
    }

    #[test]
    fn a_file_without_tab_size_and_splitters_yields_their_defaults() {
        // FR-140, FR-149, IR-011: 0.10.0.0 wrote none of these keys
        let file = case_file("before");
        fs::create_dir_all(file.parent().expect("a parent")).expect("the directory can be created");
        fs::write(
            &file,
            "idle_time: 1.4\nlong_idle_time: 9.0\nautomatic: false\n",
        )
        .expect("the file can be written");
        let settings = Settings::load(&file).expect("the file is read");
        remove_case(&file);
        assert_eq!(
            (
                settings.tab_size(),
                settings.node_left_width(),
                settings.node_upper_height(),
                settings.network_left_width(),
                settings.idle_time()
            ),
            (
                DEFAULT_TAB_SIZE,
                DEFAULT_NODE_LEFT_WIDTH,
                DEFAULT_NODE_UPPER_HEIGHT,
                DEFAULT_NETWORK_LEFT_WIDTH,
                1400
            )
        );
    }

    #[test]
    fn values_of_a_file_out_of_range_are_brought_to_the_nearest_bound() {
        // FR-140, FR-149
        let file = case_file("range");
        fs::create_dir_all(file.parent().expect("a parent")).expect("the directory can be created");
        fs::write(
            &file,
            "tab_size: 40\nnode_left_width: -5\nnode_upper_height: 0\n",
        )
        .expect("the file can be written");
        let too_great = Settings::load(&file).expect("the file is read");
        fs::write(&file, "tab_size: -3\n").expect("the file can be written");
        let too_small = Settings::load(&file).expect("the file is read");
        remove_case(&file);
        assert_eq!(
            (
                too_great.tab_size(),
                too_great.node_left_width(),
                too_great.node_upper_height(),
                too_small.tab_size()
            ),
            (16, 0, 0, 1)
        );
    }

    #[test]
    fn a_tab_size_within_the_bounds_is_set() {
        // FR-140, FR-141
        let mut settings = Settings::default();
        let least = (settings.set_tab_size(1), settings.tab_size());
        let greatest = (settings.set_tab_size(16), settings.tab_size());
        assert_eq!((least, greatest), ((Ok(()), 1), (Ok(()), 16)));
    }

    #[test]
    fn a_tab_size_outside_the_bounds_is_refused_with_its_message() {
        // FR-140, FR-141
        let mut settings = Settings::default();
        let message = "The tab size is an integer from 1 to 16.".to_owned();
        let below = settings.set_tab_size(0);
        let above = settings.set_tab_size(17);
        assert_eq!(
            (below, above, settings.tab_size()),
            (
                Err(SettingsError::Constraint(message.clone())),
                Err(SettingsError::Constraint(message)),
                DEFAULT_TAB_SIZE
            )
        );
    }

    #[test]
    fn the_tab_size_violation_is_none_within_the_bounds_only() {
        // FR-140, FR-141
        assert_eq!(
            [0, 1, 4, 16, 17].map(|tab_size| tab_size_violation(tab_size).is_none()),
            [false, true, true, true, false]
        );
    }

    #[test]
    fn the_positions_of_the_splitters_are_held_apart() {
        // FR-146, FR-147, FR-148, FR-149
        let mut settings = Settings::default();
        settings.set_node_left_width(1);
        settings.set_node_upper_height(2);
        settings.set_network_left_width(3);
        assert_eq!(
            (
                settings.node_left_width(),
                settings.node_upper_height(),
                settings.network_left_width()
            ),
            (1, 2, 3)
        );
    }

    #[test]
    fn the_settings_file_and_the_build_system_lie_in_the_folder_of_the_executable() {
        // FR-094, IR-011, IR-012
        let folder = executable_folder();
        assert_eq!(
            (
                Settings::default_path(),
                Settings::default_build_system_path()
            ),
            (
                folder.join(SETTINGS_FILE_NAME),
                folder
                    .join(DEFAULT_BUILD_SYSTEM_NAME)
                    .to_string_lossy()
                    .into_owned()
            )
        );
    }

    #[test]
    fn a_store_writes_the_settings_and_leaves_no_temporary_file() {
        // FR-479, IR-012
        let dir = case_dir("store-writes");
        let path = dir.join(SETTINGS_FILE_NAME);
        let mut store = SettingsStore::open(&path);
        store.set_network_path(Some("n.gc3n"));
        let stored = store.store();
        let read = Settings::load(&path).map(|settings| settings.network_path().map(str::to_owned));
        let files = fs::read_dir(&dir)
            .map(|entries| entries.count())
            .unwrap_or(0);
        let failure = store.failure().map(ToOwned::to_owned);
        drop(store);
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(
            (stored, read, files, failure),
            (Ok(()), Ok(Some("n.gc3n".to_owned())), 1, None)
        );
    }

    #[test]
    fn no_network_file_is_no_key_in_the_file_and_an_empty_or_null_value_reads_as_none() {
        // FR-090, FR-093, FR-535
        let file = case_file("no-network");
        let mut settings = Settings::default();
        settings.set_network_path(None);
        let saved = settings.save(&file);
        let content = fs::read_to_string(&file).unwrap_or_default();
        fs::write(&file, "network_path: ''\n").expect("the file can be written");
        let empty =
            Settings::load(&file).map(|settings| settings.network_path().map(str::to_owned));
        fs::write(&file, "network_path: null\n").expect("the file can be written");
        let null = Settings::load(&file).map(|settings| settings.network_path().map(str::to_owned));
        fs::write(&file, "network_path: /tmp/n.gc3n\n").expect("the file can be written");
        let named =
            Settings::load(&file).map(|settings| settings.network_path().map(str::to_owned));
        remove_case(&file);
        assert_eq!(
            (saved, content.contains("network_path"), empty, null, named),
            (
                Ok(()),
                false,
                Ok(None),
                Ok(None),
                Ok(Some("/tmp/n.gc3n".to_owned()))
            )
        );
    }

    #[test]
    fn two_stores_write_the_same_file_and_the_last_write_wins() {
        // FR-479
        let dir = case_dir("store-last-wins");
        let path = dir.join(SETTINGS_FILE_NAME);
        let mut first = SettingsStore::open(&path);
        first.set_network_path(Some("first.gc3n"));
        let _ = first.store();
        let mut second = SettingsStore::open(&path);
        let read = second.network_path().map(str::to_owned);
        second.set_network_path(Some("second.gc3n"));
        let stored = second.store();
        let on_disk =
            Settings::load(&path).map(|settings| settings.network_path().map(str::to_owned));
        drop(second);
        drop(first);
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(
            (read.as_deref(), stored, on_disk),
            (
                Some("first.gc3n"),
                Ok(()),
                Ok(Some("second.gc3n".to_owned()))
            )
        );
    }

    #[test]
    fn a_failed_write_holds_its_reason_until_a_write_succeeds() {
        // FR-480
        let dir = case_dir("store-fails");
        let blocking = dir.join("blocking");
        fs::write(&blocking, "a file where a folder is needed").expect("the file can be written");
        let mut store = SettingsStore::open(&blocking.join(SETTINGS_FILE_NAME));
        let failed = store.store().is_err();
        let reason = store.failure().map(|reason| !reason.is_empty());
        let mut writable = SettingsStore::open(&dir.join(SETTINGS_FILE_NAME));
        let _ = writable.store();
        let cleared = writable.failure().is_none();
        let _ = fs::remove_dir_all(&dir);
        assert_eq!((failed, reason, cleared), (true, Some(true), true));
    }

    #[test]
    fn a_store_creates_the_folder_of_the_settings_file() {
        // IR-012: the folder that does not exist is created by the writing
        let dir = case_dir("store-creates");
        let path = dir.join("absent").join(SETTINGS_FILE_NAME);
        let mut store = SettingsStore::open(&path);
        let state = (store.store(), path.is_file());
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(state, (Ok(()), true));
    }

    #[test]
    fn default_settings_hold_both_switches_of_the_editor_and_the_dialog_off() {
        // FR-094: the spaces setting and the silent creation setting are off
        let settings = Settings::default();
        assert_eq!(
            (settings.tabs_to_spaces(), settings.silent_creation()),
            (DEFAULT_TABS_TO_SPACES, DEFAULT_SILENT_CREATION)
        );
        assert_eq!(
            (DEFAULT_TABS_TO_SPACES, DEFAULT_SILENT_CREATION),
            (false, false)
        );
    }

    #[test]
    fn the_two_switches_survive_save_and_load_and_default_where_absent() {
        // FR-093, FR-095, IR-011, IR-012: both keys written and read; a file without them
        // yields them off
        let file = case_file("switches");
        let mut settings = Settings::default();
        settings.set_tabs_to_spaces(true);
        settings.set_silent_creation(true);
        settings.save(&file).expect("the settings can be stored");
        let content = fs::read_to_string(&file).expect("the settings file was written");
        let restored = Settings::load(&file).expect("the settings can be restored");
        fs::write(&file, "tab_size: 2\n").expect("the file can be written");
        let without = Settings::load(&file).expect("a file without the keys is read");
        remove_case(&file);
        assert_eq!(
            (
                content.contains("tabs_to_spaces: true"),
                content.contains("silent_creation: true"),
                restored.tabs_to_spaces(),
                restored.silent_creation(),
                without.tabs_to_spaces(),
                without.silent_creation()
            ),
            (true, true, true, true, false, false)
        );
    }

    #[test]
    fn default_settings_hold_no_playback_delay_and_the_playback_delay_dialog_enabled() {
        // FR-094: no delay, the first value in the field, the playback delay dialog setting off
        let settings = Settings::default();
        assert_eq!(
            (
                settings.playback_delay_on(),
                settings.playback_delay_index(),
                settings.playback_delay(),
                settings.playback_delay_dialog_disabled(),
                DEFAULT_PLAYBACK_DELAY_ON,
                DEFAULT_PLAYBACK_DELAY_INDEX,
                DEFAULT_PLAYBACK_DELAY_DIALOG_DISABLED
            ),
            (false, 0, 0, false, false, 0, false)
        );
    }

    #[test]
    fn the_thirteen_playback_delays_grow_by_the_square_root_of_two_from_50_milliseconds() {
        // FR-518: 0.05 · √2ⁿ seconds for n from 0 to 12, rounded to the millisecond, and their
        // texts with two decimals
        let expected: Vec<u32> = (0..13)
            .map(|n| (0.05 * 2f64.sqrt().powi(n) * 1000.0).round() as u32)
            .collect();
        let texts: Vec<String> = PLAYBACK_DELAYS
            .iter()
            .map(|value| playback_delay_text(*value))
            .collect();
        assert_eq!(
            (PLAYBACK_DELAYS.to_vec(), texts.join(" ")),
            (
                expected,
                "0.05 0.07 0.10 0.14 0.20 0.28 0.40 0.57 0.80 1.13 1.60 2.26 3.20".to_owned()
            )
        );
    }

    #[test]
    fn a_playback_delay_on_is_written_in_seconds_and_one_off_writes_no_key() {
        // FR-093, FR-095, FR-518, IR-011, IR-012: the key holds the value in seconds while the
        // delay is on and is absent while it is off; a file without the key reads as off with
        // the first value in the field
        let file = case_file("delay");
        let mut settings = Settings::default();
        settings
            .set_playback_delay(true, 7)
            .expect("the index names a value");
        settings.set_playback_delay_dialog_disabled(true);
        settings.save(&file).expect("the settings can be stored");
        let content_on = fs::read_to_string(&file).expect("the settings file was written");
        let restored_on = Settings::load(&file).expect("the settings can be restored");
        settings
            .set_playback_delay(false, 7)
            .expect("the index names a value");
        settings.save(&file).expect("the settings can be stored");
        let content_off = fs::read_to_string(&file).expect("the settings file was written");
        let restored_off = Settings::load(&file).expect("the settings can be restored");
        remove_case(&file);
        assert_eq!(
            (
                content_on.contains("playback_delay: 0.566\n"),
                content_on.contains("playback_delay_dialog_disabled: true\n"),
                restored_on.playback_delay_on(),
                restored_on.playback_delay_index(),
                restored_on.playback_delay(),
                restored_on.playback_delay_dialog_disabled()
            ),
            (true, true, true, 7, 566, true)
        );
        assert_eq!(
            (
                content_off.contains("playback_delay:"),
                content_off.contains("playback_delay_dialog_disabled: true\n"),
                restored_off.playback_delay_on(),
                restored_off.playback_delay_index(),
                restored_off.playback_delay()
            ),
            (false, true, false, 0, 0)
        );
    }

    #[test]
    fn a_playback_delay_of_an_earlier_file_is_brought_to_the_nearest_value_and_0_reads_as_off() {
        // FR-095: 0.3 of a file before 0.39.0 becomes 0.283, the integer 10 of a file before
        // 0.38.0 is 1.0 seconds and becomes 1.131, 7.5 becomes 3.2, 0 and 0.0 are no delay, and
        // a value that is no number is no delay
        let file = case_file("delay-earlier");
        fs::create_dir_all(file.parent().expect("a parent")).expect("the directory can be created");
        let read = |content: &str| {
            fs::write(&file, content).expect("the file can be written");
            let settings = Settings::load(&file).expect("the file is read");
            (
                settings.playback_delay_on(),
                settings.playback_delay_index(),
                settings.playback_delay(),
            )
        };
        let tenths_of_a_second = read("playback_delay: 0.3\n");
        let integer = read("playback_delay: 10\n");
        let beyond = read("playback_delay: 7.5\n");
        let zero = read("playback_delay: 0\n");
        let zero_decimal = read("playback_delay: 0.0\n");
        let no_number = read("playback_delay: fast\n");
        remove_case(&file);
        assert_eq!(
            (tenths_of_a_second, integer, beyond),
            ((true, 5, 283), (true, 9, 1131), (true, 12, 3200))
        );
        assert_eq!(
            (zero, zero_decimal, no_number),
            ((false, 0, 0), (false, 0, 0), (false, 0, 0))
        );
    }

    #[test]
    fn the_nearest_playback_delay_is_the_smaller_one_where_two_are_equally_near() {
        // FR-095: 60 lies between 50 and 71, 85 between 71 and 100
        assert_eq!(
            (
                nearest_playback_delay_index(0),
                nearest_playback_delay_index(60),
                nearest_playback_delay_index(85),
                nearest_playback_delay_index(1000),
                nearest_playback_delay_index(u32::MAX)
            ),
            (0, 0, 1, 9, 12)
        );
    }

    #[test]
    fn a_playback_delay_is_set_by_its_switch_and_its_index() {
        // FR-518, FR-543: on with the last index, off keeping the index
        let mut settings = Settings::default();
        let on = settings.set_playback_delay(true, 12);
        let while_on = (settings.playback_delay(), settings.playback_delay_index());
        let off = settings.set_playback_delay(false, 12);
        assert_eq!(
            (
                on,
                while_on,
                off,
                settings.playback_delay(),
                settings.playback_delay_index()
            ),
            (Ok(()), (3200, 12), Ok(()), 0, 12)
        );
    }

    #[test]
    fn an_index_that_names_no_playback_delay_is_refused_with_the_message() {
        // FR-518: the index 13 names no value, nothing being set
        let mut settings = Settings::default();
        settings
            .set_playback_delay(true, 2)
            .expect("the index names a value");
        let refused = settings.set_playback_delay(true, 13);
        assert_eq!(
            (refused, settings.playback_delay_index()),
            (
                Err(SettingsError::Constraint(
                    "The playback delay is off or one of 0.05, 0.07, 0.10, 0.14, 0.20, 0.28, 0.40, 0.57, 0.80, 1.13, 1.60, 2.26, 3.20 seconds.".to_owned()
                )),
                2
            )
        );
    }

    #[test]
    fn the_playback_delay_violation_is_none_for_off_and_the_thirteen_values_only() {
        // FR-518
        assert_eq!(
            (
                playback_delay_violation(None),
                playback_delay_violation(Some(50)),
                playback_delay_violation(Some(3200)),
                playback_delay_violation(Some(570)).is_some(),
                playback_delay_violation(Some(0)).is_some()
            ),
            (None, None, None, true, true)
        );
    }

    #[test]
    fn the_playback_delay_dialog_setting_is_switched() {
        // FR-543, FR-544
        let mut settings = Settings::default();
        settings.set_playback_delay_dialog_disabled(true);
        let on = settings.playback_delay_dialog_disabled();
        settings.set_playback_delay_dialog_disabled(false);
        assert_eq!(
            (on, settings.playback_delay_dialog_disabled()),
            (true, false)
        );
    }
}
