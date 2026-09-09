//! The paths of the two input files, of the *compiler-compiler* and of the *output files*, restored and stored.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::input_file::InputKind;

use std::collections::BTreeMap;
use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

// realises FR-031
/// The number of the first *output file*.
pub const MIN_FILE_NUMBER: u8 = 1;
/// The number of the last *output file*.
pub const MAX_FILE_NUMBER: u8 = 9;

/// The key of the path of the *compiler-compiler input file* in the settings file.
const KEY_CC_INPUT_PATH: &str = "cc_input_path";
/// The key of the path of the *compiler input file* in the settings file.
const KEY_C_INPUT_PATH: &str = "c_input_path";
/// The key of the path of the *compiler-compiler* in the settings file.
const KEY_COMPILER_COMPILER_PATH: &str = "compiler_compiler_path";
/// The prefix of the key of the path of an *output file* in the settings file, followed by its number.
const KEY_OUTPUT_PATH_PREFIX: &str = "output_path_";

// realises IR-011, IR-012
/// Why the settings could not be restored or stored.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SettingsError {
    /// the settings file could not be read or written; carries the reason of the operating system
    Io(String),
    /// a line of the settings file is not `key = value`; carries the line
    Malformed(String),
    /// an *output file* number outside `MIN_FILE_NUMBER..=MAX_FILE_NUMBER`
    FileNumberOutOfRange(u8),
}

impl fmt::Display for SettingsError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SettingsError::Io(reason) => write!(formatter, "settings file: {reason}"),
            SettingsError::Malformed(line) => {
                write!(formatter, "settings line is not key = value: {line}")
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

// realises FR-031, FR-048, FR-049
/// The paths *product* restores between sessions.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Settings {
    /// the path of each input file, indexed by `InputKind::index`
    input_paths: [String; 2],
    /// the path of the *compiler-compiler*
    compiler_compiler_path: String,
    /// the path of each enabled *output file*, by its number
    output_paths: BTreeMap<u8, String>,
}

impl Settings {
    /// Yields the path of the settings file: 'genc3wb/settings.txt' in the configuration directory
    /// of the user, or in the working directory where the user has none.
    pub fn default_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("genc3wb")
            .join("settings.txt")
    }

    // realises FR-048, IR-011
    /// Restores the paths stored at the last termination.
    ///
    /// * Where the settings file does not exist, the default settings are yielded.
    /// * A key the settings do not know is ignored, so that a later version's file is read.
    ///
    /// # Errors
    /// Returns [`SettingsError::Io`] where the file exists but cannot be read, and
    /// [`SettingsError::Malformed`] where a line is not `key = value`.
    pub fn load(path: &Path) -> Result<Settings, SettingsError> {
        if !path.is_file() {
            return Ok(Settings::default());
        }
        let content =
            fs::read_to_string(path).map_err(|error| SettingsError::Io(error.to_string()))?;
        let mut settings = Settings::default();
        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            let (key, value) = line
                .split_once('=')
                .ok_or_else(|| SettingsError::Malformed(line.to_owned()))?;
            let (key, value) = (key.trim(), value.trim());
            match key {
                KEY_CC_INPUT_PATH => {
                    settings.set_input_path(InputKind::CompilerCompilerInput, value)
                }
                KEY_C_INPUT_PATH => settings.set_input_path(InputKind::CompilerInput, value),
                KEY_COMPILER_COMPILER_PATH => settings.set_compiler_compiler_path(value),
                _ => {
                    if let Some(number) = key.strip_prefix(KEY_OUTPUT_PATH_PREFIX) {
                        let number = number.parse::<u8>().unwrap_or(0);
                        let _ = settings.set_output_path(number, value);
                    }
                }
            }
        }
        Ok(settings)
    }

    // realises FR-049, IR-012
    /// Stores the paths, to be restored at the next start.
    ///
    /// * The directory of the file is created where it does not exist.
    ///
    /// # Errors
    /// Returns [`SettingsError::Io`] where the file cannot be written.
    pub fn save(&self, path: &Path) -> Result<(), SettingsError> {
        if let Some(directory) = path.parent() {
            fs::create_dir_all(directory).map_err(|error| SettingsError::Io(error.to_string()))?;
        }
        fs::write(path, self.rendering()).map_err(|error| SettingsError::Io(error.to_string()))
    }

    /// Yields the content of the settings file: one `key = value` line per path.
    fn rendering(&self) -> String {
        let mut content = String::new();
        content.push_str(&format!("{KEY_CC_INPUT_PATH} = {}\n", self.input_paths[0]));
        content.push_str(&format!("{KEY_C_INPUT_PATH} = {}\n", self.input_paths[1]));
        content.push_str(&format!(
            "{KEY_COMPILER_COMPILER_PATH} = {}\n",
            self.compiler_compiler_path
        ));
        for (number, path) in &self.output_paths {
            content.push_str(&format!("{KEY_OUTPUT_PATH_PREFIX}{number} = {path}\n"));
        }
        content
    }

    /// Yields the path of one input file, empty where none is stored.
    pub fn input_path(&self, kind: InputKind) -> &str {
        &self.input_paths[kind.index()]
    }

    // realises FR-012, FR-049
    /// Sets the path of one input file.
    pub fn set_input_path(&mut self, kind: InputKind, path: &str) {
        self.input_paths[kind.index()] = path.to_owned();
    }

    /// Yields the path of the *compiler-compiler*, empty where none is stored.
    pub fn compiler_compiler_path(&self) -> &str {
        &self.compiler_compiler_path
    }

    // realises FR-024, FR-049
    /// Sets the path of the *compiler-compiler*.
    pub fn set_compiler_compiler_path(&mut self, path: &str) {
        self.compiler_compiler_path = path.to_owned();
    }

    // realises FR-031, FR-038
    /// Yields the path of one enabled *output file*, `None` where that number is not enabled.
    pub fn output_path(&self, number: u8) -> Option<&str> {
        self.output_paths.get(&number).map(String::as_str)
    }

    // realises FR-038, FR-049
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
        dir.join("sub").join("settings.txt")
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
        // FR-048, FR-049
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
        // FR-048, FR-049, IR-011, IR-012
        let file = case_file("roundtrip");
        let mut settings = Settings::default();
        settings.set_input_path(InputKind::CompilerCompilerInput, "/tmp/a b.cc");
        settings.set_input_path(InputKind::CompilerInput, "/tmp/b.c");
        settings.set_compiler_compiler_path("/opt/genc3");
        settings
            .set_output_path(2, "/tmp/out2.txt")
            .expect("2 is within range");
        settings.set_output_path(7, "").expect("7 is within range");
        assert_eq!(settings.save(&file), Ok(()));
        assert_eq!(Settings::load(&file), Ok(settings));
        let _ = fs::remove_dir_all(
            file.parent()
                .and_then(Path::parent)
                .expect("the case directory"),
        );
    }

    #[test]
    fn missing_settings_file_yields_the_defaults() {
        // FR-048
        let file = case_file("missing");
        assert_eq!(Settings::load(&file), Ok(Settings::default()));
    }

    #[test]
    fn malformed_line_is_reported_and_unknown_key_ignored() {
        // IR-011
        let file = case_file("malformed");
        fs::create_dir_all(file.parent().expect("a parent")).expect("the directory can be created");
        fs::write(&file, "cc_input_path = a\nfuture_key = b\n\n").expect("the file can be written");
        let settings = Settings::load(&file).expect("an unknown key is no error");
        assert_eq!(settings.input_path(InputKind::CompilerCompilerInput), "a");
        fs::write(&file, "no equals sign\n").expect("the file can be written");
        assert_eq!(
            Settings::load(&file),
            Err(SettingsError::Malformed("no equals sign".into()))
        );
        let _ = fs::remove_dir_all(
            file.parent()
                .and_then(Path::parent)
                .expect("the case directory"),
        );
    }
}
