// The modal settings dialog, in three parts: the automatic setting with the idle time and the long idle time
// of every code editor, in seconds with one decimal; the tab size and the spaces setting of every code editor;
// and the silent creation setting of the input creation confirmation dialog.
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
    padding: 8
    anchors.centerIn: Overlay.overlay
    // the dialog is left by its two buttons alone: OK stores where the constraints hold, Cancel
    // discards (FR-100, FR-135, FR-141, FR-257)
    closePolicy: Popup.NoAutoClose

    // the fields take the stored values whenever the dialog opens, so that a rejected
    // change is not carried into the next one (FR-100); the times are milliseconds at the
    // workbench object and tenths of a second in the fields
    onAboutToShow: {
        idleTimeBox.value = Workbench.idleTime / 100;
        longIdleTimeBox.value = Workbench.longIdleTime / 100;
        automaticBox.checked = Workbench.automatic;
        tabSizeBox.value = Workbench.tabSize;
        spacesBox.checked = Workbench.tabsToSpaces;
        silentBox.checked = Workbench.silentCreation;
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
                                             tabSizeBox.value, spacesBox.checked, silentBox.checked)) {
                    settingsDialog.accept();
                }
            }
        }
    }

    ColumnLayout {
        spacing: 5

        CheckBox {
            id: automaticBox

            text: qsTr("Set the times automatically from the processing time")
            checked: Workbench.automatic
        }

        GridLayout {
            id: timesLayout

            columns: 3
            // a label, its field and the text beside it belong together
            columnSpacing: 3
            rowSpacing: 5

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

                text: qsTr("Seconds until the text is saved and transmitted")
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

                text: qsTr("Seconds until the error message of a failed build is shown")
            }
        }

        // the line between the times and the settings of the code editor: 8 pixels from the
        // content on each side, the spacing of the column and its margins
        Rectangle {
            id: editorSeparator

            Layout.fillWidth: true
            Layout.topMargin: 3
            Layout.bottomMargin: 3
            implicitHeight: 1
            color: settingsDialog.palette.mid
        }

        GridLayout {
            id: editorLayout

            columns: 3
            columnSpacing: 3
            rowSpacing: 5

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

                text: qsTr("Characters per tab, from 1 to 16")
            }
        }

        // the spaces setting (FR-492)
        CheckBox {
            id: spacesBox

            text: qsTr("Converted to spaces")
            checked: Workbench.tabsToSpaces
        }

        // the line between the settings of the code editor and the one of the dialog
        Rectangle {
            id: creationSeparator

            Layout.fillWidth: true
            Layout.topMargin: 3
            Layout.bottomMargin: 3
            implicitHeight: 1
            color: settingsDialog.palette.mid
        }

        // the silent creation setting (FR-490)
        CheckBox {
            id: silentBox

            text: qsTr("Disable input creation confirmation dialog")
            checked: Workbench.silentCreation
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
