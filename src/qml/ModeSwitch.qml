// The mode switch of a code editor: the three positions txt, hex and bin, of which one is selected.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

pragma ComponentBehavior: Bound

import QtQuick
// The types of the module genc3wb are registered by the bridge crate at run time, which writes no type
// description for qmllint; the import and the unqualified access to the singleton are therefore not linted.
// qmllint disable import unqualified
import genc3wb

Rectangle {
    id: root

    // whether the octets of the document the code editor presents are valid UTF-8
    property bool documentIsText: true
    // the index of the position the user selected since the code editor came to present its
    // document; -1 where the user selected none
    property int chosen: -1
    // the index of the selected position: 0 for txt, 1 for hex, 2 for bin
    readonly property int mode: Editing.modeSelected(root.chosen, root.documentIsText)

    implicitWidth: positions.implicitWidth + 2
    implicitHeight: positions.implicitHeight + 2
    radius: 4
    color: palette.button
    border.color: palette.mid

    SystemPalette {
        id: palette
    }

    Row {
        id: positions

        x: 1
        y: 1

        Repeater {
            id: positionRepeater

            model: ["txt", "hex", "bin"]
            delegate: Rectangle {
                id: position

                required property int index
                required property string modelData

                readonly property bool selected: root.mode === position.index

                implicitWidth: positionLabel.implicitWidth + 16
                implicitHeight: positionLabel.implicitHeight + 8
                radius: 3
                color: position.selected ? palette.highlight : "transparent"

                Text {
                    id: positionLabel

                    anchors.centerIn: parent
                    text: position.modelData
                    font.family: "Courier New"
                    color: position.selected ? palette.highlightedText : palette.buttonText
                }

                TapHandler {
                    id: positionTap

                    onTapped: root.chosen = position.index
                }
            }
        }
    }
}
