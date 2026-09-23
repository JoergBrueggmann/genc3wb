//! The *build system*: its processes, the conversation on a *service socket*, the *build*, and the *node*.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::api_message::{
    Diagnostic, MessageError, NodeDescription, Request, Response, decode_response,
    delta_of_increment, encode_request, frames_of_message, read_message,
};
use crate::core::executable;
use crate::core::text_increment::TextIncrement;

use std::collections::BTreeMap;
use std::fmt;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::Duration;

/// How often *product* tries to connect to the *service socket* of a process it started.
const CONNECT_ATTEMPTS: u32 = 100;
/// The pause between two such attempts.
const CONNECT_PAUSE: Duration = Duration::from_millis(50);
/// The time after which a *response* that did not arrive is a failure.
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(10);
/// The time *product* waits for a process to end after its *shutdown request*.
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

// realises FR-070, FR-073, FR-128
/// What a *build* yields once the change of the *network file* is transmitted: the *diagnostics*
/// of that change, and the *network response* or why the network was not yielded.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BuildResult {
    /// the *diagnostics* of the open or edit request
    pub diagnostics: Vec<Diagnostic>,
    /// the *network response*, or why the *network query request* failed
    pub network: Result<NetworkDescription, BuildError>,
}

// realises FR-073, FR-074, FR-112, C-005
/// Why a *build* or a *request* failed, or why a process of the *build system* was not reached.
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
    /// the process answered with an error response; carries its message text
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

// realises FR-070, FR-072, FR-073, FR-106, FR-109, IR-018, IR-019, IR-020, IR-027, IR-028
/// The conversation with a process of the *build system* on a byte stream: the documents it
/// accepted an open request for, each with its *document version*.
///
/// * The stream is a generic bound, so that the conversation is tested on a stream in memory.
#[derive(Debug)]
pub struct Conversation<S: Read + Write> {
    /// the byte stream of the *service socket*
    stream: S,
    /// the *document version* the next edit request of each opened document applies to
    documents: BTreeMap<String, u64>,
    /// the *request identifier* of the next *request*
    next_request: u64,
}

impl<S: Read + Write> Conversation<S> {
    /// Creates the conversation on `stream`; nothing is transmitted, no document is opened.
    pub fn new(stream: S) -> Conversation<S> {
        Conversation {
            stream,
            documents: BTreeMap::new(),
            next_request: 1,
        }
    }

    // realises FR-070, FR-072, FR-106, FR-107, FR-112
    /// Transmits the change of the document `document`.
    ///
    /// * Where the document is not opened, the change is an open request carrying
    ///   `provided_after`; otherwise it is an edit request with the *edit delta* of `increment`.
    /// * A version mismatch response is answered by an open request carrying `provided_after`.
    /// * After an error response, the document counts as not opened, so that the next change
    ///   opens it again.
    /// * Yields the *diagnostics* of the *terminal response*, none for an acknowledged response.
    ///
    /// # Arguments
    /// * `document` - the *document identifier*
    /// * `provided_before` - the *provided text* `increment` applies to
    /// * `increment` - the *text increment* the code editor provided
    /// * `provided_after` - the *provided text* after `increment`
    ///
    /// # Errors
    /// Returns [`BuildError::Refused`] where the process answers with an error response,
    /// [`BuildError::Connection`] where the stream fails, and [`BuildError::Protocol`] where a
    /// *message* is not decoded or has an unexpected kind.
    pub fn transmit(
        &mut self,
        document: &str,
        provided_before: &str,
        increment: &TextIncrement,
        provided_after: &str,
    ) -> Result<Vec<Diagnostic>, BuildError> {
        let Some(version) = self.documents.get(document).copied() else {
            return self.open(document, provided_after);
        };
        let edit = Request::Edit {
            document: document.to_owned(),
            version,
            deltas: vec![delta_of_increment(provided_before, increment)],
        };
        match self.exchange(&edit)? {
            Response::Diagnostics {
                version,
                diagnostics,
            } => {
                self.documents.insert(document.to_owned(), version);
                Ok(diagnostics)
            }
            Response::VersionMismatch { .. } => self.open(document, provided_after),
            Response::Error(text) => {
                self.documents.remove(document);
                Err(BuildError::Refused(text))
            }
            Response::Acknowledged | Response::Network { .. } | Response::Other(_) => {
                Err(unexpected("the edit request"))
            }
        }
    }

    // realises FR-070, FR-073, FR-074
    /// Transmits a *network query request* and yields the *network response*.
    ///
    /// # Errors
    /// Returns [`BuildError::Refused`] where the *build system* answers with an error response,
    /// [`BuildError::Connection`] where the stream fails, and [`BuildError::Protocol`] where a
    /// *message* is not decoded or has an unexpected kind.
    pub fn query_network(&mut self) -> Result<NetworkDescription, BuildError> {
        match self.exchange(&Request::QueryNetwork)? {
            Response::Network { version, nodes } => Ok(NetworkDescription { version, nodes }),
            Response::Error(text) => Err(BuildError::Refused(text)),
            Response::Acknowledged
            | Response::Diagnostics { .. }
            | Response::VersionMismatch { .. }
            | Response::Other(_) => Err(unexpected("the network query request")),
        }
    }

    // realises FR-109, FR-111, FR-112
    /// Transmits a *store request* and awaits its acknowledged response.
    ///
    /// # Errors
    /// Returns [`BuildError::Refused`] where the *node* answers with an error response,
    /// [`BuildError::Connection`] where the stream fails, and [`BuildError::Protocol`] where a
    /// *message* is not decoded or has an unexpected kind.
    pub fn store(&mut self) -> Result<(), BuildError> {
        match self.exchange(&Request::Store)? {
            Response::Acknowledged => Ok(()),
            Response::Error(text) => Err(BuildError::Refused(text)),
            Response::Diagnostics { .. }
            | Response::VersionMismatch { .. }
            | Response::Network { .. }
            | Response::Other(_) => Err(unexpected("the store request")),
        }
    }

    // realises FR-068, FR-105
    /// Transmits a *shutdown request* and awaits its *terminal response*.
    ///
    /// # Errors
    /// Returns [`BuildError::Connection`] where the stream fails.
    pub fn shut_down(&mut self) -> Result<(), BuildError> {
        self.exchange(&Request::Shutdown).map(|_| ())
    }

    /// Transmits an open request carrying `text`; the *document version* becomes the one
    /// answered, and the *diagnostics* of the response are yielded.
    fn open(&mut self, document: &str, text: &str) -> Result<Vec<Diagnostic>, BuildError> {
        let open = Request::Open {
            document: document.to_owned(),
            text: text.to_owned(),
        };
        let (version, diagnostics) = match self.exchange(&open)? {
            Response::Acknowledged => (0, Vec::new()),
            Response::Diagnostics {
                version,
                diagnostics,
            } => (version, diagnostics),
            Response::Error(text) => return Err(BuildError::Refused(text)),
            Response::VersionMismatch { .. } | Response::Network { .. } | Response::Other(_) => {
                return Err(unexpected("the open request"));
            }
        };
        self.documents.insert(document.to_owned(), version);
        Ok(diagnostics)
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

// realises FR-068, FR-105, IR-017, IR-026
/// A process of the *build system* and its socket file.
///
/// * Dropping it ends the process where it still runs, and removes the socket file.
#[derive(Debug)]
struct ServedProcess {
    /// the process
    child: Child,
    /// the path of its *service socket*
    socket_path: PathBuf,
}

impl ServedProcess {
    // realises FR-067, FR-104, IR-017, IR-026, C-005
    /// Starts `executable` with `args` in `directory` and connects to `socket_path` once it
    /// listens.
    ///
    /// * A relative `executable` is resolved against the working directory of *product* before
    ///   the process is started, since the process is started in `directory` and the operating
    ///   system would otherwise look for it there.
    fn start(
        executable: &str,
        directory: &Path,
        args: &[String],
        socket_path: &Path,
    ) -> Result<(ServedProcess, ServiceStream), BuildError> {
        if !BuildSession::is_available() {
            return Err(BuildError::Unsupported);
        }
        if !executable::is_executable(executable) {
            return Err(BuildError::NotExecutable(executable.to_owned()));
        }
        let _ = std::fs::remove_file(socket_path);
        let mut child = Command::new(absolute_program(executable))
            .current_dir(directory)
            .args(args)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|error| BuildError::Process(error.to_string()))?;
        match connected(&mut child, socket_path) {
            Ok(stream) => Ok((
                ServedProcess {
                    child,
                    socket_path: socket_path.to_path_buf(),
                },
                stream,
            )),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = std::fs::remove_file(socket_path);
                Err(error)
            }
        }
    }

    // realises FR-068, FR-105
    /// Waits for the process to end, for at most `SHUTDOWN_TIMEOUT`; `Drop` ends it where it does
    /// not.
    fn await_end(&mut self) {
        let pause = Duration::from_millis(20);
        let mut waited = Duration::ZERO;
        while waited < SHUTDOWN_TIMEOUT && matches!(self.child.try_wait(), Ok(None)) {
            std::thread::sleep(pause);
            waited += pause;
        }
    }
}

impl Drop for ServedProcess {
    fn drop(&mut self) {
        if matches!(self.child.try_wait(), Ok(None)) {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        let _ = std::fs::remove_file(&self.socket_path);
    }
}

// realises FR-067, FR-068, IR-017, C-005
/// A *build system* *product* started in its *network role*, with the conversation on its
/// *service socket*.
#[derive(Debug)]
pub struct BuildSession {
    /// the process and its socket file
    process: ServedProcess,
    /// the conversation on its *service socket*
    conversation: Conversation<ServiceStream>,
    /// the *document identifier* of the *network file*: its file name (\[AD5\] IR-074)
    document: String,
}

impl BuildSession {
    // realises C-005
    /// Yields whether the platform offers the Unix domain socket the *builds* and the *node* need.
    pub fn is_available() -> bool {
        cfg!(unix)
    }

    // realises FR-067, IR-017
    /// Starts the *build system* `executable` on `network_path` in the *description mode*, in the
    /// directory of the *network file*, and connects to `socket_path` once it listens.
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
        let network = Path::new(network_path);
        let document = network
            .file_name()
            .map(|name| name.to_string_lossy().into_owned())
            .ok_or_else(|| BuildError::Process("no network file is named".to_owned()))?;
        let directory = match network.parent() {
            Some(parent) if !parent.as_os_str().is_empty() => parent.to_path_buf(),
            _ => PathBuf::from("."),
        };
        let args = vec![
            "--network".to_owned(),
            document.clone(),
            "--no-nodes".to_owned(),
            "--socket".to_owned(),
            socket_path.to_string_lossy().into_owned(),
        ];
        let (process, stream) = ServedProcess::start(executable, &directory, &args, socket_path)?;
        Ok(BuildSession {
            process,
            conversation: Conversation::new(stream),
            document,
        })
    }

    // realises FR-069, FR-070, FR-073, FR-128
    /// Carries out a *build*: transmits the change of the *network file*
    /// ([`Conversation::transmit`]), then queries the network ([`Conversation::query_network`]).
    ///
    /// * The *diagnostics* of the change are yielded with the outcome of the query, so that a
    ///   *network file* that is not well formed is marked where its fault lies.
    ///
    /// # Errors
    /// Returns what [`Conversation::transmit`] returns.
    pub fn build(
        &mut self,
        provided_before: &str,
        increment: &TextIncrement,
        provided_after: &str,
    ) -> Result<BuildResult, BuildError> {
        let diagnostics = self.conversation.transmit(
            &self.document,
            provided_before,
            increment,
            provided_after,
        )?;
        Ok(BuildResult {
            diagnostics,
            network: self.conversation.query_network(),
        })
    }

    // realises FR-068
    /// Transmits a *shutdown request* and waits for the process to end; ends it where it does not.
    pub fn shut_down(mut self) {
        let _ = self.conversation.shut_down();
        self.process.await_end();
    }
}

// realises FR-104, FR-105, FR-106, FR-109, IR-026, IR-027, C-005
/// A *node* *product* started, with the conversation on its *service socket*.
#[derive(Debug)]
pub struct NodeSession {
    /// the process and its socket file
    process: ServedProcess,
    /// the conversation on its *service socket*
    conversation: Conversation<ServiceStream>,
}

impl NodeSession {
    // realises FR-104, IR-026
    /// Starts the *build system* `executable` for one *node* in `directory`, the directory of the
    /// *network file*, with `--socket`, `--meta-dsl` naming `meta_dsl`, `--input` once per
    /// element of `inputs` and `--output` once per element of `outputs`, each path as given, and
    /// connects to `socket_path` once it listens.
    ///
    /// # Errors
    /// Returns what [`BuildSession::start`] returns.
    pub fn start(
        executable: &str,
        directory: &Path,
        meta_dsl: &str,
        inputs: &[String],
        outputs: &[String],
        socket_path: &Path,
    ) -> Result<NodeSession, BuildError> {
        let args = node_args(meta_dsl, inputs, outputs, socket_path);
        let (process, stream) = ServedProcess::start(executable, directory, &args, socket_path)?;
        Ok(NodeSession {
            process,
            conversation: Conversation::new(stream),
        })
    }

    // realises FR-106, FR-107, IR-027, IR-028
    /// Transmits the change of a document, as [`Conversation::transmit`].
    ///
    /// # Errors
    /// Returns what [`Conversation::transmit`] returns.
    pub fn transmit(
        &mut self,
        document: &str,
        provided_before: &str,
        increment: &TextIncrement,
        provided_after: &str,
    ) -> Result<Vec<Diagnostic>, BuildError> {
        self.conversation
            .transmit(document, provided_before, increment, provided_after)
    }

    // realises FR-109, FR-111
    /// Transmits a *store request*, as [`Conversation::store`].
    ///
    /// # Errors
    /// Returns what [`Conversation::store`] returns.
    pub fn store(&mut self) -> Result<(), BuildError> {
        self.conversation.store()
    }

    // realises FR-105
    /// Transmits a *shutdown request* and waits for the process to end; ends it where it does not.
    pub fn shut_down(mut self) {
        let _ = self.conversation.shut_down();
        self.process.await_end();
    }
}

// realises FR-104, IR-026
/// Yields the arguments a *node* is started with: `--socket`, `--meta-dsl`, `--input` once per
/// *input* and `--output` once per *output*, each path as given.
fn node_args(
    meta_dsl: &str,
    inputs: &[String],
    outputs: &[String],
    socket_path: &Path,
) -> Vec<String> {
    let mut args = vec![
        "--socket".to_owned(),
        socket_path.to_string_lossy().into_owned(),
        "--meta-dsl".to_owned(),
        meta_dsl.to_owned(),
    ];
    for input in inputs {
        args.push("--input".to_owned());
        args.push(input.clone());
    }
    for output in outputs {
        args.push("--output".to_owned());
        args.push(output.clone());
    }
    args
}

// realises FR-067, FR-094, FR-104, IR-017, IR-026
/// Yields the path by which a process of the *build system* is started: `executable` made
/// absolute against the working directory of *product*.
///
/// * The process is started in the directory of the *network file*, so a relative path would
///   otherwise be resolved against that directory instead of against the one in which the user
///   named it, which is the working directory of *product* ([`crate::core::executable::is_executable`]).
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
                     NetworkEditor::shut_down, NodeEditor::apply_increment, NodeEditor::report_arrived,
                     NodeEditor::close; the processes and the sockets of `BuildSession` and
                     `NodeSession` are covered by the ignored integration test 'tests/build_system.rs' */
#[cfg(test)]
mod tests {
    use super::*;

    use crate::core::api_message::{NodeKind, ReadPosition, Severity, encode_response};

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
            diagnostics: vec![],
        }
    }

    fn fault() -> Diagnostic {
        Diagnostic {
            severity: Severity::Error,
            start: ReadPosition { line: 1, column: 2 },
            end: ReadPosition { line: 1, column: 3 },
            text: "fault".to_owned(),
        }
    }

    /// A conversation about the document `n.gc3n` that carries out a *build*, as the build
    /// session does.
    fn build(
        conversation: &mut Conversation<Scripted>,
        before: &str,
        increment: &TextIncrement,
        after: &str,
    ) -> Result<NetworkDescription, BuildError> {
        conversation.transmit("n.gc3n", before, increment, after)?;
        conversation.query_network()
    }

    #[test]
    fn first_build_opens_the_document_with_the_provided_text_and_queries_the_network() {
        let stream = Scripted::answering(&[(1, diagnostics(0)), (2, network(0))]);
        let mut conversation = Conversation::new(stream);
        let result = build(&mut conversation, "", &increment(0, 0, "abc"), "abc");
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
        let mut conversation = Conversation::new(stream);
        let _ = build(&mut conversation, "", &increment(0, 0, "ab\ncd"), "ab\ncd");
        let _ = build(&mut conversation, "ab\ncd", &increment(3, 2, "x"), "ab\nx");
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
                conversation.documents.get("n.gc3n").copied()
            ),
            (Some(edit), Some(1))
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
        let mut conversation = Conversation::new(stream);
        let _ = build(&mut conversation, "", &increment(0, 0, "a"), "a");
        let result = build(&mut conversation, "a", &increment(1, 0, "b"), "ab");
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
        let mut conversation = Conversation::new(stream);
        assert_eq!(
            build(&mut conversation, "", &increment(0, 0, "x"), "x"),
            Err(BuildError::Refused("not well formed".to_owned()))
        );
    }

    #[test]
    fn error_response_to_an_edit_request_makes_the_next_change_open_again() {
        let stream = Scripted::answering(&[
            (1, diagnostics(0)),
            (2, network(0)),
            (3, Response::Error("invalid range".to_owned())),
        ]);
        let mut conversation = Conversation::new(stream);
        let _ = build(&mut conversation, "", &increment(0, 0, "a"), "a");
        let _ = build(&mut conversation, "a", &increment(5, 0, "b"), "ab");
        assert!(!conversation.documents.contains_key("n.gc3n"));
    }

    #[test]
    fn response_to_another_request_is_passed_over() {
        let stream = Scripted::answering(&[
            (9, Response::Acknowledged),
            (1, diagnostics(0)),
            (2, network(0)),
        ]);
        let mut conversation = Conversation::new(stream);
        assert!(build(&mut conversation, "", &increment(0, 0, "a"), "a").is_ok());
    }

    #[test]
    fn unexpected_response_to_the_network_query_is_a_protocol_error() {
        let stream = Scripted::answering(&[(1, diagnostics(0)), (2, Response::Acknowledged)]);
        let mut conversation = Conversation::new(stream);
        assert!(matches!(
            build(&mut conversation, "", &increment(0, 0, "a"), "a"),
            Err(BuildError::Protocol(_))
        ));
    }

    #[test]
    fn stream_that_ends_is_a_connection_error() {
        let mut conversation = Conversation::new(Scripted::answering(&[]));
        assert!(matches!(
            build(&mut conversation, "", &increment(0, 0, "a"), "a"),
            Err(BuildError::Connection(_))
        ));
    }

    #[test]
    fn shutdown_transmits_the_shutdown_request() {
        let mut conversation =
            Conversation::new(Scripted::answering(&[(1, Response::Acknowledged)]));
        let result = conversation.shut_down();
        assert_eq!(
            (result, conversation.stream.requests()),
            (Ok(()), vec![encode_request(1, &Request::Shutdown)])
        );
    }

    #[test]
    fn documents_of_a_node_are_opened_and_edited_apart() {
        // FR-106, IR-027: two documents on one conversation, each with its own version
        let stream = Scripted::answering(&[
            (1, diagnostics(0)),
            (2, diagnostics(0)),
            (3, diagnostics(1)),
            (4, diagnostics(1)),
        ]);
        let mut conversation = Conversation::new(stream);
        let _ = conversation.transmit("a.gc3", "", &increment(0, 0, "x"), "x");
        let _ = conversation.transmit("in.txt", "", &increment(0, 0, "y"), "y");
        let _ = conversation.transmit("in.txt", "y", &increment(1, 0, "z"), "yz");
        let _ = conversation.transmit("a.gc3", "x", &increment(1, 0, "w"), "xw");
        let kinds: Vec<u8> = conversation
            .stream
            .requests()
            .iter()
            .map(|request| request[2])
            .collect();
        assert_eq!(
            (
                kinds,
                conversation.documents.get("a.gc3").copied(),
                conversation.documents.get("in.txt").copied()
            ),
            (vec![1, 1, 2, 2], Some(1), Some(1))
        );
    }

    #[test]
    fn transmit_yields_the_diagnostics_of_the_response() {
        // FR-107
        let stream = Scripted::answering(&[(
            1,
            Response::Diagnostics {
                version: 0,
                diagnostics: vec![fault()],
            },
        )]);
        let mut conversation = Conversation::new(stream);
        assert_eq!(
            conversation.transmit("a.gc3", "", &increment(0, 0, "x"), "x"),
            Ok(vec![fault()])
        );
    }

    #[test]
    fn acknowledged_open_request_yields_no_diagnostic() {
        // FR-108
        let stream = Scripted::answering(&[(1, Response::Acknowledged)]);
        let mut conversation = Conversation::new(stream);
        let result = conversation.transmit("a.gc3", "", &increment(0, 0, "x"), "x");
        assert_eq!(
            (result, conversation.documents.get("a.gc3").copied()),
            (Ok(vec![]), Some(0))
        );
    }

    #[test]
    fn store_transmits_the_store_request_and_is_acknowledged() {
        // FR-109, FR-111, IR-027
        let mut conversation =
            Conversation::new(Scripted::answering(&[(1, Response::Acknowledged)]));
        let result = conversation.store();
        assert_eq!(
            (result, conversation.stream.requests()),
            (Ok(()), vec![encode_request(1, &Request::Store)])
        );
    }

    #[test]
    fn error_response_to_the_store_request_is_the_failure_with_its_message_text() {
        // FR-112
        let mut conversation = Conversation::new(Scripted::answering(&[(
            1,
            Response::Error("out.txt: permission denied".to_owned()),
        )]));
        assert_eq!(
            conversation.store(),
            Err(BuildError::Refused("out.txt: permission denied".to_owned()))
        );
    }

    #[test]
    fn unexpected_response_to_the_store_request_is_a_protocol_error() {
        let mut conversation = Conversation::new(Scripted::answering(&[(1, network(0))]));
        assert!(matches!(conversation.store(), Err(BuildError::Protocol(_))));
    }

    #[test]
    fn node_is_started_with_its_socket_its_meta_dsl_its_inputs_and_its_outputs() {
        // FR-104, IR-026
        assert_eq!(
            node_args(
                "a.gc3",
                &["in1.txt".to_owned(), "in2.txt".to_owned()],
                &["out.txt".to_owned()],
                Path::new("/tmp/n1.sock")
            ),
            vec![
                "--socket",
                "/tmp/n1.sock",
                "--meta-dsl",
                "a.gc3",
                "--input",
                "in1.txt",
                "--input",
                "in2.txt",
                "--output",
                "out.txt"
            ]
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

    #[test]
    fn node_of_a_build_system_that_is_not_executable_is_not_started() {
        // FR-104, FR-112
        let result = NodeSession::start(
            "/nonexistent/genc3d",
            Path::new("/tmp"),
            "a.gc3",
            &[],
            &[],
            Path::new("/tmp/unused-node.sock"),
        );
        let expected = if BuildSession::is_available() {
            BuildError::NotExecutable("/nonexistent/genc3d".to_owned())
        } else {
            BuildError::Unsupported
        };
        assert_eq!(result.err(), Some(expected));
    }
}
