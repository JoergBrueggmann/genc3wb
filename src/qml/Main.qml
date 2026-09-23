// The main window, the compiler network editor: menu bar, status bar, the network file and its graph.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
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
    onClosing: {
        Workbench.network.shutDown();
        Workbench.node.shutDown();
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

    footer: Label {
        id: statusBar

        padding: 4
        text: !Workbench.network.available
              ? qsTr("The build system is not available on this platform.")
              : Workbench.network.building ? qsTr("The build system is building.") : ""
    }

    SplitView {
        id: splitView

        anchors.fill: parent
        anchors.margins: 6
        orientation: Qt.Horizontal

        InputGroup {
            id: networkInputGroup

            group: Workbench.networkInput
            detachable: false
            SplitView.preferredWidth: 480
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

    SettingsDialog {
        id: settingsDialog
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
    }
}
