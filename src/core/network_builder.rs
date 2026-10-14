//! The builder of the *compiler network editor*: a *build* and the laying out of its *network graph*.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::api_message::{Diagnostic, NodeDescription};
use crate::core::build_system::{BuildError, BuildSession, StateSink};
use crate::core::network_graph::{
    GraphFile, GraphLayout, dot_of_network, files_of_network, render,
};
use crate::core::text_increment::TextIncrement;

use std::path::PathBuf;

/// The text of FR-112 where a *store request* is handed over while no *build system* runs.
const NOT_RUNNING: &str = "No build system runs.";

// realises FR-075, FR-077, FR-078, FR-273, FR-274, FR-275, FR-289 to FR-291
/// What a successful *build* yields for the *front end*: the two images of the *network graph*,
/// the one of a network that serves *nodes* and the one of an *invalid network*, their layout,
/// and the *nodes* and the files they show.
#[derive(Debug, Clone, PartialEq)]
pub struct BuiltGraph {
    /// the file holding the PNG image of a *valid network* or an *active network*
    pub served_image_path: PathBuf,
    /// the file holding the PNG image of an *invalid network*
    pub invalid_image_path: PathBuf,
    /// the layout, the same for both images, by which a point of an image is related to a vertex
    pub layout: GraphLayout,
    /// the *node descriptions*, in the order of the vertices `n<index>`
    pub nodes: Vec<NodeDescription>,
    /// the files of the network, in the order of the vertices `f<index>`
    pub files: Vec<GraphFile>,
}

// realises FR-069, FR-073, FR-074, FR-128
/// What a *build* yields for the *front end*: the *diagnostics* of the change of the
/// *network file*, and the laid out graph or the *error message*.
#[derive(Debug, Clone, PartialEq)]
pub struct BuildOutcome {
    /// the *diagnostics* of the open or edit request; none where it was not answered
    pub diagnostics: Vec<Diagnostic>,
    /// the graph of a successful *build*, or the *error message* of a failed one
    pub graph: Result<BuiltGraph, String>,
}

// realises FR-229
/// What yields the sink of the watch of each *build system* the builder starts: the receiver of
/// its *state responses*.
pub type StateSinks = Box<dyn FnMut() -> StateSink + Send>;

// realises FR-067, FR-068, FR-069, FR-074, FR-109, FR-228, FR-244, FR-260, FR-267
/// The builder: the *build system* it runs, and the directory it lays the *network graphs* out in.
///
/// * The builder is owned by one thread; every function of it blocks until the *build system* or
///   `dot` answered.
/// * The *state responses* of the *build system* do not pass through the builder: the sink it
///   hands to each *build system* it starts receives them on the reader thread of the
///   connection.
pub struct NetworkBuilder {
    /// the directory of the socket files and of the images
    directory: PathBuf,
    /// what yields the sink of the watch of each *build system* started
    sinks: StateSinks,
    /// the path of the *build system*, empty where none is named
    executable: String,
    /// the path of the *network file*, empty where none is named
    network_path: String,
    /// the running *build system*, `None` where none runs
    session: Option<BuildSession>,
    /// the number of *build systems* started, which names the socket file
    sessions_started: u32,
    /// the number of images written, which names the image file
    images_written: u32,
    /// the image of the last successful *build*
    /// the images of the last successful *build*, removed once the next ones are written
    last_images: Vec<PathBuf>,
}

impl NetworkBuilder {
    /// Creates the builder that writes its socket files and images into `directory`, and hands
    /// a sink of `sinks` to each *build system* it starts.
    pub fn new(directory: PathBuf, sinks: StateSinks) -> NetworkBuilder {
        NetworkBuilder {
            directory,
            sinks,
            executable: String::new(),
            network_path: String::new(),
            session: None,
            sessions_started: 0,
            images_written: 0,
            last_images: Vec::new(),
        }
    }

    /// Yields the directory a process of *product* writes its socket files and images into:
    /// 'genc3wb-' followed by the process identifier, in the temporary directory of the system.
    pub fn default_directory() -> PathBuf {
        std::env::temp_dir().join(format!("genc3wb-{}", std::process::id()))
    }

    // realises FR-067, FR-068, FR-244
    /// Names the *build system* and the *network file*: shuts the running *build system* down and
    /// starts the named one, where both are named.
    ///
    /// # Errors
    /// Returns what [`BuildSession::start`] returns.
    pub fn restart(&mut self, executable: &str, network_path: &str) -> Result<(), BuildError> {
        self.stop();
        self.executable = executable.to_owned();
        self.network_path = network_path.to_owned();
        if self.executable.is_empty() || self.network_path.is_empty() {
            return Ok(());
        }
        self.start()
    }

    // realises FR-069, FR-073, FR-074, FR-075, FR-077, FR-128, FR-273, FR-274, FR-289
    /// Carries out a *build* and lays out its *network graph*, twice: in the colours of a network
    /// that serves *nodes* and in those of an *invalid network*, the layout being the same.
    ///
    /// * Where no *build system* runs although one is named, it is started first.
    /// * Where the connection fails, the *build system* is given up, so that the next *build*
    ///   starts it again.
    /// * The images of the *build* before are removed once the new ones are written.
    /// * The graph is the *error message* where the *build* failed: that of the [`BuildError`],
    ///   or that of the [`crate::core::network_graph::GraphError`] where `dot` fails.
    pub fn build(
        &mut self,
        provided_before: &str,
        increment: &TextIncrement,
        provided_after: &str,
    ) -> BuildOutcome {
        let (diagnostics, graph) = self.built(provided_before, increment, provided_after);
        BuildOutcome { diagnostics, graph }
    }

    /// Carries out the *build* of [`NetworkBuilder::build`]; the *diagnostics* are those of the
    /// change where it was answered, none otherwise.
    fn built(
        &mut self,
        provided_before: &str,
        increment: &TextIncrement,
        provided_after: &str,
    ) -> (Vec<Diagnostic>, Result<BuiltGraph, String>) {
        let restarted = self.session.is_none();
        if restarted {
            if self.executable.is_empty() {
                return (vec![], Err("No build system is named.".to_owned()));
            }
            if self.network_path.is_empty() {
                return (vec![], Err("No compiler network file is named.".to_owned()));
            }
            if let Err(error) = self.start() {
                return (vec![], Err(error.to_string()));
            }
        }
        let session = self
            .session
            .as_mut()
            .expect("a build system runs after a successful start");
        let result = match session.build(provided_before, increment, provided_after) {
            Ok(result) => result,
            Err(error) => {
                self.give_up_on(&error);
                return (vec![], Err(error.to_string()));
            }
        };
        let description = match result.network {
            Ok(description) => description,
            Err(error) => {
                self.give_up_on(&error);
                return (result.diagnostics, Err(error.to_string()));
            }
        };
        self.images_written += 1;
        let stem = format!("graph-{}", self.images_written);
        let rendered = |served: bool, suffix: &str| {
            render(
                &dot_of_network(&description.nodes, served),
                &self.directory,
                &format!("{stem}-{suffix}"),
            )
        };
        let (served, invalid) = match (rendered(true, "served"), rendered(false, "invalid")) {
            (Ok(served), Ok(invalid)) => (served, invalid),
            (Err(error), _) | (_, Err(error)) => {
                return (result.diagnostics, Err(error.to_string()));
            }
        };
        for previous in std::mem::replace(
            &mut self.last_images,
            vec![served.image_path.clone(), invalid.image_path.clone()],
        ) {
            let _ = std::fs::remove_file(previous);
        }
        (
            result.diagnostics,
            Ok(BuiltGraph {
                served_image_path: served.image_path,
                invalid_image_path: invalid.image_path,
                layout: served.layout,
                files: files_of_network(&description.nodes),
                nodes: description.nodes,
            }),
        )
    }

    /// Gives the *build system* up where `error` says that it is not reached any more.
    fn give_up_on(&mut self, error: &BuildError) {
        match error {
            BuildError::Connection(_) | BuildError::Process(_) | BuildError::Protocol(_) => {
                self.session = None;
            }
            BuildError::Unsupported | BuildError::NotExecutable(_) | BuildError::Refused(_) => {}
        }
    }

    // realises FR-109, FR-112, FR-211
    /// Transmits a *store request* to the running *build system*, which passes it on to every
    /// *node*, as [`BuildSession::store`].
    ///
    /// * Where the connection fails, the *build system* is given up, as a *build* gives it up.
    ///
    /// # Errors
    /// Returns the text of FR-112: that no *build system* runs, or the rendering of the
    /// [`BuildError`].
    pub fn store(&mut self) -> Result<(), String> {
        let Some(session) = self.session.as_mut() else {
            return Err(NOT_RUNNING.to_owned());
        };
        let outcome = session.store();
        if let Err(error) = &outcome {
            self.give_up_on(error);
        }
        outcome.map_err(|error| error.to_string())
    }

    // realises FR-260, FR-261
    /// Transmits an *activate request* to the running *build system*, which starts, wires and
    /// activates every *node* of the network, as [`BuildSession::activate`].
    ///
    /// * Where the connection fails, the *build system* is given up, as a *build* gives it up.
    ///
    /// # Errors
    /// Returns the *error message* of FR-261: that no *build system* runs, or the rendering of
    /// the [`BuildError`], which is the message text of the error response where the
    /// *network file* or a *node* is invalid.
    pub fn activate(&mut self) -> Result<(), String> {
        let Some(session) = self.session.as_mut() else {
            return Err(NOT_RUNNING.to_owned());
        };
        let outcome = session.activate();
        if let Err(error) = &outcome {
            self.give_up_on(error);
        }
        outcome.map_err(|error| error.to_string())
    }

    // realises FR-267, FR-268
    /// Transmits a *deactivate request* to the running *build system*, which carries out the
    /// *network deactivation*, as [`BuildSession::deactivate`].
    ///
    /// * Where the connection fails, the *build system* is given up, as a *build* gives it up.
    ///
    /// # Errors
    /// Returns the *error message* of FR-268: that no *build system* runs, or the rendering of
    /// the [`BuildError`].
    pub fn deactivate(&mut self) -> Result<(), String> {
        let Some(session) = self.session.as_mut() else {
            return Err(NOT_RUNNING.to_owned());
        };
        let outcome = session.deactivate();
        if let Err(error) = &outcome {
            self.give_up_on(error);
        }
        outcome.map_err(|error| error.to_string())
    }

    // realises FR-281, FR-286
    /// Transmits a *hold request* naming `node` to the running *build system*, which starts and
    /// serves the *node* in a *valid network*, as [`BuildSession::hold`].
    ///
    /// * Where the connection fails, the *build system* is given up, as a *build* gives it up.
    ///
    /// # Errors
    /// Returns the *error message* of FR-286: that no *build system* runs, or the rendering of
    /// the [`BuildError`], which is the message text of the error response in an
    /// *invalid network*.
    pub fn hold(&mut self, node: &str) -> Result<(), String> {
        let Some(session) = self.session.as_mut() else {
            return Err(NOT_RUNNING.to_owned());
        };
        let outcome = session.hold(node);
        if let Err(error) = &outcome {
            self.give_up_on(error);
        }
        outcome.map_err(|error| error.to_string())
    }

    // realises FR-282, FR-287
    /// Transmits a *release request* naming `node` to the running *build system*, which ends
    /// the *node* in a *valid network* where no other client holds it, as
    /// [`BuildSession::release`].
    ///
    /// * Where the connection fails, the *build system* is given up, as a *build* gives it up.
    ///
    /// # Errors
    /// Returns the *error message* of FR-287: that no *build system* runs, or the rendering of
    /// the [`BuildError`].
    pub fn release(&mut self, node: &str) -> Result<(), String> {
        let Some(session) = self.session.as_mut() else {
            return Err(NOT_RUNNING.to_owned());
        };
        let outcome = session.release(node);
        if let Err(error) = &outcome {
            self.give_up_on(error);
        }
        outcome.map_err(|error| error.to_string())
    }

    // realises FR-068
    /// Shuts the running *build system* down, where one runs.
    pub fn stop(&mut self) {
        if let Some(session) = self.session.take() {
            session.shut_down();
        }
    }

    // realises FR-067, FR-228
    /// Starts the named *build system* on the named *network file*, on a socket file of its own,
    /// with a sink for its watch.
    fn start(&mut self) -> Result<(), BuildError> {
        std::fs::create_dir_all(&self.directory)
            .map_err(|error| BuildError::Process(error.to_string()))?;
        self.sessions_started += 1;
        let socket_path = self
            .directory
            .join(format!("b{}.sock", self.sessions_started));
        self.session = Some(BuildSession::start(
            &self.executable,
            &self.network_path,
            &socket_path,
            (self.sinks)(),
        )?);
        Ok(())
    }
}

impl Drop for NetworkBuilder {
    fn drop(&mut self) {
        self.stop();
        for image in self.last_images.drain(..) {
            let _ = std::fs::remove_file(image);
        }
        let _ = std::fs::remove_dir(&self.directory);
    }
}

/*  * validated        : ✅
* completeness     : ❌ a successful build needs a build system; it is covered by the ignored
                       integration test 'tests/build_system.rs'
* independence     : ✅
* edge cases       : ✅
* conforms to doc  : ✅
* covers bridge    : NetworkEditor::set_network_path, NetworkEditor::set_build_system_path,
                     NetworkEditor::apply_increment, NetworkEditor::store,
                     NetworkEditor::activate, NetworkEditor::deactivate,
                     NetworkEditor::open_node_at, NetworkEditor::open_node_named,
                     NetworkEditor::close_node, NetworkEditor::shut_down */
#[cfg(test)]
mod tests {
    use super::*;

    fn increment() -> TextIncrement {
        TextIncrement {
            position: 0,
            range: 0,
            text: "a".to_owned(),
        }
    }

    fn builder(case: &str) -> NetworkBuilder {
        NetworkBuilder::new(
            std::env::temp_dir().join(format!("genc3wb-test-{}-{case}", std::process::id())),
            Box::new(|| Box::new(|_| {})),
        )
    }

    #[test]
    fn store_without_a_running_build_system_is_reported_as_not_running() {
        // FR-112
        assert_eq!(builder("nostore").store(), Err(NOT_RUNNING.to_owned()));
    }

    #[test]
    fn activate_without_a_running_build_system_is_reported_as_not_running() {
        // FR-261
        assert_eq!(
            builder("noactivate").activate(),
            Err(NOT_RUNNING.to_owned())
        );
    }

    #[test]
    fn deactivate_without_a_running_build_system_is_reported_as_not_running() {
        // FR-268
        assert_eq!(
            builder("nodeactivate").deactivate(),
            Err(NOT_RUNNING.to_owned())
        );
    }

    #[test]
    fn hold_and_release_without_a_running_build_system_are_reported_as_not_running() {
        // FR-286, FR-287
        let mut builder = builder("nohold");
        assert_eq!(
            (builder.hold("a"), builder.release("a")),
            (Err(NOT_RUNNING.to_owned()), Err(NOT_RUNNING.to_owned()))
        );
    }

    #[test]
    fn build_without_a_named_build_system_fails_with_its_message() {
        assert_eq!(
            builder("unnamed").build("", &increment(), "a").graph,
            Err("No build system is named.".to_owned())
        );
    }

    #[test]
    fn build_without_a_named_network_file_fails_with_its_message() {
        let mut builder = builder("nofile");
        let _ = builder.restart("/bin/sh", "");
        assert_eq!(
            builder.build("", &increment(), "a").graph,
            Err("No compiler network file is named.".to_owned())
        );
    }

    #[test]
    fn restart_without_both_paths_starts_nothing() {
        let mut builder = builder("nothing");
        assert_eq!(
            (
                builder.restart("", "/tmp/n.gc3n"),
                builder.session.is_none()
            ),
            (Ok(()), true)
        );
    }

    #[test]
    fn build_system_that_is_not_executable_is_the_error_message_of_the_build() {
        let mut builder = builder("notexecutable");
        let _ = builder.restart("/nonexistent/genc3d", "/tmp/n.gc3n");
        let message = builder
            .build("", &increment(), "a")
            .graph
            .err()
            .unwrap_or_default();
        assert!(message.contains("/nonexistent/genc3d") || message.contains("not available"));
    }
}
