//! The *bridged type* `Editing`: the *editing* functions of the *core* as a code editor calls them.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::editing;
use crate::core::octet_view::{self, EditorMode};

use qtbridge::qobject;

// realises FR-136 to FR-138, FR-144, FR-145, FR-156, FR-160, FR-161, FR-173 to FR-176
/// The singleton through which a code editor reaches the *editing* functions of the *core*.
///
/// * Every offset is an offset of the text in UTF-16 code units, as a QML `TextArea` counts
///   its positions; a negative offset or line is taken as 0.
#[derive(Default)]
pub struct Editing {}

// realises FR-136 to FR-138, FR-144, FR-145, FR-156, FR-160, FR-161, FR-173 to FR-176
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

    // realises FR-156
    /// Yields the index of the position a *mode switch* selects, as
    /// [`octet_view::mode_selected`] does: `chosen` where it is the index of an *editor mode*,
    /// and otherwise that of txt where the document `is_text` and that of hex where it is not.
    #[qslot(qml_name = "modeSelected")]
    fn mode_selected(&self, chosen: i32, is_text: bool) -> i32 {
        int_of(octet_view::mode_selected(mode_of(chosen), is_text).index())
    }

    // realises FR-173
    /// Yields the number of octets per row of the *editor mode* of index `mode` for a document
    /// of `octet_count` octets in a view `columns` character columns wide, as
    /// [`octet_view::octets_per_row_fitting`] does; 0 for an index that names no mode.
    #[qslot(qml_name = "octetsPerRow")]
    fn octets_per_row(&self, mode: i32, octet_count: i32, columns: i32) -> i32 {
        mode_of(mode).map_or(0, |mode| {
            int_of(octet_view::octets_per_row_fitting(
                mode,
                count_of(octet_count),
                count_of(columns),
            ))
        })
    }

    // realises FR-174
    /// Yields the row of `after` octets per row that holds the first octet of the row `row` of
    /// `before` octets per row, as [`octet_view::row_keeping`] does.
    #[qslot(qml_name = "rowKeeping")]
    fn row_keeping(&self, row: i32, before: i32, after: i32) -> i32 {
        int_of(octet_view::row_keeping(
            count_of(row),
            count_of(before),
            count_of(after),
        ))
    }

    // realises FR-160, FR-175
    /// Yields the row label of the row `row` of `per_row` octets per row in a document of
    /// `octet_count` octets, as [`octet_view::row_label`] does.
    #[qslot(qml_name = "rowLabel")]
    fn row_label(&self, per_row: i32, row: i32, octet_count: i32) -> String {
        octet_view::row_label(count_of(row), count_of(per_row), count_of(octet_count))
    }

    // realises FR-161, FR-176
    /// Yields the column header of rows of `per_row` octets of the *editor mode* of index
    /// `mode`, as [`octet_view::column_header`] does; empty for an index that names no mode.
    #[qslot(qml_name = "columnHeader")]
    fn column_header(&self, mode: i32, per_row: i32) -> String {
        mode_of(mode).map_or_else(String::new, |mode| {
            octet_view::column_header(mode, count_of(per_row))
        })
    }
}

/// Yields the *editor mode* of the index `mode`, `None` where the index names none.
fn mode_of(mode: i32) -> Option<EditorMode> {
    usize::try_from(mode).ok().and_then(EditorMode::of_index)
}

/// Yields `value` as a count; 0 where it is negative.
fn count_of(value: i32) -> usize {
    usize::try_from(value).unwrap_or(0)
}

/// Yields `value` as the integer QML takes; the greatest one where it is larger.
fn int_of(value: usize) -> i32 {
    i32::try_from(value).unwrap_or(i32::MAX)
}
