//! The *core*: the application logic, free of the bridge crate and of QML.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

pub mod api_message;
pub mod build_system;
pub mod editing;
pub mod executable;
pub mod input_file;
pub mod network_builder;
pub mod network_graph;
pub mod node_runner;
pub mod output;
pub mod settings;
pub mod text_increment;
