// The node window: the meta compiler DSL, the inputs with their navigation, the outputs, the status line, and
// the detached windows.
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

    width: 1200
    height: 760
    title: Workbench.node.name.length > 0 ? "genc³wb - " + Workbench.node.name : "genc³wb"
    onClosing: Workbench.node.close()

    GridLayout {
        id: layout

        anchors.fill: parent
        anchors.margins: 6
        columns: 2

        InputGroup {
            id: metaDslGroup

            group: Workbench.node.metaDsl
            Layout.fillWidth: true
            Layout.fillHeight: true
            onDetachRequested: editorWindowModel.append({ "input": -1 })
        }

        OutputGroup {
            id: outputGroup

            output: Workbench.output
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.rowSpan: 2
            onDetachRequested: outputWindowModel.append({ "number": outputWindowModel.count })
        }

        GroupBox {
            id: inputsGroup

            title: qsTr("Inputs")
            Layout.fillWidth: true
            Layout.fillHeight: true

            ColumnLayout {
                anchors.fill: parent

                RowLayout {
                    Button {
                        id: previousInputButton

                        text: "<"
                        enabled: Workbench.node.hasPreviousInput
                        onClicked: Workbench.node.previousInput()
                    }

                    Label {
                        id: inputLabel

                        text: qsTr("input %1 of %2").arg(Workbench.node.inputPage + 1).arg(Workbench.node.inputCount)
                    }

                    Button {
                        id: nextInputButton

                        text: ">"
                        enabled: Workbench.node.hasNextInput
                        onClicked: Workbench.node.nextInput()
                    }
                }

                StackLayout {
                    id: inputPages

                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    currentIndex: Workbench.node.inputPage

                    Repeater {
                        id: inputRepeater

                        model: Workbench.node.inputCount
                        delegate: InputGroup {
                            required property int index

                            group: Workbench.node.inputAt(index)
                            onDetachRequested: editorWindowModel.append({ "input": index })
                        }
                    }
                }
            }
        }

        Label {
            id: statusLine

            Layout.columnSpan: 2
            text: Workbench.node.status.length > 0
                  ? Workbench.node.status
                  : !Workbench.node.served
                    ? qsTr("The node is not served.")
                    : Workbench.node.busy ? qsTr("The node is processing.") : qsTr("The node is served.")
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
            required property int input

            group: input < 0 ? Workbench.node.metaDsl : Workbench.node.inputAt(input)
            transientParent: nodeWindow
            visible: true
            onClosing: editorWindowModel.remove(index)
        }
    }
}
