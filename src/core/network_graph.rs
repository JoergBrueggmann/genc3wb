//! The *network graph*: its DOT text, its layout by `dot`, the vertex at a point, and the files of a *node*.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::api_message::{NodeDescription, NodeKind};

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

/// The name of the layout program of Graphviz.
const DOT_PROGRAM: &str = "dot";

/// The resolution of the image in dots per inch: twice the 96 of a display, so that the image,
/// presented at half its pixel size, is sharp on a display of high resolution as well.
pub const IMAGE_DPI: u32 = 192;

/// The directories searched for `dot` after those of the environment variable `PATH`: an
/// application started from the desktop of macOS does not inherit the `PATH` of the shell.
const FALLBACK_DIRECTORIES: [&str; 4] = [
    "/opt/local/bin",
    "/opt/homebrew/bin",
    "/usr/local/bin",
    "/usr/bin",
];

// realises FR-085, FR-088, IR-024
/// The place of one vertex in the layout, in the units of the layout, the origin at the top left.
#[derive(Debug, Clone, PartialEq)]
pub struct VertexBox {
    /// the identifier of the vertex in the DOT text: `n<index>` for a *node*, `f<index>` for a file
    pub identifier: String,
    /// the left edge
    pub left: f64,
    /// the top edge
    pub top: f64,
    /// the width
    pub width: f64,
    /// the height
    pub height: f64,
}

// realises FR-085, IR-022, IR-024
/// The plain-text layout `dot` wrote: the size of the graph and the place of every vertex.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct GraphLayout {
    /// the width of the graph
    pub width: f64,
    /// the height of the graph
    pub height: f64,
    /// the vertices, in the order `dot` wrote them
    pub vertices: Vec<VertexBox>,
}

// realises FR-077, IR-022
/// A *network graph* as `dot` laid it out.
#[derive(Debug, Clone, PartialEq)]
pub struct RenderedGraph {
    /// the file holding the PNG image
    pub image_path: PathBuf,
    /// the layout, by which a point of the image is related to a vertex
    pub layout: GraphLayout,
}

// realises FR-086, FR-087, FR-102, FR-113
/// One file of a *node*: the path as the *node description* carries it, which is its
/// *document identifier* (\[AD5\] IR-044), and, for an *input*, the name of its *producer*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeFile {
    /// the path as written in the *network file*
    pub identifier: String,
    /// the name of the *producer*, empty where the file has none
    pub producer: String,
}

// realises FR-086, FR-087, FR-102, FR-103
/// The files the *node window* presents for one *node*, in the order of the *node description*.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeFiles {
    /// the name of the *node*
    pub name: String,
    /// its *meta compiler DSL*
    pub meta_dsl: NodeFile,
    /// its *inputs*
    pub inputs: Vec<NodeFile>,
    /// its *outputs*
    pub outputs: Vec<NodeFile>,
}

// realises FR-077, C-006
/// Why the *network graph* could not be laid out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GraphError {
    /// `dot` was found neither on the `PATH` nor in a usual directory
    DotNotFound,
    /// `dot` could not be started, or a file could not be read or written; carries the reason
    Io(String),
    /// `dot` ended with a failure; carries what it wrote to standard error
    DotFailed(String),
    /// the plain-text layout of `dot` could not be read; carries the line
    Layout(String),
}

impl fmt::Display for GraphError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphError::DotNotFound => write!(
                formatter,
                "the program 'dot' of Graphviz was not found on the PATH"
            ),
            GraphError::Io(reason) => {
                write!(formatter, "the graph could not be laid out: {reason}")
            }
            GraphError::DotFailed(reason) => write!(formatter, "'dot' failed: {reason}"),
            GraphError::Layout(line) => {
                write!(formatter, "the layout of 'dot' is not read: {line}")
            }
        }
    }
}

impl std::error::Error for GraphError {}

impl GraphLayout {
    // realises IR-022
    /// Reads the plain-text layout of `dot`: the line `graph`, and one line `node` per vertex.
    ///
    /// * `dot` places the origin at the bottom left and names the centre of a vertex; the layout
    ///   yielded places the origin at the top left and names the top left corner.
    ///
    /// # Errors
    /// Returns [`GraphError::Layout`] where the line `graph` is absent or a number is not read.
    pub fn of_plain(plain: &str) -> Result<GraphLayout, GraphError> {
        let mut layout: Option<GraphLayout> = None;
        for line in plain.lines() {
            let words: Vec<&str> = line.split_whitespace().collect();
            let number = |index: usize| -> Result<f64, GraphError> {
                words
                    .get(index)
                    .and_then(|word| word.parse::<f64>().ok())
                    .ok_or_else(|| GraphError::Layout(line.to_owned()))
            };
            match words.first().copied() {
                Some("graph") => {
                    layout = Some(GraphLayout {
                        width: number(2)?,
                        height: number(3)?,
                        vertices: Vec::new(),
                    });
                }
                Some("node") => {
                    let graph = layout
                        .as_mut()
                        .ok_or_else(|| GraphError::Layout(line.to_owned()))?;
                    let (width, height) = (number(4)?, number(5)?);
                    graph.vertices.push(VertexBox {
                        identifier: words[1].to_owned(),
                        left: number(2)? - width / 2.0,
                        top: graph.height - number(3)? - height / 2.0,
                        width,
                        height,
                    });
                }
                _ => {}
            }
        }
        layout.ok_or_else(|| GraphError::Layout("no line 'graph'".to_owned()))
    }

    // realises FR-085, FR-088, IR-024
    /// Yields the identifier of the vertex at a point of the image, `None` where there is none.
    ///
    /// # Arguments
    /// * `horizontal` - the distance from the left edge, as a fraction of the width of the image
    /// * `vertical` - the distance from the top edge, as a fraction of the height of the image
    pub fn vertex_at(&self, horizontal: f64, vertical: f64) -> Option<&str> {
        let (x, y) = (horizontal * self.width, vertical * self.height);
        self.vertices
            .iter()
            .find(|vertex| {
                x >= vertex.left
                    && x <= vertex.left + vertex.width
                    && y >= vertex.top
                    && y <= vertex.top + vertex.height
            })
            .map(|vertex| vertex.identifier.as_str())
    }
}

// realises FR-075, FR-076, IR-021, NFR-006
/// Derives the DOT text of the *network graph* of `nodes`.
///
/// * A *node* is the vertex `n<index>`, a box; a *proxy node* is a box drawn in three dimensions.
/// * A distinct path is the vertex `f<index>`, in the order of its first use: a *source input* is
///   a note, a *sink output* a filled note, the file of a *connection* an ellipse.
/// * The graph runs from left to right, has no padding, so that the image has the size of the
///   layout, and has the resolution [`IMAGE_DPI`].
pub fn dot_of_network(nodes: &[NodeDescription]) -> String {
    let mut indices: BTreeMap<&str, usize> = BTreeMap::new();
    let mut paths: Vec<&str> = Vec::new();
    let mut produced: BTreeSet<&str> = BTreeSet::new();
    let mut consumed: BTreeSet<&str> = BTreeSet::new();
    for node in nodes {
        consumed.extend(node.inputs.iter().map(String::as_str));
        produced.extend(node.outputs.iter().map(String::as_str));
        for path in node.inputs.iter().chain(node.outputs.iter()) {
            if !indices.contains_key(path.as_str()) {
                indices.insert(path.as_str(), paths.len());
                paths.push(path.as_str());
            }
        }
    }
    let mut dot =
        String::from("digraph network {\n    node [fontname=\"Helvetica\", fontsize=11];\n");
    dot.push_str(&format!(
        "    graph [rankdir=LR, pad=0, dpi={IMAGE_DPI}];\n"
    ));
    for (index, node) in nodes.iter().enumerate() {
        let (shape, fill) = match node.kind {
            NodeKind::MetaCompilerCompiler => ("box", "lightblue"),
            NodeKind::ProxyNode => ("box3d", "lightgrey"),
        };
        dot.push_str(&format!(
            "    n{index} [label=\"{}\\n{}\", shape={shape}, style=filled, fillcolor={fill}];\n",
            escaped(&node.name),
            escaped(&node.transformation)
        ));
    }
    for (index, path) in paths.iter().enumerate() {
        let attributes = match (produced.contains(path), consumed.contains(path)) {
            (false, _) => "shape=note",
            (true, false) => "shape=note, style=filled, fillcolor=palegreen",
            (true, true) => "shape=ellipse",
        };
        dot.push_str(&format!(
            "    f{index} [label=\"{}\", {attributes}];\n",
            escaped(path)
        ));
    }
    for (index, node) in nodes.iter().enumerate() {
        for path in &node.inputs {
            dot.push_str(&format!("    f{} -> n{index};\n", indices[path.as_str()]));
        }
        for path in &node.outputs {
            dot.push_str(&format!("    n{index} -> f{};\n", indices[path.as_str()]));
        }
    }
    dot.push_str("}\n");
    dot
}

// realises FR-085, FR-088
/// Yields the index of the *node* a vertex identifier names, `None` for the vertex of a file.
pub fn node_index_of_vertex(identifier: &str) -> Option<usize> {
    identifier.strip_prefix('n')?.parse().ok()
}

// realises FR-086, FR-087, FR-088, FR-102, FR-103, FR-113
/// Yields the files the *node window* presents for `node`, `None` for a *proxy node*.
///
/// * The paths are those of the *node description*, which are the *document identifiers*.
/// * The *producer* of an *input* is the first *node* of `nodes` other than `node` that has an
///   *output* of the same path; the *meta compiler DSL* and the *outputs* have none.
pub fn files_of_node(node: &NodeDescription, nodes: &[NodeDescription]) -> Option<NodeFiles> {
    let without_producer = |path: &String| NodeFile {
        identifier: path.clone(),
        producer: String::new(),
    };
    let producer_of = |path: &String| {
        nodes
            .iter()
            .filter(|candidate| candidate.name != node.name)
            .find(|candidate| candidate.outputs.contains(path))
            .map(|producer| producer.name.clone())
            .unwrap_or_default()
    };
    match node.kind {
        NodeKind::ProxyNode => None,
        NodeKind::MetaCompilerCompiler => Some(NodeFiles {
            name: node.name.clone(),
            meta_dsl: without_producer(&node.transformation),
            inputs: node
                .inputs
                .iter()
                .map(|path| NodeFile {
                    identifier: path.clone(),
                    producer: producer_of(path),
                })
                .collect(),
            outputs: node.outputs.iter().map(without_producer).collect(),
        }),
    }
}

// realises FR-086, FR-087, FR-102
/// Yields the path of a file of a *node*: `identifier` resolved against `network_directory`, the
/// directory of the *network file*.
pub fn path_of_identifier(network_directory: &Path, identifier: &str) -> String {
    network_directory
        .join(identifier)
        .to_string_lossy()
        .into_owned()
}

// realises FR-077, IR-021, IR-022, C-006
/// Lays out the DOT text by `dot` and yields the PNG image file and the layout.
///
/// * `dot` writes the image to `<stem>.png` in `directory`; the plain-text layout is read and its
///   file removed.
///
/// # Errors
/// Returns [`GraphError::DotNotFound`] where `dot` is not found, [`GraphError::Io`] where it cannot
/// be started or a file cannot be read, [`GraphError::DotFailed`] where it ends with a failure,
/// and [`GraphError::Layout`] where its layout is not read.
pub fn render(dot: &str, directory: &Path, stem: &str) -> Result<RenderedGraph, GraphError> {
    let program = dot_path().ok_or(GraphError::DotNotFound)?;
    let io = |error: std::io::Error| GraphError::Io(error.to_string());
    std::fs::create_dir_all(directory).map_err(io)?;
    let image_path = directory.join(format!("{stem}.png"));
    let plain_path = directory.join(format!("{stem}.plain"));
    let mut child = Command::new(program)
        .arg("-Tpng")
        .arg(format!("-o{}", image_path.display()))
        .arg("-Tplain")
        .arg(format!("-o{}", plain_path.display()))
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(io)?;
    if let Some(mut stdin) = child.stdin.take() {
        stdin.write_all(dot.as_bytes()).map_err(io)?;
    }
    let output = child.wait_with_output().map_err(io)?;
    if !output.status.success() {
        return Err(GraphError::DotFailed(
            String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        ));
    }
    let plain = std::fs::read_to_string(&plain_path).map_err(io)?;
    let _ = std::fs::remove_file(&plain_path);
    Ok(RenderedGraph {
        image_path,
        layout: GraphLayout::of_plain(&plain)?,
    })
}

/// Yields the path of `dot`: the first directory of `PATH`, then of the usual ones, that holds it.
fn dot_path() -> Option<PathBuf> {
    let from_environment = std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).collect::<Vec<_>>())
        .unwrap_or_default();
    from_environment
        .into_iter()
        .chain(FALLBACK_DIRECTORIES.iter().map(PathBuf::from))
        .map(|directory| directory.join(DOT_PROGRAM))
        .find(|candidate| candidate.is_file())
}

/// Yields `text` as it stands within a double-quoted string of the DOT language.
fn escaped(text: &str) -> String {
    text.replace('\\', "\\\\").replace('"', "\\\"")
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : NetworkEditor::report_arrived, NetworkEditor::open_node_at, NodeEditor::open_node */
#[cfg(test)]
mod tests {
    use super::*;

    use std::time::Instant;

    fn node(name: &str, kind: NodeKind, inputs: &[&str], outputs: &[&str]) -> NodeDescription {
        NodeDescription {
            name: name.to_owned(),
            kind,
            transformation: format!("{name}.gc3"),
            socket: format!("{name}.gc3.sock"),
            inputs: inputs.iter().map(|path| (*path).to_owned()).collect(),
            outputs: outputs.iter().map(|path| (*path).to_owned()).collect(),
        }
    }

    fn two_stages() -> Vec<NodeDescription> {
        vec![
            node("a", NodeKind::MetaCompilerCompiler, &["a.in"], &["x.txt"]),
            node("b", NodeKind::ProxyNode, &["x.txt"], &["y.txt"]),
        ]
    }

    #[test]
    fn empty_network_yields_a_graph_without_vertices() {
        let dot = dot_of_network(&[]);
        assert!(dot.starts_with("digraph network {") && !dot.contains("->"));
    }

    #[test]
    fn every_node_and_every_distinct_path_is_one_vertex() {
        let dot = dot_of_network(&two_stages());
        let vertices = dot.lines().filter(|line| line.contains("[label=")).count();
        assert_eq!(vertices, 5);
    }

    #[test]
    fn edges_run_from_inputs_to_nodes_and_from_nodes_to_outputs() {
        let dot = dot_of_network(&two_stages());
        let edges: Vec<&str> = dot
            .lines()
            .filter(|line| line.contains("->"))
            .map(str::trim)
            .collect();
        assert_eq!(
            edges,
            vec!["f0 -> n0;", "n0 -> f1;", "f1 -> n1;", "n1 -> f2;"]
        );
    }

    #[test]
    fn source_input_sink_output_and_connection_differ_in_shape_or_fill() {
        let dot = dot_of_network(&two_stages());
        let attributes = |identifier: &str| {
            dot.lines()
                .find(|line| line.trim_start().starts_with(&format!("{identifier} [")))
                .map(|line| line.split("\", ").nth(1).unwrap_or_default().to_owned())
        };
        assert_eq!(
            (attributes("f0"), attributes("f1"), attributes("f2")),
            (
                Some("shape=note];".to_owned()),
                Some("shape=ellipse];".to_owned()),
                Some("shape=note, style=filled, fillcolor=palegreen];".to_owned())
            )
        );
    }

    #[test]
    fn proxy_node_differs_from_meta_compiler_compiler() {
        let dot = dot_of_network(&two_stages());
        assert!(dot.contains("shape=box, ") && dot.contains("shape=box3d, "));
    }

    #[test]
    fn quote_and_backslash_of_a_path_are_escaped() {
        let dot = dot_of_network(&[node(
            "a",
            NodeKind::MetaCompilerCompiler,
            &["a \"b\"\\c"],
            &[],
        )]);
        assert!(dot.contains("label=\"a \\\"b\\\"\\\\c\""));
    }

    #[test]
    fn thousand_node_descriptions_are_derived_within_the_bound() {
        let nodes: Vec<NodeDescription> = (0..1000)
            .map(|index| {
                node(
                    &format!("node_{index}"),
                    NodeKind::MetaCompilerCompiler,
                    &[&format!("file_{index}")],
                    &[&format!("file_{}", index + 1)],
                )
            })
            .collect();
        let start = Instant::now();
        let dot = dot_of_network(&nodes);
        assert!(start.elapsed().as_millis() <= 200 && !dot.is_empty());
    }

    #[test]
    fn plain_layout_is_read_with_the_origin_at_the_top_left() {
        let plain =
            "graph 1 2 0.5\nnode n0 1.625 0.25 0.75 0.5 \"a\" solid box black lightgrey\nstop\n";
        assert_eq!(
            GraphLayout::of_plain(plain),
            Ok(GraphLayout {
                width: 2.0,
                height: 0.5,
                vertices: vec![VertexBox {
                    identifier: "n0".to_owned(),
                    left: 1.25,
                    top: 0.0,
                    width: 0.75,
                    height: 0.5,
                }],
            })
        );
    }

    #[test]
    fn plain_layout_without_graph_line_is_an_error() {
        assert!(matches!(
            GraphLayout::of_plain("node n0 1 1 1 1\n"),
            Err(GraphError::Layout(_))
        ));
    }

    #[test]
    fn point_within_a_vertex_yields_it_and_a_point_beside_it_yields_none() {
        let layout = GraphLayout {
            width: 2.0,
            height: 1.0,
            vertices: vec![VertexBox {
                identifier: "n0".to_owned(),
                left: 1.0,
                top: 0.0,
                width: 1.0,
                height: 0.5,
            }],
        };
        assert_eq!(
            (layout.vertex_at(0.75, 0.25), layout.vertex_at(0.25, 0.25)),
            (Some("n0"), None)
        );
    }

    #[test]
    fn vertex_of_a_node_names_its_index_and_vertex_of_a_file_names_none() {
        assert_eq!(
            (node_index_of_vertex("n12"), node_index_of_vertex("f3")),
            (Some(12), None)
        );
    }

    fn file(identifier: &str, producer: &str) -> NodeFile {
        NodeFile {
            identifier: identifier.to_owned(),
            producer: producer.to_owned(),
        }
    }

    #[test]
    fn files_of_a_meta_compiler_compiler_are_its_paths_as_written_in_their_order() {
        // FR-086, FR-087, FR-102, FR-103
        let nodes = two_stages();
        assert_eq!(
            files_of_node(&nodes[0], &nodes),
            Some(NodeFiles {
                name: "a".to_owned(),
                meta_dsl: file("a.gc3", ""),
                inputs: vec![file("a.in", "")],
                outputs: vec![file("x.txt", "")],
            })
        );
    }

    #[test]
    fn input_that_another_node_outputs_names_that_node_as_its_producer() {
        // FR-113
        let nodes = vec![
            node("a", NodeKind::MetaCompilerCompiler, &["a.in"], &["x.txt"]),
            node(
                "b",
                NodeKind::MetaCompilerCompiler,
                &["x.txt", "b.in"],
                &["y.txt"],
            ),
            node("c", NodeKind::ProxyNode, &["y.txt"], &["b.in"]),
        ];
        let inputs = files_of_node(&nodes[1], &nodes).map(|files| files.inputs);
        assert_eq!(inputs, Some(vec![file("x.txt", "a"), file("b.in", "c")]));
    }

    #[test]
    fn input_the_node_itself_outputs_has_no_producer() {
        // FR-113: a node that reads its own output is no producer of itself
        let nodes = vec![node(
            "a",
            NodeKind::MetaCompilerCompiler,
            &["x.txt"],
            &["x.txt"],
        )];
        let inputs = files_of_node(&nodes[0], &nodes).map(|files| files.inputs);
        assert_eq!(inputs, Some(vec![file("x.txt", "")]));
    }

    #[test]
    fn proxy_node_has_no_files_to_open() {
        // FR-088
        let nodes = two_stages();
        assert_eq!(files_of_node(&nodes[1], &nodes), None);
    }

    #[test]
    fn node_without_input_and_output_has_the_meta_dsl_alone() {
        let nodes = vec![node("a", NodeKind::MetaCompilerCompiler, &[], &[])];
        let files = files_of_node(&nodes[0], &nodes);
        assert_eq!(
            files.map(|files| (files.inputs.len(), files.outputs.len())),
            Some((0, 0))
        );
    }

    #[test]
    fn identifier_is_resolved_against_the_network_directory() {
        // FR-086, FR-087, FR-102
        assert_eq!(
            (
                path_of_identifier(Path::new("/net"), "src/a.in"),
                path_of_identifier(Path::new("/net"), "/abs/a.in")
            ),
            ("/net/src/a.in".to_owned(), "/abs/a.in".to_owned())
        );
    }
}
