// The tests of the playback delay dialog (FR-542, FR-543): it opens with the switch and the value of the
// settings and the check box unchecked, the field is enabled while the switch is checked alone, OK sets
// the switch, the value and the setting and accepts, and Cancel sets nothing and rejects.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

import QtQuick
import QtQuick.Controls
import QtTest
import genc3wb

Item {
    id: root

    // the number of times the dialog was accepted and rejected
    property int accepted: 0
    property int rejected: 0

    width: 640
    height: 400

    PlaybackDelayDialog {
        id: dialog

        onAccepted: root.accepted += 1
        onRejected: root.rejected += 1
    }

    TestCase {
        id: testCase

        name: "PlaybackDelay"
        when: windowShown

        function init() {
            Workbench.playbackDelayOn = true;
            Workbench.playbackDelayIndex = 5;
            Workbench.playbackDelayDialogDisabled = false;
            Workbench.delaysSet = 0;
            root.accepted = 0;
            root.rejected = 0;
        }

        function cleanup() {
            if (dialog.visible) {
                dialog.close();
            }
        }

        // Yields the spin box, the two check boxes, the switch first, and the two buttons of the dialog.
        function items(): var {
            const boxes = [];
            const checks = [];
            const buttons = [];
            function collect(item) {
                for (let i = 0; i < item.children.length; i++) {
                    const child = item.children[i];
                    if (child instanceof SpinBox) {
                        boxes.push(child);
                    } else if (child instanceof CheckBox) {
                        checks.push(child);
                    } else if (child instanceof Button) {
                        buttons.push(child);
                    }
                    collect(child);
                }
            }
            collect(dialog.contentItem);
            collect(dialog.footer);
            return { box: boxes[0], on: checks[0], check: checks[1], cancel: buttons[0], ok: buttons[1] };
        }

        function test_opens_with_the_stored_delay_and_the_check_box_unchecked() {
            // FR-542: the switch and the field hold the playback delay of the settings, 0.283 seconds as the
            // index 5 shown with two decimals, and the check box is unchecked, also after it was checked at
            // a previous opening
            dialog.open();
            tryCompare(dialog, "visible", true);
            const first = items();
            compare(first.on.checked, true);
            compare(first.box.value, 5);
            compare(first.box.displayText, "0.28");
            compare(first.check.checked, false);
            first.check.checked = true;
            dialog.close();
            tryCompare(dialog, "visible", false);
            Workbench.playbackDelayOn = false;
            Workbench.playbackDelayIndex = 12;
            dialog.open();
            tryCompare(dialog, "visible", true);
            const second = items();
            compare(second.on.checked, false);
            compare(second.box.value, 12);
            compare(second.box.displayText, "3.20");
            compare(second.check.checked, false);
        }

        function test_the_field_is_enabled_while_the_switch_is_checked_alone() {
            // FR-542: the field follows the switch
            dialog.open();
            tryCompare(dialog, "visible", true);
            const its = items();
            compare(its.box.enabled, true);
            its.on.checked = false;
            compare(its.box.enabled, false);
            its.on.checked = true;
            compare(its.box.enabled, true);
        }

        function test_ok_sets_the_delay_and_accepts() {
            // FR-543: OK hands the switch, the index and the check box unchecked, and accepts the dialog
            dialog.open();
            tryCompare(dialog, "visible", true);
            const its = items();
            its.box.value = 9;
            mouseClick(its.ok);
            tryCompare(root, "accepted", 1);
            compare(Workbench.playbackDelayOn, true);
            compare(Workbench.playbackDelayIndex, 9);
            compare(Workbench.playbackDelay, 1131);
            compare(Workbench.playbackDelayDialogDisabled, false);
            compare(Workbench.delaysSet, 1);
            compare(root.rejected, 0);
            tryCompare(dialog, "visible", false);
        }

        function test_ok_with_the_switch_off_and_the_check_box_disables_the_dialog() {
            // FR-543: OK with the switch unchecked sets no delay, the index kept, and with the check box
            // checked switches the playback delay dialog setting on
            dialog.open();
            tryCompare(dialog, "visible", true);
            const its = items();
            its.on.checked = false;
            its.check.checked = true;
            mouseClick(its.ok);
            tryCompare(root, "accepted", 1);
            compare(Workbench.playbackDelayOn, false);
            compare(Workbench.playbackDelayIndex, 5);
            compare(Workbench.playbackDelay, 0);
            compare(Workbench.playbackDelayDialogDisabled, true);
            compare(Workbench.delaysSet, 1);
        }

        function test_cancel_sets_nothing_and_rejects() {
            // FR-543: Cancel changes no setting and rejects the dialog
            dialog.open();
            tryCompare(dialog, "visible", true);
            const its = items();
            its.box.value = 2;
            its.on.checked = false;
            its.check.checked = true;
            mouseClick(its.cancel);
            tryCompare(root, "rejected", 1);
            compare(Workbench.playbackDelayOn, true);
            compare(Workbench.playbackDelayIndex, 5);
            compare(Workbench.playbackDelayDialogDisabled, false);
            compare(Workbench.delaysSet, 0);
            compare(root.accepted, 0);
            tryCompare(dialog, "visible", false);
        }
    }
}
