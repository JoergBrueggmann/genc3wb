//! The node runner: the *node* it serves, the transmission of a change and of a *store request*, and the *originals*.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::api_message::Diagnostic;
use crate::core::build_system::{BuildError, NodeSession};
use crate::core::text_increment::TextIncrement;

use std::path::PathBuf;

/// The text of FR-112 where a change or a *store request* is handed over while no *node* is served.
const NOT_SERVED: &str = "No node is served.";

// realises FR-104, IR-026
/// What a *node* is started with: the *build system*, the directory of the *network file*, and
/// the paths of its files as the *node description* carries them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeStart {
    /// the path of the *build system*
    pub executable: String,
    /// the directory of the *network file*, in which the *node* is started
    pub directory: PathBuf,
    /// the path of the *meta compiler DSL*
    pub meta_dsl: String,
    /// the paths of the *inputs*, in the order of the *node description*
    pub inputs: Vec<String>,
    /// the paths of the *outputs*, in the order of the *node description*
    pub outputs: Vec<String>,
}

// realises FR-115
/// The *originals*: the content of each file when the *node* was started, `None` where the file
/// did not exist.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Originals {
    /// the files and their content
    files: Vec<(PathBuf, Option<Vec<u8>>)>,
}

impl Originals {
    // realises FR-115
    /// Reads the content of every file of `paths`; a file that does not exist is recorded as
    /// absent, so that nothing is written for it.
    pub fn capture(paths: &[String]) -> Originals {
        Originals {
            files: paths
                .iter()
                .map(|path| (PathBuf::from(path), std::fs::read(path).ok()))
                .collect(),
        }
    }

    // realises FR-115
    /// Writes the recorded content back to every file that existed.
    ///
    /// * A file that cannot be written is passed over, since the *node* that produces it stores
    ///   it again.
    pub fn restore(&self) {
        for (path, content) in &self.files {
            if let Some(content) = content {
                let _ = std::fs::write(path, content);
            }
        }
    }

    /// Yields the number of files recorded.
    pub fn len(&self) -> usize {
        self.files.len()
    }

    /// Yields whether no file is recorded.
    pub fn is_empty(&self) -> bool {
        self.files.is_empty()
    }
}

// realises FR-104, FR-105, FR-106, FR-109, FR-112
/// The node runner: the *node* it serves, and the directory of its socket files.
///
/// * The runner is owned by one thread; every function of it blocks until the *node* answered.
#[derive(Debug)]
pub struct NodeRunner {
    /// the directory of the socket files
    directory: PathBuf,
    /// the served *node*, `None` where none is served
    session: Option<NodeSession>,
    /// the number of *nodes* started, which names the socket file
    sessions_started: u32,
}

impl NodeRunner {
    /// Creates the runner that writes its socket files into `directory`.
    pub fn new(directory: PathBuf) -> NodeRunner {
        NodeRunner {
            directory,
            session: None,
            sessions_started: 0,
        }
    }

    // realises FR-104, FR-105
    /// Shuts the served *node* down and starts the one of `start` on a socket file of its own.
    ///
    /// # Errors
    /// Returns what [`NodeSession::start`] returns; no *node* is served then.
    pub fn start(&mut self, start: &NodeStart) -> Result<(), BuildError> {
        self.stop();
        std::fs::create_dir_all(&self.directory)
            .map_err(|error| BuildError::Process(error.to_string()))?;
        self.sessions_started += 1;
        let socket_path = self
            .directory
            .join(format!("n{}.sock", self.sessions_started));
        self.session = Some(NodeSession::start(
            &start.executable,
            &start.directory,
            &start.meta_dsl,
            &start.inputs,
            &start.outputs,
            &socket_path,
        )?);
        Ok(())
    }

    // realises FR-106, FR-107, FR-112
    /// Transmits the change of a document to the served *node*, as [`NodeSession::transmit`].
    ///
    /// * Where the connection fails, the *node* is given up, so that every later change is
    ///   reported as not served until a *node* is opened again.
    ///
    /// # Errors
    /// Returns the text of FR-112: that no *node* is served, or the rendering of the
    /// [`BuildError`].
    pub fn transmit(
        &mut self,
        document: &str,
        provided_before: &str,
        increment: &TextIncrement,
        provided_after: &str,
    ) -> Result<Vec<Diagnostic>, String> {
        let Some(session) = self.session.as_mut() else {
            return Err(NOT_SERVED.to_owned());
        };
        let outcome = session.transmit(document, provided_before, increment, provided_after);
        self.give_up_on(&outcome);
        outcome.map_err(|error| error.to_string())
    }

    // realises FR-109, FR-111, FR-112
    /// Transmits a *store request* to the served *node*, as [`NodeSession::store`].
    ///
    /// # Errors
    /// Returns the text of FR-112, as [`NodeRunner::transmit`] does.
    pub fn store(&mut self) -> Result<(), String> {
        let Some(session) = self.session.as_mut() else {
            return Err(NOT_SERVED.to_owned());
        };
        let outcome = session.store();
        self.give_up_on(&outcome);
        outcome.map_err(|error| error.to_string())
    }

    // realises FR-105
    /// Shuts the served *node* down, where one is served.
    pub fn stop(&mut self) {
        if let Some(session) = self.session.take() {
            session.shut_down();
        }
    }

    /// Gives the *node* up where `outcome` says that it is not reached any more.
    fn give_up_on<T>(&mut self, outcome: &Result<T, BuildError>) {
        if let Err(BuildError::Connection(_) | BuildError::Process(_) | BuildError::Protocol(_)) =
            outcome
        {
            self.session = None;
        }
    }
}

impl Drop for NodeRunner {
    fn drop(&mut self) {
        self.stop();
        let _ = std::fs::remove_dir(&self.directory);
    }
}

/*  * validated        : ✅
 * completeness     : ❌ a served node needs a build system; it is covered by the ignored
 *                    integration test 'tests/build_system.rs'
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : NodeEditor::open_node, NodeEditor::apply_increment, NodeEditor::report_arrived,
 *                    NodeEditor::close, NodeEditor::shut_down */
#[cfg(test)]
mod tests {
    use super::*;

    use std::fs;

    fn case_dir(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("genc3wb-node_runner-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("the temporary directory of the test can be created");
        dir
    }

    fn increment() -> TextIncrement {
        TextIncrement {
            position: 0,
            range: 0,
            text: "a".to_owned(),
        }
    }

    #[test]
    fn change_without_a_served_node_is_reported_as_not_served() {
        // FR-112
        let mut runner = NodeRunner::new(case_dir("unserved"));
        let transmitted = runner.transmit("a.gc3", "", &increment(), "a");
        let stored = runner.store();
        assert_eq!(
            (transmitted, stored),
            (Err(NOT_SERVED.to_owned()), Err(NOT_SERVED.to_owned()))
        );
    }

    #[test]
    fn node_of_a_build_system_that_is_not_executable_is_not_served() {
        // FR-104, FR-112
        let mut runner = NodeRunner::new(case_dir("notexecutable"));
        let started = runner.start(&NodeStart {
            executable: "/nonexistent/genc3d".to_owned(),
            directory: PathBuf::from("/tmp"),
            meta_dsl: "a.gc3".to_owned(),
            inputs: vec![],
            outputs: vec![],
        });
        let message = started
            .err()
            .map(|error| error.to_string())
            .unwrap_or_default();
        assert!(
            (message.contains("/nonexistent/genc3d") || message.contains("not available"))
                && runner.session.is_none()
        );
    }

    #[test]
    fn stop_without_a_served_node_does_nothing() {
        // FR-105
        let mut runner = NodeRunner::new(case_dir("stop"));
        runner.stop();
        assert!(runner.session.is_none());
    }

    #[test]
    fn originals_are_written_back_and_an_absent_file_is_left_absent() {
        // FR-115
        let dir = case_dir("originals");
        let existing = dir.join("in.txt");
        let absent = dir.join("absent.txt");
        fs::write(&existing, "original").expect("the file can be written");
        let originals = Originals::capture(&[
            existing.to_string_lossy().into_owned(),
            absent.to_string_lossy().into_owned(),
        ]);
        fs::write(&existing, "temporary").expect("the file can be written");
        originals.restore();
        let restored = fs::read_to_string(&existing).expect("the file was written back");
        let still_absent = !absent.exists();
        let _ = fs::remove_dir_all(&dir);
        assert_eq!(
            (originals.len(), restored, still_absent),
            (2, "original".to_owned(), true)
        );
    }

    #[test]
    fn originals_of_no_file_are_empty() {
        // FR-115
        let originals = Originals::capture(&[]);
        assert!(originals.is_empty() && originals == Originals::default());
    }
}
