// The modal settings dialog: the idle time and the long idle time of every code editor, in seconds with one
// decimal, the automatic setting, and the tab size of every code editor.
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
    id: settingsDialog

    title: qsTr("Settings")
    modal: true
    anchors.centerIn: Overlay.overlay
    // the dialog is left by its two buttons alone: OK stores where the constraints hold, Cancel
    // discards (FR-100, FR-135, FR-141)
    closePolicy: Popup.NoAutoClose

    // the fields take the stored values whenever the dialog opens, so that a rejected
    // change is not carried into the next one (FR-100); the times are milliseconds at the
    // workbench object and tenths of a second in the fields
    onAboutToShow: {
        idleTimeBox.value = Workbench.idleTime / 100;
        longIdleTimeBox.value = Workbench.longIdleTime / 100;
        automaticBox.checked = Workbench.automatic;
        tabSizeBox.value = Workbench.tabSize;
        errorLabel.text = "";
    }

    footer: DialogButtonBox {
        id: buttons

        Button {
            id: cancelButton

            text: qsTr("Cancel")
            onClicked: settingsDialog.reject()
        }

        Button {
            id: okButton

            text: qsTr("OK")
            onClicked: {
                if (Workbench.trySetSettings(idleTimeBox.value * 100, longIdleTimeBox.value * 100, automaticBox.checked,
                                             tabSizeBox.value)) {
                    settingsDialog.accept();
                }
            }
        }
    }

    ColumnLayout {
        GridLayout {
            id: layout

            columns: 3

            Label {
                id: idleTimeLabel

                text: qsTr("Idle time")
            }

            SpinBox {
                id: idleTimeBox

                from: 2
                to: 36000
                stepSize: 2
                editable: true
                enabled: !automaticBox.checked
                value: Workbench.idleTime / 100
                textFromValue: (value, locale) => (value / 10).toFixed(1)
                valueFromText: (text, locale) => Math.round(parseFloat(text) * 10)
            }

            Label {
                id: idleTimeUnit

                text: qsTr("seconds until the text is saved and transmitted")
            }

            Label {
                id: longIdleTimeLabel

                text: qsTr("Long idle time")
            }

            SpinBox {
                id: longIdleTimeBox

                from: 2
                to: 36000
                stepSize: 2
                editable: true
                enabled: !automaticBox.checked
                value: Workbench.longIdleTime / 100
                textFromValue: (value, locale) => (value / 10).toFixed(1)
                valueFromText: (text, locale) => Math.round(parseFloat(text) * 10)
            }

            Label {
                id: longIdleTimeUnit

                text: qsTr("seconds until the error message of a failed build is shown")
            }

            Label {
                id: tabSizeLabel

                text: qsTr("Tab size")
            }

            // the range of the box is wider than the one of the tab size, so that a value outside
            // 1..16 reaches the constraint and its message (FR-141)
            SpinBox {
                id: tabSizeBox

                from: 0
                to: 999
                editable: true
                value: Workbench.tabSize
            }

            Label {
                id: tabSizeUnit

                text: qsTr("characters per tab, from 1 to 16")
            }
        }

        CheckBox {
            id: automaticBox

            text: qsTr("Set the times automatically from the processing time")
            checked: Workbench.automatic
        }

        Label {
            id: errorLabel

            color: "#d02020"
            text: ""
        }
    }

    Connections {
        target: Workbench

        function onSettingsRejected(message) {
            errorLabel.text = message;
        }
    }
}
