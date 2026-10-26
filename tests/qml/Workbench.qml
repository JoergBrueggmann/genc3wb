// The stand-in of the bridged type Workbench for the tests of the code editor, of the output group and of
// the playback delay dialog: the typing mode, the insert key, which the test sets and which toggles the
// typing mode as Workbench::toggled_by_key does, the tab size, the spaces setting, and the playback delay
// with its dialog setting, set as Workbench::set_playback_delay sets them.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

pragma Singleton

import QtQuick

QtObject {
    id: workbench

    // the typing mode: whether typing inserts
    property bool insertMode: true
    // the insert key: its key and its modifiers, as Qt numbers them
    property int insertKey: Qt.Key_Enter
    property int insertModifiers: Qt.NoModifier
    // the number of times the typing mode was toggled by a key
    property int toggles: 0
    // the tab size of every code editor
    property int tabSize: 4
    // the spaces setting: whether the Tab key inserts spaces up to the next tab stop
    property bool tabsToSpaces: false
    // the playback delay: whether one is waited, the index of its value among the thirteen, the value in
    // milliseconds, and the playback delay dialog setting
    property bool playbackDelayOn: false
    property int playbackDelayIndex: 0
    readonly property int playbackDelay: workbench.playbackDelayOn ? workbench.playbackDelayOf(workbench.playbackDelayIndex) : 0
    property bool playbackDelayDialogDisabled: false
    // the number of times the playback delay was set by the dialog
    property int delaysSet: 0
    // the thirteen values in milliseconds, as PLAYBACK_DELAYS holds them
    readonly property var playbackDelays: [50, 71, 100, 141, 200, 283, 400, 566, 800, 1131, 1600, 2263, 3200]

    // Yields the value with the index `index` in milliseconds, 0 for an index that names none.
    function playbackDelayOf(index: int): int {
        return index >= 0 && index < workbench.playbackDelays.length ? workbench.playbackDelays[index] : 0;
    }

    // Sets the playback delay, whether one is waited and the index of its value, and switches the dialog
    // setting on where `disableDialog`, as the workbench object does.
    function setPlaybackDelay(on: bool, index: int, disableDialog: bool) {
        workbench.playbackDelayOn = on;
        workbench.playbackDelayIndex = index;
        if (disableDialog) {
            workbench.playbackDelayDialogDisabled = true;
        }
        workbench.delaysSet += 1;
    }

    // Toggles the typing mode where the key event is the insert key, and yields whether it is; the
    // modifiers kept are those KeyStroke::new keeps: Control, Shift, Alt and Meta.
    function toggledByKey(key: int, modifiers: int): bool {
        const kept = modifiers & (Qt.ControlModifier | Qt.ShiftModifier | Qt.AltModifier | Qt.MetaModifier);
        const toggled = key === workbench.insertKey && kept === workbench.insertModifiers;
        if (toggled) {
            workbench.insertMode = !workbench.insertMode;
            workbench.toggles += 1;
        }
        return toggled;
    }
}
