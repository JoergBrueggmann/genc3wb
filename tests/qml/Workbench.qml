// The stand-in of the bridged type Workbench for the tests of the code editor and of the output group: the
// typing mode, the insert key, which the test sets and which toggles the typing mode as
// Workbench::toggled_by_key does, the tab size and the spaces setting.
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
