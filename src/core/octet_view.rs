//! The *editor modes*: the octets of a document read as text, and presented as rows in hex and bin.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

/// The replacement character U+FFFD, which txt presents an octet that is no character as.
const REPLACEMENT: char = '\u{FFFD}';

/// The numbers of octets one row of hex presents, the one chosen by the width available.
const HEX_WIDTHS: [usize; 3] = [16, 32, 64];

/// The numbers of octets one row of bin presents, the one chosen by the width available.
const BIN_WIDTHS: [usize; 4] = [4, 8, 16, 32];

/// The number of octets whose numbers differ in their last hexadecimal digit alone, up to which
/// rows share a row label that drops that digit.
const LABEL_OCTETS: usize = 16;

/// The least number of hexadecimal digits of the row label of a row of more than 16 octets.
const FULL_LABEL_DIGITS: usize = 3;

// realises FR-154, FR-155
/// An *editor mode*: how a code editor presents the octets of its document.
///
/// * The order of the variants is the order of the positions of the *mode switch*.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum EditorMode {
    /// the octets read as UTF-8
    #[default]
    Txt,
    /// each octet as two hexadecimal digits, 16, 32 or 64 octets per row
    Hex,
    /// each octet as eight binary digits, 4, 8, 16 or 32 octets per row
    Bin,
}

impl EditorMode {
    /// Yields the index of the mode in the order of the variants.
    pub fn index(self) -> usize {
        match self {
            EditorMode::Txt => 0,
            EditorMode::Hex => 1,
            EditorMode::Bin => 2,
        }
    }

    /// Yields the mode of an index, `None` where the index names none.
    pub fn of_index(index: usize) -> Option<EditorMode> {
        match index {
            0 => Some(EditorMode::Txt),
            1 => Some(EditorMode::Hex),
            2 => Some(EditorMode::Bin),
            _ => None,
        }
    }

    // realises FR-173
    /// Yields the numbers of octets one row of the mode may present, the smallest first; none
    /// for txt, which has no rows.
    pub fn row_widths(self) -> &'static [usize] {
        match self {
            EditorMode::Txt => &[],
            EditorMode::Hex => &HEX_WIDTHS,
            EditorMode::Bin => &BIN_WIDTHS,
        }
    }

    /// Yields the number of character columns one octet takes in a row of the mode, the space
    /// that separates it from the next one included; 0 for txt.
    fn octet_pitch(self) -> usize {
        match self {
            EditorMode::Txt => 0,
            EditorMode::Hex => 3,
            EditorMode::Bin => 9,
        }
    }
}

// realises FR-169, FR-170, FR-171
/// The part of one row of hex or bin below which a *diagnostic* is marked.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowSegment {
    /// the first character column of the row text the mark lies below, counted from 0
    pub first_column: usize,
    /// the number of character columns the mark lies below
    pub columns: usize,
    /// the index of the *diagnostic* among the marks
    pub mark: usize,
}

// realises FR-156
/// Yields the position of the *mode switch* that no selection of the user fixes: `chosen` where
/// it names a mode, and otherwise txt where the document `is_text` and hex where it is not.
pub fn mode_selected(chosen: Option<EditorMode>, is_text: bool) -> EditorMode {
    match (chosen, is_text) {
        (Some(mode), _) => mode,
        (None, true) => EditorMode::Txt,
        (None, false) => EditorMode::Hex,
    }
}

// realises FR-156, FR-167
/// Yields whether `octets` are valid UTF-8.
pub fn is_utf8(octets: &[u8]) -> bool {
    std::str::from_utf8(octets).is_ok()
}

// realises FR-157
/// Yields `octets` read as UTF-8, each octet that belongs to no character of valid UTF-8 as one
/// replacement character U+FFFD.
pub fn text_of_octets(octets: &[u8]) -> String {
    let mut text = String::with_capacity(octets.len());
    let mut rest = octets;
    loop {
        match std::str::from_utf8(rest) {
            Ok(valid) => {
                text.push_str(valid);
                return text;
            }
            Err(error) => {
                let (valid, invalid) = rest.split_at(error.valid_up_to());
                text.push_str(&String::from_utf8_lossy(valid));
                let skipped = error.error_len().unwrap_or(invalid.len());
                text.extend(std::iter::repeat_n(REPLACEMENT, skipped));
                rest = &invalid[skipped..];
            }
        }
    }
}

// realises FR-158, FR-159
/// Yields the number of rows `octet_count` octets take, `octets_per_row` per row; 0 where
/// `octets_per_row` is 0, as for txt.
pub fn row_count(octet_count: usize, octets_per_row: usize) -> usize {
    match octets_per_row {
        0 => 0,
        per_row => octet_count.div_ceil(per_row),
    }
}

// realises FR-173
/// Yields the number of character columns a row of `octets_per_row` octets of `mode` takes: its
/// digits and the spaces between its octets; 0 for txt.
pub fn row_columns(mode: EditorMode, octets_per_row: usize) -> usize {
    (mode.octet_pitch() * octets_per_row).saturating_sub(1)
}

// realises FR-173
/// Yields the number of octets per row of `mode` for a document of `octet_count` octets in a
/// view `columns` character columns wide: the largest of [`EditorMode::row_widths`] with which
/// the row label of the last row, one column, the digits of a full row and one column fit, and
/// the smallest where none fits; 0 for txt.
pub fn octets_per_row_fitting(mode: EditorMode, octet_count: usize, columns: usize) -> usize {
    let widths = mode.row_widths();
    let fits = |per_row: usize| {
        let last_row = row_count(octet_count, per_row).saturating_sub(1);
        let label = row_label(last_row, per_row, octet_count).len();
        // the label, one column, the digits of a full row and one column
        label + 1 + row_columns(mode, per_row) < columns
    };
    widths
        .iter()
        .rev()
        .copied()
        .find(|per_row| fits(*per_row))
        .or_else(|| widths.first().copied())
        .unwrap_or(0)
}

// realises FR-174
/// Yields the row of `octets_per_row_after` octets per row that holds the first octet of the
/// row `row` of `octets_per_row_before` octets per row; 0 where either is 0.
pub fn row_keeping(row: usize, octets_per_row_before: usize, octets_per_row_after: usize) -> usize {
    match octets_per_row_after {
        0 => 0,
        after => row.saturating_mul(octets_per_row_before) / after,
    }
}

// realises FR-160, FR-175
/// Yields the row label of the row `row`, counted from 0, of `octets_per_row` octets per row in
/// a document of `octet_count` octets; empty where `octets_per_row` is 0.
///
/// * Up to 16 octets per row: the hexadecimal digits of the number of the first octet of the
///   row without the last one, followed by `x`.
/// * More than 16 octets per row: the hexadecimal digits of the number of the first octet,
///   padded with zeros to the digits of the first octet of the last row, at least three.
pub fn row_label(row: usize, octets_per_row: usize, octet_count: usize) -> String {
    let first = row.saturating_mul(octets_per_row);
    match octets_per_row {
        0 => String::new(),
        1..=LABEL_OCTETS => format!("{:X}x", first / LABEL_OCTETS),
        _ => {
            let last = row_count(octet_count, octets_per_row)
                .saturating_sub(1)
                .saturating_mul(octets_per_row)
                .max(first);
            let digits = format!("{last:X}").len().max(FULL_LABEL_DIGITS);
            format!("{first:0digits$X}")
        }
    }
}

// realises FR-161, FR-176
/// Yields the column header of rows of `octets_per_row` octets of `mode`, one entry above each
/// octet, as wide as a full row; empty for txt.
///
/// * Up to 16 octets per row: `x` followed by the last hexadecimal digits of the numbers of the
///   octets below the entry, separated by `/`: `x0/4/8/C` for 4 octets per row, `x0` for 16.
/// * More than 16 octets per row: the distance of the octet from the start of the row in two
///   hexadecimal digits, `00` to `1F` for 32.
/// * An entry is padded with spaces to the width of an octet.
pub fn column_header(mode: EditorMode, octets_per_row: usize) -> String {
    if mode == EditorMode::Txt {
        return String::new();
    }
    let width = mode.octet_pitch().saturating_sub(1);
    (0..octets_per_row)
        .map(|column| {
            let entry = match octets_per_row {
                1..=LABEL_OCTETS => {
                    let digits: Vec<String> = (column..LABEL_OCTETS)
                        .step_by(octets_per_row)
                        .map(|digit| format!("{digit:X}"))
                        .collect();
                    format!("x{}", digits.join("/"))
                }
                _ => format!("{column:02X}"),
            };
            format!("{entry:width$}")
        })
        .collect::<Vec<String>>()
        .join(" ")
}

// realises FR-158, FR-159
/// Yields the text of the row `row`, counted from 0, of `octets_per_row` octets of `mode`: its
/// octets as two hexadecimal digits each in hex and as eight binary digits each in bin,
/// separated by one space.
///
/// * A row beyond the last one, and every row of txt, is empty.
pub fn row_text(octets: &[u8], row: usize, mode: EditorMode, octets_per_row: usize) -> String {
    if mode == EditorMode::Txt {
        return String::new();
    }
    let start = row.saturating_mul(octets_per_row).min(octets.len());
    let end = start.saturating_add(octets_per_row).min(octets.len());
    octets[start..end]
        .iter()
        .map(|octet| match mode {
            EditorMode::Txt => String::new(),
            EditorMode::Hex => format!("{octet:02X}"),
            EditorMode::Bin => format!("{octet:08b}"),
        })
        .collect::<Vec<String>>()
        .join(" ")
}

// realises FR-169, FR-170, FR-171
/// Yields the segments of the row `row` of `mode` below which the *diagnostics* are marked, in
/// the order of the marks.
///
/// * In hex a *diagnostic* is marked below every octet that holds a bit of its range, in bin
///   below the bits of its range.
/// * An empty range is marked as the one bit at its start, and a range at the end of the octets
///   as the last bit; where there is no octet, nothing is marked.
///
/// # Arguments
/// * `starts`, `ends` - per *diagnostic*, the *offset* of the start and of the end of its range,
///   in bits
/// * `octet_count` - the number of octets of the document
/// * `row` - the row, counted from 0
/// * `mode` - hex or bin; txt has no rows and yields no segment
/// * `octets_per_row` - the number of octets of a row
pub fn row_segments(
    starts: &[u64],
    ends: &[u64],
    octet_count: usize,
    row: usize,
    mode: EditorMode,
    octets_per_row: usize,
) -> Vec<RowSegment> {
    let per_row = octets_per_row;
    let bit_count = 8 * octet_count as u64;
    if per_row == 0 || bit_count == 0 || mode == EditorMode::Txt {
        return Vec::new();
    }
    let row_start = 8 * (row.saturating_mul(per_row)) as u64;
    let row_end = row_start + 8 * per_row as u64;
    starts
        .iter()
        .zip(ends)
        .enumerate()
        .filter_map(|(mark, (start, end))| {
            let start = (*start).min(bit_count - 1);
            let end = (*end).clamp(start + 1, bit_count);
            let (first, last) = (start.max(row_start), (end - 1).min(row_end - 1));
            if first > last {
                return None;
            }
            let (first, last) = ((first - row_start) as usize, (last - row_start) as usize);
            let pitch = mode.octet_pitch();
            let (first_column, last_column) = match mode {
                EditorMode::Txt => return None,
                EditorMode::Hex => (pitch * (first / 8), pitch * (last / 8) + 1),
                EditorMode::Bin => (
                    pitch * (first / 8) + first % 8,
                    pitch * (last / 8) + last % 8,
                ),
            };
            Some(RowSegment {
                first_column,
                columns: last_column - first_column + 1,
                mark,
            })
        })
        .collect()
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : Editing::mode_selected, Editing::octets_per_row, Editing::row_keeping,
 *                    Editing::row_label, Editing::column_header, InputGroup::row_count,
 *                    InputGroup::row_text, InputGroup::row_marks, OutputGroup::row_count,
 *                    OutputGroup::row_text */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indices_map_both_ways() {
        let modes = [EditorMode::Txt, EditorMode::Hex, EditorMode::Bin];
        let mapped: Vec<Option<EditorMode>> = modes
            .iter()
            .map(|mode| EditorMode::of_index(mode.index()))
            .chain(std::iter::once(EditorMode::of_index(3)))
            .collect();
        assert_eq!(
            mapped,
            vec![
                Some(EditorMode::Txt),
                Some(EditorMode::Hex),
                Some(EditorMode::Bin),
                None
            ]
        );
    }

    #[test]
    fn the_default_mode_is_txt() {
        assert_eq!(EditorMode::default(), EditorMode::Txt);
    }

    #[test]
    fn without_a_selection_a_text_selects_txt_and_other_octets_select_hex() {
        assert_eq!(
            (mode_selected(None, true), mode_selected(None, false)),
            (EditorMode::Txt, EditorMode::Hex)
        );
    }

    #[test]
    fn a_selection_of_the_user_holds_whatever_the_octets_are() {
        assert_eq!(
            (
                mode_selected(Some(EditorMode::Bin), true),
                mode_selected(Some(EditorMode::Txt), false)
            ),
            (EditorMode::Bin, EditorMode::Txt)
        );
    }

    #[test]
    fn valid_utf8_is_told_from_other_octets() {
        assert_eq!(
            (
                is_utf8(b""),
                is_utf8("ä€".as_bytes()),
                is_utf8(&[b'a', 0xff])
            ),
            (true, true, false)
        );
    }

    #[test]
    fn valid_utf8_is_read_as_its_text() {
        assert_eq!(
            text_of_octets("Dear all, ä€\n".as_bytes()),
            "Dear all, ä€\n"
        );
    }

    #[test]
    fn an_octet_that_is_no_character_is_read_as_one_replacement_character() {
        assert_eq!(text_of_octets(&[b'a', 0xff, b'b']), "a\u{FFFD}b");
    }

    #[test]
    fn every_octet_of_a_truncated_character_is_read_as_a_replacement_character_of_its_own() {
        // E2 82 begins the three octets of U+20AC and is followed by no third one
        assert_eq!(
            text_of_octets(&[0xe2, 0x82, b'a', 0xe2, 0x82]),
            "\u{FFFD}\u{FFFD}a\u{FFFD}\u{FFFD}"
        );
    }

    #[test]
    fn no_octets_are_read_as_the_empty_text() {
        assert_eq!(text_of_octets(&[]), "");
    }

    #[test]
    fn rows_are_counted_by_the_octets_per_row_and_none_for_no_octets_per_row() {
        let counts: Vec<usize> = [0, 1, 4, 5, 16, 17, 33]
            .iter()
            .flat_map(|count| [4, 16, 32, 0].map(|per_row| row_count(*count, per_row)))
            .collect();
        assert_eq!(
            counts,
            vec![
                0, 0, 0, 0, 1, 1, 1, 0, 1, 1, 1, 0, 2, 1, 1, 0, 4, 1, 1, 0, 5, 2, 1, 0, 9, 3, 2, 0
            ]
        );
    }

    #[test]
    fn the_widths_of_a_row_are_16_32_64_in_hex_and_4_8_16_32_in_bin() {
        assert_eq!(
            (
                EditorMode::Txt.row_widths(),
                EditorMode::Hex.row_widths(),
                EditorMode::Bin.row_widths()
            ),
            (&[][..], &[16, 32, 64][..], &[4, 8, 16, 32][..])
        );
    }

    #[test]
    fn a_row_takes_three_columns_per_octet_in_hex_and_nine_in_bin_less_the_last_space() {
        assert_eq!(
            (
                row_columns(EditorMode::Hex, 16),
                row_columns(EditorMode::Bin, 4),
                row_columns(EditorMode::Txt, 16)
            ),
            (47, 35, 0)
        );
    }

    #[test]
    fn the_largest_width_whose_row_and_label_fit_is_chosen() {
        // 4096 octets: the label of the last row of 16 octets is 'FFx', of 32 and 64 'FE0'
        // and 'FC0'; a row takes its label, one column, its digits and one column
        let chosen: Vec<usize> = [51, 52, 99, 100, 195, 196]
            .iter()
            .map(|columns| octets_per_row_fitting(EditorMode::Hex, 4096, *columns))
            .collect();
        assert_eq!(chosen, vec![16, 16, 16, 32, 32, 64]);
    }

    #[test]
    fn the_smallest_width_is_chosen_where_none_fits() {
        assert_eq!(
            (
                octets_per_row_fitting(EditorMode::Hex, 100, 0),
                octets_per_row_fitting(EditorMode::Bin, 100, 10),
                octets_per_row_fitting(EditorMode::Txt, 100, 1000)
            ),
            (16, 4, 0)
        );
    }

    #[test]
    fn bin_steps_through_4_8_16_and_32_octets_per_row() {
        // 64 octets: the labels are '3x' up to 16 octets per row and '020' for 32
        let chosen: Vec<usize> = [38, 39, 74, 75, 146, 147, 291, 292]
            .iter()
            .map(|columns| octets_per_row_fitting(EditorMode::Bin, 64, *columns))
            .collect();
        assert_eq!(chosen, vec![4, 4, 4, 8, 8, 16, 16, 32]);
    }

    #[test]
    fn the_row_after_a_change_of_width_holds_the_first_octet_of_the_row_before() {
        assert_eq!(
            (
                row_keeping(3, 16, 32),
                row_keeping(3, 32, 16),
                row_keeping(5, 4, 64),
                row_keeping(5, 16, 0)
            ),
            (1, 6, 0, 0)
        );
    }

    #[test]
    fn the_label_of_a_row_of_16_octets_is_the_number_of_its_first_octet_without_its_last_digit() {
        let labels: Vec<String> = [0, 1, 15, 16, 256]
            .iter()
            .map(|row| row_label(*row, 16, 8192))
            .collect();
        assert_eq!(labels, vec!["0x", "1x", "Fx", "10x", "100x"]);
    }

    #[test]
    fn rows_of_4_and_8_octets_share_a_label_by_four_and_by_two() {
        let labels: Vec<String> = (0..5)
            .flat_map(|row| [row_label(row, 4, 64), row_label(row, 8, 64)])
            .collect();
        assert_eq!(
            labels,
            vec!["0x", "0x", "0x", "0x", "0x", "1x", "0x", "1x", "1x", "2x"]
        );
    }

    #[test]
    fn the_label_of_a_row_of_32_or_64_octets_is_the_full_number_of_its_first_octet() {
        assert_eq!(
            (
                row_label(1, 32, 100),
                row_label(2, 64, 200),
                row_label(3, 64, 70000)
            ),
            ("020".to_owned(), "080".to_owned(), "000C0".to_owned())
        );
    }

    #[test]
    fn no_octets_per_row_have_no_label() {
        assert_eq!(row_label(3, 0, 100), "");
    }

    #[test]
    fn the_column_headers_name_the_last_digits_of_the_octets_below_them() {
        assert_eq!(
            (
                column_header(EditorMode::Hex, 16),
                column_header(EditorMode::Bin, 4),
                column_header(EditorMode::Bin, 8)
            ),
            (
                "x0 x1 x2 x3 x4 x5 x6 x7 x8 x9 xA xB xC xD xE xF".to_owned(),
                "x0/4/8/C x1/5/9/D x2/6/A/E x3/7/B/F".to_owned(),
                "x0/8     x1/9     x2/A     x3/B     x4/C     x5/D     x6/E     x7/F    "
                    .to_owned()
            )
        );
    }

    #[test]
    fn the_column_header_of_more_than_16_octets_names_the_distance_from_the_row_start() {
        let header = column_header(EditorMode::Hex, 32);
        assert_eq!(
            (&header[..8], &header[header.len() - 5..]),
            ("00 01 02", "1E 1F")
        );
    }

    #[test]
    fn txt_and_no_octets_per_row_have_no_column_header() {
        assert_eq!(
            (
                column_header(EditorMode::Txt, 16),
                column_header(EditorMode::Hex, 0)
            ),
            (String::new(), String::new())
        );
    }

    #[test]
    fn every_column_header_is_as_wide_as_a_full_row() {
        let octets = [0u8; 64];
        let widths: Vec<(usize, usize)> = [EditorMode::Hex, EditorMode::Bin]
            .iter()
            .flat_map(|mode| {
                mode.row_widths().iter().map(|per_row| {
                    (
                        column_header(*mode, *per_row).len(),
                        row_text(&octets, 0, *mode, *per_row).len(),
                    )
                })
            })
            .collect();
        assert!(
            widths.iter().all(|(header, row)| header == row),
            "{widths:?}"
        );
    }

    #[test]
    fn a_row_of_hex_presents_its_octets_as_two_upper_case_digits_each() {
        let octets: Vec<u8> = (0..18).map(|octet| octet * 15).collect();
        assert_eq!(
            (
                row_text(&octets, 0, EditorMode::Hex, 16),
                row_text(&octets, 1, EditorMode::Hex, 16)
            ),
            (
                "00 0F 1E 2D 3C 4B 5A 69 78 87 96 A5 B4 C3 D2 E1".to_owned(),
                "F0 FF".to_owned()
            )
        );
    }

    #[test]
    fn a_row_of_bin_presents_its_octets_as_eight_digits_each_the_first_the_most_significant() {
        let octets = [0x0a, 0x0d, 0x44, 0x65, 0x80];
        assert_eq!(
            (
                row_text(&octets, 0, EditorMode::Bin, 4),
                row_text(&octets, 1, EditorMode::Bin, 4)
            ),
            (
                "00001010 00001101 01000100 01100101".to_owned(),
                "10000000".to_owned()
            )
        );
    }

    #[test]
    fn a_row_of_32_octets_presents_32_octets() {
        let octets: Vec<u8> = (0..40).collect();
        assert_eq!(
            (
                row_text(&octets, 0, EditorMode::Hex, 32).len(),
                row_text(&octets, 1, EditorMode::Hex, 32)
            ),
            (95, "20 21 22 23 24 25 26 27".to_owned())
        );
    }

    #[test]
    fn a_row_beyond_the_last_one_and_a_row_of_txt_are_empty() {
        let octets = [1, 2, 3];
        assert_eq!(
            (
                row_text(&octets, 1, EditorMode::Hex, 16),
                row_text(&octets, usize::MAX, EditorMode::Bin, 4),
                row_text(&octets, 0, EditorMode::Txt, 16)
            ),
            (String::new(), String::new(), String::new())
        );
    }

    fn segment(first_column: usize, columns: usize, mark: usize) -> RowSegment {
        RowSegment {
            first_column,
            columns,
            mark,
        }
    }

    #[test]
    fn hex_marks_every_octet_that_holds_a_bit_of_the_range() {
        // the bits 12 to 19: the second half of octet 1 and the first half of octet 2
        assert_eq!(
            row_segments(&[12], &[20], 32, 0, EditorMode::Hex, 16),
            vec![segment(3, 5, 0)]
        );
    }

    #[test]
    fn bin_marks_the_bits_of_the_range_and_the_space_between_two_octets() {
        assert_eq!(
            row_segments(&[12], &[20], 32, 0, EditorMode::Bin, 4),
            vec![segment(13, 9, 0)]
        );
    }

    #[test]
    fn a_range_over_several_rows_has_a_segment_in_each_of_them() {
        // the octets 15 to 16 in hex: the last of row 0 and the first of row 1
        let rows: Vec<Vec<RowSegment>> = (0..3)
            .map(|row| row_segments(&[120], &[136], 48, row, EditorMode::Hex, 16))
            .collect();
        assert_eq!(
            rows,
            vec![vec![segment(45, 2, 0)], vec![segment(0, 2, 0)], vec![]]
        );
    }

    #[test]
    fn a_range_over_several_rows_of_bin_ends_each_row_at_its_last_bit() {
        // the bits 30 to 33: the last two of row 0 and the first two of row 1
        let rows: Vec<Vec<RowSegment>> = (0..2)
            .map(|row| row_segments(&[30], &[34], 8, row, EditorMode::Bin, 4))
            .collect();
        assert_eq!(rows, vec![vec![segment(33, 2, 0)], vec![segment(0, 2, 0)]]);
    }

    #[test]
    fn an_empty_range_is_marked_as_the_bit_at_its_start() {
        assert_eq!(
            (
                row_segments(&[9], &[9], 4, 0, EditorMode::Bin, 4),
                row_segments(&[9], &[9], 4, 0, EditorMode::Hex, 16)
            ),
            (vec![segment(10, 1, 0)], vec![segment(3, 2, 0)])
        );
    }

    #[test]
    fn a_range_at_the_end_of_the_octets_is_marked_as_the_last_bit() {
        assert_eq!(
            (
                row_segments(&[16], &[16], 2, 0, EditorMode::Bin, 4),
                row_segments(&[99], &[120], 2, 0, EditorMode::Hex, 16)
            ),
            (vec![segment(16, 1, 0)], vec![segment(3, 2, 0)])
        );
    }

    #[test]
    fn an_end_before_the_start_is_taken_as_an_empty_range() {
        assert_eq!(
            row_segments(&[8], &[3], 2, 0, EditorMode::Bin, 4),
            vec![segment(9, 1, 0)]
        );
    }

    #[test]
    fn the_segments_of_a_row_name_their_marks_in_the_order_of_the_marks() {
        assert_eq!(
            row_segments(&[0, 200, 8], &[8, 208, 16], 32, 0, EditorMode::Hex, 16),
            vec![segment(0, 2, 0), segment(3, 2, 2)]
        );
    }

    /// One megabyte of octets of which every third is no character of UTF-8.
    fn megabyte() -> Vec<u8> {
        (0..1_000_000u32)
            .map(|index| if index % 3 == 0 { 0xff } else { b'a' })
            .collect()
    }

    #[test]
    fn one_megabyte_is_read_as_text_within_one_second() {
        // NFR-002, NFR-003
        let octets = megabyte();
        let started = std::time::Instant::now();
        let text = text_of_octets(&octets);
        let elapsed = started.elapsed();
        assert!(
            text.chars().count() == octets.len() && elapsed < std::time::Duration::from_secs(1),
            "{elapsed:?}"
        );
    }

    #[test]
    fn every_row_of_one_megabyte_is_derived_within_one_second() {
        // NFR-009: a view derives the rows it shows alone, which are far fewer
        let octets = megabyte();
        let starts = [0, 4_000_000, 7_999_999];
        let ends = [9, 4_000_100, 8_000_000];
        let started = std::time::Instant::now();
        let rows = row_count(octets.len(), 4);
        let digits: usize = (0..rows)
            .map(|row| {
                row_text(&octets, row, EditorMode::Bin, 4).len()
                    + row_segments(&starts, &ends, octets.len(), row, EditorMode::Bin, 4).len()
            })
            .sum();
        let elapsed = started.elapsed();
        assert!(
            rows == 250_000 && digits > 0 && elapsed < std::time::Duration::from_secs(1),
            "{elapsed:?}"
        );
    }

    #[test]
    fn a_range_over_the_second_half_of_a_row_of_32_octets_is_marked_there() {
        // the octets 20 to 21 of a row of 32 octets in hex
        assert_eq!(
            row_segments(&[160], &[176], 64, 0, EditorMode::Hex, 32),
            vec![segment(60, 5, 0)]
        );
    }

    #[test]
    fn no_octets_and_txt_have_no_segment() {
        assert_eq!(
            (
                row_segments(&[0], &[8], 0, 0, EditorMode::Hex, 16),
                row_segments(&[0], &[8], 4, 0, EditorMode::Txt, 16)
            ),
            (vec![], vec![])
        );
    }
}
