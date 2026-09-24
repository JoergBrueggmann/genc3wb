// The mark of one diagnostic in a code editor: a wavy line below the characters of its range, and its
// message text as a tooltip over the character cells of the range.
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
    // the distance by which the area presenting the tooltip exceeds the character cells on each side
    readonly property int hoverMargin: 2
    // the rectangles below which the line is drawn, one per line of text the range covers, each
    // with the height of its line; the
    // rectangles of the characters at the start and at the end of the range are what the text
    // area yields, converted into the segments the line consists of; an empty range covers one
    // character cell
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
            return [{ "x": startRect.x, "y": startRect.y + lineHeight, "height": lineHeight, "width": fontMetrics.averageCharacterWidth }];
        }
        if (Math.abs(endRect.y - startRect.y) < 1) {
            return [{ "x": startRect.x, "y": startRect.y + lineHeight, "height": lineHeight, "width": endRect.x - startRect.x }];
        }
        const found = [{ "x": startRect.x, "y": startRect.y + lineHeight, "height": lineHeight, "width": right - startRect.x }];
        for (let y = startRect.y + lineHeight; y < endRect.y - 1; y += lineHeight) {
            found.push({ "x": left, "y": y + lineHeight, "height": lineHeight, "width": right - left });
        }
        found.push({ "x": left, "y": endRect.y + lineHeight, "height": lineHeight, "width": endRect.x - left });
        return found;
    }

    FontMetrics {
        id: fontMetrics

        font: root.textArea.font
    }

    Repeater {
        id: segmentRepeater

        model: root.segments.length
        delegate: Item {
            id: segment

            required property int index

            readonly property var geometry: root.segments[segment.index]

            // the character cells of the segment at line height, and the margin around them
            x: segment.geometry.x - root.hoverMargin
            y: segment.geometry.y - segment.geometry.height - root.hoverMargin
            width: segment.geometry.width + 2 * root.hoverMargin
            height: segment.geometry.height + 2 * root.hoverMargin
            ToolTip.visible: hover.hovered
            ToolTip.text: root.message

            Canvas {
                id: wave

                x: root.hoverMargin
                y: root.hoverMargin + segment.geometry.height - 4
                width: segment.geometry.width
                height: 4
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
