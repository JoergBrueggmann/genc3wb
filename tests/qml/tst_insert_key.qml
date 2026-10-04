// The tests of the insert key at a code editor (FR-193): in txt, hex and bin the insert key toggles the
// typing mode, enters nothing, and its key event passes on to no item above the code editor.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

import QtQuick
import QtQuick.Controls
import QtTest
import genc3wb

Item {
    id: root

    // the number of key events of the insert key that passed on from the code editor to the item above it
    property int passedOn: 0

    width: 640
    height: 400

    Item {
        id: aboveItem

        anchors.fill: parent
        Keys.onPressed: event => {
            if (event.key === Workbench.insertKey) {
                root.passedOn += 1;
            }
        }

        CodeEditor {
            id: codeEditor

            anchors.fill: parent
            octets: octetProvider
        }
    }

    // the stand-in of an input group: a document of two octets, counting the edits of its digits
    QtObject {
        id: octetProvider

        property int revision: 0
        property int octetCount: 2
        property int bitCount: 16
        property var octetMarkSeverities: []
        property var octetMarkTexts: []
        // the number of calls that edit the digits
        property int edits: 0

        function rowCount(octetsPerRow: int): int {
            return 1;
        }

        function rowText(mode: int, octetsPerRow: int, row: int): string {
            return mode === 2 ? "00000000 00000000" : "00 00";
        }

        function rowMarks(mode: int, octetsPerRow: int, row: int): var {
            return [];
        }

        function digitsText(mode: int, start: int, end: int): string {
            return "";
        }

        function replaceDigits(mode: int, start: int, removed: int, text: string): int {
            octetProvider.edits += 1;
            return -1;
        }

        function undo(mode: int): int {
            octetProvider.edits += 1;
            return -1;
        }

        function redo(mode: int): int {
            octetProvider.edits += 1;
            return -1;
        }
    }

    TestCase {
        id: insertKeyTest

        name: "InsertKey"
        when: windowShown

        // Yields the first item below `item` that `isWanted` accepts, or null.
        function itemBelow(item: Item, isWanted: var): var {
            for (let index = 0; index < item.children.length; index++) {
                const child = item.children[index];
                const found = isWanted(child) ? child : insertKeyTest.itemBelow(child, isWanted);
                if (found) {
                    return found;
                }
            }
            return null;
        }

        // Shows the editor mode `mode` with the text `ab`, the cursor after `a`, and gives the focus
        // to the text area in txt and to the octet view in hex and bin.
        function prepare(mode: int): var {
            codeEditor.mode = mode;
            codeEditor.text = "ab";
            const textArea = insertKeyTest.itemBelow(codeEditor, item => item instanceof TextArea);
            textArea.cursorPosition = 1;
            const focused = mode === 0 ? textArea : insertKeyTest.itemBelow(codeEditor,
                                                                            item => item instanceof OctetView);
            focused.forceActiveFocus();
            Workbench.insertMode = true;
            Workbench.toggles = 0;
            root.passedOn = 0;
            octetProvider.edits = 0;
            return textArea;
        }

        // Sets the insert key of the row and presses it; a command key stroke is pressed with the
        // whole text selected, so that the text area would cut it.
        function pressInsertKey(row: var, textArea: var) {
            Workbench.insertKey = row.key;
            Workbench.insertModifiers = row.modifiers;
            if (row.modifiers !== Qt.NoModifier) {
                textArea.selectAll();
            }
            insertKeyTest.keyClick(row.key, row.modifiers);
        }

        function keys_data() {
            return [
                {tag: "printable key", key: Qt.Key_NumberSign, modifiers: Qt.NoModifier},
                {tag: "command key stroke", key: Qt.Key_X, modifiers: Qt.ControlModifier},
                {tag: "Return", key: Qt.Key_Return, modifiers: Qt.NoModifier},
                {tag: "Enter", key: Qt.Key_Enter, modifiers: Qt.NoModifier}
            ];
        }

        function modes_data() {
            const rows = [];
            for (const mode of [{name: "txt", index: 0}, {name: "hex", index: 1}, {name: "bin", index: 2}]) {
                for (const key of insertKeyTest.keys_data()) {
                    rows.push({tag: mode.name + ", " + key.tag, mode: mode.index, key: key.key,
                               modifiers: key.modifiers});
                }
            }
            return rows;
        }

        function test_insert_key_is_not_entered_as_text_data() {
            return insertKeyTest.keys_data();
        }

        function test_insert_key_is_not_entered_as_text(row: var) {
            const textArea = insertKeyTest.prepare(0);
            insertKeyTest.pressInsertKey(row, textArea);
            insertKeyTest.compare(codeEditor.text, "ab");
        }

        function test_insert_key_edits_no_digit_data() {
            return insertKeyTest.modes_data().filter(row => row.mode !== 0);
        }

        function test_insert_key_edits_no_digit(row: var) {
            const textArea = insertKeyTest.prepare(row.mode);
            insertKeyTest.pressInsertKey(row, textArea);
            insertKeyTest.compare(octetProvider.edits, 0);
        }

        function test_insert_key_toggles_the_typing_mode_data() {
            return insertKeyTest.modes_data();
        }

        function test_insert_key_toggles_the_typing_mode(row: var) {
            const textArea = insertKeyTest.prepare(row.mode);
            insertKeyTest.pressInsertKey(row, textArea);
            insertKeyTest.compare([Workbench.toggles, Workbench.insertMode], [1, false]);
        }

        function test_insert_key_does_not_pass_on_to_the_items_above_data() {
            return insertKeyTest.modes_data();
        }

        function test_insert_key_does_not_pass_on_to_the_items_above(row: var) {
            const textArea = insertKeyTest.prepare(row.mode);
            insertKeyTest.pressInsertKey(row, textArea);
            insertKeyTest.compare(root.passedOn, 0);
        }
    }
}
