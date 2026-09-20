// The node window: the four groups of one node, a status line, and the detached windows.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts
// The types of the module genc3wb are registered by the bridge crate at run time, which writes no type
// description for qmllint; the import and the unqualified access to the singleton are therefore not linted.
// qmllint disable import unqualified
import genc3wb

Window {
    id: nodeWindow

    property string nodeName: ""

    width: 1200
    height: 760
    title: nodeWindow.nodeName.length > 0 ? "genc³wb - " + nodeWindow.nodeName : "genc³wb"

    GridLayout {
        id: layout

        anchors.fill: parent
        anchors.margins: 6
        columns: 2

        InputGroup {
            id: ccInputGroup

            group: Workbench.ccInput
            Layout.fillWidth: true
            Layout.fillHeight: true
            onDetachRequested: editorWindowModel.append({ "kind": 0 })
        }

        OutputGroup {
            id: outputGroup

            output: Workbench.output
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.rowSpan: 2
            onDetachRequested: outputWindowModel.append({ "number": outputWindowModel.count })
        }

        InputGroup {
            id: cInputGroup

            group: Workbench.cInput
            Layout.fillWidth: true
            Layout.fillHeight: true
            onDetachRequested: editorWindowModel.append({ "kind": 1 })
        }

        RunnerGroup {
            id: runnerGroup

            runner: Workbench.runner
            Layout.fillWidth: true
            Layout.columnSpan: 2
        }

        Label {
            id: runLabel

            Layout.columnSpan: 2
            text: Workbench.runner.running ? qsTr("The compiler-compiler is running.") : ""
        }
    }

    ListModel {
        id: outputWindowModel
    }

    ListModel {
        id: editorWindowModel
    }

    Instantiator {
        id: outputWindows

        model: outputWindowModel
        delegate: OutputWindow {
            required property int index

            transientParent: nodeWindow
            visible: true
            onClosing: outputWindowModel.remove(index)
        }
    }

    Instantiator {
        id: editorWindows

        model: editorWindowModel
        delegate: EditorWindow {
            required property int index
            required property int kind

            group: kind === 0 ? Workbench.ccInput : Workbench.cInput
            transientParent: nodeWindow
            visible: true
            onClosing: editorWindowModel.remove(index)
        }
    }
}
