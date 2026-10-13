// The tests of the position switch (FR-259, FR-265, FR-266, IR-031, IR-039): a tap on a position emits its
// index, a disabled switch emits nothing, the highlight follows the owner and not the tap, and the tooltip
// is the text given.
//
// Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
// Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

import QtQuick
import QtQuick.Controls
import QtTest
import genc3wb

Item {
    id: root

    // the index of the position tapped last, -1 where none was
    property int tapped: -1
    // the number of taps the switch emitted
    property int taps: 0

    width: 320
    height: 120

    // the activation switch as the group for the network graph instantiates it
    PositionSwitch {
        id: positionSwitch

        x: 20
        y: 20
        labels: ["ina", "act"]
        selected: 0
        tooltip: "Inactive / Active - to deactivate or activate the network"
        onPositionTapped: index => {
            root.tapped = index;
            root.taps += 1;
        }
    }

    TestCase {
        id: testCase

        name: "PositionSwitch"
        when: windowShown

        function init() {
            root.tapped = -1;
            root.taps = 0;
            positionSwitch.enabled = true;
            positionSwitch.selected = 0;
        }

        // Clicks the position of index `index` of the two: the left or the right half of the switch.
        function clickPosition(index: int) {
            mouseClick(positionSwitch, positionSwitch.width * (index === 0 ? 0.25 : 0.75), positionSwitch.height / 2);
        }

        function test_tap_on_a_position_emits_its_index() {
            clickPosition(1);
            compare(root.tapped, 1);
            compare(root.taps, 1);
            clickPosition(0);
            compare(root.tapped, 0);
            compare(root.taps, 2);
        }

        function test_disabled_switch_emits_nothing() {
            positionSwitch.enabled = false;
            clickPosition(1);
            clickPosition(0);
            compare(root.taps, 0);
        }

        function test_highlight_follows_selected_not_the_tap() {
            clickPosition(1);
            compare(positionSwitch.selected, 0);
            positionSwitch.selected = 1;
            compare(positionSwitch.selected, 1);
        }

        function test_tooltip_is_the_text_given() {
            compare(positionSwitch.ToolTip.text, "Inactive / Active - to deactivate or activate the network");
        }
    }
}
