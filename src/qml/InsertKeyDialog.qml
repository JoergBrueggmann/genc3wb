// The modal dialog of the insert key: records a key stroke and takes it as the insert key.
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

Dialog {
    id: insertKeyDialog

    // the key stroke recorded: its key, its modifiers and its name; the name is empty before one is recorded
    property int recordedKey: 0
    property int recordedModifiers: 0
    property string recordedName: ""

    modal: true
    title: qsTr("Insert key")
    anchors.centerIn: parent
    closePolicy: Popup.NoAutoClose
    onOpened: {
        insertKeyDialog.recordedName = "";
        recorder.forceActiveFocus();
    }

    ColumnLayout {
        id: content

        Label {
            id: explanation

            text: qsTr("The insert key toggles between the insert mode and the overwrite mode of typing.")
        }

        Label {
            id: currentKey

            text: qsTr("Insert key: %1").arg(Workbench.insertKeyName)
        }

        Rectangle {
            id: recorder

            Layout.fillWidth: true
            implicitHeight: recorderLabel.implicitHeight + 16
            border.color: recorder.activeFocus ? recorder.palette.highlight : recorder.palette.mid
            border.width: recorder.activeFocus ? 2 : 1
            color: "transparent"
            focus: true
            // every key is recorded here, the keys that would close the dialog included
            Keys.onPressed: event => {
                const name = Workbench.keyName(event.key, event.modifiers);
                if (name.length > 0) {
                    insertKeyDialog.recordedKey = event.key;
                    insertKeyDialog.recordedModifiers = event.modifiers;
                    insertKeyDialog.recordedName = name;
                }
                event.accepted = true;
            }

            Label {
                id: recorderLabel

                anchors.centerIn: parent
                text: insertKeyDialog.recordedName.length > 0
                      ? qsTr("Recorded: %1").arg(insertKeyDialog.recordedName)
                      : qsTr("Press the key to be used as the insert key.")
            }

            TapHandler {
                id: recorderTap

                onTapped: recorder.forceActiveFocus()
            }
        }

        RowLayout {
            id: buttons

            Button {
                id: defaultButton

                text: qsTr("Default")
                focusPolicy: Qt.NoFocus
                onClicked: {
                    Workbench.resetInsertKey();
                    insertKeyDialog.close();
                }
            }

            Item {
                Layout.fillWidth: true
            }

            Button {
                id: cancelButton

                text: qsTr("Cancel")
                focusPolicy: Qt.NoFocus
                onClicked: insertKeyDialog.close()
            }

            Button {
                id: okButton

                text: qsTr("OK")
                focusPolicy: Qt.NoFocus
                enabled: insertKeyDialog.recordedName.length > 0
                onClicked: {
                    Workbench.setInsertKey(insertKeyDialog.recordedKey, insertKeyDialog.recordedModifiers);
                    insertKeyDialog.close();
                }
            }
        }
    }
}
