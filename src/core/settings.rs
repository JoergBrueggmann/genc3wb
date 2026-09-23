//! The *settings file*: the two times and the paths of *product*, restored and stored in YAML.
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
/// The *idle time* in seconds where the *settings file* does not hold one.
pub const DEFAULT_IDLE_TIME: u32 = 2;
/// The *long idle time* in seconds where the *settings file* does not hold one.
pub const DEFAULT_LONG_IDLE_TIME: u32 = 16;
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
}

impl fmt::Display for SettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingsError::Io(reason) => write!(formatter, "settings file: {reason}"),
            SettingsError::Malformed(reason) => {
                write!(formatter, "settings file is not read: {reason}")
            }
        }
    }
}

impl std::error::Error for SettingsError {}

// realises FR-090, FR-091, FR-093, FR-094
/// The *idle time*, the *long idle time* and the paths *product* restores between sessions.
///
/// * The two times are held in seconds, as the *settings file* and the settings dialog carry them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// the *idle time* in seconds
    idle_time: u32,
    /// the *long idle time* in seconds
    long_idle_time: u32,
    /// the path of the *network file*
    network_path: String,
    /// the path of the *build system*
    build_system_path: String,
}

impl Default for Settings {
    // realises FR-094
    /// The settings where no *settings file* exists: the two default times, the *build system* of
    /// the distribution, and no *network file*.
    fn default() -> Self {
        Settings {
            idle_time: DEFAULT_IDLE_TIME,
            long_idle_time: DEFAULT_LONG_IDLE_TIME,
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
    #[serde(default = "default_idle_time")]
    idle_time: u32,
    #[serde(default = "default_long_idle_time")]
    long_idle_time: u32,
    #[serde(default)]
    network_path: String,
    #[serde(default = "default_build_system_path")]
    build_system_path: String,
}

/// The default of the key `idle_time`, for a *settings file* that lacks it.
fn default_idle_time() -> u32 {
    DEFAULT_IDLE_TIME
}

/// The default of the key `long_idle_time`, for a *settings file* that lacks it.
fn default_long_idle_time() -> u32 {
    DEFAULT_LONG_IDLE_TIME
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
        Ok(Settings {
            idle_time: stored.idle_time.max(1),
            long_idle_time: stored.long_idle_time.max(1),
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
            idle_time: self.idle_time,
            long_idle_time: self.long_idle_time,
            network_path: self.network_path.clone(),
            build_system_path: self.build_system_path.clone(),
        };
        let content = serde_norway::to_string(&stored)
            .map_err(|error| SettingsError::Malformed(error.to_string()))?;
        fs::write(path, content).map_err(|error| SettingsError::Io(error.to_string()))
    }

    // realises FR-093, FR-095
    /// Yields the *idle time* in seconds.
    pub fn idle_time(&self) -> u32 {
        self.idle_time
    }

    // realises FR-099
    /// Sets the *idle time*; a value below 1 second becomes 1 second.
    pub fn set_idle_time(&mut self, seconds: u32) {
        self.idle_time = seconds.max(1);
    }

    // realises FR-093, FR-095
    /// Yields the *long idle time* in seconds.
    pub fn long_idle_time(&self) -> u32 {
        self.long_idle_time
    }

    // realises FR-099
    /// Sets the *long idle time*; a value below 1 second becomes 1 second.
    pub fn set_long_idle_time(&mut self, seconds: u32) {
        self.long_idle_time = seconds.max(1);
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
 * covers bridge    : Workbench::default, InputGroup::set_path, NetworkEditor::set_build_system_path */
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
    fn default_settings_hold_the_build_system_of_the_distribution_and_no_network_file() {
        // FR-094
        let settings = Settings::default();
        assert_eq!(
            (
                settings.network_path(),
                settings.build_system_path(),
                settings.idle_time(),
                settings.long_idle_time()
            ),
            (
                "",
                DEFAULT_BUILD_SYSTEM_PATH,
                DEFAULT_IDLE_TIME,
                DEFAULT_LONG_IDLE_TIME
            )
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
    fn settings_survive_save_and_load() {
        // FR-090, FR-091, FR-093, IR-011, IR-012
        let file = case_file("roundtrip");
        let mut settings = Settings::default();
        settings.set_network_path("/tmp/n.gc3n");
        settings.set_build_system_path("/opt/genc3d");
        settings.set_idle_time(3);
        settings.set_long_idle_time(9);
        let saved = settings.save(&file);
        let loaded = Settings::load(&file);
        remove_case(&file);
        assert_eq!((saved, loaded), (Ok(()), Ok(settings)));
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
                settings.build_system_path()
            ),
            (
                "a",
                DEFAULT_IDLE_TIME,
                DEFAULT_LONG_IDLE_TIME,
                DEFAULT_BUILD_SYSTEM_PATH
            )
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
    fn the_two_times_are_held_and_never_fall_below_one_second() {
        // FR-093, FR-099
        let mut settings = Settings::default();
        settings.set_idle_time(5);
        settings.set_long_idle_time(0);
        assert_eq!((settings.idle_time(), settings.long_idle_time()), (5, 1));
    }

    #[test]
    fn the_settings_file_lies_in_the_working_directory() {
        // IR-011, IR-012
        assert_eq!(Settings::default_path(), Path::new(SETTINGS_FILE_NAME));
    }
}
