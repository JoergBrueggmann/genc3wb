// A code editor with line numbers, the highlight of the current line and of the occurrences of the selection,
// the marks of the diagnostics, the editing keys, and the two idle timers.
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

Frame {
    id: root

    property alias text: textArea.text
    property alias readOnly: textArea.readOnly
    property int idleTime: 2000
    property int longIdleTime: 16000
    // the tab size: the number of characters a tab character is displayed as wide
    property int tabSize: 4
    // the marks of the diagnostics: per diagnostic its start and its end as offsets of the text,
    // the index of its severity, and its message text
    property var markStarts: []
    property var markEnds: []
    property var markSeverities: []
    property var markTexts: []
    // the width of one character of the monospaced font
    readonly property real characterWidth: fontMetrics.averageCharacterWidth
    // the height of one line of the text
    readonly property real lineHeight: Math.max(textArea.cursorRectangle.height, 1)
    // the other occurrences of the selection: per occurrence its start and its end offset in turn
    readonly property var occurrenceOffsets: Editing.occurrences(textArea.text, textArea.selectionStart,
                                                                 textArea.selectionEnd)

    signal textEdited()
    signal idleExpired()
    signal longIdleExpired()

    // Yields the line, counted from 0, at `y` of the content of the flickable.
    function lineAt(y: real): int {
        return Math.floor((y - textArea.y - textArea.topPadding) / root.lineHeight);
    }

    // Selects the lines from `from` to `to`, the cursor at the end of line `to`.
    function selectLines(from: int, to: int) {
        const span = Editing.linesSpan(textArea.text, from, to);
        if (to < from) {
            textArea.select(span[1], span[0]);
        } else {
            textArea.select(span[0], span[1]);
        }
    }

    // Scrolls the flickable so that the cursor is visible with a margin of one line and one character.
    function showCursor() {
        const cursor = textArea.cursorRectangle;
        flickable.contentY = Editing.viewOffsetShowing(flickable.contentY, flickable.height,
                                                       flickable.contentHeight,
                                                       textArea.y + cursor.y - root.lineHeight,
                                                       textArea.y + cursor.y + cursor.height + root.lineHeight);
        const left = cursor.x - root.characterWidth < textArea.leftPadding
                   ? 0 : textArea.x + cursor.x - root.characterWidth;
        flickable.contentX = Editing.viewOffsetShowing(flickable.contentX, flickable.width,
                                                       flickable.contentWidth, left,
                                                       textArea.x + cursor.x + cursor.width + root.characterWidth);
    }

    // Inserts a line break and the indentation of the line of the cursor in place of the selection.
    function breakLine(event: KeyEvent) {
        if (textArea.readOnly || (event.modifiers & ~Qt.KeypadModifier) !== Qt.NoModifier) {
            event.accepted = false;
            return;
        }
        textArea.remove(textArea.selectionStart, textArea.selectionEnd);
        const position = textArea.cursorPosition;
        const inserted = "\n" + Editing.indentationOfLine(textArea.text, position);
        textArea.insert(position, inserted);
        textArea.cursorPosition = position + inserted.length;
    }

    // Moves the cursor by the height of the view, down or up; with Shift the selection is extended.
    function movePage(event: KeyEvent, down: bool) {
        const cursor = textArea.cursorRectangle;
        const position = textArea.positionAt(cursor.x, cursor.y + (down ? flickable.height : -flickable.height));
        if (event.modifiers & Qt.ShiftModifier) {
            textArea.moveCursorSelection(position, TextEdit.SelectCharacters);
        } else {
            textArea.cursorPosition = position;
        }
    }

    padding: 1

    FontMetrics {
        id: fontMetrics

        font: textArea.font
    }

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

        Repeater {
            id: occurrenceRepeater

            model: root.occurrenceOffsets.length / 2
            delegate: Rectangle {
                id: occurrence

                required property int index

                readonly property rect startRectangle: textArea.positionToRectangle(
                    root.occurrenceOffsets[2 * occurrence.index])
                readonly property rect endRectangle: textArea.positionToRectangle(
                    root.occurrenceOffsets[2 * occurrence.index + 1])

                x: textArea.x + occurrence.startRectangle.x
                y: textArea.y + occurrence.startRectangle.y
                width: occurrence.endRectangle.x - occurrence.startRectangle.x
                height: occurrence.startRectangle.height
                color: "#dde8f8"
            }
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

        MouseArea {
            id: lineNumberArea

            // the line of the mouse-down event
            property int anchorLine: 0
            // the pointer relative to the top of the view: beyond it where negative or greater than
            // the height of the view
            property real pointerY: 0

            x: 0
            y: 0
            width: lineNumbers.width
            height: flickable.contentHeight
            preventStealing: true
            onPressed: mouse => {
                textArea.forceActiveFocus();
                lineNumberArea.anchorLine = root.lineAt(mouse.y);
                lineNumberArea.pointerY = mouse.y - flickable.contentY;
                root.selectLines(lineNumberArea.anchorLine, lineNumberArea.anchorLine);
            }
            onPositionChanged: mouse => {
                lineNumberArea.pointerY = mouse.y - flickable.contentY;
                root.selectLines(lineNumberArea.anchorLine, root.lineAt(mouse.y));
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
            tabStopDistance: root.tabSize * root.characterWidth
            background: null
            onTextChanged: {
                idleTimer.restart();
                longIdleTimer.restart();
                root.textEdited();
            }
            onCursorRectangleChanged: {
                if (textArea.activeFocus) {
                    Qt.callLater(root.showCursor);
                }
            }
            Keys.onTabPressed: event => {
                if (textArea.readOnly || event.modifiers !== Qt.NoModifier) {
                    event.accepted = false;
                    return;
                }
                textArea.remove(textArea.selectionStart, textArea.selectionEnd);
                textArea.insert(textArea.cursorPosition, "\t");
            }
            Keys.onReturnPressed: event => root.breakLine(event)
            Keys.onEnterPressed: event => root.breakLine(event)
            Keys.onPressed: event => {
                if (event.key === Qt.Key_PageDown || event.key === Qt.Key_PageUp) {
                    root.movePage(event, event.key === Qt.Key_PageDown);
                } else {
                    event.accepted = false;
                }
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

    // scrolls the view while the pointer that drags in the line-number area is beyond it: the line
    // at the pointer is selected, which moves the cursor, which the view follows
    Timer {
        id: autoScrollTimer

        interval: 100
        repeat: true
        running: lineNumberArea.pressed && (lineNumberArea.pointerY < 0 || lineNumberArea.pointerY > flickable.height)
        onTriggered: root.selectLines(lineNumberArea.anchorLine,
                                      root.lineAt(flickable.contentY + lineNumberArea.pointerY))
    }
}
