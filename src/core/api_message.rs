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

// realises IR-019
/// A *request* *product* transmits to the *build system*.
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
    /// a *network query request*
    QueryNetwork,
    /// a *shutdown request*
    Shutdown,
}

// realises IR-020
/// A *response* *product* accepts from the *build system*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Response {
    /// the *request* was carried out
    Acknowledged,
    /// a *diagnostic* response: the resulting *document version* and the message text of every
    /// *diagnostic*
    Diagnostics { version: u64, messages: Vec<String> },
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

// realises IR-018, IR-019
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
        Response::Diagnostics { version, messages } => {
            pairs.push((KEY_KIND, unsigned(KIND_DIAGNOSTICS)));
            pairs.push((KEY_VERSION, unsigned(*version)));
            let origin = Value::Array(vec![unsigned(1), unsigned(1)]);
            let range = Value::Array(vec![origin.clone(), origin]);
            pairs.push((
                KEY_DIAGNOSTICS,
                Value::Array(
                    messages
                        .iter()
                        .map(|message| {
                            Value::Array(vec![
                                unsigned(1),
                                range.clone(),
                                Value::Text(message.clone()),
                            ])
                        })
                        .collect(),
                ),
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

// realises IR-018, IR-020
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
            messages: array_under(&pairs, KEY_DIAGNOSTICS)?
                .iter()
                .map(message_of_diagnostic)
                .collect::<Option<Vec<String>>>()
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

/// Yields an *edit delta* as the array `[[start, end], text]` of the message schema.
fn value_of_delta(delta: &EditDelta) -> Value {
    let position = |position: &ReadPosition| {
        Value::Array(vec![unsigned(position.line), unsigned(position.column)])
    };
    Value::Array(vec![
        Value::Array(vec![position(&delta.start), position(&delta.end)]),
        Value::Text(delta.text.clone()),
    ])
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

/// Yields the message text of a *diagnostic* `[severity, range, text]`.
fn message_of_diagnostic(value: &Value) -> Option<String> {
    match value {
        Value::Array(elements) if elements.len() == 3 => text_of(&elements[2]),
        _ => None,
    }
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
 * covers bridge    : NetworkEditor::apply_increment, through build_system */
#[cfg(test)]
mod tests {
    use super::*;

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
                messages: vec!["fault".to_owned()],
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
