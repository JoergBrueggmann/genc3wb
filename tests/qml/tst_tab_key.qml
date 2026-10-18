// The tests of the Tab key at a code editor (FR-143, FR-492): with the spaces setting off it inserts one tab
// character, with the setting on as many spaces as reach the next tab stop, and in a read-only editor nothing.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

import QtQuick
import QtQuick.Controls
import QtTest
import genc3wb

Item {
    id: root

    width: 640
    height: 400

    CodeEditor {
        id: codeEditor

        anchors.fill: parent
        tabSize: 4
    }

    TestCase {
        id: tabKeyTest

        name: "TabKey"
        when: windowShown

        // Yields the first item below `item` that `isWanted` accepts, or null.
        function itemBelow(item: Item, isWanted: var): var {
            for (let index = 0; index < item.children.length; index++) {
                const child = item.children[index];
                const found = isWanted(child) ? child : tabKeyTest.itemBelow(child, isWanted);
                if (found) {
                    return found;
                }
            }
            return null;
        }

        // Shows the text `text` in txt with the cursor at `cursor`, the spaces setting as `spaces`, and gives
        // the focus to the text area.
        function prepare(text: string, cursor: int, spaces: bool): var {
            codeEditor.mode = 0;
            codeEditor.readOnly = false;
            codeEditor.tabsToSpaces = spaces;
            codeEditor.text = text;
            const textArea = tabKeyTest.itemBelow(codeEditor, item => item instanceof TextArea);
            textArea.cursorPosition = cursor;
            textArea.forceActiveFocus();
            return textArea;
        }

        function test_tab_inserts_a_tab_character_while_the_setting_is_off() {
            const textArea = tabKeyTest.prepare("ab", 1, false);
            keyClick(Qt.Key_Tab);
            compare(textArea.text, "a\tb");
        }

        function test_tab_inserts_spaces_to_the_next_tab_stop_while_the_setting_is_on() {
            const textArea = tabKeyTest.prepare("ab", 1, true);
            keyClick(Qt.Key_Tab);
            compare(textArea.text, "a   b");
            textArea.cursorPosition = 4;
            keyClick(Qt.Key_Tab);
            compare(textArea.text, "a       b");
        }

        function test_tab_replaces_the_selection_by_the_spaces() {
            const textArea = tabKeyTest.prepare("abcdef", 0, true);
            textArea.select(1, 5);
            keyClick(Qt.Key_Tab);
            compare(textArea.text, "a   f");
        }

        function test_tab_enters_nothing_into_a_read_only_editor() {
            const textArea = tabKeyTest.prepare("ab", 1, true);
            codeEditor.readOnly = true;
            keyClick(Qt.Key_Tab);
            compare(textArea.text, "ab");
        }
    }
}
