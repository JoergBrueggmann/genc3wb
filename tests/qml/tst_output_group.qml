// The tests of the page label of the output group (FR-036): the presented page is counted from 0 and the
// number after 'of' is the number of pages, the diagnostics page named.
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

    // the stand-in of the bridged output group: two outputs and the diagnostics page, the presented page
    // set by the test
    QtObject {
        id: outputStandIn

        property int pageIndex: 0
        property int pageCount: 3
        property bool diagnosticsPage: false
        property bool hasNext: true
        property bool hasPrevious: false
        property string filePath: "out.txt"
        property string fileContent: ""
        property bool isText: true
        property bool outdated: false
        property string storeFailure: ""
        property string diagnostics: ""
        property var origins: []
        property int revision: 0
        property int octetCount: 0
        property int bitCount: 0
        property var octetMarkSeverities: []
        property var octetMarkTexts: []

        signal documentChanged()

        function next() {
        }

        function previous() {
        }

        function rowCount(octetsPerRow: int): int {
            return 0;
        }

        function rowText(mode: int, octetsPerRow: int, row: int): string {
            return "";
        }

        function rowMarks(mode: int, octetsPerRow: int, row: int): var {
            return [];
        }
    }

    OutputGroup {
        id: outputGroup

        anchors.fill: parent
        output: outputStandIn
    }

    TestCase {
        id: testCase

        name: "OutputGroup"
        when: windowShown

        function init() {
            outputStandIn.pageIndex = 0;
            outputStandIn.diagnosticsPage = false;
        }

        // Yields the text of the page label.
        function pageLabelText(): string {
            return findChild(outputGroup, "pageLabel").text;
        }

        function test_first_output_page_is_counted_from_0() {
            compare(pageLabelText(), "Output 0 of 3");
        }

        function test_second_output_page_is_page_1() {
            outputStandIn.pageIndex = 1;
            compare(pageLabelText(), "Output 1 of 3");
        }

        function test_diagnostics_page_is_named_and_counted_from_0() {
            outputStandIn.pageIndex = 2;
            outputStandIn.diagnosticsPage = true;
            compare(pageLabelText(), "Diagnostics (page 2 of 3)");
        }
    }
}
