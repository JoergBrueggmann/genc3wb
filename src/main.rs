//! The entry point: reads the command line, embeds the front end, builds the Qt application
//! with the handler of the file-open event, and runs the event loop.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use genc3wb::bridge::editing::Editing;
use genc3wb::bridge::workbench::{self, Workbench};
use genc3wb::core::launch;

use qtbridge::{QApp, include_bytes_qml};

fn main() {
    // the handed file or the script of the switch --verify of the command line, or a refused
    // argument (IR-206, FR-517)
    match launch::arguments(std::env::args().skip(1)) {
        Ok(arguments) => {
            if let Some(path) = arguments.handed {
                workbench::hand_file(path);
            }
            if let Some(path) = arguments.verify {
                workbench::verify_script(path);
            }
        }
        Err(message) => {
            eprintln!("{message}");
            std::process::exit(1);
        }
    }
    include_bytes_qml!("qml/qmldir", "genc3wb");
    include_bytes_qml!("qml/Main.qml", "genc3wb");
    include_bytes_qml!("qml/NodeWindow.qml", "genc3wb");
    include_bytes_qml!("qml/NetworkGraphGroup.qml", "genc3wb");
    include_bytes_qml!("qml/InputGroup.qml", "genc3wb");
    include_bytes_qml!("qml/CodeEditor.qml", "genc3wb");
    include_bytes_qml!("qml/DiagnosticMark.qml", "genc3wb");
    include_bytes_qml!("qml/WavyLine.qml", "genc3wb");
    include_bytes_qml!("qml/PositionSwitch.qml", "genc3wb");
    include_bytes_qml!("qml/SplitterHandle.qml", "genc3wb");
    include_bytes_qml!("qml/ModeSwitch.qml", "genc3wb");
    include_bytes_qml!("qml/OctetView.qml", "genc3wb");
    include_bytes_qml!("qml/TypingModeLabel.qml", "genc3wb");
    include_bytes_qml!("qml/InsertKeyDialog.qml", "genc3wb");
    include_bytes_qml!("qml/ProcessingStateIndicator.qml", "genc3wb");
    include_bytes_qml!("qml/OutputGroup.qml", "genc3wb");
    include_bytes_qml!("qml/EditorWindow.qml", "genc3wb");
    include_bytes_qml!("qml/InfoDialog.qml", "genc3wb");
    include_bytes_qml!("qml/SettingsDialog.qml", "genc3wb");
    include_bytes_qml!("qml/PlaybackDelayDialog.qml", "genc3wb");
    include_bytes_qml!("qml/PathOfUrl.qml", "genc3wb");
    include_bytes_qml!("qml/icons/indicatorUntouched.png", "genc3wb");
    include_bytes_qml!("qml/icons/indicatorChanged.png", "genc3wb");
    include_bytes_qml!("qml/icons/indicatorUnknownFileUntouched.png", "genc3wb");
    include_bytes_qml!("qml/icons/indicatorUnknownFileChanged.png", "genc3wb");
    include_bytes_qml!("qml/icons/error.png", "genc3wb");
    include_bytes_qml!("qml/icons/network.png", "genc3wb");
    include_bytes_qml!("qml/icons/machine.png", "genc3wb");
    let exit_code = QApp::new()
        .application_name("genc3wb")
        // the file the operating system hands over, at launch or while running (IR-207)
        .on_file_open(workbench::file_opened)
        .register::<Workbench>()
        .register::<Editing>()
        .load_qml_from_file("qrc:/genc3wb/qml/Main.qml")
        .run();
    std::process::exit(exit_code);
}
