//! The *settings file*: the two times, the *automatic setting* and the paths of *product*, in YAML.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use serde::{Deserialize, Serialize};

use std::fmt;
use std::fs;
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
/// The path of the *build system* where the *settings file* does not hold one: the copy that the
/// distribution carries beside the workbench.
pub const DEFAULT_BUILD_SYSTEM_PATH: &str = "./genc3/bin/genc3d";

// realises IR-011, IR-012
/// Why the settings could not be restored or stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsError {
    /// the *settings file* could not be read or written; carries the reason of the operating system
    Io(String),
    /// the *settings file* is no YAML mapping of the keys of [`Settings`]; carries the reason
    Malformed(String),
    /// the two times violate FR-133 or FR-134; carries the message naming the constraint
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

// realises FR-090, FR-091, FR-093, FR-094, FR-133, FR-134
/// The *idle time*, the *long idle time*, the *automatic setting* and the paths *product*
/// restores between sessions.
///
/// * The two times are held in milliseconds, as the code editors take them; the *settings file*
///   and the settings dialog carry them in seconds with one decimal.
/// * The two times satisfy FR-133 and FR-134: the *idle time* is at least `MIN_IDLE_TIME`, and
///   the *long idle time* is greater than the *idle time*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// the *idle time* in milliseconds
    idle_time: u32,
    /// the *long idle time* in milliseconds
    long_idle_time: u32,
    /// whether the *automatic setting* is on
    automatic: bool,
    /// the path of the *network file*
    network_path: String,
    /// the path of the *build system*
    build_system_path: String,
}

impl Default for Settings {
    // realises FR-094
    /// The settings where no *settings file* exists: the two default times, the *automatic
    /// setting* on, the *build system* of the distribution, and no *network file*.
    fn default() -> Self {
        Settings {
            idle_time: DEFAULT_IDLE_TIME,
            long_idle_time: DEFAULT_LONG_IDLE_TIME,
            automatic: DEFAULT_AUTOMATIC,
            network_path: String::new(),
            build_system_path: DEFAULT_BUILD_SYSTEM_PATH.to_owned(),
        }
    }
}

// realises FR-093, C-007
/// The *settings file* as YAML holds it: one key per value, each key absent where its default
/// holds.
///
/// * No path of the *node window* is among the keys: every path of the *node window* is named
///   from the *node description* when a *node* is opened (FR-086, FR-087, FR-102).
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
    #[serde(default)]
    network_path: String,
    #[serde(default = "default_build_system_path")]
    build_system_path: String,
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

/// The default of the key `build_system_path`, for a *settings file* that lacks it.
fn default_build_system_path() -> String {
    DEFAULT_BUILD_SYSTEM_PATH.to_owned()
}

impl Settings {
    // realises IR-011, IR-012
    /// Yields the path of the *settings file*: 'genc3wb.yaml' in the working directory.
    pub fn default_path() -> PathBuf {
        PathBuf::from(SETTINGS_FILE_NAME)
    }

    // realises FR-090, FR-094, FR-095, IR-011
    /// Restores the settings stored at the last termination.
    ///
    /// * Where the *settings file* does not exist, the default settings are yielded (FR-094).
    /// * A key the settings do not know is ignored, and a key that is absent takes its default,
    ///   so that a file of another version is read.
    /// * The two times are read in seconds and held in milliseconds; where they violate FR-133
    ///   or FR-134, the *idle time* is raised to the least one and the *long idle time* to the
    ///   *idle time* plus one step.
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
        Ok(Settings {
            idle_time,
            long_idle_time,
            automatic: stored.automatic,
            network_path: stored.network_path,
            build_system_path: stored.build_system_path,
        })
    }

    // realises FR-091, FR-099, IR-012
    /// Stores the settings, to be restored at the next start.
    ///
    /// * The directory of the file is created where it does not exist.
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
            network_path: self.network_path.clone(),
            build_system_path: self.build_system_path.clone(),
        };
        let content = serde_norway::to_string(&stored)
            .map_err(|error| SettingsError::Malformed(error.to_string()))?;
        fs::write(path, content).map_err(|error| SettingsError::Io(error.to_string()))
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

    /// Yields the path of the *network file*, empty where none is stored.
    pub fn network_path(&self) -> &str {
        &self.network_path
    }

    // realises FR-012, FR-091
    /// Sets the path of the *network file*.
    pub fn set_network_path(&mut self, path: &str) {
        self.network_path = path.to_owned();
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
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : Workbench::default, Workbench::try_set_times, Workbench::report_processing_time,
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
            ("", DEFAULT_BUILD_SYSTEM_PATH, 2000, 16000, true)
        );
    }

    #[test]
    fn the_two_paths_are_held_apart() {
        // FR-091
        let mut settings = Settings::default();
        settings.set_network_path("n.gc3n");
        settings.set_build_system_path("/usr/bin/genc3d");
        assert_eq!(
            (settings.network_path(), settings.build_system_path()),
            ("n.gc3n", "/usr/bin/genc3d")
        );
    }

    #[test]
    fn settings_survive_save_and_load_with_the_times_in_seconds() {
        // FR-090, FR-091, FR-093, IR-011, IR-012
        let file = case_file("roundtrip");
        let mut settings = Settings::default();
        settings.set_network_path("/tmp/n.gc3n");
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
                "a",
                DEFAULT_IDLE_TIME,
                DEFAULT_LONG_IDLE_TIME,
                DEFAULT_AUTOMATIC,
                DEFAULT_BUILD_SYSTEM_PATH
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
    fn the_settings_file_lies_in_the_working_directory() {
        // IR-011, IR-012
        assert_eq!(Settings::default_path(), Path::new(SETTINGS_FILE_NAME));
    }
}
