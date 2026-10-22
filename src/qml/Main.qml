// The main window, the compiler network editor: menu bar, status bar, the network file and its graph, the
// dialog telling a refusal of the network file, the failure of a write of the settings file in the status
// bar, the node windows, one per node opened, the playback of an automation script: its menu, its
// dialogs, the timer that steps the player and the dispatch of the front-end actions; and the recording:
// its menu items, its dialogs and its status text.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
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

    // the holder of the network file, as a phrase, empty where there is no refusal (FR-219, FR-220)
    readonly property string networkHeld: Workbench.networkInput.refusedFile.length > 0
                                          ? heldText(Workbench.networkInput.refusedFile,
                                                     Workbench.networkInput.holderPid,
                                                     Workbench.networkInput.holderHost,
                                                     Workbench.networkInput.holderSince)
                                          : ""
    // the failure of the last write of the settings file and the refusal for the status bar: short
    // phrases (FR-220, FR-480)
    readonly property string settingsFailure: Workbench.settingsFailure.length > 0
                                              ? qsTr("Settings not stored: %1").arg(Workbench.settingsFailure)
                                              : ""
    readonly property string networkRefusal: mainWindow.networkHeld.length > 0
                                             ? qsTr("Network not opened: %1").arg(mainWindow.networkHeld)
                                             : ""
    // the failure of the last start of a recording or write of an expected file, and that a recording
    // runs, for the status bar (FR-532)
    readonly property string recordingFailure: Workbench.automation.recordingFailure.length > 0
                                               ? qsTr("Recording failed: %1").arg(Workbench.automation.recordingFailure)
                                               : ""
    readonly property string recordingText: Workbench.automation.recording
                                            ? qsTr("Recording to %1").arg(Workbench.automation.recordingPath)
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

    // Yields the sentence of the dialog of FR-219 for the refusal, empty where there is none.
    function refusalSentences(): list<string> {
        return [mainWindow.networkHeld.length > 0
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

    // Yields the window an action of a playback names: the main window, the node window of the
    // node, or the editor window on the file at the path; null where none is presented (FR-508).
    function windowOf(kind: string, name: string): var {
        if (kind === "main") {
            return mainWindow;
        }
        if (kind === "node") {
            const index = mainWindow.nodeWindowIndex(name);
            return index < 0 ? null : nodeWindows.objectAt(index);
        }
        if (kind === "file") {
            const index = mainWindow.fileWindowIndex(name);
            return index < 0 ? null : fileWindows.objectAt(index);
        }
        return null;
    }

    // Yields the component that presents the code editor an action of a playback names by its
    // four fields: the input group of the network file, the editor window on the file, or what
    // the node window yields for the item; null where none is presented (FR-505 to FR-508).
    function presenterOf(kind: string, name: string, item: string, index: int): var {
        if (kind === "network") {
            return networkInputGroup;
        }
        if (kind === "file") {
            return mainWindow.windowOf("file", name);
        }
        const window = mainWindow.windowOf("node", name);
        return window ? window.presenterOf(item, index) : null;
    }

    // Yields the code editor an action of a playback names, null where none is presented.
    function editorOf(kind: string, name: string, item: string, index: int): var {
        const presenter = mainWindow.presenterOf(kind, name, item, index);
        if (!presenter) {
            return null;
        }
        return item === "diagnostics" ? presenter.diagnosticsEditor : presenter.codeEditor;
    }

    // Tells the player that a front-end action is done, or that nothing was found to carry it out
    // on; deferred, so that the player is not re-entered while its signal is emitted.
    function frontEndDone(done: bool) {
        Qt.callLater(() => Workbench.automation.frontEndDone(done));
    }

    // Presents the dialog of FR-219 with `texts`, where one is not empty.
    function tellRefusals(texts: list<string>) {
        const told = texts.filter(text => text.length > 0);
        if (told.length > 0) {
            refusalDialog.text = told.join("\n\n");
            refusalDialog.open();
        }
    }

    Component.onCompleted: {
        // the front end is up: a file the operating system hands over reaches the workbench (FR-476)
        Workbench.started();
        tellRefusals(refusalSentences());
    }
    onClosing: {
        // the recording is completed first (FR-529), then every node editor is shut down as well
        // (FR-068, FR-105)
        Workbench.automation.stopRecording();
        Workbench.network.shutDown();
        Workbench.releaseLock();
    }
    // the main window became the active one, for the recorder (FR-526)
    onActiveChanged: {
        if (mainWindow.active) {
            Workbench.automation.recordWindowFocus("main", "");
        }
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

        // the menu of the playback and the recording (FR-498)
        Menu {
            id: automationMenu

            title: qsTr("&Automation")
            // whether a code editor holds or held the focus is state of the recorder, read when the
            // menu is about to show (FR-531)
            onAboutToShow: storeAction.enabled = Workbench.automation.recording && Workbench.automation.canStore()

            Action {
                id: playbackAction

                text: qsTr("&Play back script...")
                enabled: !Workbench.automation.recording && !Workbench.automation.running
                onTriggered: scriptDialog.open()
            }

            Action {
                id: startRecordingAction

                text: qsTr("&Start and create recording file...")
                enabled: !Workbench.automation.recording && !Workbench.automation.running
                onTriggered: recordingFileDialog.open()
            }

            Action {
                id: stopRecordingAction

                text: qsTr("S&top and save recording...")
                enabled: Workbench.automation.recording
                onTriggered: stopRecordingDialog.open()
            }

            Action {
                id: backupAction

                text: qsTr("&Backup to restore on replay...")
                enabled: Workbench.automation.recording
                onTriggered: backupDialog.open()
            }

            Action {
                id: storeAction

                text: qsTr("Store to &verify on replay")
                enabled: false
                onTriggered: Workbench.automation.storeFocused()
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
            // the failed write of the settings file and the refusal while they last (FR-220,
            // FR-480), then the state of the build system and the network mode (FR-258), or that
            // the network is being activated or deactivated or a node is being opened (FR-269,
            // FR-270, FR-285)
            text: [mainWindow.settingsFailure,
                   mainWindow.networkRefusal,
                   mainWindow.recordingFailure,
                   !Workbench.network.available
                   ? qsTr("Build system not available on this platform")
                   : Workbench.network.building ? qsTr("Building") : "",
                   mainWindow.networkModeText(Workbench.network.networkMode),
                   mainWindow.recordingText]
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

    // the input creation confirmation dialog: names the files of the nodes about to be opened or
    // activated that do not exist, each with its path (FR-487, FR-488); Create creates them with
    // their missing folders and goes on, its check box switching the silent creation setting on,
    // Cancel leaves the node, the file or the network as it is (FR-489)
    Dialog {
        id: creationDialog

        property alias text: creationText.text

        width: Math.min(mainWindow.width - 48, 640)
        modal: true
        title: qsTr("Create missing files")
        anchors.centerIn: parent
        footer: DialogButtonBox {
            Button {
                text: qsTr("Create")
                DialogButtonBox.buttonRole: DialogButtonBox.AcceptRole
            }
            Button {
                text: qsTr("Cancel")
                DialogButtonBox.buttonRole: DialogButtonBox.RejectRole
            }
        }
        onAboutToShow: silentBox.checked = false
        onAccepted: Workbench.network.answerCreation(true, silentBox.checked)
        onRejected: Workbench.network.answerCreation(false, false)

        ColumnLayout {
            width: creationDialog.availableWidth
            spacing: 5

            Label {
                id: creationText

                Layout.fillWidth: true
                wrapMode: Text.Wrap
            }

            CheckBox {
                id: silentBox

                text: qsTr("Do not show this dialog again")
            }
        }
    }

    // names the folder or the file that could not be created and the reason (FR-491); Continue
    // goes on without the files not created, Cancel leaves the node, the file or the network as
    // it is
    Dialog {
        id: creationFailureDialog

        property alias text: creationFailureText.text

        width: Math.min(mainWindow.width - 48, 560)
        modal: true
        title: qsTr("Files not created")
        anchors.centerIn: parent
        footer: DialogButtonBox {
            Button {
                text: qsTr("Continue")
                DialogButtonBox.buttonRole: DialogButtonBox.AcceptRole
            }
            Button {
                text: qsTr("Cancel")
                DialogButtonBox.buttonRole: DialogButtonBox.RejectRole
            }
        }
        onAccepted: Workbench.network.answerCreationFailure(true)
        onRejected: Workbench.network.answerCreationFailure(false)

        Label {
            id: creationFailureText

            width: creationFailureDialog.availableWidth
            wrapMode: Text.Wrap
        }
    }

    SettingsDialog {
        id: settingsDialog
    }

    // the file selector dialog of an automation script; its acceptance starts the playback
    // (FR-499, IR-211)
    FileDialog {
        id: scriptDialog

        title: qsTr("Automation script")
        nameFilters: [qsTr("Automation scripts (*.gc3s)")]
        onAccepted: Workbench.automation.play(PathOfUrl.path(scriptDialog.selectedFile))
    }

    // the file dialog of the operating system for the gc3s-file of an individual recording, which asks
    // before a file that exists is replaced; its acceptance starts the recording (FR-528, IR-217)
    FileDialog {
        id: recordingFileDialog

        title: qsTr("Recording file")
        fileMode: FileDialog.SaveFile
        defaultSuffix: "gc3s"
        nameFilters: [qsTr("Automation scripts (*.gc3s)")]
        onAccepted: Workbench.automation.startRecording(PathOfUrl.path(recordingFileDialog.selectedFile))
    }

    // asks whether to stop the recording and save it into its file (FR-529)
    Dialog {
        id: stopRecordingDialog

        property alias text: stopRecordingText.text

        width: Math.min(mainWindow.width - 48, 560)
        modal: true
        title: qsTr("Stop recording")
        standardButtons: Dialog.Ok | Dialog.Cancel
        anchors.centerIn: parent
        onAboutToShow: stopRecordingDialog.text = qsTr("Stop the recording and save it into %1?").arg(Workbench.automation.recordingPath)
        onAccepted: Workbench.automation.stopRecording()

        Label {
            id: stopRecordingText

            width: stopRecordingDialog.availableWidth
            wrapMode: Text.Wrap
        }
    }

    // the backup dialog (FR-530)
    BackupDialog {
        id: backupDialog
    }

    // the playback report of a playback started from the menu (FR-515, FR-516)
    Dialog {
        id: reportDialog

        width: Math.min(mainWindow.width - 48, 720)
        height: Math.min(mainWindow.height - 48, 480)
        modal: true
        title: qsTr("Playback report")
        standardButtons: Dialog.Ok
        anchors.centerIn: parent

        ScrollView {
            id: reportScroll

            anchors.fill: parent

            TextArea {
                id: reportArea

                readOnly: true
                wrapMode: TextEdit.NoWrap
                font.family: "Courier New"
            }
        }
    }

    // steps the player while a playback runs (FR-500)
    Timer {
        id: playerTimer

        interval: 50
        repeat: true
        running: Workbench.automation.running
        onTriggered: Workbench.automation.step()
    }

    InsertKeyDialog {
        id: insertKeyDialog
    }

    // one entry per node window presented, carrying the name of its node (FR-001, FR-085)
    ListModel {
        id: nodeWindowModel
    }

    // the node windows, one per entry, each bound to the node editor of its name; a window is
    // destroyed when its entry is removed (FR-001, FR-047, FR-263)
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

        // files of the nodes about to be opened or activated do not exist (FR-487, FR-488): the
        // text lists the meta compiler DSLs and the inputs, each a line per path
        function onCreationAsked(metaDsls, inputs) {
            let text = qsTr("The following files do not exist yet. Create creates them, with the folders of their paths that are missing, and goes on.");
            if (metaDsls.length > 0) {
                text += "\n\n" + qsTr("Meta compiler DSL, from the template:") + "\n" + metaDsls;
            }
            if (inputs.length > 0) {
                text += "\n\n" + qsTr("Input, empty:") + "\n" + inputs;
            }
            creationDialog.text = text;
            creationDialog.open();
        }

        // a folder or a file could not be created (FR-491)
        function onCreationFailed(message) {
            creationFailureDialog.text = qsTr("A file could not be created: %1. Continue goes on without the files not created.").arg(message);
            creationFailureDialog.open();
        }
    }

    Connections {
        target: Workbench.networkInput

        function onRefused() {
            mainWindow.tellRefusals([mainWindow.networkRefusal]);
        }
    }

    // the front-end actions of a playback, each carried out on the code editor, the component or
    // the window named, and the end of a playback (FR-505, FR-507, FR-508, FR-510, FR-516, FR-517)
    Connections {
        id: automationConnections

        target: Workbench.automation

        function onEditAsked(kind, name, item, index, start, end, text, delay) {
            const editor = mainWindow.editorOf(kind, name, item, index);
            if (!editor) {
                mainWindow.frontEndDone(false);
                return;
            }
            editor.playEdit(start, end, text, delay, () => mainWindow.frontEndDone(true));
        }

        function onCursorAsked(kind, name, item, index, offset) {
            const editor = mainWindow.editorOf(kind, name, item, index);
            if (editor) {
                editor.placeCursor(offset);
            }
            mainWindow.frontEndDone(editor !== null);
        }

        function onSelectAsked(kind, name, item, index, start, end) {
            const editor = mainWindow.editorOf(kind, name, item, index);
            if (editor) {
                editor.selectRange(start, end);
            }
            mainWindow.frontEndDone(editor !== null);
        }

        function onFocusAsked(kind, name, item, index, window) {
            if (window) {
                const target = mainWindow.windowOf(kind, name);
                if (target) {
                    mainWindow.raise(target);
                }
                mainWindow.frontEndDone(target !== null);
                return;
            }
            const editor = mainWindow.editorOf(kind, name, item, index);
            if (editor) {
                editor.takeFocus();
            }
            mainWindow.frontEndDone(editor !== null);
        }

        function onModeAsked(kind, name, item, index, mode) {
            const presenter = mainWindow.presenterOf(kind, name, item, index);
            if (presenter) {
                presenter.selectMode(mode);
            }
            mainWindow.frontEndDone(presenter !== null);
        }

        function onAnswerAsked(dialog, choice, silent) {
            if (dialog === "save") {
                networkInputGroup.answerSave(choice);
            } else if (dialog === "creation") {
                silentBox.checked = silent;
                if (choice) {
                    creationDialog.accept();
                } else {
                    creationDialog.reject();
                }
            } else if (dialog === "creation failure") {
                if (choice) {
                    creationFailureDialog.accept();
                } else {
                    creationFailureDialog.reject();
                }
            }
            mainWindow.frontEndDone(true);
        }

        // a verification run terminates as the main window does when closed; a playback of the
        // menu presents the report and leaves the product as it is
        function onFinished(code) {
            if (Workbench.automation.verifying) {
                Workbench.network.shutDown();
                Workbench.releaseLock();
                Qt.exit(code);
                return;
            }
            reportArea.text = Workbench.automation.report;
            reportDialog.open();
        }
    }

}
