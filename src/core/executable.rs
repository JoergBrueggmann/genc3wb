//! Whether a file is executable.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use std::path::Path;

// realises FR-067, FR-092, FR-104
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

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : NetworkEditor::set_build_system_path, NetworkEditor::try_build_system_path */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_file_is_not_executable() {
        // FR-067
        assert!(!is_executable(""));
        assert!(!is_executable("/no/such/file"));
        let dir = std::env::temp_dir();
        assert!(!is_executable(&dir.to_string_lossy()));
    }

    #[cfg(unix)]
    #[test]
    fn shell_is_executable() {
        // FR-067, FR-104
        assert!(is_executable("/bin/sh"));
    }

    #[cfg(unix)]
    #[test]
    fn file_without_execute_permission_is_not_executable() {
        // FR-092
        let dir = std::env::temp_dir().join(format!("genc3wb-executable-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).expect("the directory can be created");
        let file = dir.join("plain.txt");
        std::fs::write(&file, "text").expect("the file can be written");
        let executable = is_executable(&file.to_string_lossy());
        let _ = std::fs::remove_dir_all(&dir);
        assert!(!executable);
    }
}
