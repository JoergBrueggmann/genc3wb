// The group for the network graph: the build system with its activation switch, the graph, the error message,
// and the two buttons.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts
// The types of the module genc3wb are registered by the bridge crate at run time, which writes no type
// description for qmllint; the import is therefore not linted.
// qmllint disable import
import genc3wb

GroupBox {
    id: root

    required property var network

    title: qsTr("Compiler network")
    padding: 8

    ColumnLayout {
        anchors.fill: parent
        spacing: 5

        RowLayout {
            spacing: 5

            // the label and its field, 3 pixels apart
            RowLayout {
                id: buildSystemRow

                Layout.fillWidth: true
                spacing: 3

                Label {
                    id: buildSystemLabel

                    text: qsTr("Build system")
                }

                // the file name field accepts no text: the build system is named by the file selector
                // (FR-481)
                TextField {
                    id: buildSystemField

                    Layout.fillWidth: true
                    text: root.network.buildSystemPath
                    readOnly: true
                }
            }

            ToolButton {
                id: selectButton

                text: "..."
                onClicked: fileDialog.open()
            }

            // the activation switch: `act` highlighted while the network is active (FR-266), enabled
            // while a network mode is kept and no request of the switch awaits its response
            // (FR-259); `act` transmits an activate request, `ina` a deactivate request (FR-260,
            // FR-267)
            PositionSwitch {
                id: activationSwitch

                labels: ["ina", "act"]
                selected: root.network.networkMode === 3 ? 1 : 0
                enabled: root.network.switchable
                // FR-272
                tooltip: qsTr("Inactive / Active - to deactivate or activate the network")
                onPositionTapped: index => {
                    if (index === 1) {
                        root.network.activate();
                    } else {
                        root.network.deactivate();
                    }
                }
            }

            Image {
                id: executableIndicator

                source: root.network.executable && root.network.available
                        ? "qrc:/genc3wb/qml/icons/indicatorUntouched.png"
                        : "qrc:/genc3wb/qml/icons/indicatorUnknownFileChanged.png"
                fillMode: Image.PreserveAspectFit
                sourceSize.height: 20
            }

            ToolButton {
                id: errorButton

                icon.source: "qrc:/genc3wb/qml/icons/error.png"
                icon.color: "transparent"
                visible: root.network.errorButtonVisible
                onClicked: root.network.showError()
            }

            ToolButton {
                id: graphButton

                icon.source: "qrc:/genc3wb/qml/icons/network.png"
                icon.color: "transparent"
                visible: root.network.graphButtonVisible
                onClicked: root.network.showGraph()
            }
        }

        StackLayout {
            id: views

            Layout.fillWidth: true
            Layout.fillHeight: true
            currentIndex: root.network.showingError ? 1 : 0

            Frame {
                id: graphFrame

                padding: 1

                Flickable {
                    id: graphFlickable

                    anchors.fill: parent
                    clip: true
                    contentWidth: Math.max(width, graphImage.width + 16)
                    contentHeight: Math.max(height, graphImage.height + 16)

                    ScrollBar.vertical: ScrollBar {}
                    ScrollBar.horizontal: ScrollBar {}

                    Image {
                        id: graphImage

                        x: 8
                        y: 8
                        source: root.network.graphSource
                        cache: false
                        // the image has twice the resolution of a display (IMAGE_DPI)
                        width: sourceSize.width / 2
                        height: sourceSize.height / 2
                        smooth: true
                        mipmap: true
                        opacity: root.network.errorButtonVisible ? 0.5 : 1.0

                        TapHandler {
                            id: openHandler

                            onDoubleTapped: (eventPoint) => root.network.openNodeAt(
                                eventPoint.position.x / graphImage.width,
                                eventPoint.position.y / graphImage.height)
                        }
                    }
                }
            }

            Frame {
                id: errorFrame

                padding: 1

                ScrollView {
                    id: errorScroll

                    anchors.fill: parent

                    TextArea {
                        id: errorArea

                        readOnly: true
                        wrapMode: TextEdit.NoWrap
                        font.family: "Courier New"
                        text: root.network.errorMessage
                    }
                }
            }
        }
    }

    FileDialog {
        id: fileDialog

        title: qsTr("Build system executable")
        onAccepted: root.network.buildSystemPath = PathOfUrl.path(fileDialog.selectedFile)
    }
}
