//! The *build system*: its process, the conversation with it on its *service socket*, and the *build*.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::api_message::{
    MessageError, NodeDescription, Request, Response, decode_response, delta_of_increment,
    encode_request, frames_of_message, read_message,
};
use crate::core::runner;
use crate::core::text_increment::TextIncrement;

use std::fmt;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

/// How often *product* tries to connect to the *service socket* of a *build system* it started.
const CONNECT_ATTEMPTS: u32 = 100;
/// The pause between two such attempts.
const CONNECT_PAUSE: Duration = Duration::from_millis(50);
/// The time after which a *response* that did not arrive is a failure.
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(10);
/// The time *product* waits for a *build system* to end after its *shutdown request*.
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(2);

/// The byte stream of the *service socket*: a Unix domain stream socket (C-005).
#[cfg(unix)]
type ServiceStream = std::os::unix::net::UnixStream;
/// The byte stream where no Unix domain socket exists; never created, since `start` fails before.
#[cfg(not(unix))]
type ServiceStream = std::io::Empty;

// realises FR-073
/// What a successful *build* yields: the *network response* of the *build system*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NetworkDescription {
    /// the *document version* of the *network file* the *nodes* were derived from
    pub version: u64,
    /// the *node descriptions*, in the order of the *network file*
    pub nodes: Vec<NodeDescription>,
}

// realises FR-073, FR-074, C-005
/// Why a *build* failed, or why the *build system* was not reached.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildError {
    /// the platform offers no Unix domain socket (C-005)
    Unsupported,
    /// the named *build system* is not executable; carries the path
    NotExecutable(String),
    /// the process could not be started, or ended before it was reached; carries the reason
    Process(String),
    /// the *service socket* was not reached, or the connection failed; carries the reason
    Connection(String),
    /// a *message* could not be decoded, or a *response* of an unexpected kind arrived
    Protocol(String),
    /// the *build system* answered with an error response; carries its message text
    Refused(String),
}

impl fmt::Display for BuildError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BuildError::Unsupported => write!(
                formatter,
                "the build system is not available on this platform: it is reached through a \
                 Unix domain socket"
            ),
            BuildError::NotExecutable(path) => {
                write!(formatter, "the build system '{path}' is not executable")
            }
            BuildError::Process(reason) => {
                write!(formatter, "the build system could not be run: {reason}")
            }
            BuildError::Connection(reason) => {
                write!(formatter, "the build system was not reached: {reason}")
            }
            BuildError::Protocol(reason) => {
                write!(formatter, "the build system was not understood: {reason}")
            }
            BuildError::Refused(text) => write!(formatter, "{text}"),
        }
    }
}

impl std::error::Error for BuildError {}

impl From<MessageError> for BuildError {
    fn from(error: MessageError) -> BuildError {
        match error {
            MessageError::Io(reason) => BuildError::Connection(reason),
            MessageError::Malformed(_)
            | MessageError::UnknownKind(_)
            | MessageError::MissingKey(_)
            | MessageError::InvalidValue(_) => BuildError::Protocol(error.to_string()),
        }
    }
}

// realises FR-070, FR-072, FR-073, IR-018, IR-019, IR-020
/// The conversation with a *build system* about one *network file*, on a byte stream.
///
/// * The stream is a generic bound, so that the conversation is tested on a stream in memory.
#[derive(Debug)]
pub struct Conversation<S: Read + Write> {
    /// the byte stream of the *service socket*
    stream: S,
    /// the *document identifier* of the *network file*
    document: String,
    /// whether the *build system* accepted an open request of this conversation
    opened: bool,
    /// the *document version* the next edit request applies to
    version: u64,
    /// the *request identifier* of the next *request*
    next_request: u64,
}

impl<S: Read + Write> Conversation<S> {
    /// Creates the conversation about the document `document` on `stream`; nothing is transmitted.
    pub fn new(stream: S, document: &str) -> Conversation<S> {
        Conversation {
            stream,
            document: document.to_owned(),
            opened: false,
            version: 0,
            next_request: 1,
        }
    }

    // realises FR-069, FR-070, FR-072, FR-073, FR-074
    /// Carries out a *build*: transmits the change of the text, then queries the network.
    ///
    /// * Where no open request was accepted yet, the change is an open request carrying
    ///   `provided_after`; otherwise it is an edit request with the *edit delta* of `increment`.
    /// * A version mismatch response is answered by an open request carrying `provided_after`.
    /// * After an error response to the change, the next *build* opens the document again.
    ///
    /// # Arguments
    /// * `provided_before` - the *provided text* `increment` applies to
    /// * `increment` - the *text increment* the code editor provided
    /// * `provided_after` - the *provided text* after `increment`
    ///
    /// # Errors
    /// Returns [`BuildError::Refused`] where the *build system* answers with an error response,
    /// [`BuildError::Connection`] where the stream fails, and [`BuildError::Protocol`] where a
    /// *message* is not decoded or has an unexpected kind.
    pub fn build(
        &mut self,
        provided_before: &str,
        increment: &TextIncrement,
        provided_after: &str,
    ) -> Result<NetworkDescription, BuildError> {
        if self.opened {
            let edit = Request::Edit {
                document: self.document.clone(),
                version: self.version,
                deltas: vec![delta_of_increment(provided_before, increment)],
            };
            match self.exchange(&edit)? {
                Response::Diagnostics { version, .. } => self.version = version,
                Response::VersionMismatch { .. } => self.open(provided_after)?,
                Response::Error(text) => {
                    self.opened = false;
                    return Err(BuildError::Refused(text));
                }
                Response::Acknowledged | Response::Network { .. } | Response::Other(_) => {
                    return Err(unexpected("the edit request"));
                }
            }
        } else {
            self.open(provided_after)?;
        }
        match self.exchange(&Request::QueryNetwork)? {
            Response::Network { version, nodes } => Ok(NetworkDescription { version, nodes }),
            Response::Error(text) => Err(BuildError::Refused(text)),
            Response::Acknowledged
            | Response::Diagnostics { .. }
            | Response::VersionMismatch { .. }
            | Response::Other(_) => Err(unexpected("the network query request")),
        }
    }

    // realises FR-068
    /// Transmits a *shutdown request* and awaits its *terminal response*.
    ///
    /// # Errors
    /// Returns [`BuildError::Connection`] where the stream fails.
    pub fn shut_down(&mut self) -> Result<(), BuildError> {
        self.exchange(&Request::Shutdown).map(|_| ())
    }

    /// Transmits an open request carrying `text`; the *document version* becomes the one answered.
    fn open(&mut self, text: &str) -> Result<(), BuildError> {
        let open = Request::Open {
            document: self.document.clone(),
            text: text.to_owned(),
        };
        match self.exchange(&open)? {
            Response::Acknowledged => self.version = 0,
            Response::Diagnostics { version, .. } => self.version = version,
            Response::Error(text) => return Err(BuildError::Refused(text)),
            Response::VersionMismatch { .. } | Response::Network { .. } | Response::Other(_) => {
                return Err(unexpected("the open request"));
            }
        }
        self.opened = true;
        Ok(())
    }

    /// Transmits `request` and yields the *response* that carries its *request identifier*.
    fn exchange(&mut self, request: &Request) -> Result<Response, BuildError> {
        let identifier = self.next_request;
        self.next_request += 1;
        let frames = frames_of_message(&encode_request(identifier, request));
        self.stream
            .write_all(&frames)
            .and_then(|()| self.stream.flush())
            .map_err(|error| BuildError::Connection(error.to_string()))?;
        loop {
            let (answered, response) = decode_response(&read_message(&mut self.stream)?)?;
            if answered == identifier {
                return Ok(response);
            }
        }
    }
}

// realises FR-067, FR-068, IR-017, C-005
/// A *build system* *product* started, with the conversation on its *service socket*.
///
/// * Dropping the session ends the process where it still runs, and removes the socket file.
#[derive(Debug)]
pub struct BuildSession {
    /// the process of the *build system*
    child: Child,
    /// the conversation on its *service socket*
    conversation: Conversation<ServiceStream>,
    /// the path of its *service socket*
    socket_path: PathBuf,
}

impl BuildSession {
    // realises C-005
    /// Yields whether the platform offers the Unix domain socket the *builds* need.
    pub fn is_available() -> bool {
        cfg!(unix)
    }

    // realises FR-067, IR-017
    /// Starts the *build system* `executable` on `network_path` in the *description mode*, in the
    /// directory of the *network file*, and connects to `socket_path` once it listens.
    ///
    /// * A relative `executable` is resolved against the working directory of *product* before
    ///   the process is started, since the process is started in the directory of the *network
    ///   file* and the operating system would otherwise look for it there.
    ///
    /// # Errors
    /// Returns [`BuildError::Unsupported`] where the platform offers no Unix domain socket,
    /// [`BuildError::NotExecutable`] where `executable` is not executable,
    /// [`BuildError::Process`] where the process cannot be started or ends before it listens, and
    /// [`BuildError::Connection`] where its *service socket* is not reached.
    pub fn start(
        executable: &str,
        network_path: &str,
        socket_path: &Path,
    ) -> Result<BuildSession, BuildError> {
        if !BuildSession::is_available() {
            return Err(BuildError::Unsupported);
        }
        if !runner::is_executable(executable) {
            return Err(BuildError::NotExecutable(executable.to_owned()));
        }
        let network = Path::new(network_path);
        let document = network
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .ok_or_else(|| BuildError::Process("no network file is named".to_owned()))?;
        let directory = match network.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
            _ => PathBuf::from("."),
        };
        let _ = std::fs::remove_file(socket_path);
        let mut child = Command::new(absolute_program(executable))
            .current_dir(directory)
            .arg("--network")
            .arg(&document)
            .arg("--no-nodes")
            .arg("--socket")
            .arg(socket_path)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| BuildError::Process(error.to_string()))?;
        match connected(&mut child, socket_path) {
            Ok(stream) => Ok(BuildSession {
                child,
                conversation: Conversation::new(stream, &document),
                socket_path: socket_path.to_path_buf(),
            }),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = std::fs::remove_file(socket_path);
                Err(error)
            }
        }
    }

    // realises FR-069
    /// Carries out a *build*, as [`Conversation::build`].
    ///
    /// # Errors
    /// Returns what [`Conversation::build`] returns.
    pub fn build(
        &mut self,
        provided_before: &str,
        increment: &TextIncrement,
        provided_after: &str,
    ) -> Result<NetworkDescription, BuildError> {
        self.conversation
            .build(provided_before, increment, provided_after)
    }

    // realises FR-068
    /// Transmits a *shutdown request* and waits for the process to end; ends it where it does not.
    pub fn shut_down(mut self) {
        let _ = self.conversation.shut_down();
        let pause = Duration::from_millis(20);
        let mut waited = Duration::ZERO;
        while waited < SHUTDOWN_TIMEOUT && matches!(self.child.try_wait(), Ok(None)) {
            std::thread::sleep(pause);
            waited += pause;
        }
    }
}

impl Drop for BuildSession {
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

// realises FR-067, FR-094, IR-017
/// Yields the path by which the process of the *build system* is started: `executable` made
/// absolute against the working directory of *product*.
///
/// * The *build system* is started in the directory of the *network file*, so a relative path
///   would otherwise be resolved against that directory instead of against the one in which the
///   user named it, which is the working directory of *product* ([`crate::core::runner::is_executable`]).
/// * Where the path cannot be made absolute, it is yielded as it stands, so that the failure is
///   the one of the process and not one of this function.
fn absolute_program(executable: &str) -> PathBuf {
    let path = Path::new(executable);
    if path.is_absolute() {
        return path.to_path_buf();
    }
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// Yields the error of a *response* whose kind does not answer `request`.
fn unexpected(request: &str) -> BuildError {
    BuildError::Protocol(format!("{request} was answered by an unexpected response"))
}

/// Connects to the *service socket* of `child`, retrying until it listens or the process ends.
#[cfg(unix)]
fn connected(child: &mut Child, socket_path: &Path) -> Result<ServiceStream, BuildError> {
    let mut reason = String::from("the service socket does not exist");
    for _ in 0..CONNECT_ATTEMPTS {
        match ServiceStream::connect(socket_path) {
            Ok(stream) => {
                stream
                    .set_read_timeout(Some(RESPONSE_TIMEOUT))
                    .and_then(|()| stream.set_write_timeout(Some(RESPONSE_TIMEOUT)))
                    .map_err(|error| BuildError::Connection(error.to_string()))?;
                return Ok(stream);
            }
            Err(error) => reason = error.to_string(),
        }
        if let Ok(Some(status)) = child.try_wait() {
            return Err(BuildError::Process(format!(
                "the build system ended with {status} before it listened"
            )));
        }
        std::thread::sleep(CONNECT_PAUSE);
    }
    Err(BuildError::Connection(reason))
}

/// Yields that the platform offers no Unix domain socket; `start` fails before it calls this.
#[cfg(not(unix))]
fn connected(_child: &mut Child, _socket_path: &Path) -> Result<ServiceStream, BuildError> {
    let _ = (CONNECT_ATTEMPTS, CONNECT_PAUSE, RESPONSE_TIMEOUT);
    Err(BuildError::Unsupported)
}

/*  * validated        : ✅
* completeness     : ✅
* independence     : ✅
* edge cases       : ✅
* conforms to doc  : ✅
* covers bridge    : NetworkEditor::apply_increment, NetworkEditor::set_network_path,
                     NetworkEditor::shut_down; the process and the socket of `BuildSession`
                     are covered by the ignored integration test 'tests/build_system.rs' */
#[cfg(test)]
mod tests {
    use super::*;

    use crate::core::api_message::{NodeKind, ReadPosition, encode_response};

    use std::io::Cursor;

    /// A stream in memory: the *responses* scripted for it, and what was written to it.
    struct Scripted {
        responses: Cursor<Vec<u8>>,
        written: Vec<u8>,
    }

    impl Scripted {
        fn answering(responses: &[(u64, Response)]) -> Scripted {
            let bytes = responses
                .iter()
                .flat_map(|(identifier, response)| {
                    frames_of_message(&encode_response(*identifier, response))
                })
                .collect();
            Scripted {
                responses: Cursor::new(bytes),
                written: Vec::new(),
            }
        }

        fn requests(&self) -> Vec<Vec<u8>> {
            let mut stream = Cursor::new(self.written.clone());
            let mut requests = Vec::new();
            while let Ok(message) = read_message(&mut stream) {
                requests.push(message);
            }
            requests
        }
    }

    impl Read for Scripted {
        fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
            self.responses.read(buffer)
        }
    }

    impl Write for Scripted {
        fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
            self.written.write(buffer)
        }

        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }

    fn node() -> NodeDescription {
        NodeDescription {
            name: "a".to_owned(),
            kind: NodeKind::MetaCompilerCompiler,
            transformation: "a.gc3".to_owned(),
            socket: "a.gc3.sock".to_owned(),
            inputs: vec![],
            outputs: vec![],
        }
    }

    fn network(version: u64) -> Response {
        Response::Network {
            version,
            nodes: vec![node()],
        }
    }

    fn increment(position: usize, range: usize, text: &str) -> TextIncrement {
        TextIncrement {
            position,
            range,
            text: text.to_owned(),
        }
    }

    fn diagnostics(version: u64) -> Response {
        Response::Diagnostics {
            version,
            messages: vec![],
        }
    }

    #[test]
    fn first_build_opens_the_document_with_the_provided_text_and_queries_the_network() {
        let stream = Scripted::answering(&[(1, diagnostics(0)), (2, network(0))]);
        let mut conversation = Conversation::new(stream, "n.gc3n");
        let result = conversation.build("", &increment(0, 0, "abc"), "abc");
        let expected = vec![
            encode_request(
                1,
                &Request::Open {
                    document: "n.gc3n".to_owned(),
                    text: "abc".to_owned(),
                },
            ),
            encode_request(2, &Request::QueryNetwork),
        ];
        assert_eq!(
            (result, conversation.stream.requests()),
            (
                Ok(NetworkDescription {
                    version: 0,
                    nodes: vec![node()],
                }),
                expected
            )
        );
    }

    #[test]
    fn later_build_edits_the_document_at_the_version_answered_before() {
        let stream = Scripted::answering(&[
            (1, diagnostics(0)),
            (2, network(0)),
            (3, diagnostics(1)),
            (4, network(1)),
        ]);
        let mut conversation = Conversation::new(stream, "n.gc3n");
        let _ = conversation.build("", &increment(0, 0, "ab\ncd"), "ab\ncd");
        let _ = conversation.build("ab\ncd", &increment(3, 2, "x"), "ab\nx");
        let edit = encode_request(
            3,
            &Request::Edit {
                document: "n.gc3n".to_owned(),
                version: 0,
                deltas: vec![crate::core::api_message::EditDelta {
                    start: ReadPosition { line: 2, column: 1 },
                    end: ReadPosition { line: 2, column: 3 },
                    text: "x".to_owned(),
                }],
            },
        );
        assert_eq!(
            (
                conversation.stream.requests().get(2).cloned(),
                conversation.version
            ),
            (Some(edit), 1)
        );
    }

    #[test]
    fn version_mismatch_is_answered_by_an_open_request_with_the_provided_text() {
        let stream = Scripted::answering(&[
            (1, diagnostics(0)),
            (2, network(0)),
            (3, Response::VersionMismatch { version: 7 }),
            (4, diagnostics(0)),
            (5, network(0)),
        ]);
        let mut conversation = Conversation::new(stream, "n.gc3n");
        let _ = conversation.build("", &increment(0, 0, "a"), "a");
        let result = conversation.build("a", &increment(1, 0, "b"), "ab");
        let reopened = encode_request(
            4,
            &Request::Open {
                document: "n.gc3n".to_owned(),
                text: "ab".to_owned(),
            },
        );
        assert_eq!(
            (
                result.is_ok(),
                conversation.stream.requests().get(3).cloned()
            ),
            (true, Some(reopened))
        );
    }

    #[test]
    fn error_response_to_the_network_query_is_the_failure_with_its_message_text() {
        let stream = Scripted::answering(&[
            (1, diagnostics(0)),
            (2, Response::Error("not well formed".to_owned())),
        ]);
        let mut conversation = Conversation::new(stream, "n.gc3n");
        assert_eq!(
            conversation.build("", &increment(0, 0, "x"), "x"),
            Err(BuildError::Refused("not well formed".to_owned()))
        );
    }

    #[test]
    fn error_response_to_an_edit_request_makes_the_next_build_open_again() {
        let stream = Scripted::answering(&[
            (1, diagnostics(0)),
            (2, network(0)),
            (3, Response::Error("invalid range".to_owned())),
        ]);
        let mut conversation = Conversation::new(stream, "n.gc3n");
        let _ = conversation.build("", &increment(0, 0, "a"), "a");
        let _ = conversation.build("a", &increment(5, 0, "b"), "ab");
        assert!(!conversation.opened);
    }

    #[test]
    fn response_to_another_request_is_passed_over() {
        let stream = Scripted::answering(&[
            (9, Response::Acknowledged),
            (1, diagnostics(0)),
            (2, network(0)),
        ]);
        let mut conversation = Conversation::new(stream, "n.gc3n");
        assert!(conversation.build("", &increment(0, 0, "a"), "a").is_ok());
    }

    #[test]
    fn unexpected_response_to_the_network_query_is_a_protocol_error() {
        let stream = Scripted::answering(&[(1, diagnostics(0)), (2, Response::Acknowledged)]);
        let mut conversation = Conversation::new(stream, "n.gc3n");
        assert!(matches!(
            conversation.build("", &increment(0, 0, "a"), "a"),
            Err(BuildError::Protocol(_))
        ));
    }

    #[test]
    fn stream_that_ends_is_a_connection_error() {
        let mut conversation = Conversation::new(Scripted::answering(&[]), "n.gc3n");
        assert!(matches!(
            conversation.build("", &increment(0, 0, "a"), "a"),
            Err(BuildError::Connection(_))
        ));
    }

    #[test]
    fn shutdown_transmits_the_shutdown_request() {
        let mut conversation =
            Conversation::new(Scripted::answering(&[(1, Response::Acknowledged)]), "n.gc3n");
        let result = conversation.shut_down();
        assert_eq!(
            (result, conversation.stream.requests()),
            (Ok(()), vec![encode_request(1, &Request::Shutdown)])
        );
    }

    #[test]
    fn a_relative_build_system_is_started_by_an_absolute_path() {
        // FR-067, FR-094, IR-017: the process runs in the directory of the network file
        let relative = absolute_program("./Cargo.toml");
        let absolute = absolute_program("/bin/sh");
        assert_eq!(
            (
                relative.is_absolute(),
                relative.ends_with("Cargo.toml"),
                absolute
            ),
            (true, true, PathBuf::from("/bin/sh"))
        );
    }

    #[test]
    fn a_relative_build_system_that_does_not_exist_stays_as_it_is() {
        assert_eq!(
            absolute_program("./nowhere/genc3d"),
            PathBuf::from("./nowhere/genc3d")
        );
    }

    #[test]
    fn build_system_that_is_not_executable_is_not_started() {
        let result = BuildSession::start(
            "/nonexistent/genc3d",
            "/tmp/n.gc3n",
            Path::new("/tmp/unused.sock"),
        );
        let expected = if BuildSession::is_available() {
            BuildError::NotExecutable("/nonexistent/genc3d".to_owned())
        } else {
            BuildError::Unsupported
        };
        assert_eq!(result.err(), Some(expected));
    }
}
