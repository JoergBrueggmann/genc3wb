//! Integration tests of the build system client against a build system and `dot`.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use genc3wb::core::api_message::NodeKind;
use genc3wb::core::network_builder::NetworkBuilder;
use genc3wb::core::text_increment::increment_between;

use std::fs;
use std::path::PathBuf;

/// A *network file* of two *nodes*, the *output* of the one the *input* of the other.
const TWO_NODES: &str = "meta compiler-compiler network n.gc3n\n\
    node a\n  inputs\n    a.in\n  outputs\n    x.txt\n\
    node b\n  inputs\n    x.txt\n  outputs\n    y.txt\n  transformation\n    proxy make\n";

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
// needs the service executable genc3d of genc³ 0.18.0.0 or later, named by the environment
// variable GENC3D, and the program dot of Graphviz
#[test]
#[ignore]
fn build_yields_the_nodes_and_the_laid_out_graph() {
    let directory = case_directory("build", TWO_NODES);
    let network_path = directory.join("n.gc3n").to_string_lossy().into_owned();
    let mut builder = NetworkBuilder::new(directory.join("scratch"));
    let started = builder.restart(&build_system(), &network_path);
    let built = builder.build("", &increment_between("", TWO_NODES), TWO_NODES);
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

// FR-069, FR-070, FR-071, FR-073, FR-074
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
    drop(builder);
    let _ = fs::remove_dir_all(&directory);
    assert_eq!(
        (
            first.is_ok(),
            second.as_ref().err().map(|message| !message.is_empty()),
            third.map(|graph| graph.nodes.len())
        ),
        (true, Some(true), Ok(2))
    );
}
