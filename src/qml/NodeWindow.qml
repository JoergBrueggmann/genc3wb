// One node window as a machine, bound to its node editor: the inputs above the meta compiler DSL as its
// control, beside them the machine logo and the outputs, the two splitters between them, and the status line;
// the detach buttons of its groups open the editor windows of their files.
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

    // the node editor of this window, one per node opened (FR-001)
    required property var node

    width: 1200
    height: 760
    title: nodeWindow.node.name.length > 0 ? "genc³wb - " + nodeWindow.node.name : "genc³wb"
    // closing the window has the network editor close the node editor and release the node
    // (FR-089, FR-105, FR-282)
    onClosing: Workbench.network.closeNode(nodeWindow.node.name)

    // Yields the text of the node mode `mode` for the status line, as the message schema
    // numbers it: 1 invalid, 2 valid, 3 active; empty for 0, where none is told (FR-123)
    function nodeModeText(mode: int): string {
        switch (mode) {
        case 1:
            return qsTr("Node invalid");
        case 2:
            return qsTr("Node valid, not active");
        case 3:
            return qsTr("Node active");
        case 4:
            return qsTr("Node waiting for its meta compiler DSL");
        default:
            return "";
        }
    }

    ColumnLayout {
        id: layout

        anchors.fill: parent
        anchors.margins: 8
        spacing: 5

        // the splitter between the left part, the inputs above the meta compiler DSL, and the right
        // part, the machine and the outputs (FR-146); its position is stored when it is released
        // and restored at start-up (FR-149)
        SplitView {
            id: leftRightSplit

            Layout.fillWidth: true
            Layout.fillHeight: true
            orientation: Qt.Horizontal
            handle: SplitterHandle {
                orientation: leftRightSplit.orientation
            }
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
                handle: SplitterHandle {
                    orientation: leftPart.orientation
                }
                onResizingChanged: {
                    if (!leftPart.resizing) {
                        Workbench.setNodeUpperHeight(Math.round(inputsGroup.height));
                    }
                }

                GroupBox {
                    id: inputsGroup

                    title: qsTr("Inputs")
                    padding: 8
                    SplitView.preferredHeight: Workbench.nodeUpperHeight
                    SplitView.minimumHeight: 120

                    ColumnLayout {
                        anchors.fill: parent
                        spacing: 5

                        RowLayout {
                            spacing: 5

                            Button {
                                id: previousInputButton

                                text: "<"
                                enabled: nodeWindow.node.hasPreviousInput
                                onClicked: nodeWindow.node.previousInput()
                            }

                            Label {
                                id: inputLabel

                                text: qsTr("Input %1 of %2").arg(nodeWindow.node.inputPage + 1).arg(nodeWindow.node.inputCount)
                            }

                            Button {
                                id: nextInputButton

                                text: ">"
                                enabled: nodeWindow.node.hasNextInput
                                onClicked: nodeWindow.node.nextInput()
                            }
                        }

                        StackLayout {
                            id: inputPages

                            Layout.fillWidth: true
                            Layout.fillHeight: true
                            currentIndex: nodeWindow.node.inputPage

                            Repeater {
                                id: inputRepeater

                                model: nodeWindow.node.inputCount
                                delegate: InputGroup {
                                    required property int index

                                    group: nodeWindow.node.inputAt(index)
                                    // the detach button opens the editor window of the file (FR-045)
                                    onDetachRequested: Workbench.network.openFileNamed(group.path)
                                }
                            }
                        }
                    }
                }

                InputGroup {
                    id: metaDslGroup

                    group: nodeWindow.node.metaDsl
                    SplitView.fillHeight: true
                    SplitView.minimumHeight: 120
                    // the detach button opens the editor window of the meta compiler DSL (FR-045)
                    onDetachRequested: Workbench.network.openFileNamed(metaDslGroup.group.path)
                }
            }

            RowLayout {
                id: rightPart

                SplitView.fillWidth: true
                SplitView.minimumWidth: 240
                spacing: 5

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
                            spacing: 5

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

                    output: nodeWindow.node.output
                    Layout.fillWidth: true
                    Layout.fillHeight: true
                }
            }
        }

        RowLayout {
            id: statusRow

            spacing: 5

            Label {
                id: statusLine

                Layout.fillWidth: true
                // the text of FR-112, or whether the node is served and processing, then its
                // node mode (FR-123), as short phrases
                text: [nodeWindow.node.status.length > 0
                       ? nodeWindow.node.status
                       : !nodeWindow.node.served
                         ? qsTr("Node not served")
                         : nodeWindow.node.busy ? qsTr("Node processing") : qsTr("Node served"),
                       nodeWindow.nodeModeText(nodeWindow.node.nodeMode)]
                      .filter(part => part.length > 0).join("  ·  ")
            }

            TypingModeLabel {
                id: typingModeLabel
            }
        }
    }

    Connections {
        target: nodeWindow.node

        function onProcessingMeasured(processingTime) {
            Workbench.reportProcessingTime(processingTime);
        }
    }
}
