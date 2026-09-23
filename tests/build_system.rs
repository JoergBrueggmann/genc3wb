//! Integration tests of the build system client against a build system and `dot`.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use genc3wb::core::api_message::{NodeKind, has_error};
use genc3wb::core::network_builder::NetworkBuilder;
use genc3wb::core::text_increment::increment_between;

use std::fs;
use std::path::PathBuf;

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

// FR-067, FR-069, FR-070, FR-073, FR-075, FR-077, IR-017 to IR-022
// needs the service executable genc3d of genc³ 0.19.0.0 or later, named by the environment
// variable GENC3D, and the program dot of Graphviz
#[test]
#[ignore]
fn build_yields_the_nodes_and_the_laid_out_graph() {
    let directory = case_directory("build", TWO_NODES);
    let network_path = directory.join("n.gc3n").to_string_lossy().into_owned();
    let mut builder = NetworkBuilder::new(directory.join("scratch"));
    let started = builder.restart(&build_system(), &network_path);
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
            graph.image_path.is_file(),
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
                5,
                true
            ))
        )
    );
}

// FR-069, FR-070, FR-071, FR-073, FR-074, FR-128
// needs the service executable genc3d, named by the environment variable GENC3D, and dot
#[test]
#[ignore]
fn faulty_edit_fails_with_the_fault_and_its_correction_succeeds_again() {
    let directory = case_directory("edit", TWO_NODES);
    let network_path = directory.join("n.gc3n").to_string_lossy().into_owned();
    let mut builder = NetworkBuilder::new(directory.join("scratch"));
    let _ = builder.restart(&build_system(), &network_path);
    let faulty = TWO_NODES.replace("node b", "nod b");
    let first = builder.build("", &increment_between("", TWO_NODES), TWO_NODES);
    let second = builder.build(TWO_NODES, &increment_between(TWO_NODES, &faulty), &faulty);
    let third = builder.build(&faulty, &increment_between(&faulty, TWO_NODES), TWO_NODES);
    let marked = second.diagnostics.iter().any(|diagnostic| {
        diagnostic.start.line == 7 && diagnostic.end.column > diagnostic.start.column
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

/// A *meta compiler DSL* whose *input* is copied to its *output*: a syntax matching the letters of
/// 'hello world', and a generator emitting the matched text.
const COPY_META_DSL: &str = "syntax\n    root = chars, EOS\n    chars = ch, { ch }\n    \
    ch = 'h' | 'e' | 'l' | 'o' | 'w' | 'r' | 'd' | ' '\n\
    generator copy input \"a.in\" output \"x.txt\"\n    root => flat(#1)\n";

// FR-104 to FR-109, FR-111, IR-026 to IR-028
// needs the service executable genc3d of genc³ 0.19.0.0 or later, named by the environment
// variable GENC3D
#[test]
#[ignore]
fn node_is_served_and_answers_a_change_with_diagnostics_and_a_store() {
    use genc3wb::core::node_runner::{NodeRunner, NodeStart};

    let directory = case_directory("node", TWO_NODES);
    fs::write(directory.join("a.gc3"), COPY_META_DSL)
        .expect("the meta compiler DSL can be written");
    fs::write(directory.join("a.in"), "hello").expect("the input can be written");
    let mut runner = NodeRunner::new(directory.join("scratch"));
    let started = runner.start(&NodeStart {
        executable: build_system(),
        directory: directory.clone(),
        meta_dsl: "a.gc3".to_owned(),
        inputs: vec!["a.in".to_owned()],
        outputs: vec!["x.txt".to_owned()],
    });
    let transmitted = runner.transmit(
        "a.in",
        "",
        &increment_between("", "hello world"),
        "hello world",
    );
    let stored = runner.store();
    let output = fs::read_to_string(directory.join("x.txt")).ok();
    let faulty = runner.transmit(
        "a.in",
        "hello world",
        &increment_between("hello world", "hello x"),
        "hello x",
    );
    drop(runner);
    let _ = fs::remove_dir_all(&directory);
    assert_eq!(
        (
            started,
            transmitted,
            stored,
            output,
            faulty.map(|diagnostics| has_error(&diagnostics))
        ),
        (
            Ok(()),
            Ok(vec![]),
            Ok(()),
            Some("hello world".to_owned()),
            Ok(true)
        )
    );
}
