// The group for the network graph: the build system, the graph, the error message, and the two buttons.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts

GroupBox {
    id: root

    required property var network

    title: qsTr("Compiler network")

    ColumnLayout {
        anchors.fill: parent

        RowLayout {
            Label {
                id: buildSystemLabel

                text: qsTr("Build system")
            }

            TextField {
                id: buildSystemField

                Layout.fillWidth: true
                text: root.network.buildSystemPath
                onTextEdited: root.network.tryBuildSystemPath(buildSystemField.text)
                onEditingFinished: root.network.buildSystemPath = buildSystemField.text
            }

            ToolButton {
                id: selectButton

                text: "..."
                onClicked: fileDialog.open()
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
