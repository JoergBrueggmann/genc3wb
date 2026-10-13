// The modal info dialog: the name of the product with its version, its description, its copyright
// notice and its licence.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

import QtQuick
import QtQuick.Controls
// The types of the module genc3wb are registered by the bridge crate at run time, which writes no type
// description for qmllint; the import and the unqualified access to the singleton are therefore not linted.
// qmllint disable import unqualified
import genc3wb

Dialog {
    id: infoDialog

    modal: true
    title: "genc³wb"
    padding: 8
    standardButtons: Dialog.Ok
    anchors.centerIn: parent

    Label {
        id: infoText

        // the version is that of the running executable (FR-224)
        text: "genc³wb " + Workbench.version + "\n"
              + "The workbench of the compiler-compiler genc³.\n\n"
              + "Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026\n"
              + "Licensed under the GNU General Public License, see the file LICENSE."
    }
}
