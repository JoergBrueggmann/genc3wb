//! The *editing* of a code editor: the spans of lines, the occurrences of a selection, the
//! indentation of a line, and the scrolling that shows the cursor.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

/// The most occurrences [`occurrences`] yields.
const MAX_OCCURRENCES: usize = 1000;

// realises FR-136
/// Yields the start and the end offset of the line `line` of `text`, counted from 0.
///
/// * Offsets are counted in UTF-16 code units from 0, as the positions of a QML `TextArea`
///   and the offsets of [`crate::core::api_message::Marks`] are.
/// * The end lies after the line feed of the line, so that the span holds its line break; a
///   carriage return before the line feed belongs to the line.
/// * The last line, which has no line break, ends at the end of `text`.
/// * A line beyond the last one yields the span of the last line; an empty `text` has one
///   empty line, `(0, 0)`.
pub fn line_span(text: &str, line: usize) -> (usize, usize) {
    let mut start = 0;
    let mut current = 0;
    let mut offset = 0;
    for character in text.chars() {
        offset += character.len_utf16();
        if character == '\n' {
            if current == line {
                return (start, offset);
            }
            current += 1;
            start = offset;
        }
    }
    (start, offset)
}

// realises FR-137
/// Yields the span from the start of the line `from` to the end of the line `to` of `text`, in
/// either direction: the start of the smaller and the end of the greater line, as
/// [`line_span`] yields them.
pub fn lines_span(text: &str, from: usize, to: usize) -> (usize, usize) {
    let (first, last) = (from.min(to), from.max(to));
    (line_span(text, first).0, line_span(text, last).1)
}

// realises FR-138
/// Yields the start and the end offset of every other occurrence of the selection from `start`
/// to `end` in `text`, exact and case-sensitive, left to right.
///
/// * Offsets are counted in UTF-16 code units from 0, as [`line_span`] counts them.
/// * The occurrences overlap neither each other nor the selection: they are sought in the text
///   before the selection and in the text after it.
/// * None is yielded where the selection holds fewer than two characters, or a line feed or a
///   carriage return; a surrogate pair counts as one character.
/// * At most 1000 are yielded, the leftmost ones.
/// * Offsets beyond the end of `text` are cut to its end; `start` and `end` may be given in
///   either order.
pub fn occurrences(text: &str, start: usize, end: usize) -> Vec<(usize, usize)> {
    let units: Vec<u16> = text.encode_utf16().collect();
    let (start, end) = (start.min(end), start.max(end));
    let (start, end) = (start.min(units.len()), end.min(units.len()));
    let selection = &units[start..end];
    let has_line_break = selection
        .iter()
        .any(|unit| *unit == u16::from(b'\n') || *unit == u16::from(b'\r'));
    if has_line_break || char::decode_utf16(selection.iter().copied()).count() < 2 {
        return Vec::new();
    }
    let mut found = occurrences_in(&units[..start], selection, 0);
    found.extend(occurrences_in(&units[end..], selection, end));
    found.truncate(MAX_OCCURRENCES);
    found
}

/// Yields the non-overlapping occurrences of `needle` in `haystack`, left to right, each as its
/// start and end offset plus `base`; at most [`MAX_OCCURRENCES`].
fn occurrences_in(haystack: &[u16], needle: &[u16], base: usize) -> Vec<(usize, usize)> {
    let mut found = Vec::new();
    let mut position = 0;
    while position + needle.len() <= haystack.len() && found.len() < MAX_OCCURRENCES {
        if haystack[position..position + needle.len()] == *needle {
            found.push((base + position, base + position + needle.len()));
            position += needle.len();
        } else {
            position += 1;
        }
    }
    found
}

// realises FR-144
/// Yields the leading tabs and spaces of the line of `text` that holds the offset `cursor`,
/// exactly as they are.
///
/// * `cursor` is counted in UTF-16 code units from 0, as [`line_span`] counts offsets.
/// * The indentation is the whole of it, wherever in the line `cursor` lies.
/// * An offset at the start of a line belongs to that line; an offset beyond the end of `text`
///   to the last line.
pub fn indentation_of_line(text: &str, cursor: usize) -> String {
    let mut line_start = 0;
    let mut offset = 0;
    for (index, character) in text.char_indices() {
        if offset >= cursor {
            break;
        }
        offset += character.len_utf16();
        if character == '\n' {
            line_start = index + 1;
        }
    }
    text[line_start..]
        .chars()
        .take_while(|character| *character == ' ' || *character == '\t')
        .collect()
}

// realises FR-145
/// Yields the offset of a view that shows the span from `start` to `end` of its content, moved
/// as little as possible from `offset`.
///
/// * The view shows the content from its offset to its offset plus `view`.
/// * Where the span is larger than the view, its start is shown.
/// * The offset yielded lies from 0 to `content` minus `view`, and is 0 where the content is
///   smaller than the view.
///
/// # Arguments
/// * `offset` - the offset of the view before, along one axis
/// * `view` - the size of the view along that axis
/// * `content` - the size of the content along that axis
/// * `start`, `end` - the span of the content to show, its margin included
pub fn view_offset_showing(offset: f64, view: f64, content: f64, start: f64, end: f64) -> f64 {
    let mut result = offset;
    if end > result + view {
        result = end - view;
    }
    if start < result {
        result = start;
    }
    result.min(content - view).max(0.0)
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : Editing::line_span, Editing::lines_span, Editing::occurrences,
 *                    Editing::indentation_of_line, Editing::view_offset_showing */
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn line_span_holds_the_line_break() {
        // FR-136
        assert_eq!(line_span("ab\ncd\nef", 1), (3, 6));
    }

    #[test]
    fn first_line_starts_at_zero() {
        // FR-136
        assert_eq!(line_span("ab\ncd", 0), (0, 3));
    }

    #[test]
    fn last_line_without_break_ends_at_the_end_of_the_text() {
        // FR-136
        assert_eq!(line_span("ab\ncd", 1), (3, 5));
    }

    #[test]
    fn empty_text_has_one_empty_line() {
        // FR-136
        assert_eq!(line_span("", 0), (0, 0));
    }

    #[test]
    fn line_after_a_final_break_is_empty() {
        // FR-136
        assert_eq!(line_span("ab\n", 1), (3, 3));
    }

    #[test]
    fn line_beyond_the_last_yields_the_last_line() {
        // FR-137: the pointer below the last line
        assert_eq!(line_span("ab\ncd", 7), (3, 5));
    }

    #[test]
    fn carriage_return_and_line_feed_belong_to_the_line() {
        // FR-136
        assert_eq!(line_span("ab\r\ncd", 0), (0, 4));
    }

    #[test]
    fn line_span_counts_utf16_code_units() {
        // FR-136: ä is one unit, 😀 a surrogate pair of two
        assert_eq!(line_span("ä😀\nx", 0), (0, 4));
        assert_eq!(line_span("ä😀\nx", 1), (4, 5));
    }

    #[test]
    fn lines_span_downwards_covers_every_line() {
        // FR-137
        assert_eq!(lines_span("ab\ncd\nef\ngh", 1, 2), (3, 9));
    }

    #[test]
    fn lines_span_upwards_equals_downwards() {
        // FR-137
        assert_eq!(lines_span("ab\ncd\nef\ngh", 2, 1), (3, 9));
    }

    #[test]
    fn lines_span_of_one_line_is_its_line_span() {
        // FR-136, FR-137
        assert_eq!(lines_span("ab\ncd", 1, 1), line_span("ab\ncd", 1));
    }

    #[test]
    fn lines_span_to_the_last_line_ends_at_the_end_of_the_text() {
        // FR-137
        assert_eq!(lines_span("ab\ncd", 0, 9), (0, 5));
    }

    #[test]
    fn occurrences_are_the_other_ones_left_to_right() {
        // FR-138
        assert_eq!(occurrences("ab x ab y ab", 5, 7), vec![(0, 2), (10, 12)]);
    }

    #[test]
    fn occurrences_are_case_sensitive() {
        // FR-138
        assert_eq!(occurrences("ab AB aB ab", 0, 2), vec![(9, 11)]);
    }

    #[test]
    fn occurrences_overlap_neither_each_other_nor_the_selection() {
        // FR-138
        assert_eq!(occurrences("aaaaaaa", 3, 5), vec![(0, 2), (5, 7)]);
    }

    #[test]
    fn selection_of_one_character_has_no_occurrences() {
        // FR-138
        assert_eq!(occurrences("a a a", 0, 1), Vec::new());
    }

    #[test]
    fn empty_selection_has_no_occurrences() {
        // FR-138
        assert_eq!(occurrences("ab ab", 1, 1), Vec::new());
    }

    #[test]
    fn selection_with_a_line_break_has_no_occurrences() {
        // FR-138
        assert_eq!(occurrences("a\nb a\nb", 0, 3), Vec::new());
        assert_eq!(occurrences("a\r\nb a\r\nb", 0, 2), Vec::new());
    }

    #[test]
    fn surrogate_pair_alone_is_one_character() {
        // FR-138: 😀 is two code units but one character
        assert_eq!(occurrences("😀 😀", 0, 2), Vec::new());
    }

    #[test]
    fn occurrences_count_utf16_code_units() {
        // FR-138
        assert_eq!(occurrences("ä😀 x ä😀", 0, 3), vec![(6, 9)]);
    }

    #[test]
    fn empty_text_has_no_occurrences() {
        // FR-138
        assert_eq!(occurrences("", 0, 2), Vec::new());
    }

    #[test]
    fn offsets_beyond_the_end_are_cut() {
        // FR-138
        assert_eq!(occurrences("xy xy", 3, 99), vec![(0, 2)]);
    }

    #[test]
    fn selection_in_either_order_yields_the_same_occurrences() {
        // FR-138
        assert_eq!(occurrences("xy xy", 5, 3), occurrences("xy xy", 3, 5));
    }

    #[test]
    fn occurrences_are_limited_to_the_leftmost_thousand() {
        // FR-138
        let text = "ab ".repeat(1500);
        let found = occurrences(&text, 0, 2);
        assert_eq!(
            (found.len(), found[0], found[999]),
            (1000, (3, 5), (3000, 3002))
        );
    }

    #[test]
    fn indentation_is_the_leading_tabs_and_spaces() {
        // FR-144
        assert_eq!(indentation_of_line("a\n\t  \tb c\nd", 8), "\t  \t");
    }

    #[test]
    fn indentation_is_whole_with_the_cursor_inside_it() {
        // FR-144
        assert_eq!(indentation_of_line("    x", 2), "    ");
    }

    #[test]
    fn cursor_at_the_start_of_a_line_belongs_to_that_line() {
        // FR-144
        assert_eq!(indentation_of_line("  a\n\tb", 4), "\t");
    }

    #[test]
    fn cursor_before_the_line_break_belongs_to_its_line() {
        // FR-144
        assert_eq!(indentation_of_line("  a\n\tb", 3), "  ");
    }

    #[test]
    fn line_without_indentation_yields_none() {
        // FR-144
        assert_eq!(indentation_of_line("a\nb", 3), "");
    }

    #[test]
    fn empty_text_has_no_indentation() {
        // FR-144
        assert_eq!(indentation_of_line("", 0), "");
    }

    #[test]
    fn cursor_beyond_the_end_belongs_to_the_last_line() {
        // FR-144
        assert_eq!(indentation_of_line("a\n  b", 99), "  ");
    }

    #[test]
    fn indentation_of_a_crlf_line_is_found() {
        // FR-144
        assert_eq!(indentation_of_line("a\r\n\tb", 4), "\t");
    }

    #[test]
    fn indentation_is_found_after_characters_beyond_ascii() {
        // FR-144: 😀 is two code units, so the second line starts at offset 3
        assert_eq!(indentation_of_line("😀\n  ä", 5), "  ");
    }

    #[test]
    fn indentation_stops_at_other_whitespace() {
        // FR-144: only tabs and spaces are indentation
        assert_eq!(indentation_of_line(" \u{a0}x", 3), " ");
    }

    #[test]
    fn visible_span_leaves_the_view_where_it_is() {
        // FR-145
        assert_eq!(view_offset_showing(10.0, 100.0, 500.0, 20.0, 40.0), 10.0);
    }

    #[test]
    fn span_after_the_view_moves_it_forward() {
        // FR-145
        assert_eq!(view_offset_showing(10.0, 100.0, 500.0, 120.0, 150.0), 50.0);
    }

    #[test]
    fn span_before_the_view_moves_it_back() {
        // FR-145
        assert_eq!(view_offset_showing(100.0, 100.0, 500.0, 60.0, 90.0), 60.0);
    }

    #[test]
    fn span_larger_than_the_view_shows_its_start() {
        // FR-145
        assert_eq!(view_offset_showing(0.0, 100.0, 500.0, 200.0, 350.0), 200.0);
    }

    #[test]
    fn view_offset_stays_within_the_content() {
        // FR-145: the margin beyond the content is cut
        assert_eq!(view_offset_showing(0.0, 100.0, 500.0, 480.0, 520.0), 400.0);
        assert_eq!(view_offset_showing(50.0, 100.0, 500.0, -10.0, 20.0), 0.0);
    }

    #[test]
    fn content_smaller_than_the_view_yields_zero() {
        // FR-145
        assert_eq!(view_offset_showing(30.0, 100.0, 80.0, 10.0, 20.0), 0.0);
    }
}
