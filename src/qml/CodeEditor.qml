// A code editor with line numbers, the highlight of the current line and of the occurrences of the selection,
// the marks of the diagnostics, the editing keys, the insert mode and the overwrite mode of typing, the two
// idle timers, and the presentation and the editing of the octets of its document in hex and bin.
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
    // whether the Tab key inserts spaces up to the next tab stop instead of a tab character, the
    // spaces setting (FR-492); off where no owner binds it
    property bool tabsToSpaces: false
    // the index of the editor mode: 0 for txt, 1 for hex, 2 for bin
    property int mode: 0
    // the input group or the output group whose octets hex and bin present; null where the code
    // editor has no mode switch
    property var octets: null
    // whether the octets carry marks of diagnostics
    property bool octetsMarked: false
    // whether the two timers are held: while an edit of a playback is typed and after it, until
    // the user edits, so that the idle time and the long idle time do not expire by themselves
    // (FR-505)
    property bool held: false
    // the characters of a played edit still to be entered, and the function called after the last
    property string pendingText: ""
    property var playedDone: null
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
    // the selection and the cursor of the text area changed by the user, as offsets in UTF-16 code
    // units, emitted in the turn of the event loop after the change and not at all where the text
    // changed in the same turn, so that a move the typing makes is not recorded (FR-525)
    signal cursorMoved(int start, int end, int cursor)
    // the text area or the octet view received the active focus (FR-526)
    signal focusTaken()

    // whether the text changed in this turn of the event loop, and whether a cursor move of this
    // turn waits to be emitted (FR-525)
    property bool textChangedNow: false
    property bool cursorPending: false

    // Notes a cursor or selection change of the text area for `flushCursor`, unless the text changed
    // in the same turn or the editor is read-only or not in txt (FR-525).
    function noteCursor() {
        if (root.textChangedNow || textArea.readOnly || root.mode !== 0) {
            return;
        }
        root.cursorPending = true;
        Qt.callLater(root.flushCursor);
    }

    // Emits the cursor move noted in this turn, once, with the selection and the cursor of now.
    function flushCursor() {
        if (!root.cursorPending) {
            return;
        }
        root.cursorPending = false;
        root.cursorMoved(textArea.selectionStart, textArea.selectionEnd, textArea.cursorPosition);
    }

    // Clears the note that the text changed in this turn.
    function clearTextChangedNow() {
        root.textChangedNow = false;
    }

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

    // Yields whether the key is the insert key, which toggles the typing mode; its key event is then
    // accepted, so that the text area enters nothing for it.
    function toggledBy(event: KeyEvent): bool {
        const toggled = Workbench.toggledByKey(event.key, event.modifiers);
        if (toggled) {
            event.accepted = true;
        }
        return toggled;
    }

    // Inserts a line break and the indentation of the line of the cursor in place of the selection.
    function breakLine(event: KeyEvent) {
        root.held = false;
        if (root.toggledBy(event)) {
            return;
        }
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

    // Replaces the text between the offsets `start` and `end` by `text`, typed one character after
    // another with `delay` milliseconds between two, the cursor after the character entered, and
    // calls `done` after the last; with the delay 0 the whole text is entered at once. The two
    // timers are stopped and held meanwhile (FR-505).
    function playEdit(start: int, end: int, text: string, delay: int, done: var) {
        root.held = true;
        idleTimer.stop();
        longIdleTimer.stop();
        textArea.remove(start, end);
        textArea.cursorPosition = start;
        if (delay <= 0) {
            textArea.insert(start, text);
            textArea.cursorPosition = start + text.length;
            done();
            return;
        }
        root.pendingText = text;
        root.playedDone = done;
        typingTimer.interval = delay;
        typingTimer.start();
    }

    // Places the cursor at the offset (FR-508).
    function placeCursor(offset: int) {
        textArea.cursorPosition = offset;
    }

    // Selects the text between the two offsets, the cursor at `end` (FR-508).
    function selectRange(start: int, end: int) {
        textArea.select(start, end);
    }

    // Gives the active focus to the text area, its window raised and activated (FR-508).
    function takeFocus() {
        const window = root.Window.window;
        if (window) {
            window.show();
            window.raise();
            window.requestActivate();
        }
        textArea.forceActiveFocus();
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
        visible: root.mode === 0
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
            // in the overwrite mode a character typed replaces the one at the cursor
            overwriteMode: !textArea.readOnly && !Workbench.insertMode
            cursorDelegate: Rectangle {
                id: textCursor

                // whether the cursor is in the shown half of its blinking
                property bool shown: true

                // the cursor of a text in the insert mode, a highlighted character in the overwrite mode
                width: textArea.overwriteMode ? root.characterWidth : 1
                color: textArea.overwriteMode ? root.palette.highlight : root.palette.text
                opacity: textArea.overwriteMode ? 0.6 : 1
                visible: textArea.cursorVisible && (textCursor.shown || textArea.overwriteMode)

                Timer {
                    id: textCursorTimer

                    interval: 500
                    repeat: true
                    running: textArea.cursorVisible
                    onTriggered: textCursor.shown = !textCursor.shown
                }

                Connections {
                    target: textArea

                    function onCursorPositionChanged() {
                        textCursor.shown = true;
                    }
                }
            }
            onTextChanged: {
                if (!root.held) {
                    idleTimer.restart();
                    longIdleTimer.restart();
                }
                // a cursor move of this turn is the typing's, not the user's (FR-525)
                root.textChangedNow = true;
                root.cursorPending = false;
                Qt.callLater(root.clearTextChangedNow);
                root.textEdited();
            }
            onCursorRectangleChanged: {
                if (textArea.activeFocus) {
                    Qt.callLater(root.showCursor);
                }
            }
            onCursorPositionChanged: root.noteCursor()
            onSelectionStartChanged: root.noteCursor()
            onSelectionEndChanged: root.noteCursor()
            onActiveFocusChanged: {
                if (textArea.activeFocus) {
                    root.focusTaken();
                }
            }
            Keys.onTabPressed: event => {
                root.held = false;
                if (root.toggledBy(event)) {
                    return;
                }
                if (textArea.readOnly || event.modifiers !== Qt.NoModifier) {
                    event.accepted = false;
                    return;
                }
                textArea.remove(textArea.selectionStart, textArea.selectionEnd);
                // spaces up to the next tab stop while the spaces setting is on (FR-492), one tab
                // character otherwise (FR-143)
                const inserted = root.tabsToSpaces
                    ? " ".repeat(Editing.spacesToTabStop(textArea.text, textArea.cursorPosition, root.tabSize))
                    : "\t";
                textArea.insert(textArea.cursorPosition, inserted);
            }
            Keys.onReturnPressed: event => root.breakLine(event)
            Keys.onEnterPressed: event => root.breakLine(event)
            Keys.onPressed: event => {
                // the user edits again: the timers run as usual (FR-505)
                root.held = false;
                if (root.toggledBy(event)) {
                    return;
                }
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
                // the four arrays change one after another; a mark about to be removed may read
                // an array already shorter than its index, and takes a neutral value until then
                start: root.markStarts[index] ?? 0
                end: root.markEnds[index] ?? 0
                severity: root.markSeverities[index] ?? 0
                message: root.markTexts[index] ?? ""
            }
        }
    }

    OctetView {
        id: octetView

        anchors.fill: parent
        visible: root.mode !== 0
        provider: root.octets
        mode: root.mode === 2 ? 2 : 1
        marked: root.octetsMarked
        digitFont: textArea.font
        editable: !textArea.readOnly
        // an edit of a digit restarts the two timers as an edit of the text does
        onEdited: {
            root.held = false;
            idleTimer.restart();
            longIdleTimer.restart();
        }
        onActiveFocusChanged: {
            if (octetView.activeFocus) {
                root.focusTaken();
            }
        }
    }

    // enters the next character of a played edit every `delay` milliseconds while characters
    // remain, a surrogate pair as one, and calls the function given after the last (FR-505)
    Timer {
        id: typingTimer

        repeat: true
        onTriggered: {
            if (root.pendingText.length === 0) {
                typingTimer.stop();
                const done = root.playedDone;
                root.playedDone = null;
                if (done) {
                    done();
                }
                return;
            }
            const character = String.fromCodePoint(root.pendingText.codePointAt(0));
            root.pendingText = root.pendingText.substring(character.length);
            const position = textArea.cursorPosition;
            textArea.insert(position, character);
            textArea.cursorPosition = position + character.length;
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
