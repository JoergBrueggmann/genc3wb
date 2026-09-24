// The node window as a machine: the inputs above the meta compiler DSL as its control, beside them the machine
// logo and the outputs, the two splitters between them, the status line, and the detached windows.
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

    ColumnLayout {
        id: layout

        anchors.fill: parent
        anchors.margins: 6

        // the splitter between the left part, the inputs above the meta compiler DSL, and the right
        // part, the machine and the outputs (FR-146); its position is stored when it is released
        // and restored at start-up (FR-149)
        SplitView {
            id: leftRightSplit

            Layout.fillWidth: true
            Layout.fillHeight: true
            orientation: Qt.Horizontal
            onResizingChanged: {
                if (!leftRightSplit.resizing) {
                    Workbench.setNodeLeftWidth(Math.round(leftPart.width));
                }
            }

            // the splitter between the input group above and the group of the meta compiler DSL
            // below (FR-147)
            SplitView {
                id: leftPart

                orientation: Qt.Vertical
                SplitView.preferredWidth: Workbench.nodeLeftWidth
                SplitView.minimumWidth: 240
                onResizingChanged: {
                    if (!leftPart.resizing) {
                        Workbench.setNodeUpperHeight(Math.round(inputsGroup.height));
                    }
                }

                GroupBox {
                    id: inputsGroup

                    title: qsTr("Inputs")
                    SplitView.preferredHeight: Workbench.nodeUpperHeight
                    SplitView.minimumHeight: 120

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

                InputGroup {
                    id: metaDslGroup

                    group: Workbench.node.metaDsl
                    SplitView.fillHeight: true
                    SplitView.minimumHeight: 120
                    onDetachRequested: editorWindowModel.append({ "input": -1 })
                }
            }

            RowLayout {
                id: rightPart

                SplitView.fillWidth: true
                SplitView.minimumWidth: 240

                // the machine beside the outputs: the arrow from the inputs to the logo and the one
                // from the logo to the outputs level with the inputs, and the arrow from the meta
                // compiler DSL up to the logo level with that group (FR-124)
                ColumnLayout {
                    id: machineColumn

                    Layout.fillHeight: true
                    spacing: 0

                    Item {
                        id: machineArea

                        Layout.preferredHeight: inputsGroup.height
                        implicitWidth: machineRow.implicitWidth

                        RowLayout {
                            id: machineRow

                            anchors.centerIn: parent

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
                    }

                    Label {
                        id: controlArrow

                        Layout.alignment: Qt.AlignLeft | Qt.AlignTop
                        text: "\u2197"
                        font.pointSize: 28
                    }

                    Item {
                        Layout.fillHeight: true
                    }
                }

                OutputGroup {
                    id: outputGroup

                    output: Workbench.output
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    onDetachRequested: outputWindowModel.append({ "number": outputWindowModel.count })
                }
            }
        }

        Label {
            id: statusLine

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
