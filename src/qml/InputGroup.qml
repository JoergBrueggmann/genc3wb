// One input group: file name field, file selector, indicator, the icon of an input of a producer, code editor
// with the marks of the diagnostics, detach button, and the dialogs of that group.
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
                readOnly: !root.group.selectable
                onEditingFinished: root.group.path = pathField.text
            }

            ToolButton {
                id: selectButton

                text: "..."
                visible: root.group.selectable
                onClicked: fileDialog.open()
            }

            Image {
                id: temporaryIcon

                source: "qrc:/genc3wb/qml/icons/error.png"
                fillMode: Image.PreserveAspectFit
                sourceSize.height: 20
                visible: root.group.temporaryEdit
                ToolTip.visible: temporaryHover.hovered
                ToolTip.text: qsTr("This edit is temporary and will be overwritten, because it is also output of node %1.").arg(root.group.producer)

                HoverHandler {
                    id: temporaryHover
                }
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
            idleTime: Workbench.idleTime
            longIdleTime: Workbench.longIdleTime
            onTextEdited: root.group.text = editor.text
            onIdleExpired: root.group.idleExpired()
            onLongIdleExpired: root.group.longIdleExpired()
            markStarts: root.group.diagnosticStarts
            markEnds: root.group.diagnosticEnds
            markSeverities: root.group.diagnosticSeverities
            markTexts: root.group.diagnosticTexts
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
