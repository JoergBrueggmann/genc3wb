//! The integration tests of the component `core::text_increment`, through the public interface.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use genc3wb::core::text_increment::{IncrementTracker, TextIncrement, increment_between};

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : InputGroup::idle_expired */

#[test]
fn increments_of_an_editing_session_render_as_the_lines_a_receiver_reads() {
    // FR-054, FR-056, FR-059, FR-060, IR-015
    let mut tracker = IncrementTracker::default();
    let mut lines = Vec::new();
    for text in [
        "fn main() {}",
        "fn main() {\n}",
        "fn main() {\n    run();\n}",
    ] {
        if let Some(increment) = tracker.provide(text) {
            lines.push(increment.rendering());
        }
    }
    assert_eq!(
        lines,
        vec![
            "Position:0000, Range: 0000, \"fn main() {}\"",
            "Position:0011, Range: 0000, \"\\n\"",
            "Position:0012, Range: 0000, \"    run();\\n\"",
        ]
    );
}

#[test]
fn increment_between_is_the_pending_increment_of_a_tracker() {
    // FR-054
    let mut tracker = IncrementTracker::default();
    tracker.provide("provided");
    assert_eq!(
        tracker.pending("provoked"),
        increment_between("provided", "provoked")
    );
    assert_eq!(
        tracker.pending("provoked"),
        TextIncrement {
            position: 4,
            range: 2,
            text: "ok".into()
        }
    );
}
