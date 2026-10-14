// The main window, the compiler network editor: menu bar, status bar, the network file and its graph, the
// dialog telling a refusal of the network file or of the settings file, and the node windows, one per node
// opened.
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
    // request is pending, that the network is being activated or deactivated (FR-269, FR-270),
    // and while a hold request is pending, which node is being opened (FR-285).
    function networkModeText(mode: int): string {
        if (Workbench.network.activating) {
            return qsTr("Activating network");
        }
        if (Workbench.network.deactivating) {
            return qsTr("Deactivating network");
        }
        if (Workbench.network.opening.length > 0) {
            return qsTr("Opening node %1").arg(Workbench.network.opening);
        }
        switch (mode) {
        case 1:
            return qsTr("Network invalid: no node runs");
        case 2:
            return qsTr("Network valid, not active");
        case 3:
            return qsTr("Network active");
        default:
            return "";
        }
    }

    // Yields the index of the entry of `nodeWindowModel` that names the node `name`, -1 where
    // there is none.
    function nodeWindowIndex(name: string): int {
        for (let i = 0; i < nodeWindowModel.count; i++) {
            if (nodeWindowModel.get(i).name === name) {
                return i;
            }
        }
        return -1;
    }

    // Yields the index of the entry of `fileWindowModel` that names the file `path`, -1 where
    // there is none.
    function fileWindowIndex(path: string): int {
        for (let i = 0; i < fileWindowModel.count; i++) {
            if (fileWindowModel.get(i).path === path) {
                return i;
            }
        }
        return -1;
    }

    // Shows, raises and activates `window`, where it is one.
    function raise(window: var) {
        if (window) {
            window.show();
            window.raise();
            window.requestActivate();
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
        // shuts every node editor down as well (FR-068, FR-105)
        Workbench.network.shutDown();
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
            // or a node is being opened (FR-269, FR-270, FR-285)
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

    // one entry per node window presented, carrying the name of its node (FR-001, FR-085)
    ListModel {
        id: nodeWindowModel
    }

    // the node windows, one per entry, each bound to the node editor of its name; a window is
    // destroyed with its detached windows when its entry is removed (FR-001, FR-047, FR-263)
    Instantiator {
        id: nodeWindows

        model: nodeWindowModel
        delegate: NodeWindow {
            required property string name

            node: Workbench.network.nodeNamed(name)
            transientParent: mainWindow
            visible: true
        }
    }

    // one entry per editor window presented, carrying the path of its file (FR-290 to FR-293)
    ListModel {
        id: fileWindowModel
    }

    // the editor windows of the files opened from the graph, one per entry, each bound to the
    // input group of its path; closing one has the network editor close the file (FR-290 to FR-293)
    Instantiator {
        id: fileWindows

        model: fileWindowModel
        delegate: EditorWindow {
            required property string path

            group: Workbench.network.fileNamed(path)
            transientParent: mainWindow
            visible: true
            onClosing: Workbench.network.closeFile(path)
        }
    }

    Connections {
        target: Workbench.network

        // the node editor of the name is named and opening: its window is presented, or raised
        // where it is presented already (FR-085, FR-284)
        function onNodeOpened(name) {
            const index = mainWindow.nodeWindowIndex(name);
            if (index < 0) {
                nodeWindowModel.append({ "name": name });
                return;
            }
            mainWindow.raise(nodeWindows.objectAt(index) as Window);
        }

        // the input group of the path is named and opened: its editor window is presented, or
        // raised where it is presented already (FR-290 to FR-292)
        function onFileOpened(path) {
            const index = mainWindow.fileWindowIndex(path);
            if (index < 0) {
                fileWindowModel.append({ "path": path });
                return;
            }
            mainWindow.raise(fileWindows.objectAt(index) as Window);
        }

        // the input group of the path closed: the user closed its editor window (FR-293)
        function onFileClosed(path) {
            const index = mainWindow.fileWindowIndex(path);
            if (index >= 0) {
                fileWindowModel.remove(index);
            }
        }

        // the node editor of the name closed: the user closed its window, or the network came
        // to be invalid or the build system was lost (FR-089, FR-263)
        function onNodeClosed(name) {
            const index = mainWindow.nodeWindowIndex(name);
            if (index >= 0) {
                nodeWindowModel.remove(index);
            }
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

}
