//! The tests of the insert key and of the Tab key at the code editor of the front end and of the position
//! switch, run by the QML test runner of Qt.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

mod qml_runner;

/// The QML files of the front end of which a code editor and a position switch consist, in
/// 'src/qml'.
const COMPONENTS: [&str; 5] = [
    "CodeEditor.qml",
    "DiagnosticMark.qml",
    "WavyLine.qml",
    "OctetView.qml",
    "PositionSwitch.qml",
];

/// The stand-ins of the bridged types the code editor reaches, in 'tests/qml'.
const STAND_INS: [&str; 2] = ["Editing.qml", "Workbench.qml"];

/// The QML test file of the insert key, in 'tests/qml', with the name of its test case.
const INSERT_KEY: (&str, &str) = ("tst_insert_key.qml", "InsertKey");

/// The QML test file of the position switch, in 'tests/qml', with the name of its test case.
const POSITION_SWITCH: (&str, &str) = ("tst_position_switch.qml", "PositionSwitch");

/// The QML test file of the Tab key, in 'tests/qml', with the name of its test case.
const TAB_KEY: (&str, &str) = ("tst_tab_key.qml", "TabKey");

/// The 'qmldir' of the QML module `genc3wb` the test file imports: the components of the code
/// editor, and the stand-ins as singletons.
const QMLDIR: &str = "module genc3wb
CodeEditor 1.0 CodeEditor.qml
DiagnosticMark 1.0 DiagnosticMark.qml
WavyLine 1.0 WavyLine.qml
OctetView 1.0 OctetView.qml
PositionSwitch 1.0 PositionSwitch.qml
singleton Editing 1.0 Editing.qml
singleton Workbench 1.0 Workbench.qml
";

/// Runs the test function `function` of the QML test file `test`, named with its test case, and
/// yields the output of the test runner where it failed.
fn failures_of(case: &str, test: (&str, &str), function: &str) -> Option<String> {
    qml_runner::failures_of(case, &COMPONENTS, &STAND_INS, QMLDIR, test, function)
}

/*  * validated        : ✅
 * completeness     : ✅ txt, hex and bin; a printable key, a command key stroke, Return and Enter
 *                       as the insert key; the Tab key with the spaces setting off and on, with
 *                       a selection, and in a read-only editor; the position switch tapped,
 *                       disabled, and its tooltip
 * independence     : ✅ a temporary directory per case; every QML case prepares its items
 * edge cases       : ✅ Return and Enter, which have key handlers of their own in txt; a tap on
 *                       the highlighted position, which moves the highlight not
 * conforms to doc  : ✅ FR-193, the key handlers of CodeEditor.qml and OctetView.qml; FR-143,
 *                       FR-492, the Tab key of CodeEditor.qml; FR-259,
 *                       FR-265, FR-266, FR-272, IR-031, IR-039, PositionSwitch.qml
 * covers bridge    : Workbench::toggled_by_key, by a stand-in that toggles as it does;
 *                    Editing::spaces_to_tab_stop, by a stand-in counting over the line */

// The cases need the QML test runner of the Qt installation whose qmake the build finds.

#[test]
#[ignore]
fn insert_key_is_not_entered_as_text() {
    assert_eq!(
        failures_of(
            "insert-key-text",
            INSERT_KEY,
            "test_insert_key_is_not_entered_as_text"
        ),
        None
    );
}

#[test]
#[ignore]
fn insert_key_edits_no_digit() {
    assert_eq!(
        failures_of(
            "insert-key-digits",
            INSERT_KEY,
            "test_insert_key_edits_no_digit"
        ),
        None
    );
}

#[test]
#[ignore]
fn insert_key_toggles_the_typing_mode() {
    assert_eq!(
        failures_of(
            "insert-key-toggle",
            INSERT_KEY,
            "test_insert_key_toggles_the_typing_mode"
        ),
        None
    );
}

#[test]
#[ignore]
fn insert_key_does_not_pass_on_to_the_items_above() {
    assert_eq!(
        failures_of(
            "insert-key-passing",
            INSERT_KEY,
            "test_insert_key_does_not_pass_on_to_the_items_above"
        ),
        None
    );
}

#[test]
#[ignore]
fn tap_on_a_position_emits_its_index() {
    assert_eq!(
        failures_of(
            "position-switch-tap",
            POSITION_SWITCH,
            "test_tap_on_a_position_emits_its_index"
        ),
        None
    );
}

#[test]
#[ignore]
fn disabled_position_switch_emits_nothing() {
    assert_eq!(
        failures_of(
            "position-switch-disabled",
            POSITION_SWITCH,
            "test_disabled_switch_emits_nothing"
        ),
        None
    );
}

#[test]
#[ignore]
fn highlighted_position_follows_the_owner_not_the_tap() {
    assert_eq!(
        failures_of(
            "position-switch-highlight",
            POSITION_SWITCH,
            "test_highlight_follows_selected_not_the_tap"
        ),
        None
    );
}

#[test]
#[ignore]
fn tooltip_of_a_position_switch_is_the_text_given() {
    assert_eq!(
        failures_of(
            "position-switch-tooltip",
            POSITION_SWITCH,
            "test_tooltip_is_the_text_given"
        ),
        None
    );
}

// FR-143
#[test]
fn tab_key_inserts_a_tab_character_while_the_spaces_setting_is_off() {
    assert_eq!(
        failures_of(
            "tab-off",
            TAB_KEY,
            "test_tab_inserts_a_tab_character_while_the_setting_is_off"
        ),
        None
    );
}

// FR-492
#[test]
fn tab_key_inserts_spaces_to_the_next_tab_stop_while_the_spaces_setting_is_on() {
    assert_eq!(
        failures_of(
            "tab-on",
            TAB_KEY,
            "test_tab_inserts_spaces_to_the_next_tab_stop_while_the_setting_is_on"
        ),
        None
    );
}

// FR-492
#[test]
fn tab_key_replaces_the_selection_by_the_spaces() {
    assert_eq!(
        failures_of(
            "tab-selection",
            TAB_KEY,
            "test_tab_replaces_the_selection_by_the_spaces"
        ),
        None
    );
}

// FR-042, FR-143
#[test]
fn tab_key_enters_nothing_into_a_read_only_editor() {
    assert_eq!(
        failures_of(
            "tab-read-only",
            TAB_KEY,
            "test_tab_enters_nothing_into_a_read_only_editor"
        ),
        None
    );
}
