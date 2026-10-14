//! The *messages* and the *frames* of *genc³api* that *product* exchanges with the *build system*.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::text_increment::TextIncrement;

use ciborium::value::{Integer, Value};

use std::fmt;
use std::io::Read;

/// The largest payload of one *frame*, 16 MiB (\[AD3\] IR-008, \[AD3\] IR-091).
pub const MAX_FRAME_PAYLOAD: usize = 16 * 1024 * 1024;

/// The keys of a *message* map, as the message schema numbers them.
const KEY_KIND: u64 = 0;
const KEY_REQUEST: u64 = 1;
const KEY_DOCUMENT: u64 = 2;
const KEY_VERSION: u64 = 3;
const KEY_TEXT: u64 = 4;
const KEY_DELTAS: u64 = 5;
const KEY_TERMINAL: u64 = 10;
const KEY_DIAGNOSTICS: u64 = 11;
const KEY_ERROR: u64 = 13;
const KEY_NODES: u64 = 14;
const KEY_OCTETS: u64 = 15;
const KEY_OCTET_DELTAS: u64 = 16;
const KEY_GROUPS: u64 = 17;
const KEY_STATES: u64 = 18;
const KEY_VERSIONS: u64 = 20;
const KEY_MODE: u64 = 21;
const KEY_NODE_NAME: u64 = 23;

/// The kinds of the *messages* *product* encodes and decodes.
const KIND_OPEN: u64 = 1;
const KIND_EDIT: u64 = 2;
const KIND_STORE: u64 = 9;
const KIND_SHUTDOWN: u64 = 10;
const KIND_QUERY_NETWORK: u64 = 11;
const KIND_WATCH: u64 = 12;
const KIND_QUERY_CONTENT: u64 = 14;
const KIND_ACTIVATE: u64 = 15;
const KIND_DEACTIVATE: u64 = 16;
const KIND_HOLD: u64 = 18;
const KIND_RELEASE: u64 = 19;
const KIND_ACKNOWLEDGED: u64 = 32;
const KIND_DIAGNOSTICS: u64 = 33;
const KIND_ROOT: u64 = 34;
const KIND_CHILDREN: u64 = 35;
const KIND_CANCELLED: u64 = 36;
const KIND_VERSION_MISMATCH: u64 = 37;
const KIND_ERROR: u64 = 38;
const KIND_NETWORK: u64 = 39;
const KIND_STATE: u64 = 40;
const KIND_CONTENT: u64 = 41;

// realises FR-071
/// A *read position*: a line and a column, each counted from 1, the column in Unicode code points.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ReadPosition {
    /// the line, counted from 1; lines are separated by U+000A
    pub line: u64,
    /// the column, counted from 1 in Unicode code points
    pub column: u64,
}

// realises FR-071
/// An *edit delta*: the range of the text before it that `text` replaces.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EditDelta {
    /// the first replaced position, inclusive
    pub start: ReadPosition,
    /// the position after the last replaced one
    pub end: ReadPosition,
    /// the replacing text
    pub text: String,
}

// realises FR-076
/// The kind of a *node description*.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeKind {
    /// a *meta compiler-compiler*
    MetaCompilerCompiler,
    /// a *proxy node*
    ProxyNode,
}

// realises FR-075, IR-020
/// A *node description* of a *network response*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeDescription {
    /// the name of the *node*
    pub name: String,
    /// its kind
    pub kind: NodeKind,
    /// the path of its *meta compiler DSL*, or the tool chain of a *proxy node*
    pub transformation: String,
    /// the path of its *service socket*
    pub socket: String,
    /// its *input* paths, in the order of the *network file*
    pub inputs: Vec<String>,
    /// its *output* paths, in the order of the *network file*
    pub outputs: Vec<String>,
}

// realises FR-107, FR-246, IR-028
/// The severity of a *diagnostic* (\[AD5\] IR-023).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    /// the document is faulty
    Error,
    /// the document is doubtful
    Warning,
    /// a finding without a fault
    Information,
}

impl Severity {
    /// Yields the severity as the message schema numbers it.
    fn number(self) -> u64 {
        match self {
            Severity::Error => 1,
            Severity::Warning => 2,
            Severity::Information => 3,
        }
    }

    /// Yields the severity of a number of the message schema, `None` where the number names none.
    fn of_number(number: u64) -> Option<Severity> {
        match number {
            1 => Some(Severity::Error),
            2 => Some(Severity::Warning),
            3 => Some(Severity::Information),
            _ => None,
        }
    }

    // realises FR-126
    /// Yields the index of the severity, in the order error, warning, information, by which the
    /// *front end* colours a mark.
    pub fn index(self) -> usize {
        match self {
            Severity::Error => 0,
            Severity::Warning => 1,
            Severity::Information => 2,
        }
    }

    /// Yields the severity as its rendering names it.
    fn name(self) -> &'static str {
        match self {
            Severity::Error => "error",
            Severity::Warning => "warning",
            Severity::Information => "information",
        }
    }
}

// realises FR-107, FR-153, FR-246, FR-251, IR-028
/// A *position* of a *diagnostic*: its *offset*, the number of bits of the document preceding
/// it, and its *read position* where it has one (\[AD5\] IR-117).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    /// the *offset*, counted from 0 in bits
    pub offset: u64,
    /// the *read position*, where the *position* has one
    pub read_position: Option<ReadPosition>,
}

impl Position {
    // realises FR-153
    /// Yields the *position* as text: `<line>:<column>` where it has a *read position*, and
    /// `@<offset>` where it has none.
    pub fn rendering(&self) -> String {
        match self.read_position {
            Some(position) => format!("{}:{}", position.line, position.column),
            None => format!("@{}", self.offset),
        }
    }
}

// realises FR-107, FR-109, FR-246, IR-028, IR-029
/// A *diagnostic* of a *diagnostic* response: its severity, its range of two *positions* and
/// its message text (\[AD5\] IR-023).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// its severity
    pub severity: Severity,
    /// the first position of the range, inclusive
    pub start: Position,
    /// the position after the last one of the range
    pub end: Position,
    /// its message text
    pub text: String,
}

impl Diagnostic {
    // realises FR-107, FR-153, IR-029
    /// Yields the *diagnostic* as text: the severity, a space, the range as its two *positions*
    /// around a hyphen, each `<line>:<column>` or `@<offset>`, a colon, a space, and the
    /// message text.
    pub fn rendering(&self) -> String {
        format!(
            "{} {}-{}: {}",
            self.severity.name(),
            self.start.rendering(),
            self.end.rendering(),
            self.text
        )
    }

    // realises FR-153, FR-251
    /// Yields the *read positions* of the range, where both of its *positions* have one.
    pub fn read_positions(&self) -> Option<(ReadPosition, ReadPosition)> {
        Some((self.start.read_position?, self.end.read_position?))
    }
}

// realises FR-210
/// An *octet delta*: the span of the octets before it that `replacement` replaces
/// (\[AD3\] IR-113).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OctetDelta {
    /// the number of octets preceding the span
    pub start: u64,
    /// the number of octets preceding the end of the span
    pub end: u64,
    /// the octets replacing the span
    pub replacement: Vec<u8>,
}

// realises IR-019, IR-027
/// A *request* *product* transmits to the *build system* or to the *node*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    /// an open request carrying the full text of the document
    Open { document: String, text: String },
    /// an open request carrying the octets of the document, which is a *binary document*
    /// (\[AD3\] IR-112)
    OpenOctets { document: String, octets: Vec<u8> },
    /// an edit request carrying the *document version* it applies to and its *edit deltas*
    Edit {
        document: String,
        version: u64,
        deltas: Vec<EditDelta>,
    },
    /// an edit request of a *binary document*, carrying the *document version* it applies to
    /// and its *octet deltas* (\[AD3\] IR-114)
    EditOctets {
        document: String,
        version: u64,
        deltas: Vec<OctetDelta>,
    },
    /// a *store request*, transmitted to the *build system* alone, which passes it on to every
    /// *node* (\[AD3\] IR-078)
    Store,
    /// a *network query request*, transmitted to the *build system* alone
    QueryNetwork,
    /// a *watch request* (\[AD3\] IR-120)
    Watch,
    /// a content query request naming the document whose content is asked for (\[AD3\] IR-134)
    QueryContent { document: String },
    /// an *activate request*, transmitted to the *build system* alone, which starts, wires and
    /// activates every *node* of the network (\[AD3\] IR-138, IR-157)
    Activate,
    /// a *deactivate request*, transmitted to the *build system* alone, which carries out the
    /// *network deactivation* (\[AD3\] IR-139, IR-164)
    Deactivate,
    /// a *hold request*, transmitted to the *build system* alone, naming the *node* it holds
    /// under key 23, so that the *node* runs and is served while the network is a
    /// *valid network* (\[AD3\] IR-194, IR-196 to IR-200)
    Hold { node: String },
    /// a *release request*, transmitted to the *build system* alone, naming the *node* it
    /// releases under key 23 (\[AD3\] IR-195, IR-201, IR-202)
    Release { node: String },
    /// a *shutdown request*
    Shutdown,
}

// realises FR-225, FR-226, FR-227, FR-231, FR-232, FR-233, FR-248, FR-249, FR-250, IR-028
/// A *diagnostic group* of a *diagnostic* response (\[AD3\] IR-118): the *diagnostics* of a
/// document of the *node* other than the one the *request* names; and of a *state response*
/// (\[AD3\] IR-121): the *diagnostics* of an *input* or of the *meta compiler DSL* of the *node*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticGroup {
    /// the *document identifier* of the document the *diagnostics* lie in
    pub document: String,
    /// the *document version* of that document the *diagnostics* were computed from
    pub version: u64,
    /// every *diagnostic* of that document, in the order of the group
    pub diagnostics: Vec<Diagnostic>,
}

// realises FR-123, FR-229, IR-020, IR-028
/// The *node mode* a *node* tells in its *node state* (\[AD3\] IR-123, IR-140).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NodeMode {
    /// an *invalid node*: its *configuration* is invalid, and it refuses to activate
    Invalid,
    /// a *valid node*: its *configuration* is valid, and it is not active
    Valid,
    /// an *active node*: it transforms its *inputs* into its *outputs*
    Active,
    /// a *waiting node*: its *meta compiler DSL* is produced by another *node* and has not
    /// arrived yet (\[AD3\] IR-191)
    Waiting,
}

impl NodeMode {
    // realises FR-123
    /// Yields the *node mode* as the message schema numbers it: 1 to 4.
    pub fn number(self) -> u64 {
        match self {
            NodeMode::Invalid => 1,
            NodeMode::Valid => 2,
            NodeMode::Active => 3,
            NodeMode::Waiting => 4,
        }
    }

    /// Yields the *node mode* of a number of the message schema, `None` where the number names
    /// none.
    fn of_number(number: u64) -> Option<NodeMode> {
        match number {
            1 => Some(NodeMode::Invalid),
            2 => Some(NodeMode::Valid),
            3 => Some(NodeMode::Active),
            4 => Some(NodeMode::Waiting),
            _ => None,
        }
    }
}

// realises FR-229, FR-258, FR-259, IR-020
/// The *network mode* the *build system* tells in its *state response* (\[AD3\] IR-129, IR-152).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkMode {
    /// an *invalid network*: the *network file* is invalid, and no *node* runs
    Invalid,
    /// a *valid network*: the *network file* is valid, no *node* runs, and an *activate request*
    /// activates the network
    Valid,
    /// an *active network*: every *node* runs as an *active node*
    Active,
}

impl NetworkMode {
    // realises FR-258
    /// Yields the *network mode* as the message schema numbers it: 1, 2 and 3.
    pub fn number(self) -> u64 {
        match self {
            NetworkMode::Invalid => 1,
            NetworkMode::Valid => 2,
            NetworkMode::Active => 3,
        }
    }

    /// Yields the *network mode* of a number of the message schema, `None` where the number
    /// names none.
    fn of_number(number: u64) -> Option<NetworkMode> {
        match number {
            1 => Some(NetworkMode::Invalid),
            2 => Some(NetworkMode::Valid),
            3 => Some(NetworkMode::Active),
            _ => None,
        }
    }
}

// realises FR-123, FR-229, FR-234, IR-020, IR-028
/// A *node state* of a *state response* (\[AD3\] IR-123): what a *node* tells of itself.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeState {
    /// the name of the *node*
    pub name: String,
    /// the number of the *diagnostics* of severity error of its *inputs* and of its
    /// *meta compiler DSL*
    pub errors: u64,
    /// its *origins*: the names of the *nodes* upstream of it that count an error, in ascending
    /// order of the names where a *node* tells them, and in the order of the *network file*
    /// where the *build system* does
    pub origins: Vec<String>,
    /// its *node mode*; `None` where the *build system* tells the state of a *node* that does
    /// not run or has told none yet (\[AD3\] IR-129)
    pub mode: Option<NodeMode>,
}

// realises IR-020, IR-028
/// A *response* *product* accepts from the *build system* or from the *node*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    /// the *request* was carried out
    Acknowledged,
    /// a *diagnostic* response: the resulting *document version*, the *diagnostics* of the
    /// document the *request* named, in the order of the response, and its *diagnostic groups*,
    /// none where it carries none
    Diagnostics {
        version: u64,
        diagnostics: Vec<Diagnostic>,
        groups: Vec<DiagnosticGroup>,
    },
    /// a version mismatch response naming the current *document version*
    VersionMismatch { version: u64 },
    /// an error response with its message text
    Error(String),
    /// a *network response*: the *document version* of the *network file* and its *nodes*
    Network {
        version: u64,
        nodes: Vec<NodeDescription>,
    },
    /// a cancelled response (\[AD3\] IR-028, IR-029): the *request* will not be answered,
    /// since a cancel request named it or an edit request superseded it
    Cancelled,
    /// a *state response*, which is no *terminal response* (\[AD3\] IR-121, IR-129): the
    /// *node states*, the *diagnostic group* of every *input* and of the *meta compiler DSL*,
    /// the *document version* of every *input* and *output*, and the *network mode*; the two
    /// middle ones are none where the *build system* answers, and the last is none where a
    /// *node* answers
    State {
        states: Vec<NodeState>,
        groups: Vec<DiagnosticGroup>,
        versions: Vec<(String, u64)>,
        mode: Option<NetworkMode>,
    },
    /// a content response (\[AD3\] IR-135): the *document version* of the named document, its
    /// octets, those of the UTF-8 encoding of its text where it is a *text document*, and
    /// whether it is a *binary document*
    Content {
        version: u64,
        octets: Vec<u8>,
        binary: bool,
    },
    /// a root response or a children response, which *product* asks for by no *request*;
    /// carries the kind
    Other(u64),
}

impl Response {
    // realises IR-020, IR-028
    /// Yields whether the *response* is a *terminal response*: every one is, but a
    /// *state response* (\[AD3\] § Message schema, rule 'Terminal').
    pub fn is_terminal(&self) -> bool {
        !matches!(self, Response::State { .. })
    }
}

// realises IR-018
/// Why a *message* or a *frame* could not be decoded or read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MessageError {
    /// the bytes are no CBOR map with unsigned integer keys; carries the reason
    Malformed(String),
    /// a kind the message schema does not define for the expected direction
    UnknownKind(u64),
    /// a key the kind requires is absent
    MissingKey(u64),
    /// the value of a key does not conform to the message schema
    InvalidValue(u64),
    /// the stream ended or failed while a *frame* was read; carries the reason
    Io(String),
}

impl fmt::Display for MessageError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MessageError::Malformed(reason) => write!(formatter, "malformed message: {reason}"),
            MessageError::UnknownKind(kind) => write!(formatter, "unknown message kind {kind}"),
            MessageError::MissingKey(key) => write!(formatter, "message lacks key {key}"),
            MessageError::InvalidValue(key) => {
                write!(
                    formatter,
                    "message carries an invalid value under key {key}"
                )
            }
            MessageError::Io(reason) => write!(formatter, "the connection failed: {reason}"),
        }
    }
}

impl std::error::Error for MessageError {}

// realises FR-071
/// Yields the *read position* of the character `offset` of `text`, counted from 0 in characters.
///
/// * An offset beyond the end yields the position after the last character.
pub fn position_of_offset(text: &str, offset: usize) -> ReadPosition {
    let mut position = ReadPosition { line: 1, column: 1 };
    for character in text.chars().take(offset) {
        if character == '\n' {
            position.line += 1;
            position.column = 1;
        } else {
            position.column += 1;
        }
    }
    position
}

// realises FR-071
/// Derives the *edit delta* of `increment`, which applies to the *provided text* `provided`.
pub fn delta_of_increment(provided: &str, increment: &TextIncrement) -> EditDelta {
    EditDelta {
        start: position_of_offset(provided, increment.position),
        end: position_of_offset(provided, increment.position + increment.range),
        text: increment.text.clone(),
    }
}

// realises FR-107, FR-125, IR-029
/// Yields the *diagnostics* of several documents as text, one line per *diagnostic* carrying the
/// *document identifier*, a colon, a space and the rendering, the documents in their order;
/// empty where there is none.
pub fn rendering_of_documents(documents: &[(String, Vec<Diagnostic>)]) -> String {
    documents
        .iter()
        .flat_map(|(document, diagnostics)| {
            diagnostics
                .iter()
                .map(move |diagnostic| format!("{document}: {}", diagnostic.rendering()))
        })
        .collect::<Vec<String>>()
        .join("\n")
}

// realises FR-126, FR-169, FR-170
/// The marks of the *diagnostics* of one document, as the *front end* draws them: per
/// *diagnostic*, the start and the end of its range, the index of its severity, and its message
/// text.
///
/// * The marks of txt carry the start and the end as offsets of the text in UTF-16 code units,
///   which a QML string counts ([`marks_of_diagnostics`]); the marks of hex and bin carry them
///   as *offsets*, in bits ([`bit_marks_of_diagnostics`]).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Marks {
    pub starts: Vec<i32>,
    pub ends: Vec<i32>,
    pub severities: Vec<i32>,
    pub texts: Vec<String>,
}

// realises FR-126
/// Yields the offset of `position` in `text`, in UTF-16 code units.
///
/// * A position beyond the end of a line, or beyond the last line, yields the offset of the
///   end of that line, or of the text.
pub fn utf16_offset_of_position(text: &str, position: ReadPosition) -> usize {
    let mut offset = 0;
    let mut line = 1;
    let mut column = 1;
    for character in text.chars() {
        if line == position.line && column == position.column {
            return offset;
        }
        if character == '\n' {
            if line == position.line {
                return offset;
            }
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
        offset += character.len_utf16();
    }
    offset
}

// realises FR-126, FR-153, FR-246 to FR-251
/// Yields the marks of `diagnostics` in `text`, in their order.
///
/// * A *diagnostic* one of whose *positions* has no *read position* has no mark: it lies in no
///   line of the text.
pub fn marks_of_diagnostics(text: &str, diagnostics: &[Diagnostic]) -> Marks {
    let offset = |position: ReadPosition| {
        i32::try_from(utf16_offset_of_position(text, position)).unwrap_or(i32::MAX)
    };
    let marked: Vec<(&Diagnostic, ReadPosition, ReadPosition)> = diagnostics
        .iter()
        .filter_map(|d| d.read_positions().map(|(start, end)| (d, start, end)))
        .collect();
    Marks {
        starts: marked.iter().map(|(_, start, _)| offset(*start)).collect(),
        ends: marked.iter().map(|(_, _, end)| offset(*end)).collect(),
        severities: marked
            .iter()
            .map(|(d, _, _)| d.severity.index() as i32)
            .collect(),
        texts: marked.iter().map(|(d, _, _)| d.text.clone()).collect(),
    }
}

// realises FR-210
/// Derives the *octet delta* by which `before` becomes `after`: the span between the octets the
/// two begin with alike and those they end with alike, and the octets of `after` in its place.
///
/// * Equal octets yield an empty span at their end with no replacement.
pub fn octet_delta_between(before: &[u8], after: &[u8]) -> OctetDelta {
    let prefix = before
        .iter()
        .zip(after)
        .take_while(|(before, after)| before == after)
        .count();
    let suffix = before[prefix..]
        .iter()
        .rev()
        .zip(after[prefix..].iter().rev())
        .take_while(|(before, after)| before == after)
        .count();
    OctetDelta {
        start: prefix as u64,
        end: (before.len() - suffix) as u64,
        replacement: after[prefix..after.len() - suffix].to_vec(),
    }
}

// realises FR-169, FR-170
/// Yields the marks of `diagnostics` in hex and bin, in their order: every *diagnostic*, with
/// the *offsets* of its range in bits, whether it has a *read position* or not.
///
/// * An *offset* beyond the greatest integer QML takes is carried as that integer.
pub fn bit_marks_of_diagnostics(diagnostics: &[Diagnostic]) -> Marks {
    let offset = |position: &Position| i32::try_from(position.offset).unwrap_or(i32::MAX);
    Marks {
        starts: diagnostics.iter().map(|d| offset(&d.start)).collect(),
        ends: diagnostics.iter().map(|d| offset(&d.end)).collect(),
        severities: diagnostics
            .iter()
            .map(|d| d.severity.index() as i32)
            .collect(),
        texts: diagnostics.iter().map(|d| d.text.clone()).collect(),
    }
}

// realises FR-109
/// Yields whether a *diagnostic* of severity error is among `diagnostics`.
pub fn has_error(diagnostics: &[Diagnostic]) -> bool {
    diagnostics
        .iter()
        .any(|diagnostic| diagnostic.severity == Severity::Error)
}

// realises IR-018, IR-019, IR-027
/// Encodes a *request* with its *request identifier* as one CBOR map of the message schema.
///
/// * The map has definite length, its keys ascend, and every integer has its shortest form.
pub fn encode_request(identifier: u64, request: &Request) -> Vec<u8> {
    let mut pairs = vec![(KEY_REQUEST, unsigned(identifier))];
    match request {
        Request::Open { document, text } => {
            pairs.push((KEY_KIND, unsigned(KIND_OPEN)));
            pairs.push((KEY_DOCUMENT, Value::Text(document.clone())));
            pairs.push((KEY_TEXT, Value::Text(text.clone())));
        }
        Request::OpenOctets { document, octets } => {
            pairs.push((KEY_KIND, unsigned(KIND_OPEN)));
            pairs.push((KEY_DOCUMENT, Value::Text(document.clone())));
            pairs.push((KEY_OCTETS, Value::Bytes(octets.clone())));
        }
        Request::Edit {
            document,
            version,
            deltas,
        } => {
            pairs.push((KEY_KIND, unsigned(KIND_EDIT)));
            pairs.push((KEY_DOCUMENT, Value::Text(document.clone())));
            pairs.push((KEY_VERSION, unsigned(*version)));
            pairs.push((
                KEY_DELTAS,
                Value::Array(deltas.iter().map(value_of_delta).collect()),
            ));
        }
        Request::EditOctets {
            document,
            version,
            deltas,
        } => {
            pairs.push((KEY_KIND, unsigned(KIND_EDIT)));
            pairs.push((KEY_DOCUMENT, Value::Text(document.clone())));
            pairs.push((KEY_VERSION, unsigned(*version)));
            pairs.push((
                KEY_OCTET_DELTAS,
                Value::Array(deltas.iter().map(value_of_octet_delta).collect()),
            ));
        }
        Request::Store => pairs.push((KEY_KIND, unsigned(KIND_STORE))),
        Request::QueryNetwork => pairs.push((KEY_KIND, unsigned(KIND_QUERY_NETWORK))),
        Request::Watch => pairs.push((KEY_KIND, unsigned(KIND_WATCH))),
        Request::QueryContent { document } => {
            pairs.push((KEY_KIND, unsigned(KIND_QUERY_CONTENT)));
            pairs.push((KEY_DOCUMENT, Value::Text(document.clone())));
        }
        Request::Activate => pairs.push((KEY_KIND, unsigned(KIND_ACTIVATE))),
        Request::Deactivate => pairs.push((KEY_KIND, unsigned(KIND_DEACTIVATE))),
        Request::Hold { node } => {
            pairs.push((KEY_KIND, unsigned(KIND_HOLD)));
            pairs.push((KEY_NODE_NAME, Value::Text(node.clone())));
        }
        Request::Release { node } => {
            pairs.push((KEY_KIND, unsigned(KIND_RELEASE)));
            pairs.push((KEY_NODE_NAME, Value::Text(node.clone())));
        }
        Request::Shutdown => pairs.push((KEY_KIND, unsigned(KIND_SHUTDOWN))),
    }
    bytes_of_pairs(pairs)
}

// realises IR-018
/// Encodes a *response* with the *request identifier* it answers, as the *build system* does; the
/// counterpart of [`decode_response`].
///
/// * Key 10 is 'true' but for a *state response*, which is no *terminal response*.
/// * A [`Response::Other`] is encoded with its kind and no further key.
/// * A *diagnostic* response without a *diagnostic group* is encoded without key 17, as a
///   *build system* before \[AD3\] IR-118 encodes it.
/// * A *state response* without a *diagnostic group* and without a *document version* is
///   encoded without the keys 17 and 20, as the *build system* in its *network role* encodes it,
///   and one without a *network mode* without key 21, as a *node* encodes it.
pub fn encode_response(identifier: u64, response: &Response) -> Vec<u8> {
    let mut pairs = vec![
        (KEY_REQUEST, unsigned(identifier)),
        (KEY_TERMINAL, Value::Bool(response.is_terminal())),
    ];
    match response {
        Response::Acknowledged => pairs.push((KEY_KIND, unsigned(KIND_ACKNOWLEDGED))),
        Response::Diagnostics {
            version,
            diagnostics,
            groups,
        } => {
            pairs.push((KEY_KIND, unsigned(KIND_DIAGNOSTICS)));
            pairs.push((KEY_VERSION, unsigned(*version)));
            pairs.push((
                KEY_DIAGNOSTICS,
                Value::Array(diagnostics.iter().map(value_of_diagnostic).collect()),
            ));
            if !groups.is_empty() {
                pairs.push((
                    KEY_GROUPS,
                    Value::Array(groups.iter().map(value_of_group).collect()),
                ));
            }
        }
        Response::VersionMismatch { version } => {
            pairs.push((KEY_KIND, unsigned(KIND_VERSION_MISMATCH)));
            pairs.push((KEY_VERSION, unsigned(*version)));
        }
        Response::Error(text) => {
            pairs.push((KEY_KIND, unsigned(KIND_ERROR)));
            pairs.push((KEY_ERROR, Value::Text(text.clone())));
        }
        Response::Network { version, nodes } => {
            pairs.push((KEY_KIND, unsigned(KIND_NETWORK)));
            pairs.push((KEY_VERSION, unsigned(*version)));
            pairs.push((
                KEY_NODES,
                Value::Array(nodes.iter().map(value_of_node).collect()),
            ));
        }
        Response::Cancelled => pairs.push((KEY_KIND, unsigned(KIND_CANCELLED))),
        Response::State {
            states,
            groups,
            versions,
            mode,
        } => {
            pairs.push((KEY_KIND, unsigned(KIND_STATE)));
            pairs.push((
                KEY_STATES,
                Value::Array(states.iter().map(value_of_state).collect()),
            ));
            if !groups.is_empty() {
                pairs.push((
                    KEY_GROUPS,
                    Value::Array(groups.iter().map(value_of_group).collect()),
                ));
            }
            if !versions.is_empty() {
                pairs.push((
                    KEY_VERSIONS,
                    Value::Array(versions.iter().map(value_of_version).collect()),
                ));
            }
            if let Some(mode) = mode {
                pairs.push((KEY_MODE, unsigned(mode.number())));
            }
        }
        Response::Content {
            version,
            octets,
            binary,
        } => {
            pairs.push((KEY_KIND, unsigned(KIND_CONTENT)));
            pairs.push((KEY_VERSION, unsigned(*version)));
            if *binary {
                pairs.push((KEY_OCTETS, Value::Bytes(octets.clone())));
            } else {
                pairs.push((
                    KEY_TEXT,
                    Value::Text(String::from_utf8_lossy(octets).into_owned()),
                ));
            }
        }
        Response::Other(kind) => pairs.push((KEY_KIND, unsigned(*kind))),
    }
    bytes_of_pairs(pairs)
}

// realises FR-225, FR-229, FR-231, FR-239, IR-018, IR-020, IR-028
/// Decodes a *response*: the *request identifier* it answers, and the *response*.
///
/// * A key the kind does not define is ignored, so that a later version of the message schema
///   adds a key without breaking this decoder.
/// * A *diagnostic* response without key 17 carries no *diagnostic group*.
/// * A *state response* without key 17 carries no *diagnostic group*, and one without key 20 no
///   *document version*, as the *build system* in its *network role* transmits it; one without
///   key 21 carries no *network mode*, as a *node* transmits it.
/// * A content response carries its text under key 4 or its octets under key 15.
///
/// # Errors
/// Returns [`MessageError::Malformed`] where the bytes are no CBOR map with unsigned integer keys,
/// [`MessageError::UnknownKind`] where the kind is that of no *response*,
/// [`MessageError::MissingKey`] where a key of the kind is absent, and
/// [`MessageError::InvalidValue`] where a value does not conform.
pub fn decode_response(bytes: &[u8]) -> Result<(u64, Response), MessageError> {
    let pairs = pairs_of_bytes(bytes)?;
    let kind = unsigned_under(&pairs, KEY_KIND)?;
    let identifier = unsigned_under(&pairs, KEY_REQUEST)?;
    let response = match kind {
        KIND_ACKNOWLEDGED => Response::Acknowledged,
        KIND_DIAGNOSTICS => Response::Diagnostics {
            version: unsigned_under(&pairs, KEY_VERSION)?,
            diagnostics: array_under(&pairs, KEY_DIAGNOSTICS)?
                .iter()
                .map(diagnostic_of_value)
                .collect::<Option<Vec<Diagnostic>>>()
                .ok_or(MessageError::InvalidValue(KEY_DIAGNOSTICS))?,
            groups: groups_under(&pairs)?,
        },
        KIND_STATE => Response::State {
            states: array_under(&pairs, KEY_STATES)?
                .iter()
                .map(state_of_value)
                .collect::<Option<Vec<NodeState>>>()
                .ok_or(MessageError::InvalidValue(KEY_STATES))?,
            groups: groups_under(&pairs)?,
            versions: match value_under(&pairs, KEY_VERSIONS) {
                Ok(_) => array_under(&pairs, KEY_VERSIONS)?
                    .iter()
                    .map(version_of_value)
                    .collect::<Option<Vec<(String, u64)>>>()
                    .ok_or(MessageError::InvalidValue(KEY_VERSIONS))?,
                Err(_) => Vec::new(),
            },
            mode: match value_under(&pairs, KEY_MODE) {
                Ok(value) => Some(
                    unsigned_of(value)
                        .and_then(NetworkMode::of_number)
                        .ok_or(MessageError::InvalidValue(KEY_MODE))?,
                ),
                Err(_) => None,
            },
        },
        KIND_CONTENT => {
            let version = unsigned_under(&pairs, KEY_VERSION)?;
            match (
                value_under(&pairs, KEY_TEXT),
                value_under(&pairs, KEY_OCTETS),
            ) {
                (Ok(Value::Text(text)), _) => Response::Content {
                    version,
                    octets: text.as_bytes().to_vec(),
                    binary: false,
                },
                (Ok(_), _) => return Err(MessageError::InvalidValue(KEY_TEXT)),
                (Err(_), Ok(Value::Bytes(octets))) => Response::Content {
                    version,
                    octets: octets.clone(),
                    binary: true,
                },
                (Err(_), Ok(_)) => return Err(MessageError::InvalidValue(KEY_OCTETS)),
                (Err(_), Err(_)) => return Err(MessageError::MissingKey(KEY_TEXT)),
            }
        }
        KIND_VERSION_MISMATCH => Response::VersionMismatch {
            version: unsigned_under(&pairs, KEY_VERSION)?,
        },
        KIND_ERROR => Response::Error(text_under(&pairs, KEY_ERROR)?),
        KIND_NETWORK => Response::Network {
            version: unsigned_under(&pairs, KEY_VERSION)?,
            nodes: array_under(&pairs, KEY_NODES)?
                .iter()
                .map(node_of_value)
                .collect::<Option<Vec<NodeDescription>>>()
                .ok_or(MessageError::InvalidValue(KEY_NODES))?,
        },
        KIND_CANCELLED => Response::Cancelled,
        KIND_ROOT | KIND_CHILDREN => Response::Other(kind),
        _ => return Err(MessageError::UnknownKind(kind)),
    };
    Ok((identifier, response))
}

// realises IR-018
/// Yields the bytes of the *frames* that carry `message`: each *frame* a 4-byte little-endian
/// length, a continuation byte, and at most [`MAX_FRAME_PAYLOAD`] payload bytes.
pub fn frames_of_message(message: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::with_capacity(message.len() + 5);
    let mut chunks = message.chunks(MAX_FRAME_PAYLOAD).peekable();
    if chunks.peek().is_none() {
        bytes.extend_from_slice(&[0, 0, 0, 0, 0]);
    }
    while let Some(chunk) = chunks.next() {
        let length = u32::try_from(chunk.len()).expect("a chunk is at most 16 MiB");
        bytes.extend_from_slice(&length.to_le_bytes());
        bytes.push(u8::from(chunks.peek().is_some()));
        bytes.extend_from_slice(chunk);
    }
    bytes
}

// realises IR-018
/// Reads the *frames* of one *message* from `stream` and yields its bytes.
///
/// # Errors
/// Returns [`MessageError::Io`] where the stream ends or fails within a *frame*, and
/// [`MessageError::Malformed`] where a *frame* declares more than [`MAX_FRAME_PAYLOAD`] bytes.
pub fn read_message(stream: &mut impl Read) -> Result<Vec<u8>, MessageError> {
    let mut message = Vec::new();
    loop {
        let mut header = [0u8; 5];
        stream
            .read_exact(&mut header)
            .map_err(|error| MessageError::Io(error.to_string()))?;
        let length = u32::from_le_bytes([header[0], header[1], header[2], header[3]]) as usize;
        if length > MAX_FRAME_PAYLOAD {
            return Err(MessageError::Malformed(format!(
                "a frame declares {length} bytes"
            )));
        }
        let start = message.len();
        message.resize(start + length, 0);
        stream
            .read_exact(&mut message[start..])
            .map_err(|error| MessageError::Io(error.to_string()))?;
        if header[4] == 0 {
            return Ok(message);
        }
    }
}

/// Yields an unsigned integer as a CBOR value.
fn unsigned(number: u64) -> Value {
    Value::Integer(Integer::from(number))
}

/// Yields the CBOR map of `pairs`, its keys ascending.
fn bytes_of_pairs(mut pairs: Vec<(u64, Value)>) -> Vec<u8> {
    pairs.sort_by_key(|(key, _)| *key);
    let map = Value::Map(
        pairs
            .into_iter()
            .map(|(key, value)| (unsigned(key), value))
            .collect(),
    );
    let mut bytes = Vec::new();
    ciborium::ser::into_writer(&map, &mut bytes).expect("writing into a vector cannot fail");
    bytes
}

/// Yields the pairs of the CBOR map `bytes` encode.
fn pairs_of_bytes(bytes: &[u8]) -> Result<Vec<(u64, Value)>, MessageError> {
    let value: Value = ciborium::de::from_reader(bytes)
        .map_err(|error| MessageError::Malformed(error.to_string()))?;
    let Value::Map(entries) = value else {
        return Err(MessageError::Malformed("no map".to_owned()));
    };
    entries
        .into_iter()
        .map(|(key, value)| {
            unsigned_of(&key)
                .map(|key| (key, value))
                .ok_or_else(|| MessageError::Malformed("a key is no unsigned integer".to_owned()))
        })
        .collect()
}

/// Yields the unsigned integer of a CBOR value, `None` where it is none.
fn unsigned_of(value: &Value) -> Option<u64> {
    match value {
        Value::Integer(integer) => u64::try_from(*integer).ok(),
        _ => None,
    }
}

/// Yields the text of a CBOR value, `None` where it is none.
fn text_of(value: &Value) -> Option<String> {
    match value {
        Value::Text(text) => Some(text.clone()),
        _ => None,
    }
}

/// Yields the value under `key`.
fn value_under(pairs: &[(u64, Value)], key: u64) -> Result<&Value, MessageError> {
    pairs
        .iter()
        .find(|(candidate, _)| *candidate == key)
        .map(|(_, value)| value)
        .ok_or(MessageError::MissingKey(key))
}

/// Yields the unsigned integer under `key`.
fn unsigned_under(pairs: &[(u64, Value)], key: u64) -> Result<u64, MessageError> {
    unsigned_of(value_under(pairs, key)?).ok_or(MessageError::InvalidValue(key))
}

/// Yields the text under `key`.
fn text_under(pairs: &[(u64, Value)], key: u64) -> Result<String, MessageError> {
    text_of(value_under(pairs, key)?).ok_or(MessageError::InvalidValue(key))
}

/// Yields the elements of the array under `key`.
fn array_under(pairs: &[(u64, Value)], key: u64) -> Result<&Vec<Value>, MessageError> {
    match value_under(pairs, key)? {
        Value::Array(elements) => Ok(elements),
        _ => Err(MessageError::InvalidValue(key)),
    }
}

/// Yields a *read position* as the array `[line, column]` of the message schema.
fn value_of_position(position: &ReadPosition) -> Value {
    Value::Array(vec![unsigned(position.line), unsigned(position.column)])
}

/// Yields the *read position* of the array `[line, column]`, `None` where the value is none.
fn position_of_value(value: &Value) -> Option<ReadPosition> {
    let Value::Array(elements) = value else {
        return None;
    };
    let [line, column] = elements.as_slice() else {
        return None;
    };
    Some(ReadPosition {
        line: unsigned_of(line)?,
        column: unsigned_of(column)?,
    })
}

/// Yields an *edit delta* as the array `[[start, end], text]` of the message schema.
fn value_of_delta(delta: &EditDelta) -> Value {
    Value::Array(vec![
        Value::Array(vec![
            value_of_position(&delta.start),
            value_of_position(&delta.end),
        ]),
        Value::Text(delta.text.clone()),
    ])
}

/// Yields an *octet delta* as the array `[start, end, replacement]` of the message schema.
fn value_of_octet_delta(delta: &OctetDelta) -> Value {
    Value::Array(vec![
        unsigned(delta.start),
        unsigned(delta.end),
        Value::Bytes(delta.replacement.clone()),
    ])
}

/// Yields a *position* as the array `[offset, [line, column] / null]` of the message schema.
fn value_of_diagnostic_position(position: &Position) -> Value {
    Value::Array(vec![
        unsigned(position.offset),
        match &position.read_position {
            Some(read_position) => value_of_position(read_position),
            None => Value::Null,
        },
    ])
}

/// Yields the *position* of the array `[offset, [line, column] / null]`, `None` where the value
/// is none.
fn diagnostic_position_of_value(value: &Value) -> Option<Position> {
    let Value::Array(elements) = value else {
        return None;
    };
    let [offset, read_position] = elements.as_slice() else {
        return None;
    };
    Some(Position {
        offset: unsigned_of(offset)?,
        read_position: match read_position {
            Value::Null => None,
            value => Some(position_of_value(value)?),
        },
    })
}

/// Yields a *diagnostic* as the array `[severity, [start, end], text]` of the message schema.
fn value_of_diagnostic(diagnostic: &Diagnostic) -> Value {
    Value::Array(vec![
        unsigned(diagnostic.severity.number()),
        Value::Array(vec![
            value_of_diagnostic_position(&diagnostic.start),
            value_of_diagnostic_position(&diagnostic.end),
        ]),
        Value::Text(diagnostic.text.clone()),
    ])
}

/// Yields the *diagnostic* of the array `[severity, [start, end], text]`, `None` where the value
/// is none.
fn diagnostic_of_value(value: &Value) -> Option<Diagnostic> {
    let Value::Array(elements) = value else {
        return None;
    };
    let [severity, range, text] = elements.as_slice() else {
        return None;
    };
    let Value::Array(positions) = range else {
        return None;
    };
    let [start, end] = positions.as_slice() else {
        return None;
    };
    Some(Diagnostic {
        severity: Severity::of_number(unsigned_of(severity)?)?,
        start: diagnostic_position_of_value(start)?,
        end: diagnostic_position_of_value(end)?,
        text: text_of(text)?,
    })
}

/// Yields a *diagnostic group* as the array `[document, version, [* diagnostic]]` of the
/// message schema.
fn value_of_group(group: &DiagnosticGroup) -> Value {
    Value::Array(vec![
        Value::Text(group.document.clone()),
        unsigned(group.version),
        Value::Array(group.diagnostics.iter().map(value_of_diagnostic).collect()),
    ])
}

/// Yields the *diagnostic group* of the array `[document, version, [* diagnostic]]`, `None`
/// where the value is none.
fn group_of_value(value: &Value) -> Option<DiagnosticGroup> {
    let Value::Array(elements) = value else {
        return None;
    };
    let [document, version, diagnostics] = elements.as_slice() else {
        return None;
    };
    let Value::Array(diagnostics) = diagnostics else {
        return None;
    };
    Some(DiagnosticGroup {
        document: text_of(document)?,
        version: unsigned_of(version)?,
        diagnostics: diagnostics
            .iter()
            .map(diagnostic_of_value)
            .collect::<Option<Vec<Diagnostic>>>()?,
    })
}

/// Yields the *diagnostic groups* under key 17, none where the key is absent.
fn groups_under(pairs: &[(u64, Value)]) -> Result<Vec<DiagnosticGroup>, MessageError> {
    match value_under(pairs, KEY_GROUPS) {
        Ok(_) => array_under(pairs, KEY_GROUPS)?
            .iter()
            .map(group_of_value)
            .collect::<Option<Vec<DiagnosticGroup>>>()
            .ok_or(MessageError::InvalidValue(KEY_GROUPS)),
        Err(_) => Ok(Vec::new()),
    }
}

/// Yields a *node state* as the array `[name, errors, [* origin], mode / null]` of the message
/// schema.
fn value_of_state(state: &NodeState) -> Value {
    Value::Array(vec![
        Value::Text(state.name.clone()),
        unsigned(state.errors),
        Value::Array(state.origins.iter().cloned().map(Value::Text).collect()),
        match state.mode {
            Some(mode) => unsigned(mode.number()),
            None => Value::Null,
        },
    ])
}

/// Yields the *node state* of the array `[name, errors, [* origin], mode / null]`, `None` where
/// the value is none.
fn state_of_value(value: &Value) -> Option<NodeState> {
    let Value::Array(elements) = value else {
        return None;
    };
    let [name, errors, origins, mode] = elements.as_slice() else {
        return None;
    };
    let Value::Array(origins) = origins else {
        return None;
    };
    Some(NodeState {
        name: text_of(name)?,
        errors: unsigned_of(errors)?,
        origins: origins
            .iter()
            .map(text_of)
            .collect::<Option<Vec<String>>>()?,
        mode: match mode {
            Value::Null => None,
            value => Some(NodeMode::of_number(unsigned_of(value)?)?),
        },
    })
}

/// Yields a *document version* of a *state response* as the array `[document, version]` of the
/// message schema.
fn value_of_version((document, version): &(String, u64)) -> Value {
    Value::Array(vec![Value::Text(document.clone()), unsigned(*version)])
}

/// Yields the *document identifier* and the *document version* of the array
/// `[document, version]`, `None` where the value is none.
fn version_of_value(value: &Value) -> Option<(String, u64)> {
    let Value::Array(elements) = value else {
        return None;
    };
    let [document, version] = elements.as_slice() else {
        return None;
    };
    Some((text_of(document)?, unsigned_of(version)?))
}

/// Yields a *node description* as the array of the message schema.
fn value_of_node(node: &NodeDescription) -> Value {
    let paths = |paths: &[String]| Value::Array(paths.iter().cloned().map(Value::Text).collect());
    Value::Array(vec![
        Value::Text(node.name.clone()),
        unsigned(match node.kind {
            NodeKind::MetaCompilerCompiler => 1,
            NodeKind::ProxyNode => 2,
        }),
        Value::Text(node.transformation.clone()),
        Value::Text(node.socket.clone()),
        paths(&node.inputs),
        paths(&node.outputs),
    ])
}

/// Yields the *node description* of the array of the message schema.
fn node_of_value(value: &Value) -> Option<NodeDescription> {
    let Value::Array(elements) = value else {
        return None;
    };
    let [name, kind, transformation, socket, inputs, outputs] = elements.as_slice() else {
        return None;
    };
    let paths = |value: &Value| match value {
        Value::Array(paths) => paths.iter().map(text_of).collect::<Option<Vec<String>>>(),
        _ => None,
    };
    Some(NodeDescription {
        name: text_of(name)?,
        kind: match unsigned_of(kind)? {
            1 => NodeKind::MetaCompilerCompiler,
            2 => NodeKind::ProxyNode,
            _ => return None,
        },
        transformation: text_of(transformation)?,
        socket: text_of(socket)?,
        inputs: paths(inputs)?,
        outputs: paths(outputs)?,
    })
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : NetworkEditor::apply_increment and NodeEditor::apply_increment, through build_system;
 *                    NodeEditor::report_arrived */
#[cfg(test)]
mod tests {
    use super::*;

    fn diagnostic() -> Diagnostic {
        Diagnostic {
            severity: Severity::Warning,
            start: Position {
                offset: 0,
                read_position: Some(ReadPosition { line: 2, column: 3 }),
            },
            end: Position {
                offset: 0,
                read_position: Some(ReadPosition { line: 2, column: 7 }),
            },
            text: "doubtful".to_owned(),
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

    fn node() -> NodeDescription {
        NodeDescription {
            name: "cpp_build".to_owned(),
            kind: NodeKind::ProxyNode,
            transformation: "cpp".to_owned(),
            socket: "cpp_build.sock".to_owned(),
            inputs: vec!["src/a.cpp".to_owned(), "src/a.h".to_owned()],
            outputs: vec!["build/a.o".to_owned()],
        }
    }

    #[test]
    fn offset_in_a_later_line_counts_lines_and_code_points() {
        assert_eq!(
            position_of_offset("aä\nbc", 4),
            ReadPosition { line: 2, column: 2 }
        );
    }

    #[test]
    fn offset_beyond_the_end_yields_the_position_after_the_last_character() {
        assert_eq!(
            position_of_offset("ab", 9),
            ReadPosition { line: 1, column: 3 }
        );
    }

    #[test]
    fn delta_of_an_increment_spans_the_overwritten_characters() {
        let increment = TextIncrement {
            position: 3,
            range: 2,
            text: "x".to_owned(),
        };
        assert_eq!(
            delta_of_increment("ab\ncd\n", &increment),
            EditDelta {
                start: ReadPosition { line: 2, column: 1 },
                end: ReadPosition { line: 2, column: 3 },
                text: "x".to_owned(),
            }
        );
    }

    #[test]
    fn diagnostic_renders_its_severity_its_range_and_its_text() {
        // FR-107, IR-029
        assert_eq!(diagnostic().rendering(), "warning 2:3-2:7: doubtful");
    }

    /// A *diagnostic* behind a *bit symbol*: its *positions* have no *read position*.
    fn diagnostic_in_bits() -> Diagnostic {
        Diagnostic {
            severity: Severity::Error,
            start: Position {
                offset: 40,
                read_position: None,
            },
            end: Position {
                offset: 56,
                read_position: None,
            },
            text: "no match".to_owned(),
        }
    }

    #[test]
    fn diagnostic_without_a_read_position_renders_its_offsets() {
        // FR-107, FR-153, IR-029
        assert_eq!(diagnostic_in_bits().rendering(), "error @40-@56: no match");
    }

    #[test]
    fn diagnostic_without_a_read_position_has_no_mark() {
        // FR-126, FR-153, FR-246, FR-251
        assert_eq!(
            marks_of_diagnostics("ab\ncdefgh", &[diagnostic_in_bits(), diagnostic()]),
            Marks {
                starts: vec![5],
                ends: vec![9],
                severities: vec![1],
                texts: vec!["doubtful".to_owned()],
            }
        );
    }

    #[test]
    fn diagnostic_response_with_and_without_read_positions_decodes_as_encoded() {
        // FR-153, IR-028: [offset, [line, column]] and [offset, null]
        let response = Response::Diagnostics {
            version: 4,
            diagnostics: vec![diagnostic(), diagnostic_in_bits()],
            groups: vec![],
        };
        assert_eq!(
            decode_response(&encode_response(9, &response)),
            Ok((9, response))
        );
    }

    #[test]
    fn diagnostic_response_with_diagnostic_groups_decodes_as_encoded() {
        // FR-225, IR-028: [document, version, [* diagnostic]], a group without a diagnostic too
        let response = Response::Diagnostics {
            version: 4,
            diagnostics: vec![diagnostic()],
            groups: vec![
                DiagnosticGroup {
                    document: "b.in".to_owned(),
                    version: 7,
                    diagnostics: vec![diagnostic_in_bits(), diagnostic()],
                },
                DiagnosticGroup {
                    document: "c.in".to_owned(),
                    version: 0,
                    diagnostics: vec![],
                },
            ],
        };
        assert_eq!(
            decode_response(&encode_response(9, &response)),
            Ok((9, response))
        );
    }

    #[test]
    fn diagnostic_response_without_a_diagnostic_group_is_encoded_without_key_17() {
        // IR-028: a response of a build system before [AD3] IR-118 carries no group
        let response = Response::Diagnostics {
            version: 4,
            diagnostics: vec![diagnostic()],
            groups: vec![],
        };
        let bytes = encode_response(9, &response);
        let keys: Vec<u64> = pairs_of_bytes(&bytes)
            .expect("the response is a map")
            .iter()
            .map(|(key, _)| *key)
            .collect();
        assert_eq!(
            (keys, decode_response(&bytes)),
            (vec![0, 1, 3, 10, 11], Ok((9, response)))
        );
    }

    #[test]
    fn diagnostic_group_that_is_no_group_is_an_invalid_value_under_key_17() {
        // IR-028: a group lacking its diagnostics
        let bytes = bytes_of_pairs(vec![
            (KEY_KIND, unsigned(KIND_DIAGNOSTICS)),
            (KEY_REQUEST, unsigned(9)),
            (KEY_VERSION, unsigned(4)),
            (KEY_TERMINAL, Value::Bool(true)),
            (KEY_DIAGNOSTICS, Value::Array(vec![])),
            (
                KEY_GROUPS,
                Value::Array(vec![Value::Array(vec![
                    Value::Text("b.in".to_owned()),
                    unsigned(7),
                ])]),
            ),
        ]);
        assert_eq!(
            decode_response(&bytes),
            Err(MessageError::InvalidValue(KEY_GROUPS))
        );
    }

    #[test]
    fn key_above_17_of_a_diagnostic_response_is_ignored() {
        // IR-018: a later version of the message schema adds a key
        let bytes = bytes_of_pairs(vec![
            (KEY_KIND, unsigned(KIND_DIAGNOSTICS)),
            (KEY_REQUEST, unsigned(9)),
            (KEY_VERSION, unsigned(4)),
            (KEY_TERMINAL, Value::Bool(true)),
            (KEY_DIAGNOSTICS, Value::Array(vec![])),
            (18, Value::Text("later".to_owned())),
        ]);
        assert_eq!(
            decode_response(&bytes),
            Ok((
                9,
                Response::Diagnostics {
                    version: 4,
                    diagnostics: vec![],
                    groups: vec![],
                }
            ))
        );
    }

    #[test]
    fn diagnostics_of_documents_render_one_per_line_with_their_document() {
        // FR-107, FR-108, FR-125, IR-029
        let error = Diagnostic {
            severity: Severity::Error,
            start: Position {
                offset: 0,
                read_position: Some(ReadPosition { line: 1, column: 1 }),
            },
            end: Position {
                offset: 0,
                read_position: Some(ReadPosition { line: 1, column: 1 }),
            },
            text: "fault".to_owned(),
        };
        let documents = vec![
            ("a.gc3".to_owned(), vec![error, diagnostic()]),
            ("a.in".to_owned(), vec![]),
            ("b.in".to_owned(), vec![diagnostic()]),
        ];
        assert_eq!(
            (
                rendering_of_documents(&documents),
                rendering_of_documents(&[])
            ),
            (
                "a.gc3: error 1:1-1:1: fault\na.gc3: warning 2:3-2:7: doubtful\n\
                 b.in: warning 2:3-2:7: doubtful"
                    .to_owned(),
                String::new()
            )
        );
    }

    #[test]
    fn offset_of_a_position_counts_utf16_units_and_lines() {
        // FR-126: 'ä' is one unit, the emoji two
        let text = "a\u{e4}\n\u{1f600}bc\nd";
        assert_eq!(
            [
                utf16_offset_of_position(text, ReadPosition { line: 1, column: 1 }),
                utf16_offset_of_position(text, ReadPosition { line: 1, column: 3 }),
                utf16_offset_of_position(text, ReadPosition { line: 2, column: 2 }),
                utf16_offset_of_position(text, ReadPosition { line: 2, column: 4 }),
                utf16_offset_of_position(text, ReadPosition { line: 3, column: 2 }),
            ],
            [0, 2, 5, 7, 9]
        );
    }

    #[test]
    fn offset_beyond_a_line_or_beyond_the_text_is_the_end_of_that_line_or_of_the_text() {
        // FR-126
        assert_eq!(
            [
                utf16_offset_of_position("ab\ncd", ReadPosition { line: 1, column: 9 }),
                utf16_offset_of_position("ab\ncd", ReadPosition { line: 7, column: 1 }),
                utf16_offset_of_position("", ReadPosition { line: 1, column: 1 }),
            ],
            [2, 5, 0]
        );
    }

    #[test]
    fn marks_carry_the_offsets_the_severity_index_and_the_text_of_every_diagnostic() {
        // FR-126, FR-127
        let error = Diagnostic {
            severity: Severity::Error,
            start: Position {
                offset: 0,
                read_position: Some(ReadPosition { line: 1, column: 2 }),
            },
            end: Position {
                offset: 0,
                read_position: Some(ReadPosition { line: 1, column: 2 }),
            },
            text: "fault".to_owned(),
        };
        assert_eq!(
            marks_of_diagnostics("ab\ncdefgh", &[error, diagnostic()]),
            Marks {
                starts: vec![1, 5],
                ends: vec![1, 9],
                severities: vec![0, 1],
                texts: vec!["fault".to_owned(), "doubtful".to_owned()],
            }
        );
    }

    #[test]
    fn an_error_among_the_diagnostics_is_found() {
        // FR-109
        let error = Diagnostic {
            severity: Severity::Error,
            ..diagnostic()
        };
        let information = Diagnostic {
            severity: Severity::Information,
            ..diagnostic()
        };
        assert_eq!(
            (
                has_error(&[]),
                has_error(&[diagnostic(), information.clone()]),
                has_error(&[information, error])
            ),
            (false, false, true)
        );
    }

    #[test]
    fn store_request_is_the_canonical_map() {
        // IR-027: { 0: 9, 1: 3 }
        assert_eq!(
            encode_request(3, &Request::Store),
            vec![0xa2, 0x00, 0x09, 0x01, 0x03]
        );
    }

    #[test]
    fn diagnostic_with_a_severity_outside_one_to_three_is_an_invalid_value() {
        // IR-028
        let mut bytes = encode_response(
            1,
            &Response::Diagnostics {
                version: 0,
                diagnostics: vec![diagnostic()],
                groups: vec![],
            },
        );
        // the severity 2 precedes the range [[2, 3], [2, 7]]
        let severity = bytes
            .windows(3)
            .position(|window| window == [0x02, 0x82, 0x82])
            .expect("the severity precedes the range");
        bytes[severity] = 0x04;
        assert_eq!(decode_response(&bytes), Err(MessageError::InvalidValue(11)));
    }

    #[test]
    fn network_query_request_is_the_canonical_map() {
        // { 0: 11, 1: 7 }
        assert_eq!(
            encode_request(7, &Request::QueryNetwork),
            vec![0xa2, 0x00, 0x0b, 0x01, 0x07]
        );
    }

    #[test]
    fn every_diagnostic_has_a_mark_in_bits_with_or_without_a_read_position() {
        // FR-169, FR-170
        assert_eq!(
            bit_marks_of_diagnostics(&[diagnostic_in_bits(), diagnostic()]),
            Marks {
                starts: vec![40, diagnostic().start.offset as i32],
                ends: vec![56, diagnostic().end.offset as i32],
                severities: vec![0, 1],
                texts: vec!["no match".to_owned(), "doubtful".to_owned()],
            }
        );
    }

    #[test]
    fn octet_delta_spans_what_lies_between_the_common_start_and_the_common_end() {
        // FR-210
        assert_eq!(
            (
                octet_delta_between(&[1, 2, 3, 4, 5], &[1, 2, 9, 9, 9, 5]),
                octet_delta_between(&[1, 2, 3], &[1, 3]),
                octet_delta_between(&[], &[7]),
                octet_delta_between(&[1, 1], &[1, 1, 1])
            ),
            (
                OctetDelta {
                    start: 2,
                    end: 4,
                    replacement: vec![9, 9, 9]
                },
                OctetDelta {
                    start: 1,
                    end: 2,
                    replacement: vec![]
                },
                OctetDelta {
                    start: 0,
                    end: 0,
                    replacement: vec![7]
                },
                OctetDelta {
                    start: 2,
                    end: 2,
                    replacement: vec![1]
                }
            )
        );
    }

    #[test]
    fn octet_delta_of_equal_octets_replaces_nothing_by_nothing() {
        // FR-210
        assert_eq!(
            octet_delta_between(&[1, 2], &[1, 2]),
            OctetDelta {
                start: 2,
                end: 2,
                replacement: vec![]
            }
        );
    }

    #[test]
    fn edit_request_of_a_binary_document_carries_its_octet_deltas_under_key_16() {
        // FR-210, [AD3] IR-114
        let bytes = encode_request(
            2,
            &Request::EditOctets {
                document: "n".to_owned(),
                version: 3,
                deltas: vec![OctetDelta {
                    start: 1,
                    end: 2,
                    replacement: vec![0xff],
                }],
            },
        );
        // { 0: 2, 1: 2, 2: "n", 3: 3, 16: [[1, 2, h'ff']] }
        assert_eq!(
            bytes,
            vec![
                0xa5, 0x00, 0x02, 0x01, 0x02, 0x02, 0x61, b'n', 0x03, 0x03, 0x10, 0x81, 0x83, 0x01,
                0x02, 0x41, 0xff
            ]
        );
    }

    #[test]
    fn open_request_of_a_binary_document_carries_its_octets_as_a_byte_string() {
        // FR-167, [AD3] IR-112
        let bytes = encode_request(
            1,
            &Request::OpenOctets {
                document: "n".to_owned(),
                octets: vec![0xff, 0x00],
            },
        );
        // { 0: 1, 1: 1, 2: "n", 15: h'ff00' }
        assert_eq!(
            bytes,
            vec![
                0xa4, 0x00, 0x01, 0x01, 0x01, 0x02, 0x61, b'n', 0x0f, 0x42, 0xff, 0x00
            ]
        );
    }

    #[test]
    fn open_request_carries_document_and_text_under_ascending_keys() {
        let bytes = encode_request(
            1,
            &Request::Open {
                document: "n".to_owned(),
                text: "t".to_owned(),
            },
        );
        // { 0: 1, 1: 1, 2: "n", 4: "t" }
        assert_eq!(
            bytes,
            vec![
                0xa4, 0x00, 0x01, 0x01, 0x01, 0x02, 0x61, b'n', 0x04, 0x61, b't'
            ]
        );
    }

    #[test]
    fn edit_request_carries_the_deltas_as_ranges_and_texts() {
        let bytes = encode_request(
            2,
            &Request::Edit {
                document: "n".to_owned(),
                version: 3,
                deltas: vec![EditDelta {
                    start: ReadPosition { line: 1, column: 2 },
                    end: ReadPosition { line: 1, column: 4 },
                    text: "x".to_owned(),
                }],
            },
        );
        // { 0: 2, 1: 2, 2: "n", 3: 3, 5: [[[[1, 2], [1, 4]], "x"]] }
        assert_eq!(
            bytes,
            vec![
                0xa5, 0x00, 0x02, 0x01, 0x02, 0x02, 0x61, b'n', 0x03, 0x03, 0x05, 0x81, 0x82, 0x82,
                0x82, 0x01, 0x02, 0x82, 0x01, 0x04, 0x61, b'x'
            ]
        );
    }

    #[test]
    fn every_response_round_trips_through_its_encoding() {
        let responses = [
            Response::Acknowledged,
            Response::Diagnostics {
                version: 4,
                diagnostics: vec![diagnostic()],
                groups: vec![],
            },
            Response::Diagnostics {
                version: 4,
                diagnostics: vec![],
                groups: vec![DiagnosticGroup {
                    document: "b.in".to_owned(),
                    version: 2,
                    diagnostics: vec![diagnostic()],
                }],
            },
            Response::VersionMismatch { version: 9 },
            Response::Error("no".to_owned()),
            Response::Network {
                version: 2,
                nodes: vec![node()],
            },
            Response::State {
                states: vec![state("b", 2, &["a"])],
                groups: vec![DiagnosticGroup {
                    document: "b.in".to_owned(),
                    version: 2,
                    diagnostics: vec![diagnostic()],
                }],
                versions: vec![("b.in".to_owned(), 2), ("y.txt".to_owned(), 7)],
                mode: None,
            },
            Response::State {
                states: vec![
                    state("a", 0, &[]),
                    NodeState {
                        mode: None,
                        ..state("b", 0, &[])
                    },
                ],
                groups: vec![],
                versions: vec![],
                mode: Some(NetworkMode::Valid),
            },
            Response::Cancelled,
            Response::Content {
                version: 3,
                octets: b"text".to_vec(),
                binary: false,
            },
            Response::Content {
                version: 3,
                octets: vec![0xff, 0x00],
                binary: true,
            },
            Response::Other(34),
        ];
        let decoded: Vec<_> = responses
            .iter()
            .map(|response| decode_response(&encode_response(5, response)))
            .collect();
        let expected: Vec<_> = responses
            .iter()
            .map(|response| Ok((5, response.clone())))
            .collect();
        assert_eq!(decoded, expected);
    }

    #[test]
    fn watch_request_carries_its_kind_and_its_request_identifier_alone() {
        // FR-228, FR-230, [AD3] IR-120
        // { 0: 12, 1: 7 }
        assert_eq!(
            encode_request(7, &Request::Watch),
            vec![0xa2, 0x00, 0x0c, 0x01, 0x07]
        );
    }

    #[test]
    fn content_query_request_names_its_document() {
        // FR-239, [AD3] IR-134
        let bytes = encode_request(
            2,
            &Request::QueryContent {
                document: "n".to_owned(),
            },
        );
        // { 0: 14, 1: 2, 2: "n" }
        assert_eq!(bytes, vec![0xa3, 0x00, 0x0e, 0x01, 0x02, 0x02, 0x61, b'n']);
    }

    #[test]
    fn state_response_is_no_terminal_response_and_every_other_one_is() {
        // [AD3] § Message schema, rule 'Terminal'
        let watched = Response::State {
            states: vec![],
            groups: vec![],
            versions: vec![],
            mode: None,
        };
        // { 0: 40, 1: 1, 10: false, 18: [] }
        assert_eq!(
            (
                watched.is_terminal(),
                Response::Acknowledged.is_terminal(),
                encode_response(1, &watched)
            ),
            (
                false,
                true,
                vec![0xa4, 0x00, 0x18, 0x28, 0x01, 0x01, 0x0a, 0xf4, 0x12, 0x80]
            )
        );
    }

    #[test]
    fn state_response_of_the_build_system_carries_the_network_mode_and_no_groups_or_versions() {
        // FR-229, [AD3] IR-123, IR-129:
        // { 0: 40, 1: 3, 10: false, 18: [["a", 1, ["b"], 3], ["c", 0, [], null]], 21: 3 }
        let bytes = [
            0xa5, 0x00, 0x18, 0x28, 0x01, 0x03, 0x0a, 0xf4, 0x12, 0x82, 0x84, 0x61, b'a', 0x01,
            0x81, 0x61, b'b', 0x03, 0x84, 0x61, b'c', 0x00, 0x80, 0xf6, 0x15, 0x03,
        ];
        assert_eq!(
            decode_response(&bytes),
            Ok((
                3,
                Response::State {
                    states: vec![
                        state("a", 1, &["b"]),
                        NodeState {
                            mode: None,
                            ..state("c", 0, &[])
                        }
                    ],
                    groups: vec![],
                    versions: vec![],
                    mode: Some(NetworkMode::Active),
                }
            ))
        );
    }

    #[test]
    fn state_response_of_a_node_carries_its_node_mode_and_no_network_mode() {
        // FR-123, [AD3] IR-121, IR-123: { 0: 40, 1: 1, 10: false, 18: [["a", 0, [], 2]] }
        let bytes = [
            0xa4, 0x00, 0x18, 0x28, 0x01, 0x01, 0x0a, 0xf4, 0x12, 0x81, 0x84, 0x61, b'a', 0x00,
            0x80, 0x02,
        ];
        assert_eq!(
            decode_response(&bytes),
            Ok((
                1,
                Response::State {
                    states: vec![NodeState {
                        mode: Some(NodeMode::Valid),
                        ..state("a", 0, &[])
                    }],
                    groups: vec![],
                    versions: vec![],
                    mode: None,
                }
            ))
        );
    }

    #[test]
    fn node_state_of_three_elements_as_a_version_before_told_it_is_an_invalid_value() {
        // [AD3] § Message schema, rule 'Keys': the structure of a node state changed with the
        // node mode; { 0: 40, 1: 3, 10: false, 18: [["a", 1, ["b"]]] }
        let bytes = [
            0xa4, 0x00, 0x18, 0x28, 0x01, 0x03, 0x0a, 0xf4, 0x12, 0x81, 0x83, 0x61, b'a', 0x01,
            0x81, 0x61, b'b',
        ];
        assert_eq!(decode_response(&bytes), Err(MessageError::InvalidValue(18)));
    }

    #[test]
    fn node_mode_outside_one_to_four_is_an_invalid_value() {
        // { 0: 40, 1: 3, 10: false, 18: [["a", 0, [], 5]] }
        let bytes = [
            0xa4, 0x00, 0x18, 0x28, 0x01, 0x03, 0x0a, 0xf4, 0x12, 0x81, 0x84, 0x61, b'a', 0x00,
            0x80, 0x05,
        ];
        assert_eq!(decode_response(&bytes), Err(MessageError::InvalidValue(18)));
    }

    #[test]
    fn node_mode_four_is_the_waiting_node() {
        // FR-123, [AD3] IR-191: { 0: 40, 1: 3, 10: false, 18: [["a", 0, [], 4]] }
        let bytes = [
            0xa4, 0x00, 0x18, 0x28, 0x01, 0x03, 0x0a, 0xf4, 0x12, 0x81, 0x84, 0x61, b'a', 0x00,
            0x80, 0x04,
        ];
        let decoded = decode_response(&bytes).map(|(_, response)| match response {
            Response::State { states, .. } => states.first().and_then(|state| state.mode),
            _ => None,
        });
        assert_eq!(decoded, Ok(Some(NodeMode::Waiting)));
    }

    #[test]
    fn network_mode_outside_one_to_three_is_an_invalid_value_under_key_21() {
        // { 0: 40, 1: 3, 10: false, 18: [], 21: 0 }
        let bytes = [
            0xa5, 0x00, 0x18, 0x28, 0x01, 0x03, 0x0a, 0xf4, 0x12, 0x80, 0x15, 0x00,
        ];
        assert_eq!(decode_response(&bytes), Err(MessageError::InvalidValue(21)));
    }

    #[test]
    fn activate_request_is_the_canonical_map() {
        // FR-260, [AD3] IR-138: { 0: 15, 1: 4 }
        assert_eq!(
            encode_request(4, &Request::Activate),
            vec![0xa2, 0x00, 0x0f, 0x01, 0x04]
        );
    }

    #[test]
    fn deactivate_request_is_the_canonical_map() {
        // FR-267, [AD3] IR-139: { 0: 16, 1: 5 }
        assert_eq!(
            encode_request(5, &Request::Deactivate),
            vec![0xa2, 0x00, 0x10, 0x01, 0x05]
        );
    }

    #[test]
    fn hold_request_names_its_node_under_key_23() {
        // FR-281, [AD3] IR-194: { 0: 18, 1: 1, 23: "a" }
        assert_eq!(
            encode_request(
                1,
                &Request::Hold {
                    node: "a".to_owned()
                }
            ),
            vec![0xa3, 0x00, 0x12, 0x01, 0x01, 0x17, 0x61, 0x61]
        );
    }

    #[test]
    fn release_request_names_its_node_under_key_23() {
        // FR-282, [AD3] IR-195: { 0: 19, 1: 2, 23: "ab" }
        assert_eq!(
            encode_request(
                2,
                &Request::Release {
                    node: "ab".to_owned()
                }
            ),
            vec![0xa3, 0x00, 0x13, 0x01, 0x02, 0x17, 0x62, 0x61, 0x62]
        );
    }

    #[test]
    fn cancelled_response_is_decoded_as_cancelled() {
        // FR-264, [AD3] IR-029: { 0: 36, 1: 2, 10: true }
        assert_eq!(
            decode_response(&[0xa3, 0x00, 0x18, 0x24, 0x01, 0x02, 0x0a, 0xf5]),
            Ok((2, Response::Cancelled))
        );
    }

    #[test]
    fn content_response_without_text_and_octets_lacks_its_key() {
        // { 0: 41, 1: 1, 3: 0, 10: true }
        assert_eq!(
            decode_response(&[0xa4, 0x00, 0x18, 0x29, 0x01, 0x01, 0x03, 0x00, 0x0a, 0xf5]),
            Err(MessageError::MissingKey(4))
        );
    }

    #[test]
    fn key_above_twenty_one_is_ignored() {
        // [AD3] § Message schema, rule 'Keys': { 0: 32, 1: 1, 10: true, 22: 5 }
        assert_eq!(
            decode_response(&[0xa4, 0x00, 0x18, 0x20, 0x01, 0x01, 0x0a, 0xf5, 0x16, 0x05]),
            Ok((1, Response::Acknowledged))
        );
    }

    #[test]
    fn request_kind_is_no_response() {
        assert_eq!(
            decode_response(&encode_request(1, &Request::Shutdown)),
            Err(MessageError::UnknownKind(10))
        );
    }

    #[test]
    fn response_without_request_identifier_is_reported() {
        // { 0: 32 }
        assert_eq!(
            decode_response(&[0xa1, 0x00, 0x18, 0x20]),
            Err(MessageError::MissingKey(1))
        );
    }

    #[test]
    fn node_kind_outside_one_to_two_is_an_invalid_value() {
        let mut bytes = encode_response(
            1,
            &Response::Network {
                version: 0,
                nodes: vec![node()],
            },
        );
        let kind = bytes
            .windows(2)
            .position(|window| window == [0x02, 0x63])
            .expect("the node kind precedes the transformation");
        bytes[kind] = 0x03;
        assert_eq!(decode_response(&bytes), Err(MessageError::InvalidValue(14)));
    }

    #[test]
    fn bytes_that_are_no_map_are_malformed() {
        assert!(matches!(
            decode_response(&[0x01]),
            Err(MessageError::Malformed(_))
        ));
    }

    #[test]
    fn message_is_carried_by_one_frame_with_length_and_no_continuation() {
        assert_eq!(frames_of_message(&[1, 2, 3]), vec![3, 0, 0, 0, 0, 1, 2, 3]);
    }

    #[test]
    fn empty_message_is_carried_by_one_empty_frame() {
        assert_eq!(frames_of_message(&[]), vec![0, 0, 0, 0, 0]);
    }

    #[test]
    fn continued_frames_are_read_as_one_message() {
        let mut stream: &[u8] = &[2, 0, 0, 0, 1, 7, 8, 1, 0, 0, 0, 0, 9, 99];
        assert_eq!(read_message(&mut stream), Ok(vec![7, 8, 9]));
    }

    #[test]
    fn stream_ending_within_a_frame_is_an_io_error() {
        let mut stream: &[u8] = &[4, 0, 0, 0, 0, 1];
        assert!(matches!(
            read_message(&mut stream),
            Err(MessageError::Io(_))
        ));
    }

    #[test]
    fn frame_declaring_more_than_16_mib_is_malformed() {
        let mut stream: &[u8] = &[1, 0, 0, 1, 0];
        assert!(matches!(
            read_message(&mut stream),
            Err(MessageError::Malformed(_))
        ));
    }
}
