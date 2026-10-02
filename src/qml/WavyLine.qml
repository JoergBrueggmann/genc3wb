// A wavy line of one colour over the width of the item, as the mark of a diagnostic is drawn.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

import QtQuick

Canvas {
    id: wave

    // the colour of the line
    required property color colour

    height: 4
    onPaint: {
        const context = wave.getContext("2d");
        context.clearRect(0, 0, wave.width, wave.height);
        context.strokeStyle = wave.colour;
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
    onColourChanged: wave.requestPaint()
}
