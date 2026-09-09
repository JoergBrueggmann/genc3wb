// An image showing one of the four processing states.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

import QtQuick

Image {
    id: root

    property int processingState: 2

    readonly property var imageNames: [
        "indicatorUntouched",
        "indicatorChanged",
        "indicatorUnknownFileUntouched",
        "indicatorUnknownFileChanged"
    ]

    source: "qrc:/genc3wb/qml/icons/" + root.imageNames[root.processingState] + ".png"
    fillMode: Image.PreserveAspectFit
    sourceSize.height: 20
}
