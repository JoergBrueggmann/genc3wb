// The main window: menu bar, status bar, the four groups, the info dialog and the detached windows.
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

ApplicationWindow {
    id: mainWindow

    width: 1200
    height: 760
    visible: true
    title: "genc³wb"

    menuBar: MenuBar {
        id: menuBar

        Menu {
            id: helpMenu

            title: qsTr("&Help")

            Action {
                id: infoAction

                text: qsTr("&Info")
                onTriggered: infoDialog.open()
            }
        }
    }

    footer: Label {
        id: statusBar

        padding: 4
        text: Workbench.runner.running ? qsTr("The compiler-compiler is running.") : ""
    }

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
    }

    InfoDialog {
        id: infoDialog
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

            transientParent: mainWindow
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
            transientParent: mainWindow
            visible: true
            onClosing: editorWindowModel.remove(index)
        }
    }
}
