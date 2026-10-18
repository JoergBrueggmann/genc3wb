// The editor window of one file of the network, opened from the network graph or by a detach button of a node
// window, bound to the input group the network editor owns for the file.
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
    id: editorWindow

    required property var group

    width: 700
    height: 500
    title: "genc³wb - " + editorWindow.group.caption

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 8
        spacing: 5

        RowLayout {
            spacing: 5

            TextField {
                id: pathField

                Layout.fillWidth: true
                readOnly: true
                text: editorWindow.group.path
            }

            ModeSwitch {
                id: modeSwitch

                visible: editorWindow.group.octetModes
                documentIsText: editorWindow.group.isText
            }

            ProcessingStateIndicator {
                id: indicator

                processingState: editorWindow.group.state
            }
        }

        CodeEditor {
            id: editor

            Layout.fillWidth: true
            Layout.fillHeight: true
            // an input of a producer is not edited: its content is what the node holds (FR-238)
            readOnly: editorWindow.group.produced
            // the text is bound in txt alone, so that an edit in hex or bin does not lay it out anew
            text: editor.mode === 0 ? editorWindow.group.text : ""
            idleTime: Workbench.idleTime
            longIdleTime: Workbench.longIdleTime
            tabSize: Workbench.tabSize
            tabsToSpaces: Workbench.tabsToSpaces
            onTextEdited: {
                if (editor.mode === 0) {
                    editorWindow.group.text = editor.text;
                }
            }
            onIdleExpired: editorWindow.group.idleExpired()
            onLongIdleExpired: editorWindow.group.longIdleExpired()
            markStarts: editorWindow.group.diagnosticStarts
            markEnds: editorWindow.group.diagnosticEnds
            markSeverities: editorWindow.group.diagnosticSeverities
            markTexts: editorWindow.group.diagnosticTexts
            mode: editorWindow.group.octetModes ? modeSwitch.mode : 0
            octets: editorWindow.group
            octetsMarked: true
        }

        TypingModeLabel {
            id: typingModeLabel

            Layout.alignment: Qt.AlignRight
        }
    }

    Connections {
        target: editorWindow.group

        function onDocumentChanged() {
            modeSwitch.chosen = -1;
        }
    }
}
