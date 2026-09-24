//! The *bridged type* `Editing`: the *editing* functions of the *core* as a code editor calls them.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::editing;

use qtbridge::qobject;

// realises FR-136 to FR-138, FR-144, FR-145
/// The singleton through which a code editor reaches the *editing* functions of the *core*.
///
/// * Every offset is an offset of the text in UTF-16 code units, as a QML `TextArea` counts
///   its positions; a negative offset or line is taken as 0.
#[derive(Default)]
pub struct Editing {}

// realises FR-136 to FR-138, FR-144, FR-145
#[qobject(Singleton)]
impl Editing {
    // slots
    // realises FR-136
    /// Yields the start and the end offset of the line `line` of `text` as a list of two, as
    /// [`editing::line_span`] does.
    #[qslot(qml_name = "lineSpan")]
    fn line_span(&self, text: String, line: i32) -> Vec<i32> {
        let (start, end) = editing::line_span(&text, count_of(line));
        vec![int_of(start), int_of(end)]
    }

    // realises FR-137
    /// Yields the start and the end offset of the lines from `from` to `to` of `text` as a list
    /// of two, as [`editing::lines_span`] does.
    #[qslot(qml_name = "linesSpan")]
    fn lines_span(&self, text: String, from: i32, to: i32) -> Vec<i32> {
        let (start, end) = editing::lines_span(&text, count_of(from), count_of(to));
        vec![int_of(start), int_of(end)]
    }

    // realises FR-138
    /// Yields the other occurrences of the selection from `start` to `end` in `text` as a flat
    /// list, the start and the end offset of each in turn, as [`editing::occurrences`] does.
    #[qslot(qml_name = "occurrences")]
    fn occurrences(&self, text: String, start: i32, end: i32) -> Vec<i32> {
        editing::occurrences(&text, count_of(start), count_of(end))
            .into_iter()
            .flat_map(|(start, end)| [int_of(start), int_of(end)])
            .collect()
    }

    // realises FR-144
    /// Yields the leading tabs and spaces of the line of `text` holding `cursor`, as
    /// [`editing::indentation_of_line`] does.
    #[qslot(qml_name = "indentationOfLine")]
    fn indentation_of_line(&self, text: String, cursor: i32) -> String {
        editing::indentation_of_line(&text, count_of(cursor))
    }

    // realises FR-145
    /// Yields the offset of a view that shows the span from `start` to `end` of its content, as
    /// [`editing::view_offset_showing`] does.
    #[qslot(qml_name = "viewOffsetShowing")]
    fn view_offset_showing(
        &self,
        offset: f64,
        view: f64,
        content: f64,
        start: f64,
        end: f64,
    ) -> f64 {
        editing::view_offset_showing(offset, view, content, start, end)
    }
}

/// Yields `value` as a count; 0 where it is negative.
fn count_of(value: i32) -> usize {
    usize::try_from(value).unwrap_or(0)
}

/// Yields `value` as the integer QML takes; the greatest one where it is larger.
fn int_of(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}
