// One input group: file name field, file selector, indicator, code editor, detach button, and the dialogs of that group.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts
// The types of the module genc3wb are registered by the bridge crate at run time, which writes no type
// description for qmllint; the import and the unqualified access to the singleton are therefore not linted.
// qmllint disable import unqualified
import genc3wb

GroupBox {
    id: root

    required property var group
    property bool detachable: true

    signal detachRequested()

    title: root.group.caption

    ColumnLayout {
        anchors.fill: parent

        RowLayout {
            TextField {
                id: pathField

                Layout.fillWidth: true
                text: root.group.path
                onEditingFinished: root.group.path = pathField.text
            }

            ToolButton {
                id: selectButton

                text: "..."
                onClicked: fileDialog.open()
            }

            ToolButton {
                id: detachButton

                text: "⧉"
                visible: root.detachable
                onClicked: root.detachRequested()
            }
        }

        ProcessingStateIndicator {
            id: indicator

            processingState: root.group.state
        }

        CodeEditor {
            id: editor

            Layout.fillWidth: true
            Layout.fillHeight: true
            text: root.group.text
            idleTime: Workbench.idleTime * 1000
            longIdleTime: Workbench.longIdleTime * 1000
            onTextEdited: root.group.text = editor.text
            onIdleExpired: root.group.idleExpired()
            onLongIdleExpired: root.group.longIdleExpired()
        }
    }

    FileDialog {
        id: fileDialog

        title: root.group.caption
        nameFilters: [root.group.fileFilter]
        onAccepted: root.group.path = PathOfUrl.path(fileDialog.selectedFile)
    }

    MessageDialog {
        id: saveDialog

        title: root.group.caption
        text: qsTr("The editor holds unsaved changes. Save them before the named file is loaded?")
        buttons: MessageDialog.Yes | MessageDialog.No
        onAccepted: root.group.answerSave(true)
        onRejected: root.group.answerSave(false)
    }

    Connections {
        target: root.group

        function onAskToSave(path) {
            saveDialog.open();
        }
    }
}
