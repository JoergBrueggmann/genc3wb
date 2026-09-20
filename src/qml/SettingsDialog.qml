// The modal settings dialog: the idle time and the long idle time of every code editor, in seconds.
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
    standardButtons: Dialog.Ok | Dialog.Cancel

    // the fields take the stored values whenever the dialog opens, so that a rejected
    // change is not carried into the next one (FR-100)
    onAboutToShow: {
        idleTimeBox.value = Workbench.idleTime;
        longIdleTimeBox.value = Workbench.longIdleTime;
    }

    onAccepted: {
        Workbench.idleTime = idleTimeBox.value;
        Workbench.longIdleTime = longIdleTimeBox.value;
    }

    GridLayout {
        id: layout

        columns: 3

        Label {
            id: idleTimeLabel

            text: qsTr("Idle time")
        }

        SpinBox {
            id: idleTimeBox

            from: 1
            to: 3600
            editable: true
            value: Workbench.idleTime
        }

        Label {
            id: idleTimeUnit

            text: qsTr("seconds until the text is saved and built")
        }

        Label {
            id: longIdleTimeLabel

            text: qsTr("Long idle time")
        }

        SpinBox {
            id: longIdleTimeBox

            from: 1
            to: 3600
            editable: true
            value: Workbench.longIdleTime
        }

        Label {
            id: longIdleTimeUnit

            text: qsTr("seconds until the error message of a failed build is shown")
        }
    }
}
