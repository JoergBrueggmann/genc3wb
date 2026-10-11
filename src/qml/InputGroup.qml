// One input group: file name field, file selector, indicator, the mode switch of an input, code editor with
// the marks of the diagnostics, which is read-only for an input of a producer, detach button, and the dialogs
// of that group.
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

    // an input group whose group is gone, as while the node window rebuilds its input groups for a node
    // of another number of inputs, presents nothing: its bindings read the group through `?.`
    title: root.group?.caption ?? ""

    ColumnLayout {
        anchors.fill: parent

        RowLayout {
            TextField {
                id: pathField

                Layout.fillWidth: true
                text: root.group?.path ?? ""
                readOnly: !(root.group?.selectable ?? false)
                onEditingFinished: root.group.path = pathField.text
            }

            ToolButton {
                id: selectButton

                text: "..."
                visible: root.group?.selectable ?? false
                onClicked: fileDialog.open()
            }

            ModeSwitch {
                id: modeSwitch

                visible: root.group?.octetModes ?? false
                documentIsText: root.group?.isText ?? true
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

            processingState: root.group?.state ?? 2
        }

        CodeEditor {
            id: editor

            Layout.fillWidth: true
            Layout.fillHeight: true
            // an input of a producer is not edited: its content is what the node holds (FR-238)
            readOnly: root.group?.produced ?? false
            // the text is bound in txt alone, so that an edit in hex or bin does not lay it out anew
            text: editor.mode === 0 ? (root.group?.text ?? "") : ""
            idleTime: Workbench.idleTime
            longIdleTime: Workbench.longIdleTime
            tabSize: Workbench.tabSize
            onTextEdited: {
                if (editor.mode === 0) {
                    root.group.text = editor.text;
                }
            }
            onIdleExpired: root.group.idleExpired()
            onLongIdleExpired: root.group.longIdleExpired()
            markStarts: root.group?.diagnosticStarts ?? []
            markEnds: root.group?.diagnosticEnds ?? []
            markSeverities: root.group?.diagnosticSeverities ?? []
            markTexts: root.group?.diagnosticTexts ?? []
            mode: (root.group?.octetModes ?? false) ? modeSwitch.mode : 0
            octets: root.group
            octetsMarked: true
        }
    }

    FileDialog {
        id: fileDialog

        title: root.group?.caption ?? ""
        nameFilters: [root.group?.fileFilter ?? ""]
        onAccepted: root.group.path = PathOfUrl.path(fileDialog.selectedFile)
    }

    MessageDialog {
        id: saveDialog

        title: root.group?.caption ?? ""
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

        function onDocumentChanged() {
            modeSwitch.chosen = -1;
        }
    }
}
