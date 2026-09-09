//! The *text increment*: how the text of a code editor changed, its rendering, and its derivation.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

// realises FR-050, FR-051, FR-052, FR-057, FR-060, FR-061, FR-062
/// How the text of a code editor changed since the *text increment* before it.
///
/// * Overwriting `range` characters of the *provided text* at `position` with `text` yields the
///   text the code editor currently holds.
/// * A pure insertion carries `range` 0, a pure deletion an empty `text`.
/// * Positions and ranges count characters, not bytes.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextIncrement {
    /// the characters of the *provided text* preceding where `text` overwrites
    pub position: usize,
    /// the characters of the *provided text* that `text` overwrites, from `position`
    pub range: usize,
    /// the text overwriting that range
    pub text: String,
}

impl TextIncrement {
    // realises FR-057
    /// Yields whether the *text increment* describes no change of the *provided text*.
    pub fn is_empty(&self) -> bool {
        self.range == 0 && self.text.is_empty()
    }

    // realises FR-060, FR-061, FR-062
    /// Yields the *text increment* as one line, as FR-060 to FR-062 define it.
    ///
    /// * The position and the range are decimal numbers of at least four digits, padded with zeros.
    /// * A backslash, a double quote, a line feed, a carriage return and a horizontal tab of the
    ///   text are escaped, so that the rendering is one line.
    pub fn rendering(&self) -> String {
        format!(
            "Position:{:04}, Range: {:04}, \"{}\"",
            self.position,
            self.range,
            escaped(&self.text)
        )
    }
}

/// Yields `text` with the characters of FR-062 escaped.
fn escaped(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            other => result.push(other),
        }
    }
    result
}

// realises FR-054, NFR-005
/// Derives the *text increment* from `provided` to `current`.
///
/// * The common prefix and the common suffix of both texts are left out of the increment, so that
///   the increment carries the changed characters only.
/// * Both texts are passed over once each.
pub fn increment_between(provided: &str, current: &str) -> TextIncrement {
    let provided_chars: Vec<char> = provided.chars().collect();
    let current_chars: Vec<char> = current.chars().collect();
    let common_max = provided_chars.len().min(current_chars.len());
    let prefix = provided_chars
        .iter()
        .zip(current_chars.iter())
        .take_while(|(a, b)| a == b)
        .count();
    let suffix = provided_chars
        .iter()
        .rev()
        .zip(current_chars.iter().rev())
        .take(common_max - prefix)
        .take_while(|(a, b)| a == b)
        .count();
    TextIncrement {
        position: prefix,
        range: provided_chars.len() - prefix - suffix,
        text: current_chars[prefix..current_chars.len() - suffix]
            .iter()
            .collect(),
    }
}

// realises FR-053, FR-054, FR-055, FR-056, FR-057
/// The *provided text* of one code editor, from which its next *text increment* is derived.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IncrementTracker {
    /// the *provided text*: empty before the first *text increment*
    provided: String,
}

impl IncrementTracker {
    // realises FR-053
    /// Yields the *provided text*: empty before the first *text increment*.
    pub fn provided(&self) -> &str {
        &self.provided
    }

    // realises FR-054, FR-055
    /// Yields the *text increment* from the *provided text* to `current`, without providing it.
    pub fn pending(&self, current: &str) -> TextIncrement {
        increment_between(&self.provided, current)
    }

    // realises FR-056, FR-057
    /// Provides the pending *text increment*: the *provided text* becomes `current`.
    ///
    /// * Yields `None`, and leaves the *provided text* as it is, where `current` does not differ
    ///   from it.
    pub fn provide(&mut self, current: &str) -> Option<TextIncrement> {
        let increment = self.pending(current);
        if increment.is_empty() {
            return None;
        }
        self.provided = current.to_owned();
        Some(increment)
    }
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : InputGroup::idle_expired */
#[cfg(test)]
mod tests {
    use super::*;

    /// Yields `provided` with `increment` overwritten, as a receiver of the increments does.
    fn overwritten(provided: &str, increment: &TextIncrement) -> String {
        let chars: Vec<char> = provided.chars().collect();
        let mut result: String = chars[..increment.position].iter().collect();
        result.push_str(&increment.text);
        result.extend(chars[increment.position + increment.range..].iter());
        result
    }

    #[test]
    fn provided_text_is_empty_before_the_first_increment() {
        // FR-053
        assert_eq!(IncrementTracker::default().provided(), "");
    }

    #[test]
    fn whole_text_is_the_first_increment() {
        // FR-054, FR-056
        let mut tracker = IncrementTracker::default();
        let pending = tracker.pending("abc");
        assert_eq!(
            pending,
            TextIncrement {
                position: 0,
                range: 0,
                text: "abc".into()
            }
        );
        assert!(!pending.is_empty());
        assert_eq!(tracker.provide("abc"), Some(pending));
        assert_eq!(tracker.provided(), "abc");
    }

    #[test]
    fn insertion_overwrites_no_character() {
        // FR-050, FR-051, FR-052, FR-054
        let mut tracker = IncrementTracker::default();
        tracker.provide("abc");
        let increment = tracker.pending("abZc");
        assert_eq!(
            increment,
            TextIncrement {
                position: 2,
                range: 0,
                text: "Z".into()
            }
        );
        assert_eq!(overwritten(tracker.provided(), &increment), "abZc");
    }

    #[test]
    fn deletion_carries_an_empty_text() {
        // FR-050, FR-051, FR-052, FR-054
        let mut tracker = IncrementTracker::default();
        tracker.provide("abcd");
        let increment = tracker.pending("abd");
        assert_eq!(
            increment,
            TextIncrement {
                position: 2,
                range: 1,
                text: String::new()
            }
        );
        assert!(!increment.is_empty());
        assert_eq!(overwritten(tracker.provided(), &increment), "abd");
    }

    #[test]
    fn replacement_overwrites_the_changed_characters_only() {
        // FR-050, FR-051, FR-052, FR-054
        let mut tracker = IncrementTracker::default();
        tracker.provide("abcdef");
        let increment = tracker.pending("abXYef");
        assert_eq!(
            increment,
            TextIncrement {
                position: 2,
                range: 2,
                text: "XY".into()
            }
        );
        assert_eq!(overwritten(tracker.provided(), &increment), "abXYef");
    }

    #[test]
    fn emptied_text_is_one_increment_overwriting_all_of_it() {
        // FR-050, FR-051, FR-052
        let mut tracker = IncrementTracker::default();
        tracker.provide("abcdef");
        assert_eq!(
            tracker.pending(""),
            TextIncrement {
                position: 0,
                range: 6,
                text: String::new()
            }
        );
    }

    #[test]
    fn unmodified_text_yields_no_increment() {
        // FR-057
        let mut tracker = IncrementTracker::default();
        assert!(tracker.pending("").is_empty());
        assert_eq!(tracker.provide(""), None);
        tracker.provide("abc");
        assert!(tracker.pending("abc").is_empty());
        assert_eq!(tracker.provide("abc"), None);
        assert_eq!(tracker.provided(), "abc");
    }

    #[test]
    fn receiver_composes_the_whole_text_of_the_increments() {
        // FR-054, FR-056
        let mut tracker = IncrementTracker::default();
        let mut receiver = String::new();
        for text in ["alpha", "alpha beta", "alpha gamma beta", "gamma beta", ""] {
            receiver = overwritten(&receiver, &tracker.pending(text));
            tracker.provide(text);
            assert_eq!(receiver, text);
            assert_eq!(tracker.provided(), text);
        }
    }

    #[test]
    fn characters_beyond_ascii_are_counted_as_characters() {
        // FR-051, FR-052: positions count characters, not bytes
        let mut tracker = IncrementTracker::default();
        tracker.provide("äöü");
        let increment = tracker.pending("äXü");
        assert_eq!(
            increment,
            TextIncrement {
                position: 1,
                range: 1,
                text: "X".into()
            }
        );
        assert_eq!(overwritten(tracker.provided(), &increment), "äXü");
    }

    #[test]
    fn rendering_pads_the_numbers_to_four_digits() {
        // FR-060, FR-061
        let increment = TextIncrement {
            position: 2,
            range: 2,
            text: "XY".into(),
        };
        assert_eq!(increment.rendering(), "Position:0002, Range: 0002, \"XY\"");
        let increment = TextIncrement {
            position: 0,
            range: 0,
            text: "a".into(),
        };
        assert_eq!(increment.rendering(), "Position:0000, Range: 0000, \"a\"");
    }

    #[test]
    fn rendering_writes_a_longer_number_in_full() {
        // FR-061
        let increment = TextIncrement {
            position: 12345,
            range: 67890,
            text: String::new(),
        };
        assert_eq!(increment.rendering(), "Position:12345, Range: 67890, \"\"");
    }

    #[test]
    fn rendering_escapes_to_one_line() {
        // FR-062
        let increment = TextIncrement {
            position: 1,
            range: 0,
            text: "a\nb\r\tc\"d\\e".into(),
        };
        assert_eq!(
            increment.rendering(),
            "Position:0001, Range: 0000, \"a\\nb\\r\\tc\\\"d\\\\e\""
        );
        assert!(!increment.rendering().contains('\n'));
    }

    #[test]
    fn increment_of_one_megabyte_is_derived_in_time() {
        // NFR-005: at most 200 milliseconds for a provided text of 1 megabyte
        let provided: String = "x".repeat(1024 * 1024);
        let mut current = provided.clone();
        current.replace_range(512 * 1024..512 * 1024 + 1, "y");
        let start = std::time::Instant::now();
        let increment = increment_between(&provided, &current);
        assert!(start.elapsed() <= std::time::Duration::from_millis(200));
        assert_eq!(
            increment,
            TextIncrement {
                position: 512 * 1024,
                range: 1,
                text: "y".into()
            }
        );
    }
}
