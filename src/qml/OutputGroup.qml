// The output group: navigation, page label, detach button and the output pages.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

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

                text: qsTr("output %1 of %2").arg(root.output.pageIndex + 1).arg(root.output.pageCount)
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
            text: root.output.fileContent
        }
    }
}
