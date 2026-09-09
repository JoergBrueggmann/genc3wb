//! The entry point: embeds the front end, builds the Qt application, and runs the event loop.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use genc3wb::bridge::workbench::Workbench;

use qtbridge::{QApp, include_bytes_qml};

fn main() {
    include_bytes_qml!("qml/qmldir", "genc3wb");
    include_bytes_qml!("qml/Main.qml", "genc3wb");
    include_bytes_qml!("qml/InputGroup.qml", "genc3wb");
    include_bytes_qml!("qml/CodeEditor.qml", "genc3wb");
    include_bytes_qml!("qml/ProcessingStateIndicator.qml", "genc3wb");
    include_bytes_qml!("qml/RunnerGroup.qml", "genc3wb");
    include_bytes_qml!("qml/OutputGroup.qml", "genc3wb");
    include_bytes_qml!("qml/OutputWindow.qml", "genc3wb");
    include_bytes_qml!("qml/EditorWindow.qml", "genc3wb");
    include_bytes_qml!("qml/InfoDialog.qml", "genc3wb");
    include_bytes_qml!("qml/PathOfUrl.qml", "genc3wb");
    include_bytes_qml!("qml/icons/indicatorUntouched.png", "genc3wb");
    include_bytes_qml!("qml/icons/indicatorChanged.png", "genc3wb");
    include_bytes_qml!("qml/icons/indicatorUnknownFileUntouched.png", "genc3wb");
    include_bytes_qml!("qml/icons/indicatorUnknownFileChanged.png", "genc3wb");
    let exit_code = QApp::new()
        .application_name("genc3wb")
        .register::<Workbench>()
        .load_qml_from_file("qrc:/genc3wb/qml/Main.qml")
        .run();
    std::process::exit(exit_code);
}
