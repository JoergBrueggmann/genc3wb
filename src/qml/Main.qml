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

    // the holder of the settings file and of the network file, as a phrase, empty where there
    // is no refusal (FR-219, FR-220)
    readonly property string settingsHeld: Workbench.settingsRefusedFile.length > 0
                                           ? heldText(Workbench.settingsRefusedFile,
                                                      Workbench.settingsHolderPid,
                                                      Workbench.settingsHolderHost,
                                                      Workbench.settingsHolderSince)
                                           : ""
    readonly property string networkHeld: Workbench.networkInput.refusedFile.length > 0
                                          ? heldText(Workbench.networkInput.refusedFile,
                                                     Workbench.networkInput.holderPid,
                                                     Workbench.networkInput.holderHost,
                                                     Workbench.networkInput.holderSince)
                                          : ""
    // the refusals for the status bar: short phrases (FR-220)
    readonly property string settingsRefusal: mainWindow.settingsHeld.length > 0
                                              ? qsTr("Settings not stored: %1").arg(mainWindow.settingsHeld)
                                              : ""
    readonly property string networkRefusal: mainWindow.networkHeld.length > 0
                                             ? qsTr("Network not opened: %1").arg(mainWindow.networkHeld)
                                             : ""

    // Yields that `file` is held by the holder of its lock file, as a phrase without a period,
    // the time in the local time of the user (FR-219); a process number of 0 names no holder.
    function heldText(file: string, pid: int, host: string, since: string): string {
        if (pid <= 0) {
            return qsTr("%1 held by another genc³wb").arg(file);
        }
        const time = new Date(since).toLocaleString(Qt.locale(), Locale.ShortFormat);
        return host.length > 0
               ? qsTr("%1 held by genc³wb, process %2 on %3, since %4").arg(file).arg(pid).arg(host).arg(time)
               : qsTr("%1 held by genc³wb, process %2, since %3").arg(file).arg(pid).arg(time);
    }

    // Yields the sentences of the dialog of FR-219 for the refusals, empty where there is none.
    function refusalSentences(): list<string> {
        return [mainWindow.settingsHeld.length > 0
                ? qsTr("The settings are not stored: %1.").arg(mainWindow.settingsHeld) : "",
                mainWindow.networkHeld.length > 0
                ? qsTr("The network is not opened: %1.").arg(mainWindow.networkHeld) : ""];
    }

    // Yields the phrase of the status bar for the network mode `mode`, as the message schema
    // numbers it, 0 where none is kept (FR-258); while an activate request or a deactivate
    // request is pending, that the network is being activated or deactivated (FR-269, FR-270).
    function networkModeText(mode: int): string {
        if (Workbench.network.activating) {
            return qsTr("Activating network");
        }
        if (Workbench.network.deactivating) {
            return qsTr("Deactivating network");
        }
        switch (mode) {
        case 1:
            return qsTr("Network invalid: no node runs");
        case 2:
            return qsTr("Network valid, not active: activate to open a node");
        case 3:
            return qsTr("Network active");
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

    Component.onCompleted: tellRefusals(refusalSentences())
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

        spacing: 5

        Label {
            id: statusBar

            Layout.fillWidth: true
            padding: 3
            leftPadding: 8
            // the refusals while they last (FR-220), then the state of the build system and
            // the network mode (FR-258), or that the network is being activated or deactivated
            // (FR-269, FR-270)
            text: [mainWindow.settingsRefusal,
                   mainWindow.networkRefusal,
                   !Workbench.network.available
                   ? qsTr("Build system not available on this platform")
                   : Workbench.network.building ? qsTr("Building") : "",
                   mainWindow.networkModeText(Workbench.network.networkMode)]
                  .filter(part => part.length > 0).join("  ·  ")
        }

        TypingModeLabel {
            id: typingModeLabel

            padding: 3
            rightPadding: 8
        }
    }

    SplitView {
        id: splitView

        anchors.fill: parent
        anchors.margins: 8
        orientation: Qt.Horizontal
        handle: SplitterHandle {
            orientation: splitView.orientation
        }
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
