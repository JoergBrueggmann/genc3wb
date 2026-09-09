// A detached window of one input group.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Layouts

Window {
    id: editorWindow

    required property var group

    width: 700
    height: 500
    title: "genc³wb - " + editorWindow.group.caption

    ColumnLayout {
        anchors.fill: parent
        anchors.margins: 6

        RowLayout {
            TextField {
                id: pathField

                Layout.fillWidth: true
                text: editorWindow.group.path
                onEditingFinished: editorWindow.group.path = pathField.text
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
            text: editorWindow.group.text
            maxIdleTime: editorWindow.group.maxIdleTime
            onTextEdited: editorWindow.group.text = editor.text
            onIdleExpired: editorWindow.group.idleExpired()
        }
    }
}
