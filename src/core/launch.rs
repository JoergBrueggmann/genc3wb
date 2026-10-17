//! The launch of *product*: the folder of its executable, the *handed file* of its command line,
//! a second *product* started on a gc3n-file, and the working directory of a *network file*.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The usage line written with the message of a refused argument.
const USAGE: &str = "usage: genc3wb [<path of a gc3n-file>]";

// realises FR-094, IR-011
/// Yields the folder of the executable of this process, the symbolic links of its path resolved,
/// so that the executable of the application bundle yields the folder 'bin' it links into
/// (C-009); the working directory where the path cannot be determined.
pub fn executable_folder() -> PathBuf {
    std::env::current_exe()
        .and_then(|executable| executable.canonicalize())
        .ok()
        .and_then(|executable| executable.parent().map(Path::to_path_buf))
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_default())
}

// realises IR-206
/// Yields the *handed file* the command line `args` names, the program name left out: `None` for
/// no argument and the path for one.
///
/// * An argument of the form `-psn_…`, by which older versions of macOS named the process of an
///   application it launched, is passed over.
///
/// # Errors
/// Returns the message for standard error where there are more arguments than one, or the one
/// begins with `-`.
pub fn handed_file<I: IntoIterator<Item = String>>(args: I) -> Result<Option<String>, String> {
    let mut args = args.into_iter().filter(|arg| !arg.starts_with("-psn_"));
    let first = args.next();
    if args.next().is_some() {
        return Err(format!("genc3wb: at most one argument is taken\n{USAGE}"));
    }
    match first {
        None => Ok(None),
        Some(arg) if arg.starts_with('-') => {
            Err(format!("genc3wb: unknown option '{arg}'\n{USAGE}"))
        }
        Some(arg) => Ok(Some(arg)),
    }
}

// realises FR-477, IR-208
/// Starts a second *product* on the gc3n-file at `path`: the executable of this process with
/// `path` as its only argument, in the folder of `path`, its standard streams closed; the process
/// is waited for on a thread of its own, so that it leaves no zombie.
///
/// # Errors
/// Returns the reason where the executable cannot be determined or the process cannot be
/// started.
pub fn start_second(path: &str) -> Result<(), String> {
    let executable = std::env::current_exe().map_err(|error| error.to_string())?;
    let mut command = Command::new(executable);
    command
        .arg(path)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    if let Some(folder) = folder_of(path) {
        command.current_dir(folder);
    }
    let mut child = command.spawn().map_err(|error| error.to_string())?;
    std::thread::spawn(move || {
        let _ = child.wait();
    });
    Ok(())
}

// realises FR-478
/// Yields `path` made absolute against the working directory; `path` as it is where it is
/// absolute or empty, or the working directory cannot be determined.
pub fn absolute(path: &str) -> String {
    if path.is_empty() || Path::new(path).is_absolute() {
        return path.to_owned();
    }
    std::env::current_dir()
        .map(|directory| directory.join(path).to_string_lossy().into_owned())
        .unwrap_or_else(|_| path.to_owned())
}

// realises FR-478
/// Makes the folder of the file at `path` the working directory. The empty path and a path
/// without a folder change nothing.
///
/// # Errors
/// Returns the reason where the folder cannot be entered.
pub fn enter_folder_of(path: &str) -> Result<(), String> {
    match folder_of(path) {
        Some(folder) => std::env::set_current_dir(folder).map_err(|error| error.to_string()),
        None => Ok(()),
    }
}

/// Yields the folder of the file at `path`; `None` where `path` is empty or names no folder.
fn folder_of(path: &str) -> Option<&Path> {
    if path.is_empty() {
        return None;
    }
    Path::new(path)
        .parent()
        .filter(|folder| !folder.as_os_str().is_empty())
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : Workbench::default, workbench::file_opened, InputGroup::lock_network */
#[cfg(test)]
mod tests {
    use super::*;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|arg| (*arg).to_owned()).collect()
    }

    #[test]
    fn the_folder_of_the_executable_is_absolute_and_exists() {
        // IR-011
        let folder = executable_folder();
        assert!(
            folder.is_absolute() && folder.is_dir(),
            "{}",
            folder.display()
        );
    }

    #[test]
    fn no_argument_hands_no_file_and_one_hands_it() {
        // IR-206
        assert_eq!(
            (
                handed_file(args(&[])),
                handed_file(args(&["n.gc3n"])),
                handed_file(args(&["-psn_0_12345", "/tmp/n.gc3n"]))
            ),
            (
                Ok(None),
                Ok(Some("n.gc3n".to_owned())),
                Ok(Some("/tmp/n.gc3n".to_owned()))
            )
        );
    }

    #[test]
    fn two_arguments_and_an_option_are_refused_with_the_usage() {
        // IR-206
        let two = handed_file(args(&["a.gc3n", "b.gc3n"]));
        let option = handed_file(args(&["--help"]));
        assert_eq!(
            (
                two.as_ref().err().map(|m| m.contains(USAGE)),
                option
                    .as_ref()
                    .err()
                    .map(|m| m.contains("--help") && m.contains(USAGE))
            ),
            (Some(true), Some(true))
        );
    }

    #[test]
    fn an_absolute_or_empty_path_stays_and_a_relative_one_is_rooted() {
        // FR-478
        let rooted = absolute("n.gc3n");
        assert_eq!(
            (
                absolute("/tmp/n.gc3n"),
                absolute(""),
                Path::new(&rooted).is_absolute() && rooted.ends_with("n.gc3n")
            ),
            ("/tmp/n.gc3n".to_owned(), String::new(), true)
        );
    }

    #[test]
    fn the_folder_of_a_file_is_entered_and_a_bare_name_changes_nothing() {
        // FR-478
        let folder = std::env::temp_dir().join(format!("genc3wb-launch-{}", std::process::id()));
        std::fs::create_dir_all(&folder).expect("the temporary directory can be created");
        let file = folder.join("n.gc3n");
        let entered = enter_folder_of(&file.to_string_lossy());
        let inside = std::env::current_dir()
            .ok()
            .and_then(|dir| dir.canonicalize().ok())
            == folder.canonicalize().ok();
        let unchanged = enter_folder_of("n.gc3n");
        let missing = enter_folder_of("/nowhere/at/all/n.gc3n").is_err();
        let _ = std::fs::remove_dir_all(&folder);
        assert_eq!(
            (entered, inside, unchanged, missing),
            (Ok(()), true, Ok(()), true)
        );
    }

    #[test]
    fn a_second_product_is_started_on_a_file() {
        // FR-477, IR-208: the executable of this process is the test executable, which takes the
        // path as a filter, runs no case and exits
        assert_eq!(start_second("/tmp/genc3wb-launch-second.gc3n"), Ok(()));
    }
}
