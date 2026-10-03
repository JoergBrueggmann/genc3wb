//! The *bridged type* `Editing`: the *editing* functions of the *core* as a code editor calls them.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::editing;
use crate::core::octet_edit::{self, Movement};
use crate::core::octet_view::{self, EditorMode};

use qtbridge::qobject;

// realises FR-136 to FR-138, FR-144, FR-145, FR-156, FR-160, FR-161, FR-173 to FR-176, FR-181 to
// FR-184, FR-186, FR-188 to FR-191, FR-201, FR-205
/// The singleton through which a code editor reaches the *editing* functions of the *core*.
///
/// * Every offset is an offset of the text in UTF-16 code units, as a QML `TextArea` counts
///   its positions; a negative offset or line is taken as 0.
#[derive(Default)]
pub struct Editing {}

// realises FR-136 to FR-138, FR-144, FR-145, FR-156, FR-160, FR-161, FR-173 to FR-176, FR-181 to
// FR-184, FR-186, FR-188 to FR-191, FR-201, FR-205
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

    // realises FR-178, FR-191
    /// Yields the number of digits of the *editor mode* of index `mode` that `bit_count` bits
    /// hold, a begun digit counted; 0 for an index that names no mode, and for txt.
    #[qslot(qml_name = "digitCount")]
    fn digit_count(&self, mode: i32, bit_count: i32) -> i32 {
        match digit_bits_of(mode) {
            0 => 0,
            unit => int_of(count_of(bit_count).div_ceil(unit)),
        }
    }

    // realises FR-191
    /// Yields the digit of the *editor mode* of index `to` that holds the first bit of the
    /// digit `digit` of the *editor mode* of index `from`; 0 where either names no mode or txt.
    #[qslot(qml_name = "digitInMode")]
    fn digit_in_mode(&self, digit: i32, from: i32, to: i32) -> i32 {
        match (digit_bits_of(from), digit_bits_of(to)) {
            (0, _) | (_, 0) => 0,
            (from, to) => int_of(count_of(digit) * from / to),
        }
    }

    // realises FR-186, FR-203
    /// Yields whether `text` holds digits of the *editor mode* of index `mode` and white space
    /// alone, and at least one digit, as [`octet_edit::digits_of_text`] reads them.
    #[qslot(qml_name = "digitsValid")]
    fn digits_valid(&self, mode: i32, text: String) -> bool {
        mode_of(mode)
            .and_then(|mode| octet_edit::digits_of_text(&text, mode))
            .is_some_and(|digits| !digits.is_empty())
    }

    // realises FR-203
    /// Yields the number of digits `text` holds in the *editor mode* of index `mode`; 0 where
    /// it holds another character.
    #[qslot(qml_name = "digitsIn")]
    fn digits_in(&self, mode: i32, text: String) -> i32 {
        mode_of(mode)
            .and_then(|mode| octet_edit::digits_of_text(&text, mode))
            .map_or(0, |digits| int_of(digits.len()))
    }

    // realises FR-181, FR-182, FR-205
    /// Yields the digits typing or pasting `count` digits replaces as a list of two, the first
    /// one and their number, as [`octet_edit::typing_span`] does; a negative `anchor` is no
    /// selection.
    #[qslot(qml_name = "typingSpan")]
    fn typing_span(&self, cursor: i32, anchor: i32, count: i32, insert: bool) -> Vec<i32> {
        let (start, removed) = octet_edit::typing_span(
            count_of(cursor),
            usize::try_from(anchor).ok(),
            count_of(count),
            insert,
        );
        vec![int_of(start), int_of(removed)]
    }

    // realises FR-183, FR-184, FR-205
    /// Yields the digits Backspace, where `backward`, or Delete removes as a list of two, the
    /// first one and their number, as [`octet_edit::removal_span`] does; a negative `anchor` is
    /// no selection.
    #[qslot(qml_name = "removalSpan")]
    fn removal_span(&self, cursor: i32, anchor: i32, backward: bool) -> Vec<i32> {
        let (start, removed) =
            octet_edit::removal_span(count_of(cursor), usize::try_from(anchor).ok(), backward);
        vec![int_of(start), int_of(removed)]
    }

    // realises FR-188
    /// Yields the cursor after the movement of index `movement`, as
    /// [`octet_edit::cursor_moved`] does, in rows of `per_row` octets of a document of
    /// `bit_count` bits in the *editor mode* of index `mode`, the view showing `page_rows`
    /// rows; `cursor` where an index names no movement or no mode.
    #[qslot(qml_name = "cursorMoved")]
    fn cursor_moved(
        &self,
        cursor: i32,
        movement: i32,
        mode: i32,
        per_row: i32,
        bit_count: i32,
        page_rows: i32,
    ) -> i32 {
        let movement = usize::try_from(movement).ok().and_then(Movement::of_index);
        match (movement, digit_bits_of(mode)) {
            (None, _) | (_, 0) => cursor,
            (Some(movement), unit) => int_of(octet_edit::cursor_moved(
                count_of(cursor),
                movement,
                count_of(per_row) * 8 / unit,
                count_of(bit_count).div_ceil(unit),
                count_of(page_rows),
            )),
        }
    }

    // realises FR-190, FR-191
    /// Yields where the cursor before the digit `cursor` is shown as a list of three: its row,
    /// its character column and the index of what it stands before, 0 for a digit, 1 for a
    /// placeholder and 2 for nothing, as [`octet_edit::cursor_cell`] does.
    #[qslot(qml_name = "cursorCell")]
    fn cursor_cell(
        &self,
        cursor: i32,
        mode: i32,
        per_row: i32,
        bit_count: i32,
        octet_count: i32,
    ) -> Vec<i32> {
        let cell = octet_edit::cursor_cell(
            count_of(cursor),
            mode_of(mode).unwrap_or_default(),
            count_of(per_row),
            count_of(bit_count),
            count_of(octet_count),
        );
        vec![
            int_of(cell.row),
            int_of(cell.column),
            int_of(cell.kind.index()),
        ]
    }

    // realises FR-189
    /// Yields the digit a click at the character column `column` of the row `row` sets the
    /// cursor before, as [`octet_edit::digit_at`] does.
    #[qslot(qml_name = "digitAt")]
    fn digit_at(
        &self,
        row: i32,
        column: f64,
        between: bool,
        mode: i32,
        per_row: i32,
        bit_count: i32,
    ) -> i32 {
        int_of(octet_edit::digit_at(
            count_of(row),
            column,
            between,
            mode_of(mode).unwrap_or_default(),
            count_of(per_row),
            count_of(bit_count),
        ))
    }

    // realises FR-201
    /// Yields the character columns of the row `row` that hold the digits from `start` to
    /// `end`, exclusive, as a list of two, the first column and their number, as
    /// [`octet_edit::row_selection`] does; empty where the row holds none of them.
    #[qslot(qml_name = "rowSelection")]
    fn row_selection(&self, start: i32, end: i32, row: i32, mode: i32, per_row: i32) -> Vec<i32> {
        octet_edit::row_selection(
            count_of(start),
            count_of(end),
            count_of(row),
            mode_of(mode).unwrap_or_default(),
            count_of(per_row),
        )
        .map_or_else(Vec::new, |(first, columns)| {
            vec![int_of(first), int_of(columns)]
        })
    }
}

/// Yields the number of bits of a digit of the *editor mode* of index `mode`; 0 where the index
/// names no mode, and for txt.
fn digit_bits_of(mode: i32) -> usize {
    mode_of(mode).map_or(0, EditorMode::digit_bits)
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
