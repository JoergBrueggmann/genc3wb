// The output group: navigation, page label, detach button and the output pages.
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

    required property var output
    property bool detachable: true

    signal detachRequested()

    title: qsTr("Output")

    ColumnLayout {
        anchors.fill: parent

        RowLayout {
            Button {
                id: previousButton

                text: "<"
                enabled: root.output.hasPrevious
                onClicked: root.output.previous()
            }

            Label {
                id: pageLabel

                text: qsTr("page %1 of %2").arg(root.output.pageIndex + 1).arg(root.output.pageCount)
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
            currentIndex: root.output.pageKind

            CodeEditor {
                id: stdOutEditor

                readOnly: true
                text: root.output.stdOut
            }

            ColumnLayout {
                CodeEditor {
                    id: stdErrEditor

                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    readOnly: true
                    text: root.output.stdErr
                }

                RowLayout {
                    Label {
                        text: qsTr("Exit code:")
                    }

                    TextField {
                        id: exitCodeField

                        readOnly: true
                        text: root.output.hasRun ? root.output.exitCode : ""
                    }
                }
            }

            ColumnLayout {
                RowLayout {
                    CheckBox {
                        id: fileEnabledBox

                        text: qsTr("Output file %1").arg(root.output.fileNumber)
                        checked: root.output.fileEnabled
                        onToggled: root.output.fileEnabled = fileEnabledBox.checked
                    }

                    TextField {
                        id: filePathField

                        Layout.fillWidth: true
                        text: root.output.filePath
                        onEditingFinished: root.output.filePath = filePathField.text
                    }

                    ToolButton {
                        id: fileSelectButton

                        text: "..."
                        onClicked: fileDialog.open()
                    }
                }

                CodeEditor {
                    id: fileContentEditor

                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    readOnly: true
                    text: root.output.fileContent
                }
            }
        }
    }

    FileDialog {
        id: fileDialog

        title: qsTr("Output file %1").arg(root.output.fileNumber)
        onAccepted: root.output.filePath = PathOfUrl.path(fileDialog.selectedFile)
    }
}
