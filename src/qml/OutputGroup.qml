// The output group: navigation, page label, detach button, the output pages and the diagnostics page, which is
// presented while diagnostics appear.
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

                text: root.output.diagnosticsPage
                      ? qsTr("Diagnostics (page %1 of %2)").arg(root.output.pageIndex + 1).arg(root.output.pageCount)
                      : qsTr("output %1 of %2").arg(root.output.pageIndex + 1).arg(root.output.pageCount)
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
                TextField {
                    id: filePathField

                    Layout.fillWidth: true
                    readOnly: true
                    text: root.output.filePath
                }

                CodeEditor {
                    id: fileContentEditor

                    Layout.fillWidth: true
                    Layout.fillHeight: true
                    readOnly: true
                    tabSize: Workbench.tabSize
                    text: root.output.fileContent
                }
            }

            CodeEditor {
                id: diagnosticsEditor

                readOnly: true
                tabSize: Workbench.tabSize
                text: root.output.diagnostics
            }
        }
    }
}
