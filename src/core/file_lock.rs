//! The *lock file* of a file: taken through the operating system, holding its *holder*, and the
//! *refusal* where another genc³wb holds it.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use std::fs::{self, File, TryLockError};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;
use std::time::{SystemTime, UNIX_EPOCH};

// realises FR-214, FR-215
/// The suffix of the name of a *lock file* behind the name of the file it locks.
pub const LOCK_SUFFIX: &str = ".lock";

/// How often a *lock file* is taken again where its *holder* removed it while it was taken.
const ATTEMPTS: usize = 3;

// realises FR-214, FR-215
/// Yields the path of the *lock file* of the file at `file`: its path with `.lock` appended.
pub fn lock_path_of(file: &Path) -> PathBuf {
    let mut path = file.as_os_str().to_owned();
    path.push(LOCK_SUFFIX);
    PathBuf::from(path)
}

// realises FR-219, IR-034, IR-037
/// The *holder* of a *lock file*: the genc³wb that holds it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Holder {
    /// the process number
    pub pid: u32,
    /// the name of the host; empty where it could not be determined
    pub host: String,
    /// the time the *lock file* was taken, as `YYYY-MM-DDTHH:MM:SSZ` of ISO 8601 in UTC
    pub since: String,
}

impl Holder {
    // realises IR-034
    /// Yields this process as the *holder* of a *lock file* taken at `now`.
    pub fn of_this_process(now: SystemTime) -> Holder {
        let seconds = now
            .duration_since(UNIX_EPOCH)
            .map_or(0, |duration| duration.as_secs());
        Holder {
            pid: std::process::id(),
            host: host_name().to_owned(),
            since: iso_of_unix_seconds(seconds),
        }
    }

    // realises IR-034
    /// Yields the content of a *lock file* held by this *holder*: three lines, `pid: `, `host: `
    /// and `since: `, each followed by its value.
    pub fn text(&self) -> String {
        format!(
            "pid: {}\nhost: {}\nsince: {}\n",
            self.pid, self.host, self.since
        )
    }

    // realises IR-037
    /// Yields the *holder* a *lock file* names by `text`, as [`Holder::text`] writes it; `None`
    /// where `text` does not have that form, as a *lock file* being written has not.
    pub fn of_text(text: &str) -> Option<Holder> {
        let mut lines = text.lines();
        let pid = lines.next()?.strip_prefix("pid: ")?.parse::<u32>().ok()?;
        let host = lines.next()?.strip_prefix("host: ")?.to_owned();
        let since = lines
            .next()?
            .strip_prefix("since: ")
            .filter(|since| is_iso_time(since))?
            .to_owned();
        lines
            .next()
            .is_none()
            .then_some(Holder { pid, host, since })
    }
}

// realises FR-216 to FR-220
/// A *refusal*: the file refused, and its *holder*, `None` where the *lock file* names none.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    /// the path of the file refused, as it was named
    pub file: String,
    /// the *holder* of its *lock file*
    pub holder: Option<Holder>,
}

// realises FR-214 to FR-222
/// What taking the *lock file* of a file yields.
#[derive(Debug)]
pub enum Taking {
    /// the *lock file* is held, until the [`FileLock`] is dropped
    Held(FileLock),
    /// the *lock file* cannot be created or locked, since its folder cannot be written; carries
    /// the reason of the operating system (FR-222)
    Unlockable(String),
    /// another genc³wb holds the *lock file*
    Refused(Refusal),
}

// realises FR-214, FR-215, FR-221, C-008
/// A *lock file* this process holds, by the advisory lock of the operating system: `flock` on
/// MacOS, so that it is released when the process terminates in whatever way.
///
/// * Dropping it removes the *lock file* and releases the lock (FR-221).
/// * A file that another genc³wb locked is refused, and so is one this process locked through
///   another [`FileLock`]: the lock belongs to the open file, not to the process.
#[derive(Debug)]
pub struct FileLock {
    /// the open *lock file*, locked
    file: File,
    /// the path of the *lock file*
    path: PathBuf,
}

impl FileLock {
    // realises FR-214, FR-215, FR-216, FR-218, FR-222, IR-034, IR-037
    /// Takes the *lock file* of the file at `file` and writes this process into it as its
    /// *holder*.
    ///
    /// * Where another holds the *lock file*, its *holder* is read from it (IR-037).
    /// * Where the *holder* removes the *lock file* between its opening and its locking here,
    ///   the locked file is no longer the *lock file*, and the *lock file* is taken again.
    pub fn take(file: &Path) -> Taking {
        let path = lock_path_of(file);
        for _ in 0..ATTEMPTS {
            let opened = File::options()
                .read(true)
                .write(true)
                .create(true)
                .truncate(false)
                .open(&path);
            let lock_file = match opened {
                Ok(lock_file) => lock_file,
                Err(error) => return Taking::Unlockable(error.to_string()),
            };
            match lock_file.try_lock() {
                Ok(()) => {}
                Err(TryLockError::WouldBlock) => {
                    let holder = fs::read_to_string(&path)
                        .ok()
                        .and_then(|text| Holder::of_text(&text));
                    return Taking::Refused(Refusal {
                        file: file.to_string_lossy().into_owned(),
                        holder,
                    });
                }
                Err(TryLockError::Error(error)) => return Taking::Unlockable(error.to_string()),
            }
            if !is_same_file(&lock_file, &path) {
                continue;
            }
            let text = Holder::of_this_process(SystemTime::now()).text();
            let written = lock_file
                .set_len(0)
                .and_then(|()| (&lock_file).write_all(text.as_bytes()));
            if let Err(error) = written {
                return Taking::Unlockable(error.to_string());
            }
            return Taking::Held(FileLock {
                file: lock_file,
                path,
            });
        }
        Taking::Unlockable(format!(
            "{} is removed again and again",
            path.to_string_lossy()
        ))
    }

    /// Yields the path of the *lock file*.
    pub fn path(&self) -> &Path {
        &self.path
    }

    // realises FR-214
    /// Yields whether this is the *lock file* of the file at `file`, however its path is named.
    pub fn guards(&self, file: &Path) -> bool {
        let named = lock_path_of(file);
        named == self.path
            || matches!(
                (fs::canonicalize(&named), fs::canonicalize(&self.path)),
                (Ok(named), Ok(held)) if named == held
            )
    }
}

impl Drop for FileLock {
    // realises FR-221
    /// Removes the *lock file* where it is still the one locked; closing the file then releases
    /// the lock.
    fn drop(&mut self) {
        if is_same_file(&self.file, &self.path) {
            let _ = fs::remove_file(&self.path);
        }
    }
}

/// Yields whether the open `file` is the file at `path`, which a *holder* may have removed.
#[cfg(unix)]
fn is_same_file(file: &File, path: &Path) -> bool {
    use std::os::unix::fs::MetadataExt;
    match (file.metadata(), fs::metadata(path)) {
        (Ok(open), Ok(named)) => open.dev() == named.dev() && open.ino() == named.ino(),
        _ => false,
    }
}

/// Yields whether the open `file` is the file at `path`; taken as given where the operating
/// system offers no number of a file, as the *lock files* are specified for MacOS alone (C-008).
#[cfg(not(unix))]
fn is_same_file(_file: &File, path: &Path) -> bool {
    path.is_file()
}

// realises IR-034
/// Yields the name of the host, by the program `hostname`; empty where it cannot be run.
///
/// * Determined once per process.
pub fn host_name() -> &'static str {
    static HOST: OnceLock<String> = OnceLock::new();
    HOST.get_or_init(|| {
        Command::new("hostname")
            .output()
            .ok()
            .filter(|output| output.status.success())
            .map(|output| String::from_utf8_lossy(&output.stdout).trim().to_owned())
            .unwrap_or_default()
    })
}

// realises IR-034
/// Yields the time `seconds` after 1970-01-01T00:00:00Z as `YYYY-MM-DDTHH:MM:SSZ` of ISO 8601,
/// in UTC.
///
/// * The date is that of the proleptic Gregorian calendar, by the conversion of days to a civil
///   date of H. Hinnant.
pub fn iso_of_unix_seconds(seconds: u64) -> String {
    let days = seconds / 86_400;
    let rest = seconds % 86_400;
    let shifted = days + 719_468;
    let era = shifted / 146_097;
    let day_of_era = shifted % 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_index = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_index + 2) / 5 + 1;
    let month = if month_index < 10 {
        month_index + 3
    } else {
        month_index - 9
    };
    let year = year_of_era + era * 400 + u64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        rest / 3600,
        rest % 3600 / 60,
        rest % 60
    )
}

/// Yields whether `text` has the form `YYYY-MM-DDTHH:MM:SSZ`.
fn is_iso_time(text: &str) -> bool {
    let bytes = text.as_bytes();
    bytes.len() == 20
        && bytes.iter().enumerate().all(|(index, byte)| match index {
            4 | 7 => *byte == b'-',
            10 => *byte == b'T',
            13 | 16 => *byte == b':',
            19 => *byte == b'Z',
            _ => byte.is_ascii_digit(),
        })
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : InputGroup::apply_path, InputGroup::load_initial, the refusal properties
 *                    of InputGroup and Workbench */
#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Duration;

    fn case_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("genc3wb-lock-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("the temporary directory of the test can be created");
        dir
    }

    #[test]
    fn the_lock_file_is_named_by_the_file_and_the_suffix() {
        // FR-214, FR-215
        assert_eq!(
            (
                lock_path_of(Path::new("dir/network.gc3n")),
                lock_path_of(Path::new("genc3wb.yaml"))
            ),
            (
                PathBuf::from("dir/network.gc3n.lock"),
                PathBuf::from("genc3wb.yaml.lock")
            )
        );
    }

    #[test]
    fn unix_seconds_are_rendered_in_iso_8601() {
        // IR-034
        assert_eq!(
            [0, 951_782_400, 1_791_061_200, 4_102_444_799].map(iso_of_unix_seconds),
            [
                "1970-01-01T00:00:00Z",
                "2000-02-29T00:00:00Z",
                "2026-10-03T21:00:00Z",
                "2099-12-31T23:59:59Z"
            ]
        );
    }

    #[test]
    fn the_holder_of_this_process_is_its_number_and_the_time() {
        // IR-034
        let holder = Holder::of_this_process(UNIX_EPOCH + Duration::from_secs(86_400));
        assert_eq!(
            (holder.pid, holder.since.as_str()),
            (std::process::id(), "1970-01-02T00:00:00Z")
        );
    }

    #[test]
    fn a_holder_is_read_back_from_its_text() {
        // IR-034, IR-037
        let holder = Holder {
            pid: 4711,
            host: "Mac-mini.local".to_owned(),
            since: "2026-10-03T18:15:07Z".to_owned(),
        };
        assert_eq!(
            (
                holder.text(),
                Holder::of_text(&holder.text()),
                Holder::of_text("pid: 1\nhost: \nsince: 2026-10-03T18:15:07Z")
            ),
            (
                "pid: 4711\nhost: Mac-mini.local\nsince: 2026-10-03T18:15:07Z\n".to_owned(),
                Some(holder.clone()),
                Some(Holder {
                    pid: 1,
                    host: String::new(),
                    since: "2026-10-03T18:15:07Z".to_owned()
                })
            )
        );
    }

    #[test]
    fn a_text_of_another_form_names_no_holder() {
        // IR-037
        assert_eq!(
            [
                "",
                "pid: 1\nhost: a\n",
                "pid: x\nhost: a\nsince: 2026-10-03T18:15:07Z\n",
                "pid: 1\nhost: a\nsince: 2026-10-03 18:15:07\n",
                "pid: 1\nhost: a\nsince: 2026-10-03T18:15:07Z\nmore\n",
                "host: a\npid: 1\nsince: 2026-10-03T18:15:07Z\n",
            ]
            .map(Holder::of_text),
            [None, None, None, None, None, None]
        );
    }

    #[test]
    fn a_lock_file_is_taken_and_holds_this_process() {
        // FR-214, IR-034
        let dir = case_dir("taken");
        let file = dir.join("network.gc3n");
        let taking = FileLock::take(&file);
        let text = fs::read_to_string(lock_path_of(&file)).unwrap_or_default();
        let held = matches!(&taking, Taking::Held(lock) if lock.path() == lock_path_of(&file));
        drop(taking);
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(
            (held, Holder::of_text(&text).map(|holder| holder.pid)),
            (true, Some(std::process::id()))
        );
    }

    #[test]
    fn a_held_lock_file_is_refused_and_names_its_holder() {
        // FR-216, FR-218, IR-034, IR-037
        let dir = case_dir("refused");
        let file = dir.join("network.gc3n");
        let first = FileLock::take(&file);
        let second = FileLock::take(&file);
        let _ = fs::remove_dir_all(&dir);
        let refusal = match second {
            Taking::Refused(refusal) => Some(refusal),
            Taking::Held(_) | Taking::Unlockable(_) => None,
        };
        assert_eq!(
            (
                matches!(first, Taking::Held(_)),
                refusal.as_ref().map(|refusal| refusal.file.clone()),
                refusal
                    .and_then(|refusal| refusal.holder)
                    .map(|holder| holder.pid)
            ),
            (
                true,
                Some(file.to_string_lossy().into_owned()),
                Some(std::process::id())
            )
        );
    }

    #[test]
    fn a_lock_file_whose_text_has_another_form_is_refused_without_a_holder() {
        // FR-216, IR-034, IR-037
        let dir = case_dir("unknown");
        let file = dir.join("network.gc3n");
        let first = FileLock::take(&file);
        fs::write(lock_path_of(&file), "being written").expect("the lock file can be written");
        let second = FileLock::take(&file);
        let _ = fs::remove_dir_all(&dir);
        assert!(
            matches!(first, Taking::Held(_))
                && matches!(second, Taking::Refused(Refusal { holder: None, .. }))
        );
    }

    #[test]
    fn a_released_lock_file_is_removed_and_taken_again() {
        // FR-214, FR-221
        let dir = case_dir("released");
        let file = dir.join("network.gc3n");
        let first = FileLock::take(&file);
        drop(first);
        let removed = !lock_path_of(&file).exists();
        let again = FileLock::take(&file);
        let held_again = matches!(again, Taking::Held(_));
        drop(again);
        let _ = fs::remove_dir_all(&dir);
        assert_eq!((removed, held_again), (true, true));
    }

    #[test]
    fn a_lock_file_left_behind_unlocked_is_taken() {
        // FR-214, C-008: the lock of a holder that terminated is gone, its file may be left
        let dir = case_dir("left");
        let file = dir.join("network.gc3n");
        fs::write(
            lock_path_of(&file),
            "pid: 1\nhost: a\nsince: 2026-10-03T18:15:07Z\n",
        )
        .expect("the lock file can be written");
        let taking = FileLock::take(&file);
        let text = fs::read_to_string(lock_path_of(&file)).unwrap_or_default();
        let held = matches!(taking, Taking::Held(_));
        drop(taking);
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(
            (held, Holder::of_text(&text).map(|holder| holder.pid)),
            (true, Some(std::process::id()))
        );
    }

    #[test]
    fn a_lock_file_in_a_folder_that_does_not_exist_is_unlockable() {
        // FR-222
        let dir = case_dir("absent");
        let file = dir.join("absent").join("network.gc3n");
        let taking = FileLock::take(&file);
        let _ = fs::remove_dir_all(&dir);
        assert!(matches!(taking, Taking::Unlockable(_)));
    }

    #[test]
    fn a_lock_file_guards_its_file_however_it_is_named() {
        // FR-214
        let dir = case_dir("guards");
        fs::create_dir_all(dir.join("sub")).expect("the folder can be created");
        let file = dir.join("network.gc3n");
        let taking = FileLock::take(&file);
        let guards = match &taking {
            Taking::Held(lock) => (
                lock.guards(&file),
                lock.guards(&dir.join("sub").join("..").join("network.gc3n")),
                lock.guards(&dir.join("other.gc3n")),
            ),
            Taking::Refused(_) | Taking::Unlockable(_) => (false, false, true),
        };
        drop(taking);
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(guards, (true, true, false));
    }

    #[test]
    fn a_lock_file_removed_by_another_is_not_removed_on_release() {
        // FR-221: a lock file created anew by another genc³wb stays
        let dir = case_dir("replaced");
        let file = dir.join("network.gc3n");
        let taking = FileLock::take(&file);
        fs::remove_file(lock_path_of(&file)).expect("the lock file can be removed");
        fs::write(lock_path_of(&file), "another").expect("the lock file can be written");
        drop(taking);
        let kept = fs::read_to_string(lock_path_of(&file)).ok();
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(kept.as_deref(), Some("another"));
    }
}
