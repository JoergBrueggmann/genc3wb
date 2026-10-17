//! The tests of the page label of the output group of the front end, run by the QML test runner of
//! Qt.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

mod qml_runner;

/// The QML files of the front end of which an output group consists, in 'src/qml'.
const COMPONENTS: [&str; 7] = [
    "OutputGroup.qml",
    "ModeSwitch.qml",
    "PositionSwitch.qml",
    "CodeEditor.qml",
    "DiagnosticMark.qml",
    "WavyLine.qml",
    "OctetView.qml",
];

/// The stand-ins of the bridged types the output group reaches, in 'tests/qml'.
const STAND_INS: [&str; 2] = ["Editing.qml", "Workbench.qml"];

/// The QML test file of the page label, in 'tests/qml', with the name of its test case.
const PAGE_LABEL: (&str, &str) = ("tst_output_group.qml", "OutputGroup");

/// The 'qmldir' of the QML module `genc3wb` the test file imports: the components of the output
/// group, and the stand-ins as singletons.
const QMLDIR: &str = "module genc3wb
OutputGroup 1.0 OutputGroup.qml
ModeSwitch 1.0 ModeSwitch.qml
PositionSwitch 1.0 PositionSwitch.qml
CodeEditor 1.0 CodeEditor.qml
DiagnosticMark 1.0 DiagnosticMark.qml
WavyLine 1.0 WavyLine.qml
OctetView 1.0 OctetView.qml
singleton Editing 1.0 Editing.qml
singleton Workbench 1.0 Workbench.qml
";

/// Runs the test function `function` of the QML test file of the page label and yields the output
/// of the test runner where it failed.
fn failures_of(case: &str, function: &str) -> Option<String> {
    qml_runner::failures_of(case, &COMPONENTS, &STAND_INS, QMLDIR, PAGE_LABEL, function)
}

/*  * validated        : ✅
 * completeness     : ✅ the first output page, a further output page, the diagnostics page
 * independence     : ✅ a temporary directory per case; every QML case sets the presented page
 * edge cases       : ✅ page 0, the first; the diagnostics page, the last
 * conforms to doc  : ✅ FR-036, the page label of OutputGroup.qml
 * covers bridge    : OutputGroup::page_index and page_count, by a stand-in that holds them */

// The cases need the QML test runner of the Qt installation whose qmake the build finds.

#[test]
#[ignore]
fn first_output_page_is_counted_from_0() {
    assert_eq!(
        failures_of(
            "page-label-first",
            "test_first_output_page_is_counted_from_0"
        ),
        None
    );
}

#[test]
#[ignore]
fn second_output_page_is_page_1() {
    assert_eq!(
        failures_of("page-label-second", "test_second_output_page_is_page_1"),
        None
    );
}

#[test]
#[ignore]
fn diagnostics_page_is_named_and_counted_from_0() {
    assert_eq!(
        failures_of(
            "page-label-diagnostics",
            "test_diagnostics_page_is_named_and_counted_from_0"
        ),
        None
    );
}
