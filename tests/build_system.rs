//! Integration tests of the build system client against a build system and `dot`.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use genc3wb::core::api_message::{NetworkMode, NodeKind, NodeMode, has_error};
use genc3wb::core::build_system::{BuildError, DiagnosticAnswer, StateSink, Watched};
use genc3wb::core::network_builder::{NetworkBuilder, StateSinks};
use genc3wb::core::node_runner::{NodeAttach, NodeRunner, groups_of_state};
use genc3wb::core::text_increment::{TextIncrement, increment_between};

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender, channel};
use std::time::{Duration, Instant};

/// The time a case waits for the network to tell or to hold what it expects.
const PATIENCE: Duration = Duration::from_secs(20);

/// A *network file* of two *nodes*, the *output* of the one the *input* of the other.
const TWO_NODES: &str = "meta compiler-compiler network n.gc3n\n\
    node a\n  inputs\n    a.in\n  outputs\n    x.txt\n\
    node b\n  inputs\n    x.txt\n  outputs\n    y.txt\n  transformation proxy make\n";

/// Yields the path of the build system: the environment variable `GENC3D`.
fn build_system() -> String {
    std::env::var("GENC3D").expect("GENC3D names the service executable genc3d")
}

/// Yields a directory unique to `case`, holding the *network file* 'n.gc3n' with `text`.
fn case_directory(case: &str, text: &str) -> PathBuf {
    let directory = std::env::temp_dir().join(format!("g3wb-{}-{case}", std::process::id()));
    fs::create_dir_all(&directory).expect("the case directory can be created");
    fs::write(directory.join("n.gc3n"), text).expect("the network file can be written");
    directory
}

/// Writes the file `name` with `content` into `directory`.
fn write(directory: &Path, name: &str, content: &str) {
    fs::write(directory.join(name), content).expect("the file of the case can be written");
}

/// What a watch told, as the sink of a case receives it.
type Told = Result<Watched, BuildError>;

/// Yields a sink that hands what it is told to `told`.
fn sink_to(told: Sender<Told>) -> StateSink {
    Box::new(move |watched| {
        let _ = told.send(watched);
    })
}

/// Yields the sinks of a builder that tells nothing to the case.
fn no_sinks() -> StateSinks {
    Box::new(|| Box::new(|_| {}))
}

/// Yields the sinks of a builder that hand what the *build system* tells to `told`.
fn sinks_to(told: Sender<Told>) -> StateSinks {
    Box::new(move || sink_to(told.clone()))
}

/// Yields the first *state response* of `told` that `expected` holds for, `None` where none
/// arrives within [`PATIENCE`] or the connection is lost.
fn awaited(told: &Receiver<Told>, expected: impl Fn(&Watched) -> bool) -> Option<Watched> {
    let end = Instant::now() + PATIENCE;
    while let Ok(Ok(watched)) = told.recv_timeout(end.saturating_duration_since(Instant::now())) {
        if expected(&watched) {
            return Some(watched);
        }
    }
    None
}

/// Yields whether `told` tells the loss of its connection within [`PATIENCE`], which it does
/// once the *node* attached to has terminated.
fn awaited_loss(told: &Receiver<Told>) -> bool {
    let end = Instant::now() + PATIENCE;
    while let Ok(told) = told.recv_timeout(end.saturating_duration_since(Instant::now())) {
        if told.is_err() {
            return true;
        }
    }
    false
}

/// Starts the *build system* on the *network file* of `directory`, so that it runs every *node*
/// of it, with `sinks` for its watch.
fn started(directory: &Path, sinks: StateSinks) -> (NetworkBuilder, Result<(), BuildError>) {
    let network_path = directory.join("n.gc3n").to_string_lossy().into_owned();
    let mut builder = NetworkBuilder::new(directory.join("s"), sinks);
    let started = builder.restart(&build_system(), &network_path);
    (builder, started)
}

/// Attaches a runner to the *node* `name` of the network in `directory`, by the
/// *service socket* the *network notation* gives it where the *network file* states none,
/// `<name>.gc3.sock` (\[AD5\] FR-142), trying until the *build system* has started the *node*.
fn attached(
    directory: &Path,
    name: &str,
    inputs: &[&str],
    told: &Sender<Told>,
) -> (NodeRunner, Result<(), BuildError>) {
    let attach = NodeAttach {
        socket: directory.join(format!("{name}.gc3.sock")),
        inputs: inputs.iter().map(|input| (*input).to_owned()).collect(),
    };
    let mut runner = NodeRunner::new();
    let end = Instant::now() + PATIENCE;
    let mut outcome = runner.attach(&attach, sink_to(told.clone()));
    while outcome.is_err() && Instant::now() < end {
        outcome = runner.attach(&attach, sink_to(told.clone()));
    }
    (runner, outcome)
}

/// Yields the content the *node* of `runner` holds of `document` once it is `expected`, and
/// the content it held last where it does not come to be within [`PATIENCE`].
fn content_awaited(runner: &mut NodeRunner, document: &str, expected: &[u8]) -> Option<Vec<u8>> {
    let end = Instant::now() + PATIENCE;
    loop {
        let content = runner
            .query_content(document)
            .ok()
            .flatten()
            .map(|content| content.octets);
        if content.as_deref() == Some(expected) || Instant::now() >= end {
            return content;
        }
        std::thread::sleep(Duration::from_millis(50));
    }
}

// FR-067, FR-069, FR-070, FR-073, FR-075, FR-077, IR-017 to IR-022
// needs the service executable genc3d of genc³ 0.30.2.1 or later, named by the environment
// variable GENC3D, and the program dot of Graphviz
#[test]
#[ignore]
fn build_yields_the_nodes_and_the_laid_out_graph() {
    let directory = case_directory("build", TWO_NODES);
    write(&directory, "a.gc3", COPY_META_DSL);
    write(&directory, "a.in", "hello");
    let (mut builder, started) = started(&directory, no_sinks());
    let built = builder
        .build("", &increment_between("", TWO_NODES), TWO_NODES)
        .graph;
    let summary = built.as_ref().map(|graph| {
        (
            graph
                .nodes
                .iter()
                .map(|node| (node.name.clone(), node.kind))
                .collect::<Vec<_>>(),
            graph.layout.vertices.len(),
            graph.served_image_path.is_file() && graph.invalid_image_path.is_file(),
        )
    });
    drop(builder);
    let _ = fs::remove_dir_all(&directory);
    assert_eq!(
        (started, summary),
        (
            Ok(()),
            Ok((
                vec![
                    ("a".to_owned(), NodeKind::MetaCompilerCompiler),
                    ("b".to_owned(), NodeKind::ProxyNode)
                ],
                // a, b, a.gc3, a.in, x.txt, y.txt: the meta compiler DSL of a is a vertex (FR-075)
                6,
                true
            ))
        )
    );
}

// FR-069, FR-070, FR-071, FR-073, FR-074, FR-128
// needs the service executable genc3d of genc³ 0.30.2.1 or later, named by the environment
// variable GENC3D, and dot
#[test]
#[ignore]
fn faulty_edit_fails_with_the_fault_and_its_correction_succeeds_again() {
    let directory = case_directory("edit", TWO_NODES);
    write(&directory, "a.gc3", COPY_META_DSL);
    write(&directory, "a.in", "hello");
    let (mut builder, _) = started(&directory, no_sinks());
    let faulty = TWO_NODES.replace("node b", "nod b");
    let first = builder.build("", &increment_between("", TWO_NODES), TWO_NODES);
    let second = builder.build(TWO_NODES, &increment_between(TWO_NODES, &faulty), &faulty);
    let third = builder.build(&faulty, &increment_between(&faulty, TWO_NODES), TWO_NODES);
    let marked = second.diagnostics.iter().any(|diagnostic| {
        diagnostic
            .read_positions()
            .is_some_and(|(start, end)| start.line == 7 && end.column > start.column)
    });
    drop(builder);
    let _ = fs::remove_dir_all(&directory);
    assert_eq!(
        (
            first.graph.is_ok(),
            second
                .graph
                .as_ref()
                .err()
                .map(|message| !message.is_empty()),
            marked,
            third.graph.map(|graph| graph.nodes.len())
        ),
        (true, Some(true), true, Ok(2))
    );
}

/// A *network file* of one *node*.
const ONE_NODE: &str = "meta compiler-compiler network n.gc3n\n\
    node a\n  inputs\n    a.in\n  outputs\n    x.txt\n";

/// A *network file* of two *nodes* in which the *output* of `a` is the *meta compiler DSL* of
/// `b`: `b` is started with `--meta-dsl-produced`, waits for `a` and is activated after it.
const PRODUCED_META_DSL: &str = "meta compiler-compiler network n.gc3n\n\
    node a\n  inputs\n    a.in\n  outputs\n    b.gc3\n\
    node b\n  inputs\n    b.in\n  outputs\n    out.txt\n";

/// A *meta compiler DSL* of `a` whose *output* is the *meta compiler DSL* of `b`: a generator
/// emitting the text of [`COPY_META_DSL`] as a literal, so that `b` copies its *input* to its
/// *output*.
fn producing_meta_dsl() -> String {
    let produced = COPY_META_DSL
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n");
    format!(
        "syntax\n    Root = Chars, EOS\n    Chars = Ch, {{ Ch }}\n    \
         Ch = 'h' | 'e' | 'l' | 'o' | 'w' | 'r' | 'd' | ' '\n\
         generator\n    Root -> \"{produced}\"\n"
    )
}

/// A *network file* of a chain of three *nodes*: the *output* of `a` is the *input* of `b`, and
/// the *output* of `b` the *input* of `c`.
const CHAIN: &str = "meta compiler-compiler network n.gc3n\n\
    node a\n  inputs\n    a.in\n  outputs\n    x.txt\n\
    node b\n  inputs\n    x.txt\n  outputs\n    y.txt\n\
    node c\n  inputs\n    y.txt\n  outputs\n    z.txt\n";

/// A *meta compiler DSL* whose *input* is copied to its *output*: a syntax matching the letters
/// of 'hello world', and a generator emitting the matched text; it names no file, the *node*
/// does.
const COPY_META_DSL: &str = "syntax\n    Root = Chars, EOS\n    Chars = Ch, { Ch }\n    \
    Ch = 'h' | 'e' | 'l' | 'o' | 'w' | 'r' | 'd' | ' '\n\
    generator\n    Root -> flat(#0)\n";

/// Writes the files of the chain of [`CHAIN`] into a directory unique to `case`: the three
/// *meta compiler DSLs*, each copying its *input* to its *output*, and the *source input*
/// 'a.in' holding 'hello'.
fn chain_directory(case: &str) -> PathBuf {
    let directory = case_directory(case, CHAIN);
    write(&directory, "a.gc3", COPY_META_DSL);
    write(&directory, "b.gc3", COPY_META_DSL);
    write(&directory, "c.gc3", COPY_META_DSL);
    write(&directory, "a.in", "hello");
    directory
}

// FR-104 to FR-109, FR-110, FR-230, FR-252, IR-026 to IR-028, IR-038
// needs the service executable genc3d of genc³ 0.30.2.1 or later, named by the environment
// variable GENC3D
#[test]
#[ignore]
fn node_is_attached_to_and_answers_a_change_with_diagnostics_and_the_store_writes_its_output() {
    let directory = case_directory("node", ONE_NODE);
    write(&directory, "a.gc3", COPY_META_DSL);
    write(&directory, "a.in", "hello");
    let (mut builder, started) = started(&directory, no_sinks());
    let (told, _states) = channel();
    let (mut runner, attached) = attached(&directory, "a", &["a.in"], &told);
    let transmitted = runner.transmit(
        "a.in",
        "",
        &increment_between("", "hello world"),
        "hello world",
    );
    let stored = builder.store();
    let output = fs::read_to_string(directory.join("x.txt")).ok();
    let faulty = runner.transmit(
        "a.in",
        "hello world",
        &increment_between("hello world", "hello x"),
        "hello x",
    );
    drop(runner);
    drop(builder);
    let _ = fs::remove_dir_all(&directory);
    assert_eq!(
        (
            started,
            attached,
            transmitted,
            stored,
            output,
            faulty.map(|answer| has_error(&answer.diagnostics))
        ),
        (
            Ok(()),
            Ok(()),
            Ok(DiagnosticAnswer::default()),
            Ok(()),
            Some("hello world".to_owned()),
            Ok(true)
        )
    );
}

// FR-231, FR-242, IR-027, IR-028
// needs the service executable genc3d of genc³ 0.30.2.1 or later, named by the environment
// variable GENC3D
#[test]
#[ignore]
fn diagnostics_of_an_empty_input_are_told_and_taken_without_an_edit() {
    // the syntax asks for one character at least, and the file of the input is empty
    let directory = case_directory("empty", ONE_NODE);
    write(&directory, "a.gc3", COPY_META_DSL);
    write(&directory, "a.in", "");
    let (builder, started) = started(&directory, no_sinks());
    let (told, states) = channel();
    let (mut runner, attached) = attached(&directory, "a", &["a.in"], &told);
    // the document is registered as the node window registers it when it attaches: by an
    // open request carrying the empty text, no edit being made
    let registered = runner.transmit("a.in", "", &TextIncrement::default(), "");
    let held: BTreeMap<String, u64> = registered
        .iter()
        .map(|answer| ("a.in".to_owned(), answer.version))
        .collect();
    let taken = awaited(&states, |watched| {
        groups_of_state(watched.groups.clone(), &held, &[])
            .iter()
            .any(|group| group.document == "a.in" && has_error(&group.diagnostics))
    });
    drop(runner);
    drop(builder);
    let _ = fs::remove_dir_all(&directory);
    assert_eq!(
        (
            (started, attached),
            registered.map(|answer| has_error(&answer.diagnostics)),
            taken.is_some()
        ),
        ((Ok(()), Ok(())), Ok(true), true)
    );
}

/// A *meta compiler DSL* whose *input* is octets beginning with the octet `0xFF`, and whose
/// *binary output* is the complement of every octet after it.
const COMPLEMENT_META_DSL: &str = "syntax\n    Root = 0xFF, Octets, EOS\n    \
    Octets = Octets, Octet | Octet\n    Octet = bits 8\n\
    generator binary\n    Root -> #1\n    \
    Octets -> #0 ++ [#1] | -> [#0]\n    Octet :: U8\n    Octet -> fromInteger(255 - #0)\n";

// FR-107, FR-109, FR-111, FR-167, FR-169, FR-170, FR-210, IR-027, IR-028
// needs the service executable genc3d of genc³ 0.30.2.1 or later, named by the environment
// variable GENC3D
#[test]
#[ignore]
fn octets_are_served_as_a_binary_document_with_octet_deltas_stores_and_diagnostics_in_bits() {
    use genc3wb::core::api_message::bit_marks_of_diagnostics;

    let directory = case_directory("octets", ONE_NODE);
    write(&directory, "a.gc3", COMPLEMENT_META_DSL);
    fs::write(directory.join("a.in"), [0xff, 0x01]).expect("the input can be written");
    let (mut builder, started) = started(&directory, no_sinks());
    let (told, _states) = channel();
    let (mut runner, attached) = attached(&directory, "a", &["a.in"], &told);
    let opened = runner.open_octets("a.in", &[0xff, 0x00, 0x80, 0xfe]);
    let stored = builder.store();
    let output = fs::read(directory.join("x.txt")).ok();
    let edited = runner.edit_octets("a.in", &[0xff, 0x00, 0x80, 0xfe], &[0xff, 0x00, 0x0f, 0xfe]);
    let restored = builder.store();
    let output_after_edit = fs::read(directory.join("x.txt")).ok();
    let content = runner
        .query_content("x.txt")
        .map(|content| content.map(|content| (content.octets, content.binary)));
    let faulty = runner.edit_octets("a.in", &[0xff, 0x00, 0x0f, 0xfe], &[0xff]);
    let marks = faulty
        .as_ref()
        .map(|answer| bit_marks_of_diagnostics(&answer.diagnostics));
    let unmarked_in_txt = faulty.as_ref().map(|answer| {
        answer
            .diagnostics
            .iter()
            .any(|diagnostic| diagnostic.read_positions().is_none())
    });
    drop(runner);
    drop(builder);
    let _ = fs::remove_dir_all(&directory);
    assert_eq!(
        (
            (started, attached),
            opened.map(|answer| answer.erroneous),
            stored,
            output,
            (
                edited.map(|answer| answer.erroneous),
                restored,
                output_after_edit,
                content
            ),
            faulty.as_ref().map(|answer| has_error(&answer.diagnostics)),
            unmarked_in_txt,
            marks.map(|marks| marks.starts)
        ),
        (
            (Ok(()), Ok(())),
            Ok(false),
            Ok(()),
            Some(vec![0xff, 0x7f, 0x01]),
            (
                Ok(false),
                Ok(()),
                Some(vec![0xff, 0xf0, 0x01]),
                Ok(Some((vec![0xff, 0xf0, 0x01], true)))
            ),
            Ok(true),
            Ok(true),
            Ok(vec![8])
        )
    );
}

// FR-211, FR-112
// needs the service executable genc3d of genc³ 0.30.2.1 or later, named by the environment
// variable GENC3D
#[test]
#[ignore]
fn store_fails_where_the_directory_of_the_network_was_deleted_and_created_anew() {
    let directory = case_directory("lost", ONE_NODE);
    write(&directory, "a.gc3", COPY_META_DSL);
    write(&directory, "a.in", "hello");
    let (mut builder, started) = started(&directory, no_sinks());
    let (told, _states) = channel();
    let (mut runner, attached) = attached(&directory, "a", &["a.in"], &told);
    let stored_first = builder.store();
    fs::remove_dir_all(&directory).expect("the directory can be deleted");
    fs::create_dir_all(&directory).expect("the directory can be created anew");
    let transmitted = runner.transmit(
        "a.in",
        "",
        &increment_between("", "hello world"),
        "hello world",
    );
    let stored = builder.store();
    let output = fs::read_to_string(directory.join("x.txt")).ok();
    drop(runner);
    drop(builder);
    let _ = fs::remove_dir_all(&directory);
    // the build system passes the store request on to the node, whose working directory is
    // gone: the failure is told with a text, which is the one of the build system
    assert_eq!(
        (
            started,
            attached,
            stored_first,
            transmitted,
            stored.err().map(|message| !message.is_empty()),
            output
        ),
        (
            Ok(()),
            Ok(()),
            Ok(()),
            Ok(DiagnosticAnswer::default()),
            Some(true),
            None
        )
    );
}

// FR-067, FR-228, FR-229, FR-230, FR-231, FR-234, IR-017, IR-019, IR-020, IR-027, IR-028
// needs the service executable genc3d of genc³ 0.30.2.1 or later, named by the environment
// variable GENC3D
#[test]
#[ignore]
fn error_of_a_node_is_told_with_its_details_there_and_as_its_origin_downstream_until_corrected() {
    let directory = chain_directory("chain");
    let (network_told, network_states) = channel();
    let (builder, started) = started(&directory, sinks_to(network_told));
    let (node_told, node_states) = channel();
    let (mut runner, attached) = attached(&directory, "a", &["a.in"], &node_told);
    // the network settles first: while it is activated, b and c count errors for the produced
    // inputs that do not exist yet, which the activation clears
    let settled = awaited(&network_states, |watched| {
        watched.mode == Some(NetworkMode::Active)
            && watched.states.iter().all(|state| {
                state.errors == 0
                    && state.origins.is_empty()
                    && state.mode == Some(NodeMode::Active)
            })
    });
    // the source input of a is made erroneous: 'x' is no letter of its syntax; the meta
    // compiler DSL stays valid, since an invalid one would make a an invalid node, on which the
    // factory deactivates the network ([AD3] IR-167)
    let faulty = runner.transmit("a.in", "", &increment_between("", "hello x"), "hello x");
    // a tells its error to b, and b passes it on to c, with its own name where its input, the
    // output of a, is erroneous as well
    let erroneous = awaited(&network_states, |watched| {
        watched
            .states
            .iter()
            .any(|state| state.name == "a" && state.errors > 0)
            && watched
                .states
                .iter()
                .any(|state| state.name == "b" && state.origins.contains(&"a".to_owned()))
            && watched
                .states
                .iter()
                .any(|state| state.name == "c" && state.origins.contains(&"a".to_owned()))
    });
    // the network stays active while a counts an error in its input, its configuration
    // being valid (FR-229, [AD3] IR-123, IR-129)
    let modes = erroneous.as_ref().map(|watched| {
        (
            watched.mode,
            watched
                .states
                .iter()
                .map(|state| state.mode)
                .collect::<Vec<_>>(),
        )
    });
    let details = awaited(&node_states, |watched| {
        watched.states.iter().any(|state| state.errors > 0)
            && watched
                .groups
                .iter()
                .any(|group| group.document == "a.in" && has_error(&group.diagnostics))
    });
    let corrected = runner.transmit(
        "a.in",
        "hello x",
        &increment_between("hello x", "hello"),
        "hello",
    );
    let clear = awaited(&network_states, |watched| {
        watched
            .states
            .iter()
            .all(|state| state.errors == 0 && state.origins.is_empty())
    });
    let clear_at_the_node = awaited(&node_states, |watched| {
        watched
            .states
            .iter()
            .all(|state| state.errors == 0 && state.origins.is_empty())
            && watched
                .groups
                .iter()
                .all(|group| group.diagnostics.is_empty())
    });
    drop(runner);
    drop(builder);
    let _ = fs::remove_dir_all(&directory);
    assert_eq!(
        (
            (started, attached, settled.is_some()),
            faulty.map(|answer| answer.erroneous),
            erroneous.map(|watched| {
                watched
                    .states
                    .iter()
                    .map(|state| state.name.clone())
                    .collect::<Vec<_>>()
            }),
            modes,
            details.is_some(),
            corrected.map(|answer| answer.erroneous),
            clear.map(|watched| watched.states.len()),
            clear_at_the_node.is_some()
        ),
        (
            (Ok(()), Ok(()), true),
            Ok(true),
            Some(vec!["a".to_owned(), "b".to_owned(), "c".to_owned()]),
            Some((Some(NetworkMode::Active), vec![Some(NodeMode::Active); 3])),
            true,
            Ok(false),
            Some(3),
            true
        )
    );
}

// FR-067, FR-229, FR-258 to FR-261, IR-017, IR-019, IR-020
// needs the service executable genc3d of genc³ 0.30.2.1 or later, named by the environment
// variable GENC3D
#[test]
#[ignore]
fn activation_fails_while_a_node_is_invalid_and_succeeds_once_it_is_corrected() {
    // the meta compiler DSL of the one node is no meta compiler DSL: the activation at
    // start-up ends with the node told invalid, and the network stays a valid network
    let directory = case_directory("activate", ONE_NODE);
    write(&directory, "a.gc3", "this is no meta compiler DSL\n");
    write(&directory, "a.in", "hello");
    let (network_told, network_states) = channel();
    let (mut builder, started) = started(&directory, sinks_to(network_told));
    let invalid = awaited(&network_states, |watched| {
        watched.mode == Some(NetworkMode::Valid)
            && watched
                .states
                .iter()
                .any(|state| state.name == "a" && state.mode == Some(NodeMode::Invalid))
    });
    // the factory shuts the invalid node down and remains a valid network
    let shut_down = awaited(&network_states, |watched| {
        watched.mode == Some(NetworkMode::Valid)
            && watched
                .states
                .iter()
                .any(|state| state.name == "a" && state.mode.is_none())
    });
    let refused = builder.activate();
    write(&directory, "a.gc3", COPY_META_DSL);
    let activated = builder.activate();
    let active = awaited(&network_states, |watched| {
        watched.mode == Some(NetworkMode::Active)
            && watched
                .states
                .iter()
                .all(|state| state.mode == Some(NodeMode::Active))
    });
    // the node tells its own node mode in its node state (FR-123, [AD3] IR-121, IR-123)
    let (node_told, node_states) = channel();
    let (runner, attached) = attached(&directory, "a", &["a.in"], &node_told);
    let node_active = awaited(&node_states, |watched| {
        watched.mode.is_none()
            && watched
                .states
                .first()
                .is_some_and(|state| state.mode == Some(NodeMode::Active))
    });
    let again = builder.activate();
    // the deactivate request ends every node: the network is a valid network again, no node
    // runs, and the attachment to the node is lost (FR-267, [AD3] IR-164, IR-165)
    let deactivated = builder.deactivate();
    let deactivated_network = awaited(&network_states, |watched| {
        watched.mode == Some(NetworkMode::Valid)
            && watched.states.iter().all(|state| state.mode.is_none())
    });
    let node_lost = awaited_loss(&node_states);
    // a deactivate request while the network is not active is acknowledged and changes nothing
    // ([AD3] IR-166)
    let deactivated_again = builder.deactivate();
    drop(runner);
    drop(builder);
    let _ = fs::remove_dir_all(&directory);
    assert_eq!(
        (
            started,
            invalid.is_some(),
            shut_down.is_some(),
            refused.err().map(|message| message.contains('a')),
            activated,
            active.map(|watched| watched.states.len()),
            attached,
            node_active.is_some(),
            again
        ),
        (
            Ok(()),
            true,
            true,
            Some(true),
            Ok(()),
            Some(1),
            Ok(()),
            true,
            Ok(())
        )
    );
    assert_eq!(
        (
            deactivated,
            deactivated_network.is_some(),
            node_lost,
            deactivated_again
        ),
        (Ok(()), true, true, Ok(()))
    );
}

// FR-281, FR-282, FR-283, FR-286, FR-104, [AD3] IR-194 to IR-204
// needs the service executable genc3d of genc³ 0.32.1.0 or later, named by the environment
// variable GENC3D
#[test]
#[ignore]
fn node_held_in_a_valid_network_is_served_and_released() {
    // the network activates at start-up; the deactivation ends the node, which the hold request
    // starts again and serves while the network stays a valid network
    let directory = case_directory("hold", ONE_NODE);
    write(&directory, "a.gc3", COPY_META_DSL);
    write(&directory, "a.in", "hello");
    let (network_told, network_states) = channel();
    let (mut builder, started) = started(&directory, sinks_to(network_told));
    let active = awaited(&network_states, |watched| {
        watched.mode == Some(NetworkMode::Active)
    });
    let deactivated = builder.deactivate();
    let valid_without_nodes = awaited(&network_states, |watched| {
        watched.mode == Some(NetworkMode::Valid)
            && watched.states.iter().all(|state| state.mode.is_none())
    });
    // a hold request names a node the network does not state ([AD3] IR-200)
    let refused = builder.hold("zzz");
    // the hold request is acknowledged once the started node has told its node mode, and the
    // node is activated alone while the network stays a valid network ([AD3] IR-196 to IR-198)
    let held = builder.hold("a");
    let held_active = awaited(&network_states, |watched| {
        watched.mode == Some(NetworkMode::Valid)
            && watched
                .states
                .iter()
                .any(|state| state.name == "a" && state.mode == Some(NodeMode::Active))
    });
    // the held node is served: it answers a change and a content query (FR-104)
    let (node_told, node_states) = channel();
    let (mut runner, attached) = attached(&directory, "a", &["a.in"], &node_told);
    let answered = runner
        .transmit(
            "a.in",
            "",
            &increment_between("", "hello world"),
            "hello world",
        )
        .map(|answer| answer.erroneous);
    let content = content_awaited(&mut runner, "x.txt", b"hello world");
    // a hold of a node that runs is acknowledged at once ([AD3] IR-199)
    let held_again = builder.hold("a");
    // the release ends the node, the network staying a valid network ([AD3] IR-201)
    let released = builder.release("a");
    let node_lost = awaited_loss(&node_states);
    let ended = awaited(&network_states, |watched| {
        watched.mode == Some(NetworkMode::Valid)
            && watched.states.iter().all(|state| state.mode.is_none())
    });
    // a release of a node not held is acknowledged ([AD3] IR-202)
    let released_again = builder.release("a");
    drop(runner);
    drop(builder);
    let _ = fs::remove_dir_all(&directory);
    assert_eq!(
        (
            started,
            active.is_some(),
            deactivated,
            valid_without_nodes.is_some(),
            refused.err().map(|message| message.contains("zzz")),
            held,
            held_active.is_some()
        ),
        (Ok(()), true, Ok(()), true, Some(true), Ok(()), true)
    );
    assert_eq!(
        (
            attached,
            answered,
            content,
            held_again,
            released,
            node_lost,
            ended.is_some(),
            released_again
        ),
        (
            Ok(()),
            Ok(false),
            Some(b"hello world".to_vec()),
            Ok(()),
            Ok(()),
            true,
            true,
            Ok(())
        )
    );
}

/// A *network file* of one *node* with two *inputs*, which its *meta compiler DSL* parses
/// together.
const JOIN: &str = "meta compiler-compiler network n.gc3n\n\
    node join\n  inputs\n    names.txt\n    greeting.txt\n  \
    outputs\n    letters.txt\n    both.txt\n";

/// The *meta compiler DSL* of the *node* `join`, with an *input section* of two *parts*: every
/// name of the first *input* is greeted with the word of the second.
const JOIN_META_DSL: &str = "input as\n    NamesFile, GreetingFile\nsyntax\n    \
    Names = NameLines, EOS\n    NameLines = NameLine, NameLines | ()\n    \
    NameLine! = Name, NEWLINE\n    Name! = Letter, { Letter }\n    \
    Letter = 'a' .. 'z' | 'A' .. 'Z'\n    \
    Greeting = Word, [ NEWLINE ], EOS\n    Word! = Letter, { Letter }\n\n\
    generator from input by (Names, Greeting)\n    input -> text(#0)\n    Names -> text(#0)\n    \
    NameLines -> text(#0) ++ text(#1) | -> \"\"\n    \
    NameLine -> flat(@~/Greeting/Word) ++ \", \" ++ flat(#0) ++ \"!\\n\"\n\n\
    generator from input by (Names, Greeting)\n    \
    input -> \"names:\\n\" ++ flat(#0) ++ \"greeting:\\n\" ++ flat(#1)\n";
// FR-225, FR-231, IR-028
// needs the service executable genc3d of genc³ 0.30.2.1 or later, named by the environment
// variable GENC3D
#[test]
#[ignore]
fn error_in_the_second_input_is_told_in_its_group_when_the_first_input_is_edited() {
    let directory = case_directory("join", JOIN);
    write(&directory, "join.gc3", JOIN_META_DSL);
    write(&directory, "names.txt", "Ada\n");
    write(&directory, "greeting.txt", "Hello\n");
    let (builder, started) = started(&directory, no_sinks());
    let (told, states) = channel();
    let (mut runner, attached) =
        attached(&directory, "join", &["names.txt", "greeting.txt"], &told);
    // the second input holds a digit, which is no letter
    let second = runner.transmit(
        "greeting.txt",
        "",
        &increment_between("", "Hello1\n"),
        "Hello1\n",
    );
    let first = runner.transmit(
        "names.txt",
        "",
        &increment_between("", "Ada\nGrace\n"),
        "Ada\nGrace\n",
    );
    let watched = awaited(&states, |watched| {
        watched
            .groups
            .iter()
            .any(|group| group.document == "greeting.txt" && has_error(&group.diagnostics))
    });
    drop(runner);
    drop(builder);
    let _ = fs::remove_dir_all(&directory);
    assert_eq!(
        (
            (started, attached),
            second.map(|answer| has_error(&answer.diagnostics)),
            first.map(|answer| {
                (
                    has_error(&answer.diagnostics),
                    answer
                        .groups
                        .iter()
                        .filter(|group| has_error(&group.diagnostics))
                        .map(|group| group.document.clone())
                        .collect::<Vec<_>>(),
                    answer.erroneous,
                )
            }),
            watched.is_some()
        ),
        (
            (Ok(()), Ok(())),
            Ok(true),
            Ok((false, vec!["greeting.txt".to_owned()], true)),
            true
        )
    );
}

// FR-110, FR-111, FR-239, IR-027, IR-028
// needs the service executable genc3d of genc³ 0.31.0.0 or later, named by the environment
// variable GENC3D
#[test]
#[ignore]
fn content_of_an_input_of_a_producer_and_of_an_output_is_what_the_node_holds() {
    let directory = chain_directory("content");
    let (builder, started) = started(&directory, no_sinks());
    let (told, states) = channel();
    let (mut runner, attached) = attached(&directory, "b", &["x.txt"], &told);
    // the state of b states the document version of its input, of its meta compiler DSL and
    // of its output ([AD3] IR-121)
    let versions = awaited(&states, |watched| watched.versions.len() == 3).map(|watched| {
        watched
            .versions
            .iter()
            .map(|(document, _)| document.clone())
            .collect::<Vec<_>>()
    });
    let input = content_awaited(&mut runner, "x.txt", b"hello");
    let output = content_awaited(&mut runner, "y.txt", b"hello");
    let unknown = runner.query_content("nowhere.txt").is_err();
    drop(runner);
    drop(builder);
    let _ = fs::remove_dir_all(&directory);
    assert_eq!(
        ((started, attached), versions, input, output, unknown),
        (
            (Ok(()), Ok(())),
            Some(vec![
                "x.txt".to_owned(),
                "b.gc3".to_owned(),
                "y.txt".to_owned()
            ]),
            Some(b"hello".to_vec()),
            Some(b"hello".to_vec()),
            true
        )
    );
}

// FR-109, FR-111, FR-239, IR-019
// needs the service executable genc3d of genc³ 0.30.2.1 or later, named by the environment
// variable GENC3D
#[test]
#[ignore]
fn store_through_the_build_system_writes_the_output_of_every_node() {
    let directory = chain_directory("store");
    let (mut builder, started) = started(&directory, no_sinks());
    let (told, _states) = channel();
    let (mut first, attached_first) = attached(&directory, "a", &["a.in"], &told);
    let (mut last, attached_last) = attached(&directory, "c", &["y.txt"], &told);
    let transmitted = first.transmit(
        "a.in",
        "",
        &increment_between("", "hello world"),
        "hello world",
    );
    // the change has passed the chain once the last node holds it as its output
    let passed = content_awaited(&mut last, "z.txt", b"hello world");
    let stored = builder.store();
    let outputs: Vec<Option<String>> = ["x.txt", "y.txt", "z.txt"]
        .iter()
        .map(|name| fs::read_to_string(directory.join(name)).ok())
        .collect();
    drop(first);
    drop(last);
    drop(builder);
    let _ = fs::remove_dir_all(&directory);
    assert_eq!(
        (
            (started, attached_first, attached_last),
            transmitted.map(|answer| answer.erroneous),
            passed,
            stored,
            outputs
        ),
        (
            (Ok(()), Ok(()), Ok(())),
            Ok(false),
            Some(b"hello world".to_vec()),
            Ok(()),
            vec![Some("hello world".to_owned()); 3]
        )
    );
}

// FR-238 to FR-240, FR-123, [AD3] IR-077, IR-158, IR-189, IR-191, IR-192
// needs the service executable genc3d of genc³ 0.31.0.0 or later, named by the environment
// variable GENC3D
#[test]
#[ignore]
fn meta_compiler_dsl_produced_by_another_node_is_received_and_the_network_activates_in_order() {
    let directory = case_directory("produced", PRODUCED_META_DSL);
    write(&directory, "a.gc3", &producing_meta_dsl());
    write(&directory, "a.in", "hello");
    write(&directory, "b.in", "world");
    let (network_told, network_states) = channel();
    let (mut builder, started) = started(&directory, sinks_to(network_told));
    // the factory activates a first and then b, which waits for its meta compiler DSL and tells
    // valid node once a has transmitted it; the network is active once both are
    let active = awaited(&network_states, |watched| {
        watched.mode == Some(NetworkMode::Active)
            && watched.states.len() == 2
            && watched
                .states
                .iter()
                .all(|state| state.mode == Some(NodeMode::Active))
    });
    // the graph shows the meta compiler DSL of b with a as its producer
    let graph = builder
        .build(
            "",
            &increment_between("", PRODUCED_META_DSL),
            PRODUCED_META_DSL,
        )
        .graph;
    let producer = graph.as_ref().ok().and_then(|graph| {
        genc3wb::core::network_graph::files_of_node(&graph.nodes[1], &graph.nodes)
            .map(|files| files.meta_dsl.producer)
    });
    // the node b holds the meta compiler DSL a produced, which the workbench reads as the
    // content of an input of a producer; b.gc3 is not read from a file
    let (node_told, node_states) = channel();
    let (mut runner, attached) = attached(&directory, "b", &["b.in"], &node_told);
    // b tells its own mode active node and the document version of its meta compiler DSL
    // ([AD3] IR-121), by which the workbench fetches the content
    let b_active = awaited(&node_states, |watched| {
        watched
            .states
            .first()
            .is_some_and(|state| state.mode == Some(NodeMode::Active))
            && watched
                .versions
                .iter()
                .any(|(document, _)| document == "b.gc3")
    });
    let meta_dsl = content_awaited(&mut runner, "b.gc3", COPY_META_DSL.as_bytes());
    // the store writes the output of b, copied from its input by the produced meta compiler
    // DSL, and the meta compiler DSL file of b, written by a
    let stored = builder.store();
    let out = fs::read_to_string(directory.join("out.txt"));
    let gc3 = fs::read_to_string(directory.join("b.gc3"));
    drop(runner);
    drop(builder);
    let _ = fs::remove_dir_all(&directory);
    assert_eq!(
        (
            started,
            active.is_some(),
            producer,
            attached,
            b_active.is_some(),
            meta_dsl.is_some(),
            stored,
            out.ok(),
            gc3.ok()
        ),
        (
            Ok(()),
            true,
            Some("a".to_owned()),
            Ok(()),
            true,
            true,
            Ok(()),
            Some("world".to_owned()),
            Some(COPY_META_DSL.to_owned())
        )
    );
}
