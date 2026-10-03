// The indication of the typing mode: whether typing inserts or overwrites.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

import QtQuick
import QtQuick.Controls
// The types of the module genc3wb are registered by the bridge crate at run time, which writes no type
// description for qmllint; the import and the unqualified access to the singleton are therefore not linted.
// qmllint disable import unqualified
import genc3wb

Label {
    id: typingModeLabel

    text: Workbench.insertMode ? qsTr("Insert") : qsTr("Overwrite")
    ToolTip.visible: typingModeHover.hovered
    ToolTip.text: qsTr("The typing mode; toggled by the insert key %1 and by the menu Edit.")
                  .arg(Workbench.insertKeyName)

    HoverHandler {
        id: typingModeHover
    }
}
