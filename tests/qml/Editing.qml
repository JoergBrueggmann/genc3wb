// The stand-in of the bridged type Editing for the tests of the code editor: every function the code
// editor and the octet view call, yielding a value of the right form for a document of two octets.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

pragma Singleton

import QtQuick

QtObject {
    id: editing

    function occurrences(text: string, start: int, end: int): var {
        return [];
    }

    function linesSpan(text: string, from: int, to: int): var {
        return [0, text.length];
    }

    function indentationOfLine(text: string, position: int): string {
        return "";
    }

    function viewOffsetShowing(offset: real, view: real, content: real, start: real, end: real): real {
        return offset;
    }

    function octetsPerRow(mode: int, octetCount: int, columns: int): int {
        return 16;
    }

    function rowLabel(octetsPerRow: int, row: int, octetCount: int): string {
        return "00";
    }

    function columnHeader(mode: int, octetsPerRow: int): string {
        return mode === 2 ? "00000000 00000000" : "00 01";
    }

    function digitCount(mode: int, bitCount: int): int {
        return mode === 2 ? bitCount : bitCount / 4;
    }

    function cursorCell(cursor: int, mode: int, octetsPerRow: int, bitCount: int, octetCount: int): var {
        return [0, cursor, 0];
    }

    function cursorMoved(cursor: int, movement: int, mode: int, octetsPerRow: int, bitCount: int,
                         pageRows: int): int {
        return cursor;
    }

    function digitAt(row: int, column: real, between: bool, mode: int, octetsPerRow: int, bitCount: int): int {
        return 0;
    }

    function digitInMode(digit: int, from: int, to: int): int {
        return digit;
    }

    function digitsIn(mode: int, text: string): int {
        return text.length;
    }

    function digitsValid(mode: int, text: string): bool {
        return text.length > 0 && (mode === 2 ? /^[01]+$/ : /^[0-9a-fA-F]+$/).test(text);
    }

    function removalSpan(cursor: int, anchor: int, backward: bool): var {
        return [cursor, 0];
    }

    function rowKeeping(row: int, width: int, octetsPerRow: int): int {
        return row;
    }

    function rowSelection(start: int, end: int, row: int, mode: int, octetsPerRow: int): var {
        return [0, 0];
    }

    function typingSpan(cursor: int, anchor: int, count: int, insert: bool): var {
        return [cursor, insert ? 0 : count];
    }
}
