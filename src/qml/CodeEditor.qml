// A code editor with line numbers, the highlight of the current line, and the idle timer.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

import QtQuick
import QtQuick.Controls

Frame {
    id: root

    property alias text: textArea.text
    property alias readOnly: textArea.readOnly
    property int maxIdleTime: 1000

    signal textEdited()
    signal idleExpired()

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
                root.textEdited();
            }
        }
    }

    Timer {
        id: idleTimer

        interval: root.maxIdleTime
        onTriggered: root.idleExpired()
    }
}
