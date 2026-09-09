// The group of the compiler-compiler: file name field, file selector and indicator.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls
import QtQuick.Dialogs
import QtQuick.Layouts

GroupBox {
    id: root

    required property var runner

    title: qsTr("Compiler-compiler")

    RowLayout {
        anchors.fill: parent

        TextField {
            id: pathField

            Layout.fillWidth: true
            text: root.runner.path
            onEditingFinished: root.runner.path = pathField.text
        }

        ToolButton {
            id: selectButton

            text: "..."
            onClicked: fileDialog.open()
        }

        Image {
            id: indicator

            source: root.runner.ready ? "qrc:/genc3wb/qml/icons/indicatorUntouched.png"
                                      : "qrc:/genc3wb/qml/icons/indicatorUnknownFileChanged.png"
            fillMode: Image.PreserveAspectFit
            sourceSize.height: 20
        }
    }

    FileDialog {
        id: fileDialog

        title: qsTr("Compiler-compiler executable")
        onAccepted: root.runner.path = PathOfUrl.path(fileDialog.selectedFile)
    }
}
