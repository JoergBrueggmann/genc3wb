//! The *build system*: its processes, the conversation on a *service socket*, the *build*, and the *node*.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::api_message::{
    Diagnostic, DiagnosticGroup, MessageError, NetworkMode, NodeDescription, NodeState, Request,
    Response, decode_response, delta_of_increment, encode_request, frames_of_message, has_error,
    octet_delta_between, read_message,
};
use crate::core::executable;
use crate::core::text_increment::TextIncrement;

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, RecvTimeoutError, Sender, channel};
use std::time::Duration;

/// How often *product* tries to connect to the *service socket* of a process it started.
const CONNECT_ATTEMPTS: u32 = 100;
/// How often *product* tries to connect to the *service socket* of a *node* it attaches to,
/// which the *build system* has started or is starting.
const ATTACH_ATTEMPTS: u32 = 20;
/// The pause between two such attempts.
const CONNECT_PAUSE: Duration = Duration::from_millis(50);
/// The time after which a *response* that did not arrive is a failure.
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(10);
/// The time *product* waits for the *build system* to end after its *shutdown request*, in
/// which the *build system* stores and shuts down every *node* it runs (\[AD3\] IR-080 to
/// IR-082); ending it before would leave its *nodes* running.
const SHUTDOWN_TIMEOUT: Duration = Duration::from_secs(10);

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

// realises FR-229, FR-231, FR-232, FR-233, FR-111, FR-239
/// What a *state response* tells: the *node states*, and, where a *node* answers, the
/// *diagnostic group* of each of its *inputs* and of its *meta compiler DSL* and the
/// *document version* of each of its *inputs* and *outputs*, and, where the *build system*
/// answers, its *network mode*.
///
/// * Every *state response* carries the whole of it anew, so that the latest is the state.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Watched {
    /// the *node states*: the one of the *node*, or those of every *node* of the network in
    /// the order of the *network file*
    pub states: Vec<NodeState>,
    /// the *diagnostic groups*, none where the *build system* answers
    pub groups: Vec<DiagnosticGroup>,
    /// the *document identifier* and the *document version* of every *input* and *output*,
    /// none where the *build system* answers
    pub versions: Vec<(String, u64)>,
    /// the *network mode*, `None` where a *node* answers
    pub mode: Option<NetworkMode>,
}

// realises FR-110, FR-111, FR-239
/// What a content query request yields: the content the *node* holds of a document.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DocumentContent {
    /// the *document version* of the content
    pub version: u64,
    /// the octets of the document: those of the UTF-8 encoding of its text where it is a
    /// *text document*
    pub octets: Vec<u8>,
    /// whether the document is a *binary document*
    pub binary: bool,
}

// realises FR-229, FR-231, FR-112
/// The receiver of what a watch tells: every *state response* as it arrives, and, once, why the
/// connection of the watch was lost.
///
/// * It is called on the reader thread of the [`WatchedConnection`], so that it hands what it
///   receives over to the thread that owns the state, and holds nothing of that thread.
pub type StateSink = Box<dyn FnMut(Result<Watched, BuildError>) + Send>;

// realises FR-107, FR-109, FR-225, FR-226, FR-227, FR-246, FR-248, FR-252
/// What the change of a document yields: the *diagnostics* of that document, and those of the
/// other documents the *diagnostic* response carried a *diagnostic group* for.
///
/// * `groups` holds the *diagnostic groups* that are taken alone: one that names the changed
///   document itself, a document the conversation holds no *document version* of, or another
///   *document version* than the one it holds, is left out (FR-226, FR-227).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct DiagnosticAnswer {
    /// the *document version* the *node* answered for the changed document, which is the one
    /// *product* holds of it from then on (FR-231, FR-232)
    pub version: u64,
    /// the *diagnostics* of the changed document, in the order of the response
    pub diagnostics: Vec<Diagnostic>,
    /// the *diagnostic groups* taken, in the order of the response
    pub groups: Vec<DiagnosticGroup>,
    /// whether a *diagnostic* of severity error is among those the response carried, those of
    /// every *diagnostic group* included, the ones left out as well (FR-109)
    pub erroneous: bool,
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

// realises IR-018
/// The connection a [`Conversation`] transmits its *requests* on and receives their *responses*
/// from.
///
/// * It is a generic bound of the conversation: every byte stream is one, which reads the
///   *responses* in the thread of the conversation, so that the conversation is tested on a
///   stream in memory; the [`WatchedConnection`] is the one of a *service socket*, which a
///   reader thread reads.
pub trait Connection {
    /// Transmits the *frames* of one *request*.
    ///
    /// # Errors
    /// Returns [`BuildError::Connection`] where the connection fails.
    fn send(&mut self, frames: &[u8]) -> Result<(), BuildError>;

    /// Yields the next *response* with the *request identifier* it answers, waiting for it.
    ///
    /// # Errors
    /// Returns [`BuildError::Connection`] where the connection fails or no *response* arrives
    /// in time, and [`BuildError::Protocol`] where a *message* is not decoded.
    fn receive(&mut self) -> Result<(u64, Response), BuildError>;
}

impl<S: Read + Write> Connection for S {
    fn send(&mut self, frames: &[u8]) -> Result<(), BuildError> {
        self.write_all(frames)
            .and_then(|()| self.flush())
            .map_err(|error| BuildError::Connection(error.to_string()))
    }

    fn receive(&mut self) -> Result<(u64, Response), BuildError> {
        Ok(decode_response(&read_message(self)?)?)
    }
}

// realises FR-228, FR-229, FR-230, FR-231, IR-018, IR-020, IR-028
/// The connection to a *service socket* that is watched: a reader thread of its own receives
/// every *response*, hands each *state response* to the [`StateSink`] as it arrives, asked for
/// or not, and each other one to the conversation, which awaits it.
///
/// * The reader thread ends when the connection does: it then hands the reason to the
///   conversation and to the sink, unless the connection was closed or silenced by *product*.
/// * Dropping it closes the connection, which ends the watch (\[AD3\] § Message schema, rule
///   'Watch') and the reader thread.
#[derive(Debug)]
pub struct WatchedConnection {
    /// the half of the *service socket* the *requests* are written to
    writer: ServiceStream,
    /// the *responses* the reader thread received that are no *state responses*, or why the
    /// connection ended
    answers: Receiver<Result<(u64, Response), BuildError>>,
    /// whether the sink is told nothing any more, the connection being closed by *product*
    silenced: Arc<AtomicBool>,
}

impl WatchedConnection {
    // realises FR-228, FR-230
    /// Takes `stream`, the connection to a *service socket*, and starts the reader thread, which
    /// hands what it is told to `sink`.
    ///
    /// # Errors
    /// Returns [`BuildError::Connection`] where the stream cannot be read by a second thread.
    #[cfg(unix)]
    fn open(stream: ServiceStream, sink: StateSink) -> Result<WatchedConnection, BuildError> {
        let connection = |error: std::io::Error| BuildError::Connection(error.to_string());
        stream.set_read_timeout(None).map_err(connection)?;
        let mut reader = stream.try_clone().map_err(connection)?;
        let (received, answers) = channel();
        let silenced = Arc::new(AtomicBool::new(false));
        let silenced_for_reader = Arc::clone(&silenced);
        std::thread::spawn(move || {
            let mut sink = sink;
            read_responses(&mut reader, &received, &mut |told| {
                if !silenced_for_reader.load(Ordering::Acquire) {
                    sink(told);
                }
            });
        });
        Ok(WatchedConnection {
            writer: stream,
            answers,
            silenced,
        })
    }

    /// Yields that the platform offers no Unix domain socket; `start` fails before it calls this.
    #[cfg(not(unix))]
    fn open(_stream: ServiceStream, _sink: StateSink) -> Result<WatchedConnection, BuildError> {
        Err(BuildError::Unsupported)
    }

    /// Tells the sink nothing from now on: *product* is about to end the connection itself, so
    /// that its end is no loss to report.
    fn silence(&self) {
        self.silenced.store(true, Ordering::Release);
    }
}

impl Connection for WatchedConnection {
    fn send(&mut self, frames: &[u8]) -> Result<(), BuildError> {
        self.writer
            .write_all(frames)
            .and_then(|()| self.writer.flush())
            .map_err(|error| BuildError::Connection(error.to_string()))
    }

    fn receive(&mut self) -> Result<(u64, Response), BuildError> {
        match self.answers.recv_timeout(RESPONSE_TIMEOUT) {
            Ok(answer) => answer,
            Err(RecvTimeoutError::Timeout) => Err(BuildError::Connection(
                "the response did not arrive in time".to_owned(),
            )),
            Err(RecvTimeoutError::Disconnected) => Err(BuildError::Connection(
                "the connection is closed".to_owned(),
            )),
        }
    }
}

impl Drop for WatchedConnection {
    fn drop(&mut self) {
        self.silence();
        #[cfg(unix)]
        let _ = self.writer.shutdown(std::net::Shutdown::Both);
    }
}

// realises FR-229, FR-231, FR-112
/// Reads the *responses* of `reader` until it ends: hands each *state response*, which is no
/// *terminal response*, to `sink`, and each other one to `answers`, for the conversation that
/// awaits it.
///
/// * Where the stream ends or fails, or a *message* is not decoded, the reason is handed to
///   `answers` and to `sink`, and the reading ends.
/// * The reading ends as well where the conversation is gone, `answers` taking nothing more.
fn read_responses(
    reader: &mut impl Read,
    answers: &Sender<Result<(u64, Response), BuildError>>,
    sink: &mut dyn FnMut(Result<Watched, BuildError>),
) {
    loop {
        let received = read_message(reader)
            .and_then(|message| decode_response(&message))
            .map_err(BuildError::from);
        match received {
            Ok((
                _,
                Response::State {
                    states,
                    groups,
                    versions,
                    mode,
                },
            )) => sink(Ok(Watched {
                states,
                groups,
                versions,
                mode,
            })),
            Ok(answer) => {
                if answers.send(Ok(answer)).is_err() {
                    return;
                }
            }
            Err(error) => {
                let _ = answers.send(Err(error.clone()));
                sink(Err(error));
                return;
            }
        }
    }
}

// realises FR-070, FR-072, FR-073, FR-106, FR-109, FR-225, FR-226, FR-227, FR-228, FR-230,
// FR-239, IR-018, IR-019, IR-020, IR-027, IR-028
/// The conversation with a process of the *build system* on a [`Connection`]: the documents it
/// accepted an open request for, each with its *document version*.
///
/// * The connection is a generic bound, so that the conversation is tested on a stream in
///   memory.
/// * A *response* that is no *terminal response*, a *state response*, answers no *request* the
///   conversation awaits: on a byte stream it is passed over, and on a [`WatchedConnection`]
///   the reader thread hands it to the sink before the conversation sees it.
#[derive(Debug)]
pub struct Conversation<S: Connection> {
    /// the connection to the *service socket*
    connection: S,
    /// the *document version* the next edit request of each opened document applies to
    documents: BTreeMap<String, u64>,
    /// the opened documents that are *binary documents*, opened by their octets
    binary_documents: BTreeSet<String>,
    /// the *request identifier* of the next *request*
    next_request: u64,
}

impl<S: Connection> Conversation<S> {
    /// Creates the conversation on `connection`; nothing is transmitted, no document is opened.
    pub fn new(connection: S) -> Conversation<S> {
        Conversation {
            connection,
            documents: BTreeMap::new(),
            binary_documents: BTreeSet::new(),
            next_request: 1,
        }
    }

    // realises FR-070, FR-072, FR-106, FR-107, FR-112, FR-225
    /// Transmits the change of the document `document`.
    ///
    /// * Where the document is not opened, or is opened as a *binary document*, which takes no
    ///   *edit delta* (\[AD3\] IR-111), the change is an open request carrying `provided_after`;
    ///   otherwise it is an edit request with the *edit delta* of `increment`.
    /// * A version mismatch response is answered by an open request carrying `provided_after`.
    /// * After an error response, the document counts as not opened, so that the next change
    ///   opens it again.
    /// * Yields the *diagnostics* of the *terminal response* and its *diagnostic groups* that
    ///   are taken ([`DiagnosticAnswer`]), none for an acknowledged response.
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
    ) -> Result<DiagnosticAnswer, BuildError> {
        let version = match self.documents.get(document).copied() {
            Some(version) if !self.binary_documents.contains(document) => version,
            Some(_) | None => return self.open(document, provided_after),
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
                groups,
            } => {
                self.documents.insert(document.to_owned(), version);
                Ok(self.answer_of(document, diagnostics, groups))
            }
            Response::VersionMismatch { .. } => self.open(document, provided_after),
            Response::Error(text) => {
                self.documents.remove(document);
                Err(BuildError::Refused(text))
            }
            Response::Acknowledged
            | Response::Network { .. }
            | Response::State { .. }
            | Response::Content { .. }
            | Response::Cancelled
            | Response::Other(_) => Err(unexpected("the edit request")),
        }
    }

    // realises FR-167, FR-107, FR-112
    /// Transmits the octets of the document `document` by an open request, the document being a
    /// *binary document* from then on (\[AD3\] IR-112).
    ///
    /// * Yields the *diagnostics* of the *terminal response* and its *diagnostic groups* that
    ///   are taken ([`DiagnosticAnswer`]), none for an acknowledged response.
    ///
    /// # Errors
    /// Returns what [`Conversation::transmit`] returns.
    pub fn open_octets(
        &mut self,
        document: &str,
        octets: &[u8],
    ) -> Result<DiagnosticAnswer, BuildError> {
        let open = Request::OpenOctets {
            document: document.to_owned(),
            octets: octets.to_vec(),
        };
        let answer = self.opened(document, &open)?;
        self.binary_documents.insert(document.to_owned());
        Ok(answer)
    }

    // realises FR-210, FR-107, FR-112
    /// Transmits the change of the *binary document* `document` from `octets_before` to
    /// `octets_after`.
    ///
    /// * Where the document is opened as a *binary document*, the change is an edit request
    ///   with the *octet delta* between the two (\[AD3\] IR-114); otherwise it is an open
    ///   request carrying `octets_after`, as [`Conversation::open_octets`].
    /// * A version mismatch response is answered by an open request carrying `octets_after`.
    /// * After an error response, the document counts as not opened, so that the next change
    ///   opens it again.
    /// * Yields the *diagnostics* of the *terminal response* and its *diagnostic groups* that
    ///   are taken ([`DiagnosticAnswer`]), none for an acknowledged response.
    ///
    /// # Errors
    /// Returns what [`Conversation::transmit`] returns.
    pub fn edit_octets(
        &mut self,
        document: &str,
        octets_before: &[u8],
        octets_after: &[u8],
    ) -> Result<DiagnosticAnswer, BuildError> {
        let version = match self.documents.get(document).copied() {
            Some(version) if self.binary_documents.contains(document) => version,
            Some(_) | None => return self.open_octets(document, octets_after),
        };
        let edit = Request::EditOctets {
            document: document.to_owned(),
            version,
            deltas: vec![octet_delta_between(octets_before, octets_after)],
        };
        match self.exchange(&edit)? {
            Response::Diagnostics {
                version,
                diagnostics,
                groups,
            } => {
                self.documents.insert(document.to_owned(), version);
                Ok(self.answer_of(document, diagnostics, groups))
            }
            Response::VersionMismatch { .. } => self.open_octets(document, octets_after),
            Response::Error(text) => {
                self.documents.remove(document);
                self.binary_documents.remove(document);
                Err(BuildError::Refused(text))
            }
            Response::Acknowledged
            | Response::Network { .. }
            | Response::State { .. }
            | Response::Content { .. }
            | Response::Cancelled
            | Response::Other(_) => Err(unexpected("the edit request")),
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
            | Response::State { .. }
            | Response::Content { .. }
            | Response::Cancelled
            | Response::Other(_) => Err(unexpected("the network query request")),
        }
    }

    // realises FR-109, FR-112, FR-211
    /// Transmits a *store request* and awaits its acknowledged response.
    ///
    /// # Errors
    /// Returns [`BuildError::Refused`] where the *build system* answers with an error response,
    /// [`BuildError::Connection`] where the stream fails, and [`BuildError::Protocol`] where a
    /// *message* is not decoded or has an unexpected kind.
    pub fn store(&mut self) -> Result<(), BuildError> {
        match self.exchange(&Request::Store)? {
            Response::Acknowledged => Ok(()),
            Response::Error(text) => Err(BuildError::Refused(text)),
            Response::Diagnostics { .. }
            | Response::VersionMismatch { .. }
            | Response::Network { .. }
            | Response::State { .. }
            | Response::Content { .. }
            | Response::Cancelled
            | Response::Other(_) => Err(unexpected("the store request")),
        }
    }

    // realises FR-260, FR-261
    /// Transmits an *activate request* and awaits its acknowledged response, which the
    /// *build system* transmits once every *node* of the network is an *active node*
    /// (\[AD3\] IR-157 to IR-159).
    ///
    /// # Errors
    /// Returns [`BuildError::Refused`] where the *build system* answers with an error response,
    /// since the *network file* or the *configuration* of a *node* is invalid (\[AD3\] IR-161,
    /// IR-162), [`BuildError::Connection`] where the stream fails, and [`BuildError::Protocol`]
    /// where a *message* is not decoded or has an unexpected kind.
    pub fn activate(&mut self) -> Result<(), BuildError> {
        match self.exchange(&Request::Activate)? {
            Response::Acknowledged => Ok(()),
            Response::Error(text) => Err(BuildError::Refused(text)),
            Response::Diagnostics { .. }
            | Response::VersionMismatch { .. }
            | Response::Network { .. }
            | Response::State { .. }
            | Response::Content { .. }
            | Response::Cancelled
            | Response::Other(_) => Err(unexpected("the activate request")),
        }
    }

    // realises FR-267, FR-268
    /// Transmits a *deactivate request* and awaits its acknowledged response, which the
    /// *build system* transmits once the last *node* has terminated after the
    /// *network deactivation*, or at once where the network is not active
    /// (\[AD3\] IR-164 to IR-166).
    ///
    /// # Errors
    /// Returns [`BuildError::Refused`] where the *build system* answers with an error response,
    /// [`BuildError::Connection`] where the stream fails, and [`BuildError::Protocol`] where a
    /// *message* is not decoded or has an unexpected kind.
    pub fn deactivate(&mut self) -> Result<(), BuildError> {
        match self.exchange(&Request::Deactivate)? {
            Response::Acknowledged => Ok(()),
            Response::Error(text) => Err(BuildError::Refused(text)),
            Response::Diagnostics { .. }
            | Response::VersionMismatch { .. }
            | Response::Network { .. }
            | Response::State { .. }
            | Response::Content { .. }
            | Response::Cancelled
            | Response::Other(_) => Err(unexpected("the deactivate request")),
        }
    }

    // realises FR-281, FR-286
    /// Transmits a *hold request* naming `node` and awaits its acknowledged response, which the
    /// *build system* transmits once the *node* it started has told its *node mode*, or at once
    /// where the *node* runs (\[AD3\] IR-198, IR-199).
    ///
    /// # Errors
    /// Returns [`BuildError::Refused`] where the *build system* answers with an error response,
    /// in an *invalid network* or for a *node* the gc3n document does not state
    /// (\[AD3\] IR-200), [`BuildError::Connection`] where the stream fails, and
    /// [`BuildError::Protocol`] where a *message* is not decoded or has an unexpected kind.
    pub fn hold(&mut self, node: &str) -> Result<(), BuildError> {
        let hold = Request::Hold {
            node: node.to_owned(),
        };
        acknowledged(self.exchange(&hold)?, "the hold request")
    }

    // realises FR-282, FR-287
    /// Transmits a *release request* naming `node` and awaits its acknowledged response, which
    /// the *build system* transmits once the *node* has ended, or at once (\[AD3\] IR-201,
    /// IR-202).
    ///
    /// # Errors
    /// Returns [`BuildError::Refused`] where the *build system* answers with an error response,
    /// [`BuildError::Connection`] where the stream fails, and [`BuildError::Protocol`] where a
    /// *message* is not decoded or has an unexpected kind.
    pub fn release(&mut self, node: &str) -> Result<(), BuildError> {
        let release = Request::Release {
            node: node.to_owned(),
        };
        acknowledged(self.exchange(&release)?, "the release request")
    }

    // realises FR-228, FR-230
    /// Transmits a *watch request* and awaits nothing: its *state responses* are no
    /// *terminal responses*, the first follows at once and a further one whenever the state
    /// changes (\[AD3\] IR-121, IR-122, IR-129, IR-130), and the [`WatchedConnection`] hands
    /// each to its sink.
    ///
    /// # Errors
    /// Returns [`BuildError::Connection`] where the stream fails.
    pub fn watch(&mut self) -> Result<(), BuildError> {
        self.request(&Request::Watch).map(|_| ())
    }

    // realises FR-110, FR-111, FR-239, FR-264
    /// Transmits a content query request naming `document` and yields the content the process
    /// holds of it (\[AD3\] IR-134, IR-135), or `None` where the process answers with a
    /// cancelled response, an edit request of the *producer* having superseded the query
    /// (\[AD3\] IR-029); the next *state response* has the content asked for anew.
    ///
    /// # Errors
    /// Returns [`BuildError::Refused`] where the process holds no such document and answers
    /// with an error response, [`BuildError::Connection`] where the stream fails, and
    /// [`BuildError::Protocol`] where a *message* is not decoded or has an unexpected kind.
    pub fn query_content(&mut self, document: &str) -> Result<Option<DocumentContent>, BuildError> {
        let query = Request::QueryContent {
            document: document.to_owned(),
        };
        match self.exchange(&query)? {
            Response::Content {
                version,
                octets,
                binary,
            } => Ok(Some(DocumentContent {
                version,
                octets,
                binary,
            })),
            Response::Cancelled => Ok(None),
            Response::Error(text) => Err(BuildError::Refused(text)),
            Response::Acknowledged
            | Response::Diagnostics { .. }
            | Response::VersionMismatch { .. }
            | Response::Network { .. }
            | Response::State { .. }
            | Response::Other(_) => Err(unexpected("the content query request")),
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

    /// Transmits an open request carrying `text`, the document being a *text document* from
    /// then on; the *document version* becomes the one answered, and the *diagnostics* of the
    /// response are yielded.
    fn open(&mut self, document: &str, text: &str) -> Result<DiagnosticAnswer, BuildError> {
        let open = Request::Open {
            document: document.to_owned(),
            text: text.to_owned(),
        };
        let answer = self.opened(document, &open)?;
        self.binary_documents.remove(document);
        Ok(answer)
    }

    /// Transmits the open request `open` of `document`; the *document version* becomes the one
    /// answered, and the *diagnostics* of the response are yielded.
    fn opened(&mut self, document: &str, open: &Request) -> Result<DiagnosticAnswer, BuildError> {
        let (version, diagnostics, groups) = match self.exchange(open)? {
            Response::Acknowledged => (0, Vec::new(), Vec::new()),
            Response::Diagnostics {
                version,
                diagnostics,
                groups,
            } => (version, diagnostics, groups),
            Response::Error(text) => {
                self.documents.remove(document);
                self.binary_documents.remove(document);
                return Err(BuildError::Refused(text));
            }
            Response::VersionMismatch { .. }
            | Response::Network { .. }
            | Response::State { .. }
            | Response::Content { .. }
            | Response::Cancelled
            | Response::Other(_) => {
                return Err(unexpected("the open request"));
            }
        };
        self.documents.insert(document.to_owned(), version);
        Ok(self.answer_of(document, diagnostics, groups))
    }

    // realises FR-109, FR-225, FR-226, FR-227
    /// Yields the answer to the change of `document`: its `diagnostics`, and those of `groups`
    /// that are taken.
    ///
    /// * A *diagnostic group* is taken where it names another document than `document`, and
    ///   the *document version* it states is the one the conversation holds of that document;
    ///   one the conversation holds another *document version* of was computed from a state
    ///   that is changed since, and the response to that change brings its *diagnostics*.
    /// * An error in a *diagnostic group* that is left out counts as well, since it is an error
    ///   of what the *node* holds.
    fn answer_of(
        &self,
        document: &str,
        diagnostics: Vec<Diagnostic>,
        groups: Vec<DiagnosticGroup>,
    ) -> DiagnosticAnswer {
        let erroneous =
            has_error(&diagnostics) || groups.iter().any(|group| has_error(&group.diagnostics));
        DiagnosticAnswer {
            version: self.documents.get(document).copied().unwrap_or(0),
            diagnostics,
            groups: groups
                .into_iter()
                .filter(|group| {
                    group.document != document
                        && self.documents.get(&group.document) == Some(&group.version)
                })
                .collect(),
            erroneous,
        }
    }

    /// Transmits `request` and yields the *terminal response* that carries its
    /// *request identifier*.
    ///
    /// * A *response* to another *request*, and one that is no *terminal response*, is passed
    ///   over.
    fn exchange(&mut self, request: &Request) -> Result<Response, BuildError> {
        let identifier = self.request(request)?;
        loop {
            let (answered, response) = self.connection.receive()?;
            if answered == identifier && response.is_terminal() {
                return Ok(response);
            }
        }
    }

    /// Transmits `request` under the next *request identifier*, and yields that identifier.
    fn request(&mut self, request: &Request) -> Result<u64, BuildError> {
        let identifier = self.next_request;
        self.next_request += 1;
        self.connection
            .send(&frames_of_message(&encode_request(identifier, request)))?;
        Ok(identifier)
    }
}

// realises FR-068, IR-017
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
    // realises FR-067, IR-017, C-005
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
        match connected(Some(&mut child), socket_path, CONNECT_ATTEMPTS) {
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

    // realises FR-068
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

// realises FR-067, FR-068, FR-109, FR-228, FR-229, FR-244, FR-260, FR-267, IR-017, C-005
/// A *build system* *product* started in its *network role*, which activates the network and
/// runs every *node* of the *network file* while the network is an *active network*, with the
/// conversation on its *service socket* and the watch of its *node states* and its
/// *network mode*.
#[derive(Debug)]
pub struct BuildSession {
    /// the process and its socket file
    process: ServedProcess,
    /// the conversation on its *service socket*, which is watched
    conversation: Conversation<WatchedConnection>,
    /// the *document identifier* of the *network file*: its file name (\[AD5\] IR-074)
    document: String,
}

impl BuildSession {
    // realises C-005
    /// Yields whether the platform offers the Unix domain socket the *builds* and the *node* need.
    pub fn is_available() -> bool {
        cfg!(unix)
    }

    // realises FR-067, FR-228, IR-017
    /// Starts the *build system* `executable` on `network_path` in its *network role*, in the
    /// directory of the *network file*, with `--startup-inactive`, so that it starts no *node*
    /// until the *product* transmits the *activate request* of FR-534 (\[AD3\] IR-173), connects to
    /// `socket_path` once it listens, and transmits the one *watch request*, whose
    /// *state responses* `sink` receives.
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
        sink: StateSink,
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
            "--socket".to_owned(),
            socket_path.to_string_lossy().into_owned(),
            "--startup-inactive".to_owned(),
        ];
        let (process, stream) = ServedProcess::start(executable, &directory, &args, socket_path)?;
        let mut conversation = Conversation::new(WatchedConnection::open(stream, sink)?);
        conversation.watch()?;
        Ok(BuildSession {
            process,
            conversation,
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
        let answer = self.conversation.transmit(
            &self.document,
            provided_before,
            increment,
            provided_after,
        )?;
        Ok(BuildResult {
            diagnostics: answer.diagnostics,
            network: self.conversation.query_network(),
        })
    }

    // realises FR-109, FR-112, FR-211
    /// Transmits a *store request* to the *build system*, which passes it on to every *node* in
    /// the order of the *connections* (\[AD3\] IR-078), as [`Conversation::store`].
    ///
    /// # Errors
    /// Returns what [`Conversation::store`] returns.
    pub fn store(&mut self) -> Result<(), BuildError> {
        self.conversation.store()
    }

    // realises FR-260, FR-261
    /// Transmits an *activate request* to the *build system*, which starts, wires and activates
    /// every *node* of the network (\[AD3\] IR-157, IR-158), as [`Conversation::activate`].
    ///
    /// # Errors
    /// Returns what [`Conversation::activate`] returns.
    pub fn activate(&mut self) -> Result<(), BuildError> {
        self.conversation.activate()
    }

    // realises FR-267, FR-268
    /// Transmits a *deactivate request* to the *build system*, which carries out the
    /// *network deactivation* (\[AD3\] IR-164), as [`Conversation::deactivate`].
    ///
    /// # Errors
    /// Returns what [`Conversation::deactivate`] returns.
    pub fn deactivate(&mut self) -> Result<(), BuildError> {
        self.conversation.deactivate()
    }

    // realises FR-281, FR-286
    /// Transmits a *hold request* naming `node` to the *build system*, which starts and serves
    /// the *node* in a *valid network* and marks it in an *active network* (\[AD3\] IR-196 to
    /// IR-199), as [`Conversation::hold`].
    ///
    /// # Errors
    /// Returns what [`Conversation::hold`] returns.
    pub fn hold(&mut self, node: &str) -> Result<(), BuildError> {
        self.conversation.hold(node)
    }

    // realises FR-282, FR-287
    /// Transmits a *release request* naming `node` to the *build system*, which ends the *node*
    /// in a *valid network* where no other client holds it (\[AD3\] IR-201, IR-202), as
    /// [`Conversation::release`].
    ///
    /// # Errors
    /// Returns what [`Conversation::release`] returns.
    pub fn release(&mut self, node: &str) -> Result<(), BuildError> {
        self.conversation.release(node)
    }

    // realises FR-068
    /// Transmits a *shutdown request* and waits for the process to end; ends it where it does not.
    ///
    /// * The sink of the watch is told nothing of the end of the connection, which *product*
    ///   brings about itself.
    pub fn shut_down(mut self) {
        self.conversation.connection.silence();
        let _ = self.conversation.shut_down();
        self.process.await_end();
    }
}

// realises FR-104, FR-105, FR-106, FR-230, IR-026, IR-027, IR-038, C-005
/// A *node* *product* is attached to: the conversation on the *service socket* of a *node* the
/// *build system* runs, and the watch of that *node*.
///
/// * Dropping it closes the connection and leaves the *node* running (FR-105).
#[derive(Debug)]
pub struct NodeSession {
    /// the conversation on its *service socket*, which is watched
    conversation: Conversation<WatchedConnection>,
}

impl NodeSession {
    // realises FR-104, FR-230, IR-026, IR-038
    /// Attaches to the *node* that listens on `socket_path`: connects to it, retrying while the
    /// *build system* is starting the *node*, and transmits the *watch request*, whose
    /// *state responses* `sink` receives. No process is started.
    ///
    /// # Errors
    /// Returns [`BuildError::Unsupported`] where the platform offers no Unix domain socket, and
    /// [`BuildError::Connection`] where the *service socket* is not reached.
    pub fn attach(socket_path: &Path, sink: StateSink) -> Result<NodeSession, BuildError> {
        if !BuildSession::is_available() {
            return Err(BuildError::Unsupported);
        }
        let stream = connected(None, socket_path, ATTACH_ATTEMPTS)?;
        let mut conversation = Conversation::new(WatchedConnection::open(stream, sink)?);
        conversation.watch()?;
        Ok(NodeSession { conversation })
    }

    // realises FR-106, FR-107, FR-225, IR-027, IR-028
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
    ) -> Result<DiagnosticAnswer, BuildError> {
        self.conversation
            .transmit(document, provided_before, increment, provided_after)
    }

    // realises FR-167, IR-027, IR-028
    /// Transmits the octets of a document, as [`Conversation::open_octets`].
    ///
    /// # Errors
    /// Returns what [`Conversation::open_octets`] returns.
    pub fn open_octets(
        &mut self,
        document: &str,
        octets: &[u8],
    ) -> Result<DiagnosticAnswer, BuildError> {
        self.conversation.open_octets(document, octets)
    }

    // realises FR-210, IR-027, IR-028
    /// Transmits the change of a *binary document*, as [`Conversation::edit_octets`].
    ///
    /// # Errors
    /// Returns what [`Conversation::edit_octets`] returns.
    pub fn edit_octets(
        &mut self,
        document: &str,
        octets_before: &[u8],
        octets_after: &[u8],
    ) -> Result<DiagnosticAnswer, BuildError> {
        self.conversation
            .edit_octets(document, octets_before, octets_after)
    }

    // realises FR-110, FR-111, FR-239, FR-264
    /// Reads the content the *node* holds of a document, as [`Conversation::query_content`]:
    /// `None` where the *node* cancelled the query.
    ///
    /// # Errors
    /// Returns what [`Conversation::query_content`] returns.
    pub fn query_content(&mut self, document: &str) -> Result<Option<DocumentContent>, BuildError> {
        self.conversation.query_content(document)
    }
}

// realises FR-067, FR-094, IR-017
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

/// Yields `Ok(())` where `response` is the acknowledged response, [`BuildError::Refused`] with
/// its message text where it is an error response, and [`BuildError::Protocol`] for any other
/// *terminal response* to `request`.
fn acknowledged(response: Response, request: &str) -> Result<(), BuildError> {
    match response {
        Response::Acknowledged => Ok(()),
        Response::Error(text) => Err(BuildError::Refused(text)),
        Response::Diagnostics { .. }
        | Response::VersionMismatch { .. }
        | Response::Network { .. }
        | Response::State { .. }
        | Response::Content { .. }
        | Response::Cancelled
        | Response::Other(_) => Err(unexpected(request)),
    }
}

/// Yields the error of a *response* whose kind does not answer `request`.
fn unexpected(request: &str) -> BuildError {
    BuildError::Protocol(format!("{request} was answered by an unexpected response"))
}

/// Connects to the *service socket* at `socket_path`, trying `attempts` times until it listens;
/// where `child` is the process that is to listen there, until that process ends as well.
#[cfg(unix)]
fn connected(
    child: Option<&mut Child>,
    socket_path: &Path,
    attempts: u32,
) -> Result<ServiceStream, BuildError> {
    let mut child = child;
    let mut reason = String::from("the service socket does not exist");
    for _ in 0..attempts {
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
        if let Some(Ok(Some(status))) = child.as_mut().map(|child| child.try_wait()) {
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
fn connected(
    _child: Option<&mut Child>,
    _socket_path: &Path,
    _attempts: u32,
) -> Result<ServiceStream, BuildError> {
    let _ = (CONNECT_PAUSE, RESPONSE_TIMEOUT);
    Err(BuildError::Unsupported)
}

/*  * validated        : ✅
* completeness     : ✅
* independence     : ✅
* edge cases       : ✅
* conforms to doc  : ✅
* covers bridge    : NetworkEditor::apply_increment, NetworkEditor::set_network_path,
                     NetworkEditor::store, NetworkEditor::activate, NetworkEditor::deactivate,
                     NetworkEditor::report_arrived,
                     NetworkEditor::shut_down, NodeEditor::apply_increment, NodeEditor::report_arrived,
                     NodeEditor::close; the process, the sockets and the reader threads of
                     `BuildSession` and `NodeSession` are covered by the ignored integration test
                     'tests/build_system.rs' */
#[cfg(test)]
mod tests {
    use super::*;

    use crate::core::api_message::{
        NodeKind, NodeMode, OctetDelta, Position, ReadPosition, Severity, encode_response,
    };

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
            groups: vec![],
        }
    }

    /// The answer at the *document version* 0 that carries `diagnostics` and no
    /// *diagnostic group*.
    fn answer(diagnostics: Vec<Diagnostic>) -> DiagnosticAnswer {
        answer_at(0, diagnostics)
    }

    /// The answer at the *document version* `version` that carries `diagnostics` and no
    /// *diagnostic group*.
    fn answer_at(version: u64, diagnostics: Vec<Diagnostic>) -> DiagnosticAnswer {
        DiagnosticAnswer {
            version,
            erroneous: has_error(&diagnostics),
            diagnostics,
            groups: vec![],
        }
    }

    fn state(name: &str, errors: u64, origins: &[&str]) -> NodeState {
        NodeState {
            name: name.to_owned(),
            errors,
            origins: origins.iter().map(|origin| (*origin).to_owned()).collect(),
            mode: Some(NodeMode::Active),
        }
    }

    /// What the *node* `b` tells while it counts `errors` errors in 'b.in' at version 2.
    fn watched(errors: u64) -> Watched {
        Watched {
            states: vec![state("b", errors, &[])],
            groups: vec![group("b.in", 2, (0..errors).map(|_| fault()).collect())],
            versions: vec![("b.in".to_owned(), 2), ("y.txt".to_owned(), 5)],
            mode: None,
        }
    }

    fn state_response(watched: &Watched) -> Response {
        Response::State {
            states: watched.states.clone(),
            groups: watched.groups.clone(),
            versions: watched.versions.clone(),
            mode: watched.mode,
        }
    }

    /// The bytes of the *frames* of `responses`, as a process of the *build system* writes
    /// them.
    fn frames(responses: &[(u64, Response)]) -> Vec<u8> {
        responses
            .iter()
            .flat_map(|(identifier, response)| {
                frames_of_message(&encode_response(*identifier, response))
            })
            .collect()
    }

    /// What `read_responses` hands over while it reads `bytes`: to the conversation, and to
    /// the sink.
    #[allow(clippy::type_complexity)] // the two results of one reading, compared as one value
    fn read(
        bytes: Vec<u8>,
    ) -> (
        Vec<Result<(u64, Response), BuildError>>,
        Vec<Result<Watched, BuildError>>,
    ) {
        let (answers, received) = channel();
        let mut told = Vec::new();
        read_responses(&mut Cursor::new(bytes), &answers, &mut |watched| {
            told.push(watched);
        });
        drop(answers);
        (received.iter().collect(), told)
    }

    fn group(document: &str, version: u64, diagnostics: Vec<Diagnostic>) -> DiagnosticGroup {
        DiagnosticGroup {
            document: document.to_owned(),
            version,
            diagnostics,
        }
    }

    fn fault() -> Diagnostic {
        Diagnostic {
            severity: Severity::Error,
            start: Position {
                offset: 0,
                read_position: Some(ReadPosition { line: 1, column: 2 }),
            },
            end: Position {
                offset: 0,
                read_position: Some(ReadPosition { line: 1, column: 3 }),
            },
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
            (result, conversation.connection.requests()),
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
                conversation.connection.requests().get(2).cloned(),
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
                conversation.connection.requests().get(3).cloned()
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
            (result, conversation.connection.requests()),
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
            .connection
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
    fn octets_are_opened_as_a_binary_document_with_the_diagnostics_of_the_response() {
        // FR-167, FR-107
        let stream = Scripted::answering(&[(
            1,
            Response::Diagnostics {
                version: 4,
                diagnostics: vec![fault()],
                groups: vec![],
            },
        )]);
        let mut conversation = Conversation::new(stream);
        let result = conversation.open_octets("in.bin", &[0xff, 0x00]);
        assert_eq!(
            (
                result,
                conversation.connection.requests(),
                conversation.documents.get("in.bin").copied(),
                conversation.binary_documents.contains("in.bin")
            ),
            (
                Ok(answer_at(4, vec![fault()])),
                vec![encode_request(
                    1,
                    &Request::OpenOctets {
                        document: "in.bin".to_owned(),
                        octets: vec![0xff, 0x00],
                    }
                )],
                Some(4),
                true
            )
        );
    }

    #[test]
    fn change_of_a_binary_document_is_an_edit_request_with_its_octet_delta() {
        // FR-210
        let stream = Scripted::answering(&[(1, diagnostics(0)), (2, diagnostics(1))]);
        let mut conversation = Conversation::new(stream);
        let _ = conversation.open_octets("in.bin", &[1, 2, 3]);
        let result = conversation.edit_octets("in.bin", &[1, 2, 3], &[1, 9, 3]);
        assert_eq!(
            (
                result,
                conversation.connection.requests().get(1).cloned(),
                conversation.documents.get("in.bin").copied()
            ),
            (
                Ok(answer_at(1, vec![])),
                Some(encode_request(
                    2,
                    &Request::EditOctets {
                        document: "in.bin".to_owned(),
                        version: 0,
                        deltas: vec![OctetDelta {
                            start: 1,
                            end: 2,
                            replacement: vec![9],
                        }],
                    }
                )),
                Some(1)
            )
        );
    }

    #[test]
    fn change_of_octets_of_a_document_not_opened_as_binary_opens_it_with_its_octets() {
        // FR-210: not opened at all, and opened as a text document
        let stream = Scripted::answering(&[
            (1, diagnostics(0)),
            (2, diagnostics(0)),
            (3, diagnostics(1)),
        ]);
        let mut conversation = Conversation::new(stream);
        let _ = conversation.edit_octets("a.bin", &[1], &[2]);
        let _ = conversation.transmit("b.txt", "", &increment(0, 0, "x"), "x");
        let _ = conversation.edit_octets("b.txt", b"x", &[0xff]);
        let requests = conversation.connection.requests();
        assert_eq!(
            (requests.first().cloned(), requests.get(2).cloned()),
            (
                Some(encode_request(
                    1,
                    &Request::OpenOctets {
                        document: "a.bin".to_owned(),
                        octets: vec![2],
                    }
                )),
                Some(encode_request(
                    3,
                    &Request::OpenOctets {
                        document: "b.txt".to_owned(),
                        octets: vec![0xff],
                    }
                ))
            )
        );
    }

    #[test]
    fn version_mismatch_of_an_octet_edit_is_answered_by_an_open_request_with_the_octets() {
        // FR-210
        let stream = Scripted::answering(&[
            (1, diagnostics(0)),
            (2, Response::VersionMismatch { version: 7 }),
            (3, diagnostics(8)),
        ]);
        let mut conversation = Conversation::new(stream);
        let _ = conversation.open_octets("in.bin", &[1]);
        let result = conversation.edit_octets("in.bin", &[1], &[2]);
        assert_eq!(
            (
                result,
                conversation.connection.requests().get(2).cloned(),
                conversation.documents.get("in.bin").copied()
            ),
            (
                Ok(answer_at(8, vec![])),
                Some(encode_request(
                    3,
                    &Request::OpenOctets {
                        document: "in.bin".to_owned(),
                        octets: vec![2],
                    }
                )),
                Some(8)
            )
        );
    }

    #[test]
    fn error_response_to_an_octet_edit_makes_the_next_change_open_again() {
        // FR-210, FR-112
        let stream = Scripted::answering(&[
            (1, diagnostics(0)),
            (2, Response::Error("refused".to_owned())),
        ]);
        let mut conversation = Conversation::new(stream);
        let _ = conversation.open_octets("in.bin", &[1]);
        let result = conversation.edit_octets("in.bin", &[1], &[2]);
        assert_eq!(
            (
                result,
                conversation.documents.contains_key("in.bin"),
                conversation.binary_documents.contains("in.bin")
            ),
            (Err(BuildError::Refused("refused".to_owned())), false, false)
        );
    }

    #[test]
    fn text_change_of_a_binary_document_opens_it_again_as_a_text_document() {
        // FR-106: a binary document takes no edit delta
        let stream = Scripted::answering(&[
            (1, diagnostics(0)),
            (2, diagnostics(1)),
            (3, diagnostics(2)),
        ]);
        let mut conversation = Conversation::new(stream);
        let _ = conversation.open_octets("in.bin", &[0xff]);
        let _ = conversation.transmit("in.bin", "", &increment(0, 0, "x"), "x");
        let _ = conversation.transmit("in.bin", "x", &increment(1, 0, "y"), "xy");
        let requests = conversation.connection.requests();
        assert_eq!(
            (
                requests.get(1).cloned(),
                requests.get(2).map(|request| request[2]),
                conversation.binary_documents.contains("in.bin")
            ),
            (
                Some(encode_request(
                    2,
                    &Request::Open {
                        document: "in.bin".to_owned(),
                        text: "x".to_owned(),
                    }
                )),
                Some(2),
                false
            )
        );
    }

    #[test]
    fn error_response_to_the_octets_is_the_failure_with_its_message_text() {
        // FR-112
        let stream = Scripted::answering(&[(1, Response::Error("refused".to_owned()))]);
        let mut conversation = Conversation::new(stream);
        let result = conversation.open_octets("in.bin", &[0xff]);
        assert_eq!(
            (result, conversation.binary_documents.contains("in.bin")),
            (Err(BuildError::Refused("refused".to_owned())), false)
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
                groups: vec![],
            },
        )]);
        let mut conversation = Conversation::new(stream);
        assert_eq!(
            conversation.transmit("a.gc3", "", &increment(0, 0, "x"), "x"),
            Ok(answer(vec![fault()]))
        );
    }

    #[test]
    fn diagnostic_group_of_another_document_at_the_version_held_is_taken() {
        // FR-225: the edit of 'a.in' brings the diagnostics of 'b.in', and none of 'c.in'
        let stream = Scripted::answering(&[
            (1, diagnostics(0)),
            (2, diagnostics(3)),
            (3, diagnostics(0)),
            (
                4,
                Response::Diagnostics {
                    version: 1,
                    diagnostics: vec![],
                    groups: vec![group("b.in", 3, vec![fault()]), group("c.in", 0, vec![])],
                },
            ),
        ]);
        let mut conversation = Conversation::new(stream);
        let opened = conversation
            .transmit("a.in", "", &increment(0, 0, "a"), "a")
            .and(conversation.transmit("b.in", "", &increment(0, 0, "b"), "b"))
            .and(conversation.transmit("c.in", "", &increment(0, 0, "c"), "c"));
        let edited = conversation.transmit("a.in", "a", &increment(1, 0, "x"), "ax");
        assert_eq!(
            (opened, edited, conversation.documents.get("b.in").copied()),
            (
                Ok(answer(vec![])),
                Ok(DiagnosticAnswer {
                    version: 1,
                    diagnostics: vec![],
                    groups: vec![group("b.in", 3, vec![fault()]), group("c.in", 0, vec![])],
                    erroneous: true,
                }),
                Some(3)
            )
        );
    }

    #[test]
    fn diagnostic_group_at_another_version_than_the_one_held_is_left_out() {
        // FR-227: 'b.in' is held at version 3; the error of the group counts all the same
        let stream = Scripted::answering(&[
            (1, diagnostics(3)),
            (
                2,
                Response::Diagnostics {
                    version: 0,
                    diagnostics: vec![],
                    groups: vec![group("b.in", 2, vec![fault()])],
                },
            ),
        ]);
        let mut conversation = Conversation::new(stream);
        let opened = conversation.transmit("b.in", "", &increment(0, 0, "b"), "b");
        let answered = conversation.transmit("a.in", "", &increment(0, 0, "a"), "a");
        assert_eq!(
            (opened, answered),
            (
                Ok(answer_at(3, vec![])),
                Ok(DiagnosticAnswer {
                    version: 0,
                    diagnostics: vec![],
                    groups: vec![],
                    erroneous: true,
                })
            )
        );
    }

    #[test]
    fn diagnostic_group_of_a_document_not_opened_or_of_the_changed_one_is_left_out() {
        // FR-226, FR-227: no version of 'b.in' is held; 'a.in' is the document of the request
        let stream = Scripted::answering(&[(
            1,
            Response::Diagnostics {
                version: 0,
                diagnostics: vec![],
                groups: vec![group("b.in", 0, vec![]), group("a.in", 0, vec![fault()])],
            },
        )]);
        let mut conversation = Conversation::new(stream);
        assert_eq!(
            conversation.open_octets("a.in", &[0xff]),
            Ok(DiagnosticAnswer {
                version: 0,
                diagnostics: vec![],
                groups: vec![],
                erroneous: true,
            })
        );
    }

    #[test]
    fn diagnostic_group_of_the_response_to_octets_is_taken() {
        // FR-225, FR-210: the open request and the edit request of a binary document
        let stream = Scripted::answering(&[
            (1, diagnostics(5)),
            (
                2,
                Response::Diagnostics {
                    version: 0,
                    diagnostics: vec![],
                    groups: vec![group("b.in", 5, vec![])],
                },
            ),
            (
                3,
                Response::Diagnostics {
                    version: 1,
                    diagnostics: vec![fault()],
                    groups: vec![group("b.in", 5, vec![fault()])],
                },
            ),
        ]);
        let mut conversation = Conversation::new(stream);
        let opened = conversation.transmit("b.in", "", &increment(0, 0, "b"), "b");
        let octets = conversation.open_octets("a.bin", &[0xff]);
        let edited = conversation.edit_octets("a.bin", &[0xff], &[0xfe]);
        assert_eq!(
            (opened, octets, edited),
            (
                Ok(answer_at(5, vec![])),
                Ok(DiagnosticAnswer {
                    version: 0,
                    diagnostics: vec![],
                    groups: vec![group("b.in", 5, vec![])],
                    erroneous: false,
                }),
                Ok(DiagnosticAnswer {
                    version: 1,
                    diagnostics: vec![fault()],
                    groups: vec![group("b.in", 5, vec![fault()])],
                    erroneous: true,
                })
            )
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
            (Ok(answer(vec![])), Some(0))
        );
    }

    #[test]
    fn store_transmits_the_store_request_and_is_acknowledged() {
        // FR-109, FR-111, IR-027
        let mut conversation =
            Conversation::new(Scripted::answering(&[(1, Response::Acknowledged)]));
        let result = conversation.store();
        assert_eq!(
            (result, conversation.connection.requests()),
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
    fn activate_transmits_the_activate_request_and_is_acknowledged() {
        // FR-260, [AD3] IR-138, IR-157, IR-159
        let mut conversation =
            Conversation::new(Scripted::answering(&[(1, Response::Acknowledged)]));
        let result = conversation.activate();
        assert_eq!(
            (result, conversation.connection.requests()),
            (Ok(()), vec![encode_request(1, &Request::Activate)])
        );
    }

    #[test]
    fn deactivate_transmits_the_deactivate_request_and_is_acknowledged() {
        // FR-267, [AD3] IR-139, IR-164, IR-165
        let mut conversation =
            Conversation::new(Scripted::answering(&[(1, Response::Acknowledged)]));
        let result = conversation.deactivate();
        assert_eq!(
            (result, conversation.connection.requests()),
            (Ok(()), vec![encode_request(1, &Request::Deactivate)])
        );
    }

    #[test]
    fn error_response_to_the_deactivate_request_is_the_failure_with_its_message_text() {
        // FR-268
        let mut conversation = Conversation::new(Scripted::answering(&[(
            1,
            Response::Error("the network could not be deactivated".to_owned()),
        )]));
        assert_eq!(
            conversation.deactivate(),
            Err(BuildError::Refused(
                "the network could not be deactivated".to_owned()
            ))
        );
    }

    #[test]
    fn hold_and_release_requests_are_acknowledged_refused_or_a_protocol_error() {
        // FR-281, FR-282, FR-286, FR-287, [AD3] IR-198 to IR-202
        let mut acknowledged = Conversation::new(Scripted::answering(&[
            (1, Response::Acknowledged),
            (2, Response::Acknowledged),
        ]));
        let held = (acknowledged.hold("a"), acknowledged.release("a"));
        let mut refused = Conversation::new(Scripted::answering(&[
            (
                1,
                Response::Error("n.gc3n: the network is invalid".to_owned()),
            ),
            (
                2,
                Response::Error("node z: the network does not state it".to_owned()),
            ),
        ]));
        let refusals = (refused.hold("a"), refused.release("z"));
        let mut unexpected =
            Conversation::new(Scripted::answering(&[(1, network(0)), (2, network(0))]));
        let protocol = (
            matches!(unexpected.hold("a"), Err(BuildError::Protocol(_))),
            matches!(unexpected.release("a"), Err(BuildError::Protocol(_))),
        );
        assert_eq!(
            (held, refusals, protocol),
            (
                (Ok(()), Ok(())),
                (
                    Err(BuildError::Refused(
                        "n.gc3n: the network is invalid".to_owned()
                    )),
                    Err(BuildError::Refused(
                        "node z: the network does not state it".to_owned()
                    ))
                ),
                (true, true)
            )
        );
    }

    #[test]
    fn unexpected_response_to_the_deactivate_request_is_a_protocol_error() {
        // FR-268
        let mut conversation = Conversation::new(Scripted::answering(&[(1, network(0))]));
        assert!(matches!(
            conversation.deactivate(),
            Err(BuildError::Protocol(_))
        ));
    }

    #[test]
    fn error_response_to_the_activate_request_is_the_failure_with_its_message_text() {
        // FR-261, [AD3] IR-161, IR-162
        let mut conversation = Conversation::new(Scripted::answering(&[(
            1,
            Response::Error("node b: the configuration is invalid".to_owned()),
        )]));
        assert_eq!(
            conversation.activate(),
            Err(BuildError::Refused(
                "node b: the configuration is invalid".to_owned()
            ))
        );
    }

    #[test]
    fn unexpected_response_to_the_activate_request_is_a_protocol_error() {
        let mut conversation = Conversation::new(Scripted::answering(&[(1, network(0))]));
        assert!(matches!(
            conversation.activate(),
            Err(BuildError::Protocol(_))
        ));
    }

    #[test]
    fn answer_carries_the_document_version_the_node_answered() {
        // FR-231, FR-232: the version the product holds of the document from then on
        let stream = Scripted::answering(&[(1, diagnostics(0)), (2, diagnostics(6))]);
        let mut conversation = Conversation::new(stream);
        let opened = conversation.transmit("a.in", "", &increment(0, 0, "a"), "a");
        let edited = conversation.transmit("a.in", "a", &increment(1, 0, "b"), "ab");
        assert_eq!(
            (opened, edited),
            (Ok(answer_at(0, vec![])), Ok(answer_at(6, vec![])))
        );
    }

    #[test]
    fn empty_document_is_registered_by_an_open_request_with_an_empty_text() {
        // FR-242: an increment that changes nothing opens a document not opened yet, so that
        // its document version is held and its diagnostics are answered
        let stream = Scripted::answering(&[(
            1,
            Response::Diagnostics {
                version: 0,
                diagnostics: vec![fault()],
                groups: vec![],
            },
        )]);
        let mut conversation = Conversation::new(stream);
        let registered = conversation.transmit("a.in", "", &TextIncrement::default(), "");
        assert_eq!(
            (
                registered,
                conversation.connection.requests(),
                conversation.documents.get("a.in").copied()
            ),
            (
                Ok(answer(vec![fault()])),
                vec![encode_request(
                    1,
                    &Request::Open {
                        document: "a.in".to_owned(),
                        text: String::new(),
                    }
                )],
                Some(0)
            )
        );
    }

    #[test]
    fn watch_transmits_the_watch_request_and_awaits_no_response() {
        // FR-228, FR-230: the stream holds no response
        let mut conversation = Conversation::new(Scripted::answering(&[]));
        let result = conversation.watch();
        assert_eq!(
            (result, conversation.connection.requests()),
            (Ok(()), vec![encode_request(1, &Request::Watch)])
        );
    }

    #[test]
    fn state_response_is_passed_over_by_the_request_that_awaits_its_terminal_response() {
        // [AD3] rule 'Watch': the watch is request 1, the store request 2; a state response
        // is no terminal response, whatever request identifier it carries
        let stream = Scripted::answering(&[
            (1, state_response(&watched(1))),
            (2, state_response(&watched(0))),
            (2, Response::Acknowledged),
        ]);
        let mut conversation = Conversation::new(stream);
        let watching = conversation.watch();
        assert_eq!((watching, conversation.store()), (Ok(()), Ok(())));
    }

    #[test]
    fn content_query_yields_the_content_and_its_document_version() {
        // FR-110, FR-111, FR-239
        let stream = Scripted::answering(&[
            (
                1,
                Response::Content {
                    version: 5,
                    octets: b"generated".to_vec(),
                    binary: false,
                },
            ),
            (
                2,
                Response::Content {
                    version: 0,
                    octets: vec![0xff],
                    binary: true,
                },
            ),
        ]);
        let mut conversation = Conversation::new(stream);
        let text = conversation.query_content("y.txt");
        let octets = conversation.query_content("x.bin");
        assert_eq!(
            (text, octets, conversation.connection.requests()),
            (
                Ok(Some(DocumentContent {
                    version: 5,
                    octets: b"generated".to_vec(),
                    binary: false,
                })),
                Ok(Some(DocumentContent {
                    version: 0,
                    octets: vec![0xff],
                    binary: true,
                })),
                vec![
                    encode_request(
                        1,
                        &Request::QueryContent {
                            document: "y.txt".to_owned(),
                        }
                    ),
                    encode_request(
                        2,
                        &Request::QueryContent {
                            document: "x.bin".to_owned(),
                        }
                    )
                ]
            )
        );
    }

    #[test]
    fn content_query_of_a_document_the_node_does_not_hold_is_the_failure_with_its_text() {
        // FR-112, [AD3] IR-135
        let stream = Scripted::answering(&[(1, Response::Error("no such document".to_owned()))]);
        let mut conversation = Conversation::new(stream);
        assert_eq!(
            conversation.query_content("z.txt"),
            Err(BuildError::Refused("no such document".to_owned()))
        );
    }

    #[test]
    fn cancelled_response_to_the_content_query_yields_no_content() {
        // FR-264, [AD3] IR-029: an edit request of the producer superseded the query
        let stream = Scripted::answering(&[(1, Response::Cancelled)]);
        let mut conversation = Conversation::new(stream);
        assert_eq!(conversation.query_content("x.txt"), Ok(None));
    }

    #[test]
    fn unexpected_response_to_the_content_query_is_a_protocol_error() {
        let stream = Scripted::answering(&[(1, Response::Acknowledged)]);
        let mut conversation = Conversation::new(stream);
        assert!(matches!(
            conversation.query_content("y.txt"),
            Err(BuildError::Protocol(_))
        ));
    }

    #[test]
    fn reader_hands_state_responses_to_the_sink_and_the_others_to_the_conversation() {
        // FR-229, FR-231: a state response before, between and after the answers, unasked
        let (answers, told) = read(frames(&[
            (1, state_response(&watched(0))),
            (2, diagnostics(3)),
            (1, state_response(&watched(1))),
            (3, Response::Acknowledged),
            (1, state_response(&watched(0))),
        ]));
        assert_eq!(
            (
                answers.iter().take(2).cloned().collect::<Vec<_>>(),
                answers.len(),
                told.iter().take(3).cloned().collect::<Vec<_>>(),
                told.len()
            ),
            (
                vec![Ok((2, diagnostics(3))), Ok((3, Response::Acknowledged))],
                3,
                vec![Ok(watched(0)), Ok(watched(1)), Ok(watched(0))],
                4
            )
        );
    }

    #[test]
    fn reader_hands_the_network_mode_of_a_state_response_to_the_sink() {
        // FR-229: the build system tells its network mode under key 21
        let of_network = Watched {
            states: vec![state("a", 0, &[]), state("b", 0, &[])],
            groups: vec![],
            versions: vec![],
            mode: Some(NetworkMode::Active),
        };
        let (_, told) = read(frames(&[(1, state_response(&of_network))]));
        assert_eq!(told.first().cloned(), Some(Ok(of_network)));
    }

    #[test]
    fn reader_tells_the_end_of_the_stream_to_the_conversation_and_to_the_sink() {
        // FR-112: the stream ends
        let (answers, told) = read(frames(&[(1, state_response(&watched(0)))]));
        assert!(
            matches!(answers.as_slice(), [Err(BuildError::Connection(_))])
                && matches!(told.as_slice(), [Ok(_), Err(BuildError::Connection(_))])
        );
    }

    #[test]
    fn reader_tells_a_message_that_is_not_decoded_and_reads_no_further() {
        // a frame of one octet that is no map, then a state response that is not read
        let mut bytes = frames_of_message(&[0x01]);
        bytes.extend(frames(&[(1, state_response(&watched(0)))]));
        let (answers, told) = read(bytes);
        assert!(
            matches!(answers.as_slice(), [Err(BuildError::Protocol(_))])
                && matches!(told.as_slice(), [Err(BuildError::Protocol(_))])
        );
    }

    #[test]
    fn reader_ends_where_the_conversation_is_gone() {
        // the answers are taken by nobody: the state response behind the answer is not told
        let (answers, received) = channel();
        drop(received);
        let mut told = Vec::new();
        let bytes = frames(&[
            (2, Response::Acknowledged),
            (1, state_response(&watched(0))),
        ]);
        read_responses(&mut Cursor::new(bytes), &answers, &mut |watched| {
            told.push(watched);
        });
        assert_eq!(told, vec![]);
    }

    /// A *service socket* in the process: the end *product* connects to, and the end a
    /// process of the *build system* would hold.
    #[cfg(unix)]
    fn socket_pair() -> (ServiceStream, ServiceStream) {
        ServiceStream::pair().expect("a pair of connected sockets can be created")
    }

    #[cfg(unix)]
    #[test]
    fn watched_connection_receives_a_state_response_unasked_while_a_request_is_made() {
        // FR-229, FR-231: the peer tells a state before and after it answers the store request
        let (stream, mut peer) = socket_pair();
        let (states, told) = channel();
        let sink: StateSink = Box::new(move |watched| {
            let _ = states.send(watched);
        });
        let mut conversation = Conversation::new(
            WatchedConnection::open(stream, sink).expect("the socket can be read by a thread"),
        );
        let watching = conversation.watch();
        let served = std::thread::spawn(move || {
            let watch = read_message(&mut peer).expect("the watch request arrives");
            let _ = peer.write_all(&frames(&[(1, state_response(&watched(1)))]));
            let store = read_message(&mut peer).expect("the store request arrives");
            let _ = peer.write_all(&frames(&[
                (1, state_response(&watched(0))),
                (2, Response::Acknowledged),
                (1, state_response(&watched(2))),
            ]));
            (watch, store, peer)
        });
        let first = told.recv_timeout(RESPONSE_TIMEOUT);
        let stored = conversation.store();
        let second = told.recv_timeout(RESPONSE_TIMEOUT);
        let third = told.recv_timeout(RESPONSE_TIMEOUT);
        let (watch, store, _peer) = served.join().expect("the peer ends");
        assert_eq!(
            (watching, first, stored, second, third, watch, store),
            (
                Ok(()),
                Ok(Ok(watched(1))),
                Ok(()),
                Ok(Ok(watched(0))),
                Ok(Ok(watched(2))),
                encode_request(1, &Request::Watch),
                encode_request(2, &Request::Store)
            )
        );
    }

    #[cfg(unix)]
    #[test]
    fn connection_the_peer_closes_is_told_to_the_sink_and_fails_the_next_request() {
        // FR-112: the node ended
        let (stream, peer) = socket_pair();
        let (states, told) = channel();
        let sink: StateSink = Box::new(move |watched| {
            let _ = states.send(watched);
        });
        let mut conversation = Conversation::new(
            WatchedConnection::open(stream, sink).expect("the socket can be read by a thread"),
        );
        drop(peer);
        let lost = told.recv_timeout(RESPONSE_TIMEOUT);
        assert!(
            matches!(lost, Ok(Err(BuildError::Connection(_))))
                && matches!(conversation.store(), Err(BuildError::Connection(_)))
        );
    }

    #[cfg(unix)]
    #[test]
    fn connection_the_product_closes_tells_the_sink_nothing_and_ends_for_the_peer() {
        // FR-105: closing the connection ends the watch, and is no loss to report
        let (stream, mut peer) = socket_pair();
        let (states, told) = channel();
        let sink: StateSink = Box::new(move |watched| {
            let _ = states.send(watched);
        });
        let connection =
            WatchedConnection::open(stream, sink).expect("the socket can be read by a thread");
        drop(connection);
        let mut rest = Vec::new();
        let ended = peer.read_to_end(&mut rest);
        // the sink is dropped with the reader thread, having been told nothing
        assert_eq!(
            (ended.ok(), told.recv_timeout(RESPONSE_TIMEOUT).ok()),
            (Some(0), None)
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
            Box::new(|_| {}),
        );
        let expected = if BuildSession::is_available() {
            BuildError::NotExecutable("/nonexistent/genc3d".to_owned())
        } else {
            BuildError::Unsupported
        };
        assert_eq!(result.err(), Some(expected));
    }

    #[cfg(unix)]
    #[test]
    fn node_is_attached_to_by_its_service_socket_watched_and_left_running() {
        // FR-104, FR-105, FR-230, IR-026,
        // IR-038: a listener stands for the node the build system runs
        let directory = std::env::temp_dir().join(format!("genc3wb-attach-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&directory);
        std::fs::create_dir_all(&directory).expect("the directory of the case can be created");
        let socket_path = directory.join("b.gc3.sock");
        let listener = std::os::unix::net::UnixListener::bind(&socket_path)
            .expect("the socket of the case can be bound");
        let node = std::thread::spawn(move || {
            let (mut peer, _) = listener.accept().expect("the product connects");
            let watch = read_message(&mut peer).expect("the watch request arrives");
            let _ = peer.write_all(&frames(&[(1, state_response(&watched(1)))]));
            let query = read_message(&mut peer).expect("the content query request arrives");
            let _ = peer.write_all(&frames(&[(
                2,
                Response::Content {
                    version: 5,
                    octets: b"out".to_vec(),
                    binary: false,
                },
            )]));
            let mut rest = Vec::new();
            let ended = peer.read_to_end(&mut rest);
            (watch, query, ended.ok(), rest)
        });
        let (states, told) = channel();
        let attached = NodeSession::attach(
            &socket_path,
            Box::new(move |watched| {
                let _ = states.send(watched);
            }),
        );
        let first = told.recv_timeout(RESPONSE_TIMEOUT);
        let content = attached
            .map_err(|error| error.to_string())
            .and_then(|mut session| {
                session
                    .query_content("y.txt")
                    .map_err(|error| error.to_string())
            });
        let (watch, query, ended, rest) = node.join().expect("the node of the case ends");
        let _ = std::fs::remove_dir_all(&directory);
        assert_eq!(
            (first, content, watch, query, ended, rest),
            (
                Ok(Ok(watched(1))),
                Ok(Some(DocumentContent {
                    version: 5,
                    octets: b"out".to_vec(),
                    binary: false,
                })),
                encode_request(1, &Request::Watch),
                encode_request(
                    2,
                    &Request::QueryContent {
                        document: "y.txt".to_owned(),
                    }
                ),
                Some(0),
                vec![]
            )
        );
    }

    #[test]
    fn node_whose_service_socket_is_not_reached_is_not_attached_to() {
        // FR-104, FR-112
        let result = NodeSession::attach(
            Path::new("/nonexistent/genc3wb/a.gc3.sock"),
            Box::new(|_| {}),
        );
        assert!(matches!(
            result,
            Err(BuildError::Connection(_) | BuildError::Unsupported)
        ));
    }
}
