// The presentation of the octets of a document in hex or bin: the column header, and one row per as many
// octets as fit into the width of the view, with its row label, its digits and the marks of the diagnostics.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
// The types of the module genc3wb are registered by the bridge crate at run time, which writes no type
// description for qmllint; the import and the unqualified access to the singleton are therefore not linted.
// qmllint disable import unqualified
import genc3wb

Item {
    id: root

    // the input group or the output group whose octets are presented; null where there is none
    property var provider: null
    // the index of the editor mode: 1 for hex, 2 for bin
    property int mode: 1
    // whether the provider carries marks of diagnostics
    property bool marked: false
    // the font of the digits, which is monospaced
    property font digitFont

    // the number of changes of the octets, on which everything read through a slot depends
    readonly property int revision: root.provider ? root.provider.revision : 0
    // the number of octets of the document
    readonly property int octetCount: root.provider ? root.provider.octetCount : 0
    // the width of one character of the monospaced font
    readonly property real characterWidth: fontMetrics.averageCharacterWidth
    // the number of character columns the view shows beside the vertical scroll bar and the margin
    readonly property int columns: Math.floor(Math.max(rows.width - verticalBar.width - 4, 0)
                                              / Math.max(root.characterWidth, 1))
    // the number of octets per row: the most that fit into the view
    readonly property int octetsPerRow: Editing.octetsPerRow(root.mode, root.octetCount, root.columns)
    // the number of rows
    readonly property int rowCount: root.provider && root.revision >= 0 ? root.provider.rowCount(root.octetsPerRow) : 0
    // the width of the row labels: that of the label of the last row, and a margin on each side
    readonly property real labelWidth: (Editing.rowLabel(root.octetsPerRow, Math.max(root.rowCount - 1, 0),
                                                         root.octetCount).length + 1) * root.characterWidth + 4
    // the width of a row: its label and the digits of a full row
    readonly property real rowWidth: root.labelWidth + (header.text.length + 1) * root.characterWidth

    // the row at the top of the view, and the number of octets per row it was counted with,
    // by which the octet at the top stays in view when the number of octets per row changes
    property int topRow: 0
    property int topRowWidth: 0
    // whether the view is being moved to the octet at the top after a change of the octets per row
    property bool rewrapping: false

    // Moves the view to the row that holds the octet that was at its top.
    function keepTopOctet() {
        const row = Editing.rowKeeping(root.topRow, root.topRowWidth, root.octetsPerRow);
        rows.positionViewAtIndex(row, ListView.Beginning);
        root.topRow = row;
        root.topRowWidth = root.octetsPerRow;
        root.rewrapping = false;
    }

    clip: true
    onOctetsPerRowChanged: {
        if (root.topRowWidth > 0 && root.topRowWidth !== root.octetsPerRow) {
            root.rewrapping = true;
            Qt.callLater(root.keepTopOctet);
        }
    }

    SystemPalette {
        id: palette
    }

    FontMetrics {
        id: fontMetrics

        font: root.digitFont
    }

    Text {
        id: header

        x: root.labelWidth - rows.contentX
        y: 2
        font: root.digitFont
        color: "gray"
        text: Editing.columnHeader(root.mode, root.octetsPerRow)
    }

    ListView {
        id: rows

        anchors.fill: parent
        anchors.topMargin: header.height + 4
        clip: true
        boundsBehavior: Flickable.StopAtBounds
        flickableDirection: Flickable.AutoFlickIfNeeded
        contentWidth: Math.max(rows.width, root.rowWidth)
        onContentYChanged: {
            if (!root.rewrapping) {
                root.topRow = Math.max(rows.indexAt(0, rows.contentY + 1), 0);
                root.topRowWidth = root.octetsPerRow;
            }
        }
        model: root.rowCount

        ScrollBar.vertical: ScrollBar {
            id: verticalBar
        }
        ScrollBar.horizontal: ScrollBar {}

        delegate: Item {
            id: row

            required property int index

            // the segments below which the diagnostics are marked: per segment its first
            // character column, its number of character columns and the index of its mark
            readonly property var segments: root.marked && root.revision >= 0
                                            && root.provider.octetMarkTexts.length > 0
                                            ? root.provider.rowMarks(root.mode, root.octetsPerRow, row.index) : []

            width: root.rowWidth
            height: digits.implicitHeight

            Text {
                id: label

                x: 0
                width: root.labelWidth - root.characterWidth
                horizontalAlignment: Text.AlignRight
                font: root.digitFont
                color: "gray"
                text: Editing.rowLabel(root.octetsPerRow, row.index, root.octetCount)
            }

            Text {
                id: digits

                x: root.labelWidth
                font: root.digitFont
                color: palette.text
                text: root.revision >= 0 ? root.provider.rowText(root.mode, root.octetsPerRow, row.index) : ""
            }

            Repeater {
                id: markRepeater

                model: row.segments.length / 3
                delegate: Item {
                    id: mark

                    required property int index

                    // the index of the diagnostic among the marks of the provider
                    readonly property int diagnostic: row.segments[3 * mark.index + 2] ?? 0
                    // the index of the severity: 0 for error, 1 for warning, 2 for information
                    readonly property int severity: root.provider.octetMarkSeverities[mark.diagnostic] ?? 0

                    x: digits.x + (row.segments[3 * mark.index] ?? 0) * root.characterWidth
                    y: 0
                    width: (row.segments[3 * mark.index + 1] ?? 0) * root.characterWidth
                    height: row.height
                    ToolTip.visible: markHover.hovered
                    ToolTip.text: root.provider.octetMarkTexts[mark.diagnostic] ?? ""

                    WavyLine {
                        id: wave

                        y: mark.height - wave.height
                        width: mark.width
                        colour: mark.severity === 0 ? "#d02020" : mark.severity === 1 ? "#e08000" : "#2060d0"
                    }

                    HoverHandler {
                        id: markHover
                    }
                }
            }
        }
    }
}
