// The modal playback delay dialog: whether the playback about to start waits a delay and which of the
// thirteen values, and the check box that disables the dialog; OK stores them at the workbench object and
// accepts, Cancel rejects (FR-542, FR-543).
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
    id: playbackDelayDialog

    title: qsTr("Playback delay")
    modal: true
    padding: 8
    anchors.centerIn: Overlay.overlay
    // the dialog is left by its two buttons alone (FR-543)
    closePolicy: Popup.NoAutoClose

    // the switch and the field take the stored delay and the check box is unchecked whenever the dialog
    // opens (FR-542); the field holds the index of the value among the thirteen
    onAboutToShow: {
        delayOnBox.checked = Workbench.playbackDelayOn;
        delayBox.value = Workbench.playbackDelayIndex;
        disableBox.checked = false;
    }

    footer: DialogButtonBox {
        id: buttons

        Button {
            id: cancelButton

            text: qsTr("Cancel")
            onClicked: playbackDelayDialog.reject()
        }

        Button {
            id: okButton

            text: qsTr("OK")
            onClicked: {
                Workbench.setPlaybackDelay(delayOnBox.checked, delayBox.value, disableBox.checked);
                playbackDelayDialog.accept();
            }
        }
    }

    ColumnLayout {
        id: content

        spacing: 5

        // whether a playback delay is waited (FR-518, FR-542)
        CheckBox {
            id: delayOnBox

            text: qsTr("Delay")
            checked: Workbench.playbackDelayOn
        }

        RowLayout {
            id: delayRow

            // the label, its field and the unit belong together
            spacing: 3

            Label {
                id: delayLabel

                text: qsTr("Playback delay")
            }

            // the thirteen values by their index, shown as seconds with two decimals (FR-518, FR-542)
            SpinBox {
                id: delayBox

                from: 0
                to: 12
                enabled: delayOnBox.checked
                value: Workbench.playbackDelayIndex
                textFromValue: (value, locale) => (Workbench.playbackDelayOf(value) / 1000).toFixed(2)
            }

            Label {
                id: delayUnit

                text: qsTr("s")
            }
        }

        // checked with OK, it switches the playback delay dialog setting on (FR-543)
        CheckBox {
            id: disableBox

            text: qsTr("Do not show this dialog again")
            checked: false
        }
    }
}
