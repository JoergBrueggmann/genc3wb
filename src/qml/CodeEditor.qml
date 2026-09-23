// A code editor with line numbers, the highlight of the current line, the marks of the diagnostics, and the two
// idle timers.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls

Frame {
    id: root

    property alias text: textArea.text
    property alias readOnly: textArea.readOnly
    property int idleTime: 2000
    property int longIdleTime: 16000
    // the marks of the diagnostics: per diagnostic its start and its end as offsets of the text,
    // the index of its severity, and its message text
    property var markStarts: []
    property var markEnds: []
    property var markSeverities: []
    property var markTexts: []

    signal textEdited()
    signal idleExpired()
    signal longIdleExpired()

    padding: 1

    Flickable {
        id: flickable

        anchors.fill: parent
        clip: true
        contentWidth: Math.max(width, lineNumbers.width + textArea.contentWidth + 12)
        contentHeight: Math.max(height, textArea.contentHeight + 8)

        ScrollBar.vertical: ScrollBar {}
        ScrollBar.horizontal: ScrollBar {}

        Rectangle {
            id: currentLine

            x: lineNumbers.width
            y: textArea.y + textArea.cursorRectangle.y
            width: Math.max(flickable.contentWidth - lineNumbers.width, 0)
            height: textArea.cursorRectangle.height
            color: "#ffffb0"
            visible: textArea.activeFocus && !textArea.readOnly
        }

        Text {
            id: lineNumbers

            x: 0
            y: textArea.y + textArea.topPadding
            width: implicitWidth
            leftPadding: 4
            rightPadding: 4
            font: textArea.font
            color: "gray"
            horizontalAlignment: Text.AlignRight
            text: {
                let numbers = [];
                for (let line = 1; line <= textArea.lineCount; line++) {
                    numbers.push(line);
                }
                return numbers.join("\n");
            }
        }

        TextArea {
            id: textArea

            x: lineNumbers.width
            y: 0
            width: Math.max(flickable.contentWidth - lineNumbers.width, 0)
            height: flickable.contentHeight
            font.family: "Courier New"
            wrapMode: TextEdit.NoWrap
            selectByMouse: true
            background: null
            onTextChanged: {
                idleTimer.restart();
                longIdleTimer.restart();
                root.textEdited();
            }
        }

        Repeater {
            id: markRepeater

            model: root.markTexts.length
            delegate: DiagnosticMark {
                required property int index

                x: textArea.x
                y: textArea.y
                textArea: textArea
                start: root.markStarts[index]
                end: root.markEnds[index]
                severity: root.markSeverities[index]
                message: root.markTexts[index]
            }
        }
    }

    Timer {
        id: idleTimer

        interval: root.idleTime
        onTriggered: root.idleExpired()
    }

    Timer {
        id: longIdleTimer

        interval: root.longIdleTime
        onTriggered: root.longIdleExpired()
    }
}
