// The conversion of the URL a file dialog yields into the path the bridge takes.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

pragma Singleton

import QtQuick

QtObject {
    id: root

    function path(url) {
        let text = url.toString();
        if (text.startsWith("file:///")) {
            text = text.substring(7);
            if (text.length > 2 && text.charAt(2) === ':') {
                text = text.substring(1);
            }
        } else if (text.startsWith("file://")) {
            text = text.substring(7);
        }
        return decodeURIComponent(text);
    }
}
