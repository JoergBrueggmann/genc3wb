// A position switch: positions side by side, labelled by abbreviations, of which one is highlighted; its
// owner binds the highlighted position and the enabled state, and answers the tap on a position.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls

Rectangle {
    id: root

    // the abbreviations of the positions, from left to right
    required property list<string> labels
    // the index of the highlighted position
    property int selected: 0
    // the text of the tooltip; empty where the switch presents none (FR-271, FR-272, IR-039)
    property string tooltip: ""

    // the index of the position the user tapped (IR-031)
    signal positionTapped(int index)

    implicitWidth: positions.implicitWidth + 2
    implicitHeight: positions.implicitHeight + 2
    radius: 4
    color: palette.button
    border.color: palette.mid

    // the tooltip covers the whole switch
    ToolTip.visible: root.tooltip.length > 0 && hover.hovered
    ToolTip.text: root.tooltip

    // the colours of the disabled controls while the switch is disabled, which greys it (FR-259)
    SystemPalette {
        id: palette

        colorGroup: root.enabled ? SystemPalette.Active : SystemPalette.Disabled
    }

    HoverHandler {
        id: hover
    }

    Row {
        id: positions

        x: 1
        y: 1

        Repeater {
            id: positionRepeater

            model: root.labels
            delegate: Rectangle {
                id: position

                required property int index
                required property string modelData

                readonly property bool highlighted: root.selected === position.index

                implicitWidth: positionLabel.implicitWidth + 16
                implicitHeight: positionLabel.implicitHeight + 8
                radius: 3
                color: position.highlighted ? palette.highlight : "transparent"

                Text {
                    id: positionLabel

                    anchors.centerIn: parent
                    text: position.modelData
                    font.family: "Courier New"
                    color: position.highlighted ? palette.highlightedText : palette.buttonText
                }

                // a disabled switch receives no tap
                TapHandler {
                    id: positionTap

                    onTapped: root.positionTapped(position.index)
                }
            }
        }
    }
}
