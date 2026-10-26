//! The tests of the playback delay dialog of the front end, run by the QML test runner of Qt.
//!
//! Copyright 2021-2026 Jörg Karl-Heinz Walter Brüggmann
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

mod qml_runner;

/// The QML files of the front end of which the dialog consists, in 'src/qml'.
const COMPONENTS: [&str; 1] = ["PlaybackDelayDialog.qml"];

/// The stand-ins of the bridged types the dialog reaches, in 'tests/qml'.
const STAND_INS: [&str; 1] = ["Workbench.qml"];

/// The QML test file of the dialog, in 'tests/qml', with the name of its test case.
const PLAYBACK_DELAY: (&str, &str) = ("tst_playback_delay.qml", "PlaybackDelay");

/// The 'qmldir' of the QML module `genc3wb` the test file imports: the dialog, and the stand-in
/// as a singleton.
const QMLDIR: &str = "module genc3wb
PlaybackDelayDialog 1.0 PlaybackDelayDialog.qml
singleton Workbench 1.0 Workbench.qml
";

/// Runs the test function `function` of the QML test file of the dialog and yields the output of
/// the test runner where it failed.
fn failures_of(case: &str, function: &str) -> Option<String> {
    qml_runner::failures_of(
        case,
        &COMPONENTS,
        &STAND_INS,
        QMLDIR,
        PLAYBACK_DELAY,
        function,
    )
}

/*  * validated        : ✅
 * completeness     : ✅ the opening with the stored switch and value and the check box
 *                       unchecked, also after a checked one; the field enabled by the switch;
 *                       OK with the switch on, and off with the check box checked; Cancel
 * independence     : ✅ a temporary directory per case; every QML case resets the stand-in
 * edge cases       : ✅ the last of the thirteen values, the switch off keeping the index
 * conforms to doc  : ✅ FR-542, FR-543, PlaybackDelayDialog.qml
 * covers bridge    : Workbench::set_playback_delay, by a stand-in that sets as it does */

// The cases need the QML test runner of the Qt installation whose qmake the build finds.

#[test]
#[ignore]
fn the_dialog_opens_with_the_stored_delay_and_the_check_box_unchecked() {
    assert_eq!(
        failures_of(
            "playback-delay-opening",
            "test_opens_with_the_stored_delay_and_the_check_box_unchecked"
        ),
        None
    );
}

#[test]
#[ignore]
fn the_field_is_enabled_while_the_switch_is_checked_alone() {
    assert_eq!(
        failures_of(
            "playback-delay-switch",
            "test_the_field_is_enabled_while_the_switch_is_checked_alone"
        ),
        None
    );
}

#[test]
#[ignore]
fn ok_sets_the_delay_and_accepts() {
    assert_eq!(
        failures_of("playback-delay-ok", "test_ok_sets_the_delay_and_accepts"),
        None
    );
}

#[test]
#[ignore]
fn ok_with_the_switch_off_and_the_check_box_disables_the_dialog() {
    assert_eq!(
        failures_of(
            "playback-delay-disable",
            "test_ok_with_the_switch_off_and_the_check_box_disables_the_dialog"
        ),
        None
    );
}

#[test]
#[ignore]
fn cancel_sets_nothing_and_rejects() {
    assert_eq!(
        failures_of(
            "playback-delay-cancel",
            "test_cancel_sets_nothing_and_rejects"
        ),
        None
    );
}
