// The handle of a split view: the distance between two groups, 13 pixels, with the line of the splitter in
// its middle.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

import QtQuick
import QtQuick.Controls

Rectangle {
    id: root

    // the orientation of the split view the handle belongs to
    required property int orientation

    implicitWidth: 13
    implicitHeight: 13
    color: "transparent"

    SystemPalette {
        id: palette
    }

    // the line of the splitter: one pixel, highlighted while the handle is pressed or hovered
    Rectangle {
        id: line

        anchors.centerIn: parent
        width: root.orientation === Qt.Horizontal ? 1 : parent.width
        height: root.orientation === Qt.Horizontal ? parent.height : 1
        color: root.SplitHandle.pressed || root.SplitHandle.hovered ? palette.highlight : palette.mid
    }
}
