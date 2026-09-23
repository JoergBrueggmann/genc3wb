// The mark of one diagnostic in a code editor: a wavy line below the characters of its range, and its
// message text as a tooltip.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Controls

Item {
    id: root

    required property var textArea
    required property int start
    required property int end
    required property int severity
    required property string message

    // the colour of the severity: red for error, orange for warning, blue for information
    readonly property color colour: root.severity === 0 ? "#d02020" : root.severity === 1 ? "#e08000" : "#2060d0"
    // the rectangles below which the line is drawn, one per line of text the range covers; the
    // rectangles of the characters at the start and at the end of the range are what the text
    // area yields, converted into the segments the line consists of
    readonly property var segments: {
        const textLength = root.textArea.length;
        const rangeStart = Math.min(root.start, textLength);
        const rangeEnd = Math.min(Math.max(root.end, rangeStart), textLength);
        const startRect = root.textArea.positionToRectangle(rangeStart);
        const endRect = root.textArea.positionToRectangle(rangeEnd);
        const lineHeight = startRect.height;
        const left = root.textArea.leftPadding;
        const right = root.textArea.contentWidth + left;
        if (rangeEnd === rangeStart) {
            return [{ "x": startRect.x, "y": startRect.y + lineHeight, "width": Math.max(lineHeight / 2, 6) }];
        }
        if (Math.abs(endRect.y - startRect.y) < 1) {
            return [{ "x": startRect.x, "y": startRect.y + lineHeight, "width": endRect.x - startRect.x }];
        }
        const found = [{ "x": startRect.x, "y": startRect.y + lineHeight, "width": right - startRect.x }];
        for (let y = startRect.y + lineHeight; y < endRect.y - 1; y += lineHeight) {
            found.push({ "x": left, "y": y + lineHeight, "width": right - left });
        }
        found.push({ "x": left, "y": endRect.y + lineHeight, "width": endRect.x - left });
        return found;
    }

    Repeater {
        id: segmentRepeater

        model: root.segments.length
        delegate: Item {
            id: segment

            required property int index

            readonly property var geometry: root.segments[segment.index]

            x: segment.geometry.x
            y: segment.geometry.y - 4
            width: segment.geometry.width
            height: 4
            ToolTip.visible: hover.hovered
            ToolTip.text: root.message

            Canvas {
                id: wave

                anchors.fill: parent
                onPaint: {
                    const context = wave.getContext("2d");
                    context.clearRect(0, 0, wave.width, wave.height);
                    context.strokeStyle = root.colour;
                    context.lineWidth = 1;
                    context.beginPath();
                    context.moveTo(0, wave.height - 1);
                    let up = true;
                    for (let x = 2; x <= wave.width; x += 2) {
                        context.lineTo(x, up ? 1 : wave.height - 1);
                        up = !up;
                    }
                    context.stroke();
                }
                onWidthChanged: wave.requestPaint()
            }

            HoverHandler {
                id: hover
            }
        }
    }
}
