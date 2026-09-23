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

/// The kinds of the *messages* *product* encodes and decodes.
const KIND_OPEN: u64 = 1;
const KIND_EDIT: u64 = 2;
const KIND_STORE: u64 = 9;
const KIND_SHUTDOWN: u64 = 10;
const KIND_QUERY_NETWORK: u64 = 11;
const KIND_ACKNOWLEDGED: u64 = 32;
const KIND_DIAGNOSTICS: u64 = 33;
const KIND_ROOT: u64 = 34;
const KIND_CHILDREN: u64 = 35;
const KIND_CANCELLED: u64 = 36;
const KIND_VERSION_MISMATCH: u64 = 37;
const KIND_ERROR: u64 = 38;
const KIND_NETWORK: u64 = 39;

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

// realises FR-107, IR-028
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

// realises FR-107, FR-109, IR-028, IR-029
/// A *diagnostic* of a *diagnostic* response: its severity, its *read position* range and its
/// message text (\[AD5\] IR-023).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// its severity
    pub severity: Severity,
    /// the first position of the range, inclusive
    pub start: ReadPosition,
    /// the position after the last one of the range
    pub end: ReadPosition,
    /// its message text
    pub text: String,
}

impl Diagnostic {
    // realises FR-107, IR-029
    /// Yields the *diagnostic* as text: the severity, a space, the range as
    /// `<line>:<column>-<line>:<column>`, a colon, a space, and the message text.
    pub fn rendering(&self) -> String {
        format!(
            "{} {}:{}-{}:{}: {}",
            self.severity.name(),
            self.start.line,
            self.start.column,
            self.end.line,
            self.end.column,
            self.text
        )
    }
}

// realises IR-019, IR-027
/// A *request* *product* transmits to the *build system* or to the *node*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Request {
    /// an open request carrying the full text of the document
    Open { document: String, text: String },
    /// an edit request carrying the *document version* it applies to and its *edit deltas*
    Edit {
        document: String,
        version: u64,
        deltas: Vec<EditDelta>,
    },
    /// a *store request*, transmitted to the *node* alone
    Store,
    /// a *network query request*, transmitted to the *build system* alone
    QueryNetwork,
    /// a *shutdown request*
    Shutdown,
}

// realises IR-020, IR-028
/// A *response* *product* accepts from the *build system* or from the *node*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    /// the *request* was carried out
    Acknowledged,
    /// a *diagnostic* response: the resulting *document version* and the *diagnostics*, in the
    /// order of the response
    Diagnostics {
        version: u64,
        diagnostics: Vec<Diagnostic>,
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
    /// a *response* of a kind *product* asks for by no *request*; carries the kind
    Other(u64),
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

// realises FR-126
/// The marks of the *diagnostics* of one document, as the *front end* draws them: per
/// *diagnostic*, the start and the end of its range as offsets of the text in UTF-16 code units,
/// which a QML string counts, the index of its severity, and its message text.
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

// realises FR-126
/// Yields the marks of `diagnostics` in `text`, in their order.
pub fn marks_of_diagnostics(text: &str, diagnostics: &[Diagnostic]) -> Marks {
    let offset = |position: ReadPosition| {
        i32::try_from(utf16_offset_of_position(text, position)).unwrap_or(i32::MAX)
    };
    Marks {
        starts: diagnostics.iter().map(|d| offset(d.start)).collect(),
        ends: diagnostics.iter().map(|d| offset(d.end)).collect(),
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
        Request::Store => pairs.push((KEY_KIND, unsigned(KIND_STORE))),
        Request::QueryNetwork => pairs.push((KEY_KIND, unsigned(KIND_QUERY_NETWORK))),
        Request::Shutdown => pairs.push((KEY_KIND, unsigned(KIND_SHUTDOWN))),
    }
    bytes_of_pairs(pairs)
}

// realises IR-018
/// Encodes a *terminal response* with the *request identifier* it answers, as the *build system*
/// does; the counterpart of [`decode_response`].
///
/// * A [`Response::Other`] is encoded with its kind and no further key.
pub fn encode_response(identifier: u64, response: &Response) -> Vec<u8> {
    let mut pairs = vec![
        (KEY_REQUEST, unsigned(identifier)),
        (KEY_TERMINAL, Value::Bool(true)),
    ];
    match response {
        Response::Acknowledged => pairs.push((KEY_KIND, unsigned(KIND_ACKNOWLEDGED))),
        Response::Diagnostics {
            version,
            diagnostics,
        } => {
            pairs.push((KEY_KIND, unsigned(KIND_DIAGNOSTICS)));
            pairs.push((KEY_VERSION, unsigned(*version)));
            pairs.push((
                KEY_DIAGNOSTICS,
                Value::Array(diagnostics.iter().map(value_of_diagnostic).collect()),
            ));
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
        Response::Other(kind) => pairs.push((KEY_KIND, unsigned(*kind))),
    }
    bytes_of_pairs(pairs)
}

// realises IR-018, IR-020, IR-028
/// Decodes a *response*: the *request identifier* it answers, and the *response*.
///
/// * A key the kind does not define is ignored, so that a later version of the message schema
///   adds a key without breaking this decoder.
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
        },
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
        KIND_ROOT | KIND_CHILDREN | KIND_CANCELLED => Response::Other(kind),
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

/// Yields a *diagnostic* as the array `[severity, [start, end], text]` of the message schema.
fn value_of_diagnostic(diagnostic: &Diagnostic) -> Value {
    Value::Array(vec![
        unsigned(diagnostic.severity.number()),
        Value::Array(vec![
            value_of_position(&diagnostic.start),
            value_of_position(&diagnostic.end),
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
        start: position_of_value(start)?,
        end: position_of_value(end)?,
        text: text_of(text)?,
    })
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
            start: ReadPosition { line: 2, column: 3 },
            end: ReadPosition { line: 2, column: 7 },
            text: "doubtful".to_owned(),
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

    #[test]
    fn diagnostics_of_documents_render_one_per_line_with_their_document() {
        // FR-107, FR-108, FR-125, IR-029
        let error = Diagnostic {
            severity: Severity::Error,
            start: ReadPosition { line: 1, column: 1 },
            end: ReadPosition { line: 1, column: 1 },
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
            start: ReadPosition { line: 1, column: 2 },
            end: ReadPosition { line: 1, column: 2 },
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
            },
            Response::VersionMismatch { version: 9 },
            Response::Error("no".to_owned()),
            Response::Network {
                version: 2,
                nodes: vec![node()],
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
