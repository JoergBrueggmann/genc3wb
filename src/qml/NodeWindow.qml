// The node window as a machine: the inputs, the machine logo, the outputs, the meta compiler DSL as its control,
// the status line, and the detached windows.
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
        columns: 3

        GroupBox {
            id: inputsGroup

            title: qsTr("Inputs")
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.preferredWidth: 1

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

        ColumnLayout {
            id: machineColumn

            Layout.fillHeight: true
            Layout.alignment: Qt.AlignVCenter

            RowLayout {
                Layout.alignment: Qt.AlignHCenter

                Label {
                    id: inputArrow

                    text: "\u2192"
                    font.pointSize: 28
                }

                Image {
                    id: machineLogo

                    source: "qrc:/genc3wb/qml/icons/machine.png"
                    sourceSize.width: 96
                    sourceSize.height: 96
                }

                Label {
                    id: outputArrow

                    text: "\u2192"
                    font.pointSize: 28
                }
            }

            Label {
                id: controlArrow

                Layout.alignment: Qt.AlignHCenter
                text: "\u2191"
                font.pointSize: 28
            }
        }

        OutputGroup {
            id: outputGroup

            output: Workbench.output
            Layout.fillWidth: true
            Layout.fillHeight: true
            Layout.preferredWidth: 1
            onDetachRequested: outputWindowModel.append({ "number": outputWindowModel.count })
        }

        InputGroup {
            id: metaDslGroup

            group: Workbench.node.metaDsl
            Layout.columnSpan: 3
            Layout.fillWidth: true
            Layout.fillHeight: true
            onDetachRequested: editorWindowModel.append({ "input": -1 })
        }

        Label {
            id: statusLine

            Layout.columnSpan: 3
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
