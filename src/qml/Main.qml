// The main window, the compiler network editor: menu bar, status bar, the network file and its graph, and the
// dialog telling a refusal of the network file or of the settings file.
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

ApplicationWindow {
    id: mainWindow

    width: 1280
    height: 800
    visible: true
    title: "genc³wb"

    // the refusal of the settings file and of the network file, empty where there is none
    // (FR-219, FR-220)
    readonly property string settingsRefusal: Workbench.settingsRefusedFile.length > 0
                                              ? qsTr("The settings are not stored: %1")
                                                .arg(heldText(Workbench.settingsRefusedFile,
                                                              Workbench.settingsHolderPid,
                                                              Workbench.settingsHolderHost,
                                                              Workbench.settingsHolderSince))
                                              : ""
    readonly property string networkRefusal: Workbench.networkInput.refusedFile.length > 0
                                             ? qsTr("The network is not opened: %1")
                                               .arg(heldText(Workbench.networkInput.refusedFile,
                                                             Workbench.networkInput.holderPid,
                                                             Workbench.networkInput.holderHost,
                                                             Workbench.networkInput.holderSince))
                                             : ""

    // Yields that `file` is held by the holder of its lock file, the time in the local time of
    // the user (FR-219); a process number of 0 names no holder.
    function heldText(file: string, pid: int, host: string, since: string): string {
        if (pid <= 0) {
            return qsTr("%1 is held by another genc³wb.").arg(file);
        }
        const time = new Date(since).toLocaleString(Qt.locale(), Locale.ShortFormat);
        return host.length > 0
               ? qsTr("%1 is held by genc³wb, process %2 on %3, since %4.").arg(file).arg(pid).arg(host).arg(time)
               : qsTr("%1 is held by genc³wb, process %2, since %3.").arg(file).arg(pid).arg(time);
    }

    // Yields the text of the network mode `mode` for the status bar, as the message schema
    // numbers it: 1 invalid, 2 valid, 3 active; empty for 0, where none is kept (FR-258)
    function networkModeText(mode: int): string {
        switch (mode) {
        case 1:
            return qsTr("The network is invalid: no node runs.");
        case 2:
            return qsTr("The network is valid and not active: no node runs, activate the network to open a node.");
        case 3:
            return qsTr("The network is active.");
        default:
            return "";
        }
    }

    // Presents the dialog of FR-219 with `texts`, where one is not empty.
    function tellRefusals(texts: list<string>) {
        const told = texts.filter(text => text.length > 0);
        if (told.length > 0) {
            refusalDialog.text = told.join("\n\n");
            refusalDialog.open();
        }
    }

    Component.onCompleted: tellRefusals([settingsRefusal, networkRefusal])
    onClosing: {
        Workbench.network.shutDown();
        Workbench.node.shutDown();
        Workbench.releaseLocks();
    }

    menuBar: MenuBar {
        id: menuBar

        Menu {
            id: settingsMenu

            title: qsTr("&Settings")

            Action {
                id: settingsAction

                text: qsTr("&Settings...")
                onTriggered: settingsDialog.open()
            }

            Action {
                id: insertKeyAction

                text: qsTr("Insert &key...")
                onTriggered: insertKeyDialog.open()
            }
        }

        Menu {
            id: editMenu

            title: qsTr("&Edit")

            Action {
                id: insertModeAction

                text: Workbench.insertMode ? qsTr("Switch to &overwrite mode") : qsTr("Switch to &insert mode")
                onTriggered: Workbench.insertMode = !Workbench.insertMode
            }
        }

        Menu {
            id: helpMenu

            title: qsTr("&Help")

            Action {
                id: infoAction

                text: qsTr("&Info")
                onTriggered: infoDialog.open()
            }
        }
    }

    footer: RowLayout {
        id: footerRow

        Label {
            id: statusBar

            Layout.fillWidth: true
            padding: 4
            // the refusals while they last (FR-220), then the state of the build system and
            // the network mode (FR-258)
            text: [mainWindow.settingsRefusal,
                   mainWindow.networkRefusal,
                   !Workbench.network.available
                   ? qsTr("The build system is not available on this platform.")
                   : Workbench.network.building ? qsTr("The build system is building.") : "",
                   mainWindow.networkModeText(Workbench.network.networkMode)]
                  .filter(part => part.length > 0).join("  ·  ")
        }

        TypingModeLabel {
            id: typingModeLabel

            padding: 4
        }
    }

    SplitView {
        id: splitView

        anchors.fill: parent
        anchors.margins: 6
        orientation: Qt.Horizontal
        // the splitter between the network file and its graph (FR-148); its position is stored
        // when it is released and restored at start-up (FR-149)
        onResizingChanged: {
            if (!splitView.resizing) {
                Workbench.setNetworkLeftWidth(Math.round(networkInputGroup.width));
            }
        }

        InputGroup {
            id: networkInputGroup

            group: Workbench.networkInput
            detachable: false
            SplitView.preferredWidth: Workbench.networkLeftWidth
            SplitView.minimumWidth: 240
        }

        NetworkGraphGroup {
            id: networkGraphGroup

            network: Workbench.network
            SplitView.fillWidth: true
            SplitView.minimumWidth: 240
        }
    }

    InfoDialog {
        id: infoDialog
    }

    Dialog {
        id: refusalDialog

        property alias text: refusalText.text

        // a width of its own, since the width of the style follows the text and the text would
        // follow the width
        width: Math.min(mainWindow.width - 48, 560)
        modal: true
        title: qsTr("Held by another genc³wb")
        standardButtons: Dialog.Ok
        anchors.centerIn: parent

        Label {
            id: refusalText

            width: refusalDialog.availableWidth
            wrapMode: Text.Wrap
        }
    }

    SettingsDialog {
        id: settingsDialog
    }

    InsertKeyDialog {
        id: insertKeyDialog
    }

    NodeWindow {
        id: nodeWindow

        transientParent: mainWindow
        visible: false
    }

    Connections {
        target: Workbench.network

        function onNodeOpened(name) {
            nodeWindow.show();
            nodeWindow.raise();
            nodeWindow.requestActivate();
        }

        // the network ceased to be active: no node runs, so the node window and its detached
        // windows close (FR-263)
        function onNetworkDeactivated() {
            nodeWindow.closeWithDetachedWindows();
        }

        function onProcessingMeasured(processingTime) {
            Workbench.reportProcessingTime(processingTime);
        }
    }

    Connections {
        target: Workbench.networkInput

        function onRefused() {
            mainWindow.tellRefusals([mainWindow.networkRefusal]);
        }
    }

    Connections {
        target: Workbench.node

        function onProcessingMeasured(processingTime) {
            Workbench.reportProcessingTime(processingTime);
        }
    }
}
