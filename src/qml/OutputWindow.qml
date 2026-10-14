// A detached window of the output group of one node window.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

import QtQuick
// The types of the module genc3wb are registered by the bridge crate at run time, which writes no type
// description for qmllint; the import is therefore not linted.
// qmllint disable import
import genc3wb

Window {
    id: outputWindow

    // the output group of the node editor of the node window this window was detached from
    required property var output

    width: 700
    height: 500
    title: qsTr("genc³wb - Output")

    OutputGroup {
        id: outputGroup

        anchors.fill: parent
        anchors.margins: 8
        output: outputWindow.output
        detachable: false
    }
}
