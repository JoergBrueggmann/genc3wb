// The backup dialog: the source folder and the backup folder of a precondition backup, each with a button
// that presents the folder selector dialog of the operating system, the reason of a failure, and OK, which
// has the recorder copy the source folder into the backup folder and record the restoring.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts
// The types of the module genc3wb are registered by the bridge crate at run time, which writes no type
// description for qmllint; the import and the unqualified access to the singleton are therefore not linted.
// qmllint disable import unqualified
import genc3wb

Dialog {
    id: backupDialog

    title: qsTr("Backup to restore on replay")
    modal: true
    padding: 8
    width: Math.min(parent.width - 48, 640)
    anchors.centerIn: parent
    // the dialog is left by its two buttons alone: OK copies and records where that succeeds, Cancel
    // discards (FR-530)
    closePolicy: Popup.NoAutoClose

    // the fields take the folder of the network file and the folder 'backup' beside the script whenever
    // the dialog opens, and the reason of the last failure is cleared (FR-530)
    onAboutToShow: {
        sourceField.text = Workbench.automation.networkFolder;
        backupField.text = Workbench.automation.defaultBackupFolder;
        reasonLabel.text = "";
    }

    footer: DialogButtonBox {
        id: buttons

        Button {
            id: cancelButton

            text: qsTr("Cancel")
            onClicked: backupDialog.reject()
        }

        Button {
            id: okButton

            text: qsTr("OK")
            onClicked: {
                const reason = Workbench.automation.backUp(sourceField.text, backupField.text);
                if (reason.length === 0) {
                    backupDialog.accept();
                } else {
                    reasonLabel.text = reason;
                }
            }
        }
    }

    GridLayout {
        id: grid

        width: backupDialog.availableWidth
        columns: 3
        columnSpacing: 5
        rowSpacing: 5

        Label {
            id: sourceLabel

            text: qsTr("Source folder")
        }

        TextField {
            id: sourceField

            Layout.fillWidth: true
        }

        ToolButton {
            id: sourceButton

            text: "..."
            onClicked: sourceFolderDialog.open()
        }

        Label {
            id: backupLabel

            text: qsTr("Backup folder")
        }

        TextField {
            id: backupField

            Layout.fillWidth: true
        }

        ToolButton {
            id: backupButton

            text: "..."
            onClicked: backupFolderDialog.open()
        }

        // the reason why the backup was not made, in the colour of an error; empty otherwise
        Label {
            id: reasonLabel

            Layout.columnSpan: 3
            Layout.fillWidth: true
            wrapMode: Text.Wrap
            color: "#c00000"
        }
    }

    FolderDialog {
        id: sourceFolderDialog

        title: qsTr("Source folder")
        onAccepted: sourceField.text = PathOfUrl.path(sourceFolderDialog.selectedFolder)
    }

    FolderDialog {
        id: backupFolderDialog

        title: qsTr("Backup folder")
        onAccepted: backupField.text = PathOfUrl.path(backupFolderDialog.selectedFolder)
    }
}
