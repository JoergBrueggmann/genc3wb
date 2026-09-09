// A detached window of the output group.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

import QtQuick
// The types of the module genc3wb are registered by the bridge crate at run time, which writes no type
// description for qmllint; the import and the unqualified access to the singleton are therefore not linted.
// qmllint disable import unqualified
import genc3wb

Window {
    id: outputWindow

    width: 700
    height: 500
    title: qsTr("genc³wb - output")

    OutputGroup {
        id: outputGroup

        anchors.fill: parent
        anchors.margins: 6
        output: Workbench.output
        detachable: false
    }
}
