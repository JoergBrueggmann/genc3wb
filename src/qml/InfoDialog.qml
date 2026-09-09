// The modal info dialog.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

import QtQuick
import QtQuick.Controls

Dialog {
    id: infoDialog

    modal: true
    title: "genc³wb"
    standardButtons: Dialog.Ok
    anchors.centerIn: parent

    Label {
        id: infoText

        text: "genc³wb 0.4.0.0\n"
              + "The workbench of the compiler-compiler genc³.\n\n"
              + "Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026\n"
              + "Licensed under the GNU General Public License, see the file LICENSE."
    }
}
