// The output group: navigation, page label, detach button, the output pages with their mode switch and
// their mark of an outdated file, and the diagnostics page with the icons of the origins, which is presented
// while diagnostics or origins appear.
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

GroupBox {
    id: root

    required property var output
    property bool detachable: true

    signal detachRequested()

    title: qsTr("Outputs")
    padding: 8

    ColumnLayout {
        anchors.fill: parent
        spacing: 5

        RowLayout {
            spacing: 5

            Button {
                id: previousButton

                text: "<"
                enabled: root.output.hasPrevious
                onClicked: root.output.previous()
            }

            Label {
                id: pageLabel

                text: root.output.diagnosticsPage
                      ? qsTr("Diagnostics (page %1 of %2)").arg(root.output.pageIndex + 1).arg(root.output.pageCount)
                      : qsTr("Output %1 of %2").arg(root.output.pageIndex + 1).arg(root.output.pageCount)
            }

            Button {
                id: nextButton

                text: ">"
                enabled: root.output.hasNext
                onClicked: root.output.next()
            }

            Item {
                Layout.fillWidth: true
            }

            ToolButton {
                id: detachButton

                text: "⧉"
                visible: root.detachable
                onClicked: root.detachRequested()
            }
        }

        StackLayout {
            id: pages

            Layout.fillWidth: true
            Layout.fillHeight: true
            // the presented page is the one of the output group, which switches to the diagnostics
            // page and back as diagnostics appear and are gone, and which the two buttons select by
            // hand (FR-150 to FR-152)
            currentIndex: root.output.diagnosticsPage ? 1 : 0

            ColumnLayout {
                spacing: 5

                RowLayout {
                    spacing: 5

                    TextField {
                        id: filePathField

                        Layout.fillWidth: true
                        readOnly: true
                        text: root.output.filePath
                    }

                    Image {
                        id: outdatedIcon

                        source: "qrc:/genc3wb/qml/icons/error.png"
                        fillMode: Image.PreserveAspectFit
                        sourceSize.height: 20
                        visible: root.output.outdated
                        ToolTip.visible: outdatedHover.hovered
                        ToolTip.text: qsTr("The file of this output is outdated: the outputs of the last change could not be stored. %1").arg(root.output.storeFailure)

                        HoverHandler {
                            id: outdatedHover
                        }
                    }

                    ModeSwitch {
                        id: modeSwitch

                        documentIsText: root.output.isText
                    }
                }

                CodeEditor {
                    id: fileContentEditor

                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    readOnly: true
                    tabSize: Workbench.tabSize
                    text: root.output.fileContent
                    mode: modeSwitch.mode
                    octets: root.output
                }
            }

            ColumnLayout {
                spacing: 5

                // one icon per origin of the opened node, a node upstream that counts an error:
                // its tooltip names that node, and activating it opens that node (FR-234, FR-235)
                RowLayout {
                    id: originsRow

                    visible: root.output.origins.length > 0
                    spacing: 5

                    Repeater {
                        id: originRepeater

                        model: root.output.origins
                        delegate: Image {
                            id: originIcon

                            required property string modelData

                            source: "qrc:/genc3wb/qml/icons/error.png"
                            fillMode: Image.PreserveAspectFit
                            sourceSize.height: 20
                            ToolTip.visible: originHover.hovered
                            ToolTip.text: qsTr("Errors originate at node %1 ...").arg(originIcon.modelData)

                            HoverHandler {
                                id: originHover

                                cursorShape: Qt.PointingHandCursor
                            }

                            TapHandler {
                                id: originTap

                                onTapped: Workbench.network.openNodeNamed(originIcon.modelData)
                            }
                        }
                    }

                    Item {
                        Layout.fillWidth: true
                    }
                }

                CodeEditor {
                    id: diagnosticsEditor

                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    readOnly: true
                    tabSize: Workbench.tabSize
                    text: root.output.diagnostics
                }
            }
        }
    }

    Connections {
        target: root.output

        function onDocumentChanged() {
            modeSwitch.chosen = -1;
        }
    }
}
