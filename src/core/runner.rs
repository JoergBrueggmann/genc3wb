//! Whether the *compiler-compiler* is executable and can be run, and running it as a process.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::input_file::ProcessingState;

use std::fmt;
use std::path::Path;
use std::process::Command;

// realises FR-028, FR-029, FR-040, IR-014
/// What one run of the *compiler-compiler* produced.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RunResult {
    /// what the process wrote to standard output
    pub stdout: String,
    /// what the process wrote to standard error
    pub stderr: String,
    /// the exit code of the process
    pub exit_code: i32,
}

// realises FR-026, IR-013
/// Why the *compiler-compiler* could not be run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunError {
    /// the named file is not executable; carries the path
    NotExecutable(String),
    /// the process could not be started or waited for; carries the reason of the operating system
    Process(String),
}

impl fmt::Display for RunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RunError::NotExecutable(path) => write!(formatter, "{path}: not executable"),
            RunError::Process(reason) => write!(formatter, "process: {reason}"),
        }
    }
}

impl std::error::Error for RunError {}

// realises FR-025
/// Yields whether the file at `path` exists and is executable by the user.
pub fn is_executable(path: &str) -> bool {
    let path = Path::new(path);
    if !path.is_file() {
        return false;
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        path.metadata()
            .map(|metadata| metadata.permissions().mode() & 0o111 != 0)
            .unwrap_or(false)
    }
    #[cfg(not(unix))]
    {
        true
    }
}

// realises FR-026
/// Yields whether the *compiler-compiler* can be run: the file is executable and every
/// *input group* is in the state `ValidFileTextUntouched`.
pub fn is_ready(executable: bool, input_states: [ProcessingState; 2]) -> bool {
    executable
        && input_states
            .iter()
            .all(|state| *state == ProcessingState::ValidFileTextUntouched)
}

// realises IR-013, IR-014, C-003
/// Runs the *compiler-compiler* at `path` as a separate process and waits for it.
///
/// * `args` are the arguments handed to the process; none until the Specification names them.
/// * Standard output and standard error are read as UTF-8; a byte sequence that is not UTF-8 is
///   replaced by the replacement character.
/// * A process ended by a signal has no exit code; `-1` is yielded for it.
///
/// # Errors
/// Returns [`RunError::NotExecutable`] where `path` is not executable, and [`RunError::Process`]
/// where the process cannot be started or waited for.
pub fn run(path: &str, args: &[String]) -> Result<RunResult, RunError> {
    if !is_executable(path) {
        return Err(RunError::NotExecutable(path.to_owned()));
    }
    let output = Command::new(path)
        .args(args)
        .output()
        .map_err(|error| RunError::Process(error.to_string()))?;
    Ok(RunResult {
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        exit_code: output.status.code().unwrap_or(-1),
    })
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : RunnerGroup::set_path, RunnerGroup::set_input_state, RunnerGroup::completed */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readiness_needs_an_executable_and_two_untouched_valid_files() {
        // FR-026
        let ready = [ProcessingState::ValidFileTextUntouched; 2];
        assert!(is_ready(true, ready));
        assert!(!is_ready(false, ready));
        assert!(!is_ready(
            true,
            [
                ProcessingState::ValidFileTextUntouched,
                ProcessingState::ValidFileTextChanged
            ]
        ));
        assert!(!is_ready(
            true,
            [
                ProcessingState::UnknownFileTextUntouched,
                ProcessingState::ValidFileTextUntouched
            ]
        ));
    }

    #[test]
    fn missing_file_is_not_executable() {
        // FR-025
        assert!(!is_executable(""));
        assert!(!is_executable("/no/such/file"));
        let dir = std::env::temp_dir();
        assert!(!is_executable(&dir.to_string_lossy()));
    }

    #[test]
    fn run_of_a_file_that_is_not_executable_fails() {
        // FR-025, IR-013
        assert_eq!(
            run("/no/such/file", &[]),
            Err(RunError::NotExecutable("/no/such/file".into()))
        );
    }

    #[cfg(unix)]
    #[test]
    fn shell_is_executable_and_its_output_is_read() {
        // FR-025, IR-013, IR-014, C-003
        assert!(is_executable("/bin/sh"));
        let result = run(
            "/bin/sh",
            &["-c".into(), "printf out; printf err 1>&2; exit 3".into()],
        )
        .expect("the shell runs");
        assert_eq!(
            result,
            RunResult {
                stdout: "out".into(),
                stderr: "err".into(),
                exit_code: 3
            }
        );
    }
}
