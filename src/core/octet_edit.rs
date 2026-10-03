//! The editing of octets digit by digit: the bits of a document with its placeholders, the edits
//! with undo and redo, the digits as text, and the cursor of hex and bin.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

use crate::core::octet_view::EditorMode;

/// The most steps the *edit history* keeps for undo.
const MAX_STEPS: usize = 1000;

// realises FR-178, FR-179
/// The bits of a document: its octets, and how many of their bits are digits of the document.
///
/// * The bits of the last octet behind the last digit are the placeholders; they are 0.
/// * A bit is counted from 0, the first bit of an octet being its most significant one.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Bits {
    /// the octets, as many as the bits need
    octets: Vec<u8>,
    /// the number of bits that are digits of the document
    len: usize,
}

impl Bits {
    // initialisation
    /// Creates the bits of `octets`, every bit a digit.
    pub fn of_octets(octets: Vec<u8>) -> Bits {
        Bits {
            len: 8 * octets.len(),
            octets,
        }
    }

    // realises FR-178
    /// Creates the bits of `digits`, each a value of `unit` bits, in their order.
    ///
    /// * The bits of a digit beyond its `unit` lowest ones are passed over.
    pub fn of_digits(digits: &[u8], unit: usize) -> Bits {
        let mut writer = Writer::default();
        for digit in digits {
            writer.push_chunk(*digit, unit.min(8));
        }
        writer.bits()
    }

    // state
    /// Yields the number of bits that are digits of the document.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Yields whether the document has no digit.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    // realises FR-179
    /// Yields the octets, the placeholders being 0.
    pub fn octets(&self) -> &[u8] {
        &self.octets
    }

    // realises FR-179
    /// Yields the number of placeholder bits of the last octet.
    pub fn padding(&self) -> usize {
        8 * self.octets.len() - self.len
    }

    // realises FR-185
    /// Yields the bits with every placeholder taken as a digit 0.
    pub fn completed(&self) -> Bits {
        Bits::of_octets(self.octets.clone())
    }

    // realises FR-202
    /// Yields the digits of `unit` bits from the digit `start` to the digit `end`, exclusive,
    /// as their values; digits beyond the last one are left out.
    pub fn digits(&self, unit: usize, start: usize, end: usize) -> Vec<u8> {
        let unit = unit.clamp(1, 8);
        let last = self.len / unit;
        (start.min(last)..end.min(last))
            .map(|digit| chunk(&self.octets, digit * unit, unit))
            .collect()
    }

    /// Yields the bits from the bit `start` to the bit `end`, exclusive, cut to the bits there
    /// are.
    pub fn slice(&self, start: usize, end: usize) -> Bits {
        let end = end.min(self.len);
        let start = start.min(end);
        let mut writer = Writer::default();
        writer.push_bits(&self.octets, start, end - start);
        writer.bits()
    }

    // realises FR-180 to FR-184
    /// Yields the bits with `removed` bits from the bit `start` replaced by `inserted`.
    ///
    /// * A `start` beyond the last bit is taken as the end, and `removed` is cut to the bits
    ///   there are.
    pub fn spliced(&self, start: usize, removed: usize, inserted: &Bits) -> Bits {
        let start = start.min(self.len);
        let rest = (start + removed).min(self.len);
        let mut writer = Writer::default();
        writer.push_bits(&self.octets, 0, start);
        writer.push_bits(&inserted.octets, 0, inserted.len);
        writer.push_bits(&self.octets, rest, self.len - rest);
        writer.bits()
    }
}

/// Bits being written one after the other.
#[derive(Debug, Default)]
struct Writer {
    /// the octets written so far, the bits behind the last one written being 0
    octets: Vec<u8>,
    /// the number of bits written
    len: usize,
}

impl Writer {
    /// Appends the `count` lowest bits of `value`, at most 8.
    fn push_chunk(&mut self, value: u8, count: usize) {
        if count == 0 {
            return;
        }
        let value = u16::from(value) & ((1u16 << count) - 1);
        let offset = self.len % 8;
        let word = value << (16 - offset - count);
        if offset == 0 {
            self.octets.push(0);
        }
        if let Some(last) = self.octets.last_mut() {
            *last |= (word >> 8) as u8;
        }
        if offset + count > 8 {
            self.octets.push((word & 0xff) as u8);
        }
        self.len += count;
    }

    /// Appends `count` bits of `source` from its bit `start` on.
    fn push_bits(&mut self, source: &[u8], start: usize, count: usize) {
        if count > 0 && start.is_multiple_of(8) && self.len.is_multiple_of(8) {
            let whole = count / 8;
            self.octets
                .extend_from_slice(&source[start / 8..start / 8 + whole]);
            self.len += 8 * whole;
            self.push_chunk(chunk(source, start + 8 * whole, count % 8), count % 8);
            return;
        }
        let mut position = start;
        let end = start + count;
        while position < end {
            let step = (end - position).min(8);
            self.push_chunk(chunk(source, position, step), step);
            position += step;
        }
    }

    /// Yields the bits written.
    fn bits(self) -> Bits {
        Bits {
            octets: self.octets,
            len: self.len,
        }
    }
}

/// Yields the `count` bits of `source` from its bit `start` on as a value, at most 8; bits
/// beyond `source` are 0.
fn chunk(source: &[u8], start: usize, count: usize) -> u8 {
    if count == 0 {
        return 0;
    }
    let first = u16::from(source.get(start / 8).copied().unwrap_or(0));
    let second = u16::from(source.get(start / 8 + 1).copied().unwrap_or(0));
    let word = (first << 8) | second;
    ((word >> (16 - start % 8 - count)) & ((1u16 << count) - 1)) as u8
}

// realises FR-207, FR-208
/// One replacement of bits: `removed` from the bit `start` on replaced by `inserted`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Edit {
    /// the bit the replacement starts at
    pub start: usize,
    /// the bits that were there
    pub removed: Bits,
    /// the bits that are there after it
    pub inserted: Bits,
}

impl Edit {
    /// Yields `bits` with the replacement made.
    pub fn applied(&self, bits: &Bits) -> Bits {
        bits.spliced(self.start, self.removed.len(), &self.inserted)
    }

    // realises FR-207
    /// Yields `bits` with the replacement taken back.
    pub fn reverted(&self, bits: &Bits) -> Bits {
        bits.spliced(self.start, self.inserted.len(), &self.removed)
    }
}

// realises FR-180 to FR-184, FR-187, FR-205
/// Yields the step that replaces `removed` digits of `unit` bits from the digit `start` on by
/// `digits`: the edits of the step in their order; none where nothing is replaced by nothing.
///
/// * Where the bits end within a digit of `unit` bits, the first edit completes that digit with
///   bits 0, so that the placeholders within a begun digit are taken as 0 (FR-187).
/// * A `start` beyond the last digit is taken as the end, and `removed` is cut to the digits
///   there are: a digit typed at the end is appended, whatever the mode of typing is.
pub fn replacement(
    bits: &Bits,
    unit: usize,
    start: usize,
    removed: usize,
    digits: &[u8],
) -> Vec<Edit> {
    let unit = unit.clamp(1, 8);
    let mut step = Vec::new();
    let begun = bits.len() % unit;
    let len = if begun == 0 {
        bits.len()
    } else {
        step.push(Edit {
            start: bits.len(),
            removed: Bits::default(),
            inserted: Bits::of_digits(&vec![0; unit - begun], 1),
        });
        bits.len() + unit - begun
    };
    let count = len / unit;
    let start = start.min(count);
    let removed = removed.min(count - start);
    if removed == 0 && digits.is_empty() {
        return Vec::new();
    }
    let completed = step
        .iter()
        .fold(bits.clone(), |bits, edit| edit.applied(&bits));
    step.push(Edit {
        start: start * unit,
        removed: completed.slice(start * unit, (start + removed) * unit),
        inserted: Bits::of_digits(digits, unit),
    });
    step
}

/// Yields `bits` with every edit of `step` made, in their order.
pub fn applied(bits: &Bits, step: &[Edit]) -> Bits {
    step.iter()
        .fold(bits.clone(), |bits, edit| edit.applied(&bits))
}

// realises FR-207, FR-208, FR-209
/// The *edit history*: the steps made in hex and bin, and those taken back.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct History {
    /// the steps made, the last one last
    done: Vec<Vec<Edit>>,
    /// the steps taken back, the one taken back last last
    undone: Vec<Vec<Edit>>,
}

impl History {
    /// Records `step` as made; the steps taken back are forgotten, and the oldest step beyond
    /// the most the history keeps.
    pub fn record(&mut self, step: Vec<Edit>) {
        if step.is_empty() {
            return;
        }
        self.done.push(step);
        self.undone.clear();
        if self.done.len() > MAX_STEPS {
            self.done.remove(0);
        }
    }

    // realises FR-207
    /// Takes the last step back: yields `bits` without it and the bit behind what it restored,
    /// `None` where no step is made.
    pub fn undo(&mut self, bits: &Bits) -> Option<(Bits, usize)> {
        let step = self.done.pop()?;
        let restored = step
            .iter()
            .rev()
            .fold(bits.clone(), |bits, edit| edit.reverted(&bits));
        let cursor = step
            .last()
            .map_or(0, |edit| edit.start + edit.removed.len());
        self.undone.push(step);
        let cursor = cursor.min(restored.len());
        Some((restored, cursor))
    }

    // realises FR-208
    /// Makes the step taken back last again: yields `bits` with it and the bit behind what it
    /// inserted, `None` where no step is taken back.
    pub fn redo(&mut self, bits: &Bits) -> Option<(Bits, usize)> {
        let step = self.undone.pop()?;
        let made = applied(bits, &step);
        let cursor = step
            .last()
            .map_or(0, |edit| edit.start + edit.inserted.len());
        self.done.push(step);
        let cursor = cursor.min(made.len());
        Some((made, cursor))
    }

    // realises FR-209
    /// Forgets every step.
    pub fn clear(&mut self) {
        self.done.clear();
        self.undone.clear();
    }

    /// Yields whether a step can be taken back, and whether one can be made again.
    pub fn can_undo(&self) -> bool {
        !self.done.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.undone.is_empty()
    }
}

// realises FR-186, FR-203
/// Yields the digits `text` holds in `mode`, hexadecimal digits of either case in hex and the
/// digits 0 and 1 in bin, white space passed over; `None` where `text` holds another character,
/// and for txt.
pub fn digits_of_text(text: &str, mode: EditorMode) -> Option<Vec<u8>> {
    let radix = match mode {
        EditorMode::Txt => return None,
        EditorMode::Hex => 16,
        EditorMode::Bin => 2,
    };
    text.chars()
        .filter(|character| !character.is_whitespace())
        .map(|character| character.to_digit(radix).map(|digit| digit as u8))
        .collect()
}

// realises FR-202
/// Yields `digits` as text in `mode`: hexadecimal digits in upper case in hex, the digits 0 and
/// 1 in bin; empty for txt.
pub fn text_of_digits(digits: &[u8], mode: EditorMode) -> String {
    let radix = match mode {
        EditorMode::Txt => return String::new(),
        EditorMode::Hex => 16,
        EditorMode::Bin => 2,
    };
    digits
        .iter()
        .filter_map(|digit| char::from_digit(u32::from(*digit) % radix, radix))
        .map(|character| character.to_ascii_uppercase())
        .collect()
}

// realises FR-181, FR-182, FR-205
/// Yields the digits typing or pasting `count` digits replaces: the first one and their number.
///
/// * A selection, from `anchor` to `cursor`, is replaced as a whole.
/// * Without a selection, nothing is replaced in the insert mode, and `count` digits from the
///   cursor on in the overwrite mode.
pub fn typing_span(
    cursor: usize,
    anchor: Option<usize>,
    count: usize,
    insert: bool,
) -> (usize, usize) {
    match anchor.filter(|anchor| *anchor != cursor) {
        Some(anchor) => (cursor.min(anchor), cursor.abs_diff(anchor)),
        None => (cursor, if insert { 0 } else { count }),
    }
}

// realises FR-183, FR-184, FR-205
/// Yields the digits Backspace, where `backward`, or Delete removes: the first one and their
/// number.
///
/// * A selection, from `anchor` to `cursor`, is removed as a whole.
/// * Without a selection, Backspace removes the digit before the cursor, none at the start,
///   and Delete the digit at the cursor.
pub fn removal_span(cursor: usize, anchor: Option<usize>, backward: bool) -> (usize, usize) {
    match (anchor.filter(|anchor| *anchor != cursor), backward) {
        (Some(anchor), _) => (cursor.min(anchor), cursor.abs_diff(anchor)),
        (None, true) => (cursor.saturating_sub(1), cursor.min(1)),
        (None, false) => (cursor, 1),
    }
}

// realises FR-188
/// A movement of the cursor of hex and bin.
///
/// * The order of the variants is the order of their indices at the bridge.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Movement {
    /// one digit toward the start
    Left,
    /// one digit toward the end
    Right,
    /// one row up
    Up,
    /// one row down
    Down,
    /// to the first digit of the row
    RowStart,
    /// to the last digit of the row
    RowEnd,
    /// by the rows of the view up
    PageUp,
    /// by the rows of the view down
    PageDown,
    /// to the first digit of the document
    Start,
    /// behind the last digit of the document
    End,
}

impl Movement {
    /// Yields the movement of an index, `None` where the index names none.
    pub fn of_index(index: usize) -> Option<Movement> {
        [
            Movement::Left,
            Movement::Right,
            Movement::Up,
            Movement::Down,
            Movement::RowStart,
            Movement::RowEnd,
            Movement::PageUp,
            Movement::PageDown,
            Movement::Start,
            Movement::End,
        ]
        .get(index)
        .copied()
    }
}

// realises FR-188
/// Yields the cursor after `movement`, as the digit it stands before.
///
/// * The cursor stands before a digit, or behind the last one, which is the digit `digits`.
/// * A movement up or down that leaves the document moves as far as there are rows; one down
///   from the last row leaves the cursor where it is.
///
/// # Arguments
/// * `cursor` - the digit the cursor stands before
/// * `per_row` - the number of digits of a row
/// * `digits` - the number of digits of the document
/// * `page_rows` - the number of rows of the view
pub fn cursor_moved(
    cursor: usize,
    movement: Movement,
    per_row: usize,
    digits: usize,
    page_rows: usize,
) -> usize {
    let cursor = cursor.min(digits);
    let per_row = per_row.max(1);
    let row_start = cursor / per_row * per_row;
    let last_row = digits / per_row;
    let down = |rows: usize| {
        let rows = rows.min(last_row - cursor / per_row);
        (cursor + rows * per_row).min(digits)
    };
    match movement {
        Movement::Left => cursor.saturating_sub(1),
        Movement::Right => (cursor + 1).min(digits),
        Movement::Up => cursor - per_row.min(row_start),
        Movement::Down => down(1),
        Movement::RowStart => row_start,
        Movement::RowEnd => (row_start + per_row - 1).min(digits),
        Movement::PageUp => cursor - page_rows.min(cursor / per_row) * per_row,
        Movement::PageDown => down(page_rows),
        Movement::Start => 0,
        Movement::End => digits,
    }
}

// realises FR-190
/// What the cursor of hex and bin stands before.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CursorKind {
    /// a digit of the document
    Digit,
    /// a placeholder
    Placeholder,
    /// nothing: the cursor stands behind the last digit of a document without a placeholder
    End,
}

impl CursorKind {
    /// Yields the index of the kind in the order of the variants.
    pub fn index(self) -> usize {
        match self {
            CursorKind::Digit => 0,
            CursorKind::Placeholder => 1,
            CursorKind::End => 2,
        }
    }
}

// realises FR-190, FR-191
/// Where the cursor of hex and bin is shown: its row, its character column in the text of the
/// row, and what it stands before.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CursorCell {
    /// the row, counted from 0
    pub row: usize,
    /// the character column in the text of the row, counted from 0
    pub column: usize,
    /// what the cursor stands before
    pub kind: CursorKind,
}

// realises FR-190, FR-191
/// Yields where the cursor standing before the digit `cursor` of `mode` is shown, in rows of
/// `octets_per_row` octets of a document of `bits` bits in `octet_count` octets.
///
/// * Behind the last digit of a full row the cursor is shown behind that row, not in a row of
///   its own.
/// * For txt, and without octets per row, the cursor is shown at the start.
pub fn cursor_cell(
    cursor: usize,
    mode: EditorMode,
    octets_per_row: usize,
    bits: usize,
    octet_count: usize,
) -> CursorCell {
    let unit = mode.digit_bits();
    if unit == 0 || octets_per_row == 0 {
        return CursorCell {
            row: 0,
            column: 0,
            kind: CursorKind::End,
        };
    }
    let bit = (cursor * unit).min(bits.div_ceil(unit) * unit);
    let octet = bit / 8;
    let digit = bit % 8 / unit;
    let pitch = mode.octet_pitch();
    let kind = match (bit < bits, bit < 8 * octet_count) {
        (true, _) => CursorKind::Digit,
        (false, true) => CursorKind::Placeholder,
        (false, false) => CursorKind::End,
    };
    if kind == CursorKind::End && octet > 0 && octet.is_multiple_of(octets_per_row) {
        return CursorCell {
            row: octet / octets_per_row - 1,
            column: pitch * octets_per_row,
            kind,
        };
    }
    CursorCell {
        row: octet / octets_per_row,
        column: pitch * (octet % octets_per_row) + digit,
        kind,
    }
}

// realises FR-189
/// Yields the digit of `mode` a click sets the cursor before: the one at the character column
/// `column` of the row `row`, in rows of `octets_per_row` octets of a document of `bits` bits.
///
/// * Where `between`, as in the insert mode, the cursor is set before the digit whose start is
///   nearest; otherwise before the digit clicked.
/// * A click behind the digits of a row, or behind the last digit, sets the cursor behind the
///   last digit of that row, or of the document.
pub fn digit_at(
    row: usize,
    column: f64,
    between: bool,
    mode: EditorMode,
    octets_per_row: usize,
    bits: usize,
) -> usize {
    let unit = mode.digit_bits();
    if unit == 0 || octets_per_row == 0 {
        return 0;
    }
    let per_octet = 8 / unit;
    let pitch = mode.octet_pitch() as f64;
    let column = column.max(0.0);
    let octet = ((column / pitch).floor() as usize).min(octets_per_row - 1);
    let within = column - pitch * octet as f64;
    let digit = if between {
        (within.round() as usize).min(per_octet)
    } else {
        (within.floor() as usize).min(per_octet - 1)
    };
    let last_of_row = (row + 1).saturating_mul(octets_per_row * per_octet);
    ((row.saturating_mul(octets_per_row) + octet) * per_octet + digit)
        .min(if between {
            last_of_row
        } else {
            last_of_row.saturating_sub(1)
        })
        .min(bits.div_ceil(unit))
}

// realises FR-201
/// Yields the character columns of the row `row` that hold the digits of `mode` from the digit
/// `start` to the digit `end`, exclusive: the first column and their number, `None` where the
/// row holds none of them.
pub fn row_selection(
    start: usize,
    end: usize,
    row: usize,
    mode: EditorMode,
    octets_per_row: usize,
) -> Option<(usize, usize)> {
    let unit = mode.digit_bits();
    if unit == 0 || octets_per_row == 0 || end <= start {
        return None;
    }
    let per_octet = 8 / unit;
    let per_row = octets_per_row * per_octet;
    let first = start.max(row.saturating_mul(per_row));
    let last = (end - 1).min((row + 1).saturating_mul(per_row).saturating_sub(1));
    if first > last {
        return None;
    }
    let column = |digit: usize| {
        let within = digit - row * per_row;
        mode.octet_pitch() * (within / per_octet) + within % per_octet
    };
    Some((column(first), column(last) - column(first) + 1))
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : InputGroup::replace_digits, InputGroup::undo, InputGroup::redo,
 *                    InputGroup::digits_text, InputGroup::long_idle_expired,
 *                    Editing::typing_span, Editing::removal_span, Editing::cursor_moved,
 *                    Editing::cursor_cell, Editing::digit_at, Editing::row_selection,
 *                    Editing::digits_valid */
#[cfg(test)]
mod tests {
    use super::*;

    fn bits(octets: &[u8], len: usize) -> Bits {
        Bits {
            octets: octets.to_vec(),
            len,
        }
    }

    fn made(before: &Bits, unit: usize, start: usize, removed: usize, digits: &[u8]) -> Bits {
        applied(before, &replacement(before, unit, start, removed, digits))
    }

    #[test]
    fn octets_are_bits_without_a_placeholder() {
        let of_octets = Bits::of_octets(vec![0xab, 0xcd]);
        assert_eq!(
            (of_octets.len(), of_octets.padding(), of_octets.octets()),
            (16, 0, &[0xab, 0xcd][..])
        );
    }

    #[test]
    fn digits_are_written_one_after_the_other_and_pad_the_last_octet_with_zeros() {
        assert_eq!(
            (
                Bits::of_digits(&[0xa, 0xb, 0xc], 4),
                Bits::of_digits(&[1, 0, 1], 1),
                Bits::of_digits(&[], 4)
            ),
            (bits(&[0xab, 0xc0], 12), bits(&[0xa0], 3), Bits::default())
        );
    }

    #[test]
    fn a_digit_typed_at_the_end_begins_an_octet_with_a_placeholder() {
        // FR-180
        let typed = made(&Bits::of_octets(vec![0x12]), 4, 2, 1, &[0xa]);
        assert_eq!(
            (typed.clone(), typed.padding()),
            (bits(&[0x12, 0xa0], 12), 4)
        );
    }

    #[test]
    fn a_digit_typed_before_a_placeholder_fills_it_in_either_mode() {
        // FR-180
        let before = bits(&[0x12, 0xa0], 12);
        assert_eq!(
            (
                made(&before, 4, 3, 1, &[0xb]),
                made(&before, 4, 3, 0, &[0xb])
            ),
            (bits(&[0x12, 0xab], 16), bits(&[0x12, 0xab], 16))
        );
    }

    #[test]
    fn a_digit_typed_in_the_overwrite_mode_replaces_the_digit_at_the_cursor() {
        // FR-181
        assert_eq!(
            made(&Bits::of_octets(vec![0x12, 0x34]), 4, 1, 1, &[0xf]),
            bits(&[0x1f, 0x34], 16)
        );
    }

    #[test]
    fn a_digit_typed_in_the_insert_mode_moves_the_digits_behind_it_toward_the_end() {
        // FR-182
        assert_eq!(
            made(&Bits::of_octets(vec![0x12, 0x34]), 4, 1, 0, &[0xf]),
            bits(&[0x1f, 0x23, 0x40], 20)
        );
    }

    #[test]
    fn a_digit_inserted_before_a_document_with_a_placeholder_uses_the_placeholder() {
        // FR-182
        assert_eq!(
            made(&bits(&[0x12, 0x30], 12), 4, 0, 0, &[0xf]),
            bits(&[0xf1, 0x23], 16)
        );
    }

    #[test]
    fn a_removed_digit_moves_the_digits_behind_it_toward_the_start_and_leaves_a_placeholder() {
        // FR-183, FR-184
        assert_eq!(
            made(&Bits::of_octets(vec![0x12, 0x34]), 4, 1, 1, &[]),
            bits(&[0x13, 0x40], 12)
        );
    }

    #[test]
    fn a_removed_digit_of_an_octet_with_a_placeholder_removes_the_octet() {
        // FR-183
        assert_eq!(
            made(&bits(&[0x12, 0x30], 12), 4, 2, 1, &[]),
            bits(&[0x12], 8)
        );
    }

    #[test]
    fn a_bit_is_inserted_and_removed_as_a_nibble_is() {
        // FR-178, FR-182, FR-183
        let inserted = made(&Bits::of_octets(vec![0b1000_0001]), 1, 1, 0, &[1]);
        assert_eq!(
            (inserted.clone(), made(&inserted, 1, 1, 1, &[])),
            (
                bits(&[0b1100_0000, 0b1000_0000], 9),
                bits(&[0b1000_0001], 8)
            )
        );
    }

    #[test]
    fn an_edit_in_hex_takes_the_placeholders_within_a_begun_nibble_as_zero() {
        // FR-187: 1010 1 and three placeholder bits of the nibble, then the digit F
        let before = bits(&[0b1010_1000], 5);
        let step = replacement(&before, 4, 2, 0, &[0xf]);
        assert_eq!(
            (step.len(), applied(&before, &step)),
            (2, bits(&[0b1010_1000, 0xf0], 12))
        );
    }

    #[test]
    fn nothing_replaced_by_nothing_is_no_step() {
        assert_eq!(
            (
                replacement(&Bits::of_octets(vec![0x12]), 4, 2, 1, &[]),
                replacement(&Bits::default(), 4, 0, 1, &[])
            ),
            (vec![], vec![])
        );
    }

    #[test]
    fn a_start_beyond_the_end_and_too_many_digits_removed_are_cut_to_the_document() {
        assert_eq!(
            made(&Bits::of_octets(vec![0x12, 0x34]), 4, 9, 9, &[0xa]),
            bits(&[0x12, 0x34, 0xa0], 20)
        );
    }

    #[test]
    fn several_digits_replace_a_selection_as_one_step() {
        // FR-205
        let before = Bits::of_octets(vec![0x12, 0x34, 0x56]);
        let step = replacement(&before, 4, 1, 3, &[0xa, 0xb]);
        assert_eq!(
            (step.len(), applied(&before, &step)),
            (1, bits(&[0x1a, 0xb5, 0x60], 20))
        );
    }

    #[test]
    fn the_placeholders_are_completed_as_digits_zero() {
        // FR-185
        let completed = bits(&[0x12, 0xa0], 12).completed();
        assert_eq!((completed.len(), completed.padding()), (16, 0));
    }

    #[test]
    fn the_digits_of_a_span_are_read_as_their_values() {
        // FR-202
        let document = bits(&[0x12, 0xa0], 12);
        assert_eq!(
            (
                document.digits(4, 1, 9),
                document.digits(1, 6, 9),
                document.digits(4, 2, 2)
            ),
            (vec![2, 0xa], vec![1, 0, 1], vec![])
        );
    }

    #[test]
    fn a_slice_beyond_the_bits_is_cut_to_them() {
        assert_eq!(
            Bits::of_octets(vec![0x12, 0x34]).slice(4, 99),
            bits(&[0x23, 0x40], 12)
        );
    }

    #[test]
    fn splicing_one_megabyte_takes_less_than_200_milliseconds() {
        // NFR-010
        let document = Bits::of_octets(vec![0x5a; 1_000_000]);
        let started = std::time::Instant::now();
        let inserted = made(&document, 4, 1, 0, &[0xf]);
        let elapsed = started.elapsed();
        assert!(
            inserted.len() == 8_000_004 && elapsed < std::time::Duration::from_millis(200),
            "{elapsed:?}"
        );
    }

    #[test]
    fn undo_takes_the_last_step_back_and_redo_makes_it_again() {
        // FR-207, FR-208
        let before = Bits::of_octets(vec![0x12, 0x34]);
        let step = replacement(&before, 4, 1, 0, &[0xf]);
        let after = applied(&before, &step);
        let mut history = History::default();
        history.record(step);
        let undone = history.undo(&after);
        let redone = history.redo(&before);
        assert_eq!(
            (undone, redone, history.can_undo(), history.can_redo()),
            (Some((before, 4)), Some((after, 8)), true, false)
        );
    }

    #[test]
    fn undo_of_an_edit_that_completed_a_nibble_restores_the_placeholders() {
        // FR-187, FR-207
        let before = bits(&[0b1010_1000], 5);
        let step = replacement(&before, 4, 2, 0, &[0xf]);
        let after = applied(&before, &step);
        let mut history = History::default();
        history.record(step);
        assert_eq!(history.undo(&after).map(|(bits, _)| bits), Some(before));
    }

    #[test]
    fn a_new_step_forgets_the_steps_taken_back() {
        // FR-208
        let before = Bits::of_octets(vec![0x12]);
        let step = replacement(&before, 4, 0, 1, &[0xf]);
        let after = applied(&before, &step);
        let mut history = History::default();
        history.record(step.clone());
        let _ = history.undo(&after);
        history.record(step);
        assert_eq!((history.can_redo(), history.redo(&after)), (false, None));
    }

    #[test]
    fn an_empty_history_takes_nothing_back_and_an_empty_step_is_not_recorded() {
        let mut history = History::default();
        history.record(vec![]);
        assert_eq!(
            (history.can_undo(), history.undo(&Bits::default())),
            (false, None)
        );
    }

    #[test]
    fn a_cleared_history_holds_no_step() {
        // FR-209
        let before = Bits::of_octets(vec![0x12]);
        let mut history = History::default();
        history.record(replacement(&before, 4, 0, 1, &[0xf]));
        history.clear();
        assert_eq!((history.can_undo(), history.can_redo()), (false, false));
    }

    #[test]
    fn the_history_keeps_at_most_a_thousand_steps() {
        let before = Bits::of_octets(vec![0x12]);
        let mut history = History::default();
        for _ in 0..MAX_STEPS + 5 {
            history.record(replacement(&before, 4, 0, 1, &[0xf]));
        }
        assert_eq!(history.done.len(), MAX_STEPS);
    }

    #[test]
    fn hexadecimal_digits_of_either_case_and_white_space_are_read_as_digits() {
        // FR-186, FR-203
        assert_eq!(
            (
                digits_of_text("0a F\n9", EditorMode::Hex),
                digits_of_text("01 1", EditorMode::Bin),
                digits_of_text("", EditorMode::Hex)
            ),
            (
                Some(vec![0, 0xa, 0xf, 9]),
                Some(vec![0, 1, 1]),
                Some(vec![])
            )
        );
    }

    #[test]
    fn a_text_with_another_character_holds_no_digits() {
        // FR-186, FR-203
        assert_eq!(
            (
                digits_of_text("0g", EditorMode::Hex),
                digits_of_text("012", EditorMode::Bin),
                digits_of_text("01", EditorMode::Txt)
            ),
            (None, None, None)
        );
    }

    #[test]
    fn digits_are_written_as_upper_case_text() {
        // FR-202
        assert_eq!(
            (
                text_of_digits(&[0, 0xa, 0xf], EditorMode::Hex),
                text_of_digits(&[1, 0, 1], EditorMode::Bin),
                text_of_digits(&[1], EditorMode::Txt)
            ),
            ("0AF".to_owned(), "101".to_owned(), String::new())
        );
    }

    #[test]
    fn typing_replaces_nothing_in_the_insert_mode_and_as_many_digits_in_the_overwrite_mode() {
        // FR-181, FR-182
        assert_eq!(
            (
                typing_span(5, None, 1, true),
                typing_span(5, None, 3, false),
                typing_span(5, Some(5), 1, true)
            ),
            ((5, 0), (5, 3), (5, 0))
        );
    }

    #[test]
    fn typing_replaces_a_selection_in_either_direction() {
        // FR-205
        assert_eq!(
            (
                typing_span(7, Some(3), 1, true),
                typing_span(3, Some(7), 1, false)
            ),
            ((3, 4), (3, 4))
        );
    }

    #[test]
    fn backspace_removes_the_digit_before_the_cursor_and_delete_the_one_at_it() {
        // FR-183, FR-184
        assert_eq!(
            (
                removal_span(5, None, true),
                removal_span(0, None, true),
                removal_span(5, None, false),
                removal_span(2, Some(6), true)
            ),
            ((4, 1), (0, 0), (5, 1), (2, 4))
        );
    }

    #[test]
    fn movements_map_from_their_indices() {
        assert_eq!(
            (
                Movement::of_index(0),
                Movement::of_index(9),
                Movement::of_index(10)
            ),
            (Some(Movement::Left), Some(Movement::End), None)
        );
    }

    #[test]
    fn the_cursor_moves_by_a_digit_and_stops_at_both_ends() {
        // FR-188: 70 digits in rows of 32
        let moved: Vec<usize> = [
            (0, Movement::Left),
            (5, Movement::Left),
            (5, Movement::Right),
            (70, Movement::Right),
        ]
        .iter()
        .map(|(cursor, movement)| cursor_moved(*cursor, *movement, 32, 70, 4))
        .collect();
        assert_eq!(moved, vec![0, 4, 6, 70]);
    }

    #[test]
    fn the_cursor_moves_by_a_row_and_stays_in_the_first_and_in_the_last_row() {
        // FR-188
        let moved: Vec<usize> = [
            (5, Movement::Up),
            (37, Movement::Up),
            (5, Movement::Down),
            (60, Movement::Down),
            (66, Movement::Down),
        ]
        .iter()
        .map(|(cursor, movement)| cursor_moved(*cursor, *movement, 32, 70, 4))
        .collect();
        assert_eq!(moved, vec![5, 5, 37, 70, 66]);
    }

    #[test]
    fn the_cursor_moves_to_the_start_and_to_the_last_digit_of_its_row() {
        // FR-188
        let moved: Vec<usize> = [
            (37, Movement::RowStart),
            (37, Movement::RowEnd),
            (66, Movement::RowEnd),
            (37, Movement::Start),
            (37, Movement::End),
        ]
        .iter()
        .map(|(cursor, movement)| cursor_moved(*cursor, *movement, 32, 70, 4))
        .collect();
        assert_eq!(moved, vec![32, 63, 70, 0, 70]);
    }

    #[test]
    fn the_cursor_moves_by_the_rows_of_the_view_as_far_as_there_are_rows() {
        // FR-188: 1000 digits in rows of 32, a view of 4 rows
        let moved: Vec<usize> = [
            (200, Movement::PageUp),
            (40, Movement::PageUp),
            (200, Movement::PageDown),
            (900, Movement::PageDown),
        ]
        .iter()
        .map(|(cursor, movement)| cursor_moved(*cursor, *movement, 32, 1000, 4))
        .collect();
        assert_eq!(moved, vec![72, 8, 328, 996]);
    }

    #[test]
    fn a_cursor_beyond_the_digits_and_no_digits_per_row_are_brought_into_range() {
        assert_eq!(
            (
                cursor_moved(99, Movement::Left, 32, 10, 4),
                cursor_moved(3, Movement::Down, 0, 10, 4)
            ),
            (9, 4)
        );
    }

    fn cell(row: usize, column: usize, kind: CursorKind) -> CursorCell {
        CursorCell { row, column, kind }
    }

    #[test]
    fn the_cursor_before_a_digit_is_shown_at_the_column_of_that_digit() {
        // FR-190: 40 octets in rows of 16 octets
        assert_eq!(
            (
                cursor_cell(35, EditorMode::Hex, 16, 320, 40),
                cursor_cell(35, EditorMode::Bin, 4, 320, 40)
            ),
            (cell(1, 4, CursorKind::Digit), cell(1, 3, CursorKind::Digit))
        );
    }

    #[test]
    fn the_cursor_before_a_placeholder_is_shown_at_the_placeholder() {
        // FR-190: 12 bits in two octets
        assert_eq!(
            (
                cursor_cell(3, EditorMode::Hex, 16, 12, 2),
                cursor_cell(12, EditorMode::Bin, 4, 12, 2)
            ),
            (
                cell(0, 4, CursorKind::Placeholder),
                cell(0, 13, CursorKind::Placeholder)
            )
        );
    }

    #[test]
    fn the_cursor_behind_the_last_digit_is_shown_where_the_next_octet_would_begin() {
        // FR-190
        assert_eq!(
            (
                cursor_cell(4, EditorMode::Hex, 16, 16, 2),
                cursor_cell(0, EditorMode::Hex, 16, 0, 0)
            ),
            (cell(0, 6, CursorKind::End), cell(0, 0, CursorKind::End))
        );
    }

    #[test]
    fn the_cursor_behind_a_full_last_row_is_shown_behind_that_row() {
        // FR-190: 32 octets in rows of 16
        assert_eq!(
            cursor_cell(64, EditorMode::Hex, 16, 256, 32),
            cell(1, 48, CursorKind::End)
        );
    }

    #[test]
    fn the_cursor_of_txt_is_shown_at_the_start() {
        assert_eq!(
            cursor_cell(5, EditorMode::Txt, 16, 64, 8),
            cell(0, 0, CursorKind::End)
        );
    }

    #[test]
    fn a_click_in_the_overwrite_mode_sets_the_cursor_before_the_digit_clicked() {
        // FR-189: 40 octets, rows of 16 octets in hex
        assert_eq!(
            (
                digit_at(1, 4.7, false, EditorMode::Hex, 16, 320),
                digit_at(1, 5.2, false, EditorMode::Hex, 16, 320),
                digit_at(0, 13.9, false, EditorMode::Bin, 4, 320)
            ),
            (35, 35, 12)
        );
    }

    #[test]
    fn a_click_in_the_insert_mode_sets_the_cursor_before_the_nearest_digit() {
        // FR-189
        assert_eq!(
            (
                digit_at(1, 4.4, true, EditorMode::Hex, 16, 320),
                digit_at(1, 4.6, true, EditorMode::Hex, 16, 320),
                digit_at(1, 5.9, true, EditorMode::Hex, 16, 320)
            ),
            (35, 36, 36)
        );
    }

    #[test]
    fn a_click_behind_the_digits_sets_the_cursor_behind_the_last_one() {
        // FR-189: 3 octets
        assert_eq!(
            (
                digit_at(0, 40.0, true, EditorMode::Hex, 16, 24),
                digit_at(5, 0.0, false, EditorMode::Hex, 16, 24),
                digit_at(0, 99.0, true, EditorMode::Hex, 16, 1024),
                digit_at(0, 99.0, false, EditorMode::Hex, 16, 1024),
                digit_at(0, 3.0, true, EditorMode::Txt, 16, 24)
            ),
            (6, 6, 32, 31, 0)
        );
    }

    #[test]
    fn a_selection_within_a_row_covers_its_digits_and_the_spaces_between_them() {
        // FR-201: the digits 1 to 4 of row 0 in hex: '2 34 5' of '12 34 56'
        assert_eq!(
            (
                row_selection(1, 5, 0, EditorMode::Hex, 16),
                row_selection(6, 10, 0, EditorMode::Bin, 4)
            ),
            (Some((1, 6)), Some((6, 5)))
        );
    }

    #[test]
    fn a_selection_over_several_rows_has_its_part_in_each_of_them() {
        // FR-201: rows of 16 octets in hex hold 32 digits
        let rows: Vec<Option<(usize, usize)>> = (0..4)
            .map(|row| row_selection(30, 66, row, EditorMode::Hex, 16))
            .collect();
        assert_eq!(rows, vec![Some((45, 2)), Some((0, 47)), Some((0, 2)), None]);
    }

    #[test]
    fn an_empty_selection_and_txt_cover_nothing() {
        assert_eq!(
            (
                row_selection(5, 5, 0, EditorMode::Hex, 16),
                row_selection(0, 5, 0, EditorMode::Txt, 16),
                row_selection(0, 5, 0, EditorMode::Hex, 0)
            ),
            (None, None, None)
        );
    }
}
