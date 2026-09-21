//! The *settings file*: the two times and the paths of *product*, restored and stored in YAML.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::input_file::InputKind;

use serde::{Deserialize, Serialize};

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

// realises FR-031
/// The number of the first *output file*.
pub const MIN_FILE_NUMBER: u8 = 1;
/// The number of the last *output file*.
pub const MAX_FILE_NUMBER: u8 = 9;

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
    /// an *output file* number outside `MIN_FILE_NUMBER..=MAX_FILE_NUMBER`
    FileNumberOutOfRange(u8),
}

impl fmt::Display for SettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingsError::Io(reason) => write!(formatter, "settings file: {reason}"),
            SettingsError::Malformed(reason) => {
                write!(formatter, "settings file is not read: {reason}")
            }
            SettingsError::FileNumberOutOfRange(number) => {
                write!(
                    formatter,
                    "output file number {number} is not within {MIN_FILE_NUMBER}..={MAX_FILE_NUMBER}"
                )
            }
        }
    }
}

impl std::error::Error for SettingsError {}

// realises FR-031, FR-090, FR-091, FR-093, FR-094
/// The *idle time*, the *long idle time* and the paths *product* restores between sessions.
///
/// * The two times are held in seconds, as the *settings file* and the settings dialog carry them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Settings {
    /// the *idle time* in seconds
    idle_time: u32,
    /// the *long idle time* in seconds
    long_idle_time: u32,
    /// the path of each input file, indexed by `InputKind::index`
    input_paths: [String; 3],
    /// the path of the *compiler-compiler*
    compiler_compiler_path: String,
    /// the path of the *build system*
    build_system_path: String,
    /// the path of each enabled *output file*, by its number
    output_paths: BTreeMap<u8, String>,
}

impl Default for Settings {
    // realises FR-094
    /// The settings where no *settings file* exists: the two default times, the *build system* of
    /// the distribution, and no other path.
    fn default() -> Self {
        Settings {
            idle_time: DEFAULT_IDLE_TIME,
            long_idle_time: DEFAULT_LONG_IDLE_TIME,
            input_paths: Default::default(),
            compiler_compiler_path: String::new(),
            build_system_path: DEFAULT_BUILD_SYSTEM_PATH.to_owned(),
            output_paths: BTreeMap::new(),
        }
    }
}

// realises FR-093, C-007
/// The *settings file* as YAML holds it: one key per value, each key absent where its default
/// holds.
///
/// * No path of the *node window* is among the keys: neither those of its two input files, which
///   opening a *node* names from the *node description* (FR-086, FR-087), nor those of the
///   *compiler-compiler* and of the *output files*, since a *node* is served by the
///   *build system* and its outputs stand in the *network file*. They are held for the session
///   alone.
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
    /// * No path of the *node window* is restored; those are held for the session alone.
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
            input_paths: [String::new(), String::new(), stored.network_path],
            compiler_compiler_path: String::new(),
            build_system_path: stored.build_system_path,
            output_paths: BTreeMap::new(),
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
            network_path: self.input_paths[2].clone(),
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

    /// Yields the path of one input file, empty where none is stored.
    pub fn input_path(&self, kind: InputKind) -> &str {
        &self.input_paths[kind.index()]
    }

    // realises FR-012
    /// Sets the path of one input file.
    pub fn set_input_path(&mut self, kind: InputKind, path: &str) {
        self.input_paths[kind.index()] = path.to_owned();
    }

    /// Yields the path of the *compiler-compiler*, empty where none is stored.
    pub fn compiler_compiler_path(&self) -> &str {
        &self.compiler_compiler_path
    }

    // realises FR-024
    /// Sets the path of the *compiler-compiler*.
    pub fn set_compiler_compiler_path(&mut self, path: &str) {
        self.compiler_compiler_path = path.to_owned();
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

    // realises FR-031, FR-038
    /// Yields the path of one enabled *output file*, `None` where that number is not enabled.
    pub fn output_path(&self, number: u8) -> Option<&str> {
        self.output_paths.get(&number).map(String::as_str)
    }

    // realises FR-038
    /// Sets the path of one *output file*, enabling it.
    ///
    /// # Errors
    /// Returns [`SettingsError::FileNumberOutOfRange`] where `number` is not within
    /// `MIN_FILE_NUMBER..=MAX_FILE_NUMBER`.
    pub fn set_output_path(&mut self, number: u8, path: &str) -> Result<(), SettingsError> {
        if !(MIN_FILE_NUMBER..=MAX_FILE_NUMBER).contains(&number) {
            return Err(SettingsError::FileNumberOutOfRange(number));
        }
        self.output_paths.insert(number, path.to_owned());
        Ok(())
    }

    // realises FR-037
    /// Removes the path of one *output file*, so that it is no longer presented.
    pub fn remove_output_path(&mut self, number: u8) {
        self.output_paths.remove(&number);
    }

    /// Yields the numbers of the enabled *output files*, in ascending order.
    pub fn output_numbers(&self) -> Vec<u8> {
        self.output_paths.keys().copied().collect()
    }
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : Workbench::default, InputGroup::set_path, RunnerGroup::set_path, OutputGroup::set_file_path */
#[cfg(test)]
mod tests {
    use super::*;

    fn case_file(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("genc3wb-settings-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        dir.join("sub").join(SETTINGS_FILE_NAME)
    }

    #[test]
    fn default_settings_hold_no_path() {
        let settings = Settings::default();
        assert_eq!(settings.input_path(InputKind::CompilerCompilerInput), "");
        assert_eq!(settings.input_path(InputKind::CompilerInput), "");
        assert_eq!(settings.compiler_compiler_path(), "");
        assert_eq!(settings.output_numbers(), Vec::<u8>::new());
    }

    #[test]
    fn paths_of_each_kind_are_held_apart() {
        // FR-091
        let mut settings = Settings::default();
        settings.set_input_path(InputKind::CompilerCompilerInput, "a.cc");
        settings.set_input_path(InputKind::CompilerInput, "b.c");
        settings.set_compiler_compiler_path("/usr/bin/genc3");
        assert_eq!(
            settings.input_path(InputKind::CompilerCompilerInput),
            "a.cc"
        );
        assert_eq!(settings.input_path(InputKind::CompilerInput), "b.c");
        assert_eq!(settings.compiler_compiler_path(), "/usr/bin/genc3");
    }

    #[test]
    fn output_path_within_range_is_held() {
        // FR-031, FR-038
        let mut settings = Settings::default();
        assert_eq!(settings.set_output_path(MIN_FILE_NUMBER, "one.txt"), Ok(()));
        assert_eq!(
            settings.set_output_path(MAX_FILE_NUMBER, "nine.txt"),
            Ok(())
        );
        assert_eq!(settings.output_path(1), Some("one.txt"));
        assert_eq!(settings.output_path(9), Some("nine.txt"));
        assert_eq!(settings.output_path(5), None);
        assert_eq!(settings.output_numbers(), vec![1, 9]);
    }

    #[test]
    fn output_number_out_of_range_is_rejected() {
        // FR-031
        let mut settings = Settings::default();
        assert_eq!(
            settings.set_output_path(0, "x"),
            Err(SettingsError::FileNumberOutOfRange(0))
        );
        assert_eq!(
            settings.set_output_path(10, "x"),
            Err(SettingsError::FileNumberOutOfRange(10))
        );
        assert_eq!(settings.output_numbers(), Vec::<u8>::new());
    }

    #[test]
    fn removed_output_path_is_no_longer_held() {
        // FR-037
        let mut settings = Settings::default();
        settings
            .set_output_path(3, "three.txt")
            .expect("3 is within range");
        settings.remove_output_path(3);
        assert_eq!(settings.output_path(3), None);
        settings.remove_output_path(3);
    }

    #[test]
    fn settings_survive_save_and_load() {
        // FR-090, FR-091, FR-093, IR-011, IR-012
        let file = case_file("roundtrip");
        let mut settings = Settings::default();
        settings.set_input_path(InputKind::CompilerCompilerInput, "/tmp/a b.cc");
        settings.set_input_path(InputKind::CompilerInput, "/tmp/b.c");
        settings.set_input_path(InputKind::Network, "/tmp/n.gc3n");
        settings.set_build_system_path("/opt/genc3d");
        settings.set_idle_time(3);
        settings.set_long_idle_time(9);
        settings.set_compiler_compiler_path("/opt/genc3");
        settings
            .set_output_path(2, "/tmp/out2.txt")
            .expect("2 is within range");
        settings.set_output_path(7, "").expect("7 is within range");
        assert_eq!(settings.save(&file), Ok(()));
        // no path of the *node window* is stored: they are held for the session alone
        let mut expected = Settings::default();
        expected.set_input_path(InputKind::Network, "/tmp/n.gc3n");
        expected.set_build_system_path("/opt/genc3d");
        expected.set_idle_time(3);
        expected.set_long_idle_time(9);
        assert_eq!(Settings::load(&file), Ok(expected));
        let _ = fs::remove_dir_all(
            file.parent()
                .and_then(Path::parent)
                .expect("the case directory"),
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
        assert_eq!(
            (
                settings.input_path(InputKind::Network),
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
        let _ = fs::remove_dir_all(
            file.parent()
                .and_then(Path::parent)
                .expect("the case directory"),
        );
    }

    #[test]
    fn a_file_that_is_no_yaml_mapping_is_reported() {
        // IR-011
        let file = case_file("notyaml");
        fs::create_dir_all(file.parent().expect("a parent")).expect("the directory can be created");
        fs::write(&file, "idle_time: [1, 2\n").expect("the file can be written");
        assert!(matches!(
            Settings::load(&file),
            Err(SettingsError::Malformed(_))
        ));
        let _ = fs::remove_dir_all(
            file.parent()
                .and_then(Path::parent)
                .expect("the case directory"),
        );
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
