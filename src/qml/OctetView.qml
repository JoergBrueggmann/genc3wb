// The presentation of the octets of a document in hex or bin: the column header, and one row per as many
// octets as fit into the width of the view, with its row label, its digits and the marks of the diagnostics;
// where it is editable, the cursor, the selection, the keys that edit the digits, and the clipboard.
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
    // whether the digits are edited: for an input group, not for an output group
    property bool editable: false

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
    // the number of rows; an editable view has one row at least, in which its cursor stands
    readonly property int rowCount: root.provider && root.revision >= 0
                                    ? Math.max(root.provider.rowCount(root.octetsPerRow), root.editable ? 1 : 0) : 0
    // the width of the row labels: that of the label of the last row, and a margin on each side
    readonly property real labelWidth: (Editing.rowLabel(root.octetsPerRow, Math.max(root.rowCount - 1, 0),
                                                         root.octetCount).length + 1) * root.characterWidth + 4
    // the width of a row: its label, the digits of a full row, and the cell of the cursor behind them
    readonly property real rowWidth: root.labelWidth + (header.text.length + 2) * root.characterWidth

    // the number of bits of the octets that are digits of the document
    readonly property int bitCount: root.editable && root.provider ? root.provider.bitCount : 0
    // the number of digits of the document in the editor mode
    readonly property int digitCount: Editing.digitCount(root.mode, root.bitCount)
    // the digit the cursor was set before, and the editor mode it was counted in
    property int cursorDigit: 0
    property int cursorMode: 1
    // the digit the cursor stands before: the one it was set before, within the document
    readonly property int cursor: Math.min(root.cursorDigit, root.digitCount)
    // the digit at which the selection is anchored; -1 where there is none
    property int anchor: -1
    // the selection: its first digit and the digit behind its last one; equal where there is none
    readonly property int selectionStart: root.anchor < 0 ? root.cursor
                                                           : Math.min(root.cursor, root.anchor, root.digitCount)
    readonly property int selectionEnd: root.anchor < 0 ? root.cursor
                                                         : Math.min(Math.max(root.cursor, root.anchor),
                                                                    root.digitCount)
    // where the cursor is shown: its row, its character column, and what it stands before
    readonly property var cursorCell: Editing.cursorCell(root.cursor, root.mode, root.octetsPerRow, root.bitCount,
                                                         root.octetCount)
    // whether the cursor is in the shown half of its blinking
    property bool cursorShown: true
    // the number of rows the view shows
    readonly property int pageRows: Math.max(Math.floor(rows.height / Math.max(fontMetrics.height, 1)) - 1, 1)

    // the user edited a digit, which restarts the idle timers of the code editor
    signal edited()

    // Sets the cursor before `digit`; where `selecting`, the selection reaches from where it was
    // anchored, or from the cursor before, to `digit`, and there is none otherwise.
    function setCursor(digit: int, selecting: bool) {
        root.anchor = selecting ? (root.anchor < 0 ? root.cursor : root.anchor) : -1;
        root.cursorDigit = digit;
        root.cursorMode = root.mode;
        root.cursorShown = true;
        rows.positionViewAtIndex(Editing.cursorCell(digit, root.mode, root.octetsPerRow, root.bitCount,
                                                    root.octetCount)[0], ListView.Contain);
    }

    // Replaces `removed` digits from `start` on by the digits of `text`, and sets the cursor behind them.
    function replace(start: int, removed: int, text: string) {
        const cursor = root.provider.replaceDigits(root.mode, start, removed, text);
        if (cursor >= 0) {
            root.setCursor(cursor, false);
            root.edited();
        }
    }

    // Types or pastes the digits of `text` at the cursor, in place of the selection where there is one.
    function type(text: string) {
        const span = Editing.typingSpan(root.cursor, root.anchor, Editing.digitsIn(root.mode, text),
                                        Workbench.insertMode);
        root.replace(span[0], span[1], text);
    }

    // Removes the selection, or the digit before the cursor where `backward` and the one at it otherwise.
    function remove(backward: bool) {
        const span = Editing.removalSpan(root.cursor, root.anchor, backward);
        root.replace(span[0], span[1], "");
    }

    // Puts the digits of the selection on the clipboard as text.
    function copySelection() {
        if (root.selectionEnd > root.selectionStart) {
            clipboard.text = root.provider.digitsText(root.mode, root.selectionStart, root.selectionEnd);
            clipboard.selectAll();
            clipboard.copy();
        }
    }

    // Pastes the digits the clipboard holds as text.
    function pasteDigits() {
        clipboard.text = "";
        clipboard.paste();
        if (Editing.digitsValid(root.mode, clipboard.text)) {
            root.type(clipboard.text);
        }
    }

    // Takes the last edit back, or makes the one taken back last again where `again`.
    function undoEdit(again: bool) {
        const cursor = again ? root.provider.redo(root.mode) : root.provider.undo(root.mode);
        if (cursor >= 0) {
            root.setCursor(cursor, false);
            root.edited();
        }
    }

    // Moves the cursor by the movement of index `movement`, selecting where Shift is held.
    function move(movement: int, event: KeyEvent) {
        root.setCursor(Editing.cursorMoved(root.cursor, movement, root.mode, root.octetsPerRow, root.bitCount,
                                           root.pageRows), (event.modifiers & Qt.ShiftModifier) !== 0);
    }

    // Yields the digit at a point of the view, between two digits where `between`.
    function digitAtPoint(x: real, y: real, between: bool): int {
        const index = rows.indexAt(rows.contentX + 1, rows.contentY + y);
        const row = index < 0 ? Math.max(root.rowCount - 1, 0) : index;
        return Editing.digitAt(row, (rows.contentX + x - root.labelWidth) / Math.max(root.characterWidth, 1),
                               between, root.mode, root.octetsPerRow, root.bitCount);
    }

    // Handles a key: the insert key, the movements, the clipboard, undo and redo, the removals, and the digits.
    function handleKey(event: KeyEvent) {
        const movements = [Qt.Key_Left, Qt.Key_Right, Qt.Key_Up, Qt.Key_Down, Qt.Key_Home, Qt.Key_End,
                           Qt.Key_PageUp, Qt.Key_PageDown];
        const movement = movements.indexOf(event.key);
        const whole = (event.modifiers & Qt.ControlModifier) !== 0;
        if (Workbench.toggledByKey(event.key, event.modifiers)) {
            return;
        }
        // on MacOS the command key with an arrow key moves to the start or the end of the row or of the document
        const mac = Qt.platform.os === "osx" && whole;
        if (movement >= 0) {
            root.move(whole && event.key === Qt.Key_Home || mac && event.key === Qt.Key_Up ? 8
                      : whole && event.key === Qt.Key_End || mac && event.key === Qt.Key_Down ? 9
                      : mac && event.key === Qt.Key_Left ? 4 : mac && event.key === Qt.Key_Right ? 5 : movement, event);
        } else if (event.matches(StandardKey.SelectAll)) {
            root.anchor = 0;
            root.cursorDigit = root.digitCount;
            root.cursorMode = root.mode;
        } else if (event.matches(StandardKey.Copy)) {
            root.copySelection();
        } else if (event.matches(StandardKey.Cut)) {
            root.copySelection();
            if (root.selectionEnd > root.selectionStart) {
                root.remove(true);
            }
        } else if (event.matches(StandardKey.Paste)) {
            root.pasteDigits();
        } else if (event.matches(StandardKey.Undo)) {
            root.undoEdit(false);
        } else if (event.matches(StandardKey.Redo)) {
            root.undoEdit(true);
        } else if (event.key === Qt.Key_Backspace || event.key === Qt.Key_Delete) {
            root.remove(event.key === Qt.Key_Backspace);
        } else if ((event.modifiers & ~(Qt.ShiftModifier | Qt.KeypadModifier)) === Qt.NoModifier
                   && Editing.digitsValid(root.mode, event.text)) {
            root.type(event.text);
        } else {
            event.accepted = false;
        }
    }

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
    activeFocusOnTab: root.editable
    Keys.onPressed: event => {
        if (root.editable) {
            root.handleKey(event);
        } else {
            event.accepted = false;
        }
    }
    onModeChanged: {
        root.cursorDigit = Editing.digitInMode(root.cursorDigit, root.cursorMode, root.mode);
        root.cursorMode = root.mode;
        root.anchor = -1;
    }
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

    // the clipboard is reached through a text edit, which copies and pastes its own text
    TextEdit {
        id: clipboard

        visible: false
    }

    Timer {
        id: blinkTimer

        interval: 500
        repeat: true
        running: root.editable && root.activeFocus
        onTriggered: root.cursorShown = !root.cursorShown
    }

    Connections {
        target: root.editable ? root.provider : null

        function onDocumentChanged() {
            root.cursorDigit = 0;
            root.anchor = -1;
        }
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

            // the character columns of the selection in this row: the first one and their number
            readonly property var selection: root.editable && root.selectionEnd > root.selectionStart
                                             ? Editing.rowSelection(root.selectionStart, root.selectionEnd,
                                                                    row.index, root.mode, root.octetsPerRow) : []
            // whether the cursor stands in this row
            readonly property bool holdsCursor: root.editable && root.activeFocus && root.cursorCell[0] === row.index
            // whether the cursor is shown as a highlighted cell: in the overwrite mode before a
            // digit, and before a placeholder in either mode; it is the cursor of a text otherwise
            readonly property bool cursorIsCell: root.cursorCell[2] === 1
                                                 || (root.cursorCell[2] === 0 && !Workbench.insertMode)

            width: root.rowWidth
            height: digits.implicitHeight

            Rectangle {
                id: selectionHighlight

                x: digits.x + (row.selection[0] ?? 0) * root.characterWidth
                width: (row.selection[1] ?? 0) * root.characterWidth
                height: row.height
                color: palette.highlight
                opacity: 0.4
                visible: row.selection.length === 2
            }

            Rectangle {
                id: cursorMark

                x: digits.x + root.cursorCell[1] * root.characterWidth
                width: row.cursorIsCell ? root.characterWidth : 1
                height: row.height
                color: row.cursorIsCell ? palette.highlight : palette.text
                opacity: row.cursorIsCell ? 0.6 : 1
                visible: row.holdsCursor && (root.cursorShown || row.cursorIsCell)
            }

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

    MouseArea {
        id: pointerArea

        anchors.fill: rows
        anchors.rightMargin: verticalBar.width
        enabled: root.editable
        cursorShape: root.editable ? Qt.IBeamCursor : Qt.ArrowCursor
        preventStealing: true
        onPressed: mouse => {
            root.forceActiveFocus();
            root.setCursor(root.digitAtPoint(mouse.x, mouse.y, Workbench.insertMode),
                           (mouse.modifiers & Qt.ShiftModifier) !== 0);
        }
        onPositionChanged: mouse => root.setCursor(root.digitAtPoint(mouse.x, mouse.y, true), true)
    }
}
