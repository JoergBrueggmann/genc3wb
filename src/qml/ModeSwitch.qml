// The mode switch of a code editor: a position switch of the three positions txt, hex and bin, of which one
// is selected.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

import QtQuick
// The types of the module genc3wb are registered by the bridge crate at run time, which writes no type
// description for qmllint; the import and the unqualified access to the singleton are therefore not linted.
// qmllint disable import unqualified
import genc3wb

PositionSwitch {
    id: root

    // whether the octets of the document the code editor presents are valid UTF-8
    property bool documentIsText: true
    // the index of the position the user selected since the code editor came to present its
    // document; -1 where the user selected none
    property int chosen: -1
    // the index of the selected position: 0 for txt, 1 for hex, 2 for bin
    readonly property int mode: Editing.modeSelected(root.chosen, root.documentIsText)

    labels: ["txt", "hex", "bin"]
    selected: root.mode
    // FR-271
    tooltip: qsTr("Text / Hexadecimal / Binary - to see UTF-8 text decoded, octets hexadecimal, or binary as 0 and 1 positions")
    // the tap that selects the position (FR-155)
    onPositionTapped: index => root.chosen = index
}
