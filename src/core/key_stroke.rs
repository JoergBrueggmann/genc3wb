//! A key stroke: the key and the modifiers of a key event, its name, and the insert key by default.
//!
//! Copyright (c) Jörg Karl-Heinz Walter Brüggmann, 2021-2026
//! Author: Jörg Karl-Heinz Walter Brüggmann <info@joerg-brueggmann.de>

/// The codes of the keys of Qt that have a name of their own here.
const KEY_ESCAPE: u32 = 0x0100_0000;
const KEY_TAB: u32 = 0x0100_0001;
const KEY_BACKSPACE: u32 = 0x0100_0003;
const KEY_RETURN: u32 = 0x0100_0004;
const KEY_ENTER: u32 = 0x0100_0005;
const KEY_INSERT: u32 = 0x0100_0006;
const KEY_DELETE: u32 = 0x0100_0007;
const KEY_F1: u32 = 0x0100_0030;
const KEY_F35: u32 = 0x0100_0052;

/// The keys of Qt that are modifiers themselves: Shift, Control, Meta, Alt, Caps Lock, Num
/// Lock and Scroll Lock, and the keys of AltGr and of the layers of a keyboard.
const MODIFIER_KEYS: [u32; 12] = [
    0x0100_0020,
    0x0100_0021,
    0x0100_0022,
    0x0100_0023,
    0x0100_0024,
    0x0100_0025,
    0x0100_0026,
    0x0100_1103,
    0x0100_117e,
    0x0100_117f,
    0x0100_1120,
    0x01ff_ffff,
];

/// The modifiers of Qt a key stroke keeps, with their names in the order of a name.
const MODIFIERS: [(u32, &str); 4] = [
    (0x0400_0000, "Ctrl"),
    (0x0200_0000, "Shift"),
    (0x0800_0000, "Alt"),
    (0x1000_0000, "Meta"),
];

// realises FR-193, FR-198, FR-199, FR-200
/// A key stroke: a key of Qt with the modifiers held, as a key event of the *front end* carries
/// them.
///
/// * The modifiers kept are Control, Shift, Alt and Meta; that of the keypad and every other
///   one is passed over, so that the key of the keypad and that of the main block are one key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyStroke {
    /// the code of the key, as Qt numbers it
    key: u32,
    /// the modifiers kept, as Qt numbers them
    modifiers: u32,
}

impl KeyStroke {
    // initialisation
    /// Creates the key stroke of `key` with `modifiers`, of which the modifiers kept are taken.
    pub fn new(key: u32, modifiers: u32) -> KeyStroke {
        KeyStroke {
            key,
            modifiers: modifiers & MODIFIERS.iter().fold(0, |all, (bit, _)| all | bit),
        }
    }

    // realises FR-198
    /// Yields the insert key where none is stored: the Enter key, which fn + Return yields on a
    /// keyboard of MacOS, on MacOS, and the Insert key elsewhere.
    pub fn default_insert_key() -> KeyStroke {
        if cfg!(target_os = "macos") {
            KeyStroke::new(KEY_ENTER, 0)
        } else {
            KeyStroke::new(KEY_INSERT, 0)
        }
    }

    // state
    /// Yields the code of the key and the modifiers kept.
    pub fn key(&self) -> u32 {
        self.key
    }

    pub fn modifiers(&self) -> u32 {
        self.modifiers
    }

    // realises FR-193
    /// Yields whether a key event of `key` with `modifiers` is this key stroke.
    pub fn matches(&self, key: u32, modifiers: u32) -> bool {
        *self == KeyStroke::new(key, modifiers)
    }

    // realises FR-199
    /// Yields whether `key` is a modifier itself, which is no key stroke of its own.
    pub fn is_modifier_key(key: u32) -> bool {
        MODIFIER_KEYS.contains(&key)
    }

    // realises FR-199, FR-200
    /// Yields the name of the key stroke: the names of its modifiers in the order Ctrl, Shift,
    /// Alt, Meta and the name of its key, joined by `+`.
    ///
    /// * A key is named `Insert`, `Enter`, `Return`, `Escape`, `Tab`, `Backspace`, `Delete`,
    ///   `F1` to `F35`, `Space`, by its character where it is a printable character of ASCII,
    ///   and by its code in hexadecimal digits behind `0x` otherwise.
    pub fn name(&self) -> String {
        let key = match self.key {
            KEY_INSERT => "Insert".to_owned(),
            KEY_ENTER => "Enter".to_owned(),
            KEY_RETURN => "Return".to_owned(),
            KEY_ESCAPE => "Escape".to_owned(),
            KEY_TAB => "Tab".to_owned(),
            KEY_BACKSPACE => "Backspace".to_owned(),
            KEY_DELETE => "Delete".to_owned(),
            KEY_F1..=KEY_F35 => format!("F{}", self.key - KEY_F1 + 1),
            0x20 => "Space".to_owned(),
            0x21..=0x7e => char::from_u32(self.key).map_or_else(String::new, String::from),
            other => format!("0x{other:X}"),
        };
        MODIFIERS
            .iter()
            .filter(|(bit, _)| self.modifiers & bit != 0)
            .map(|(_, name)| (*name).to_owned())
            .chain(std::iter::once(key))
            .collect::<Vec<String>>()
            .join("+")
    }

    // realises FR-200
    /// Yields the key stroke of a name as [`KeyStroke::name`] yields it, `None` where `name` is
    /// none.
    pub fn of_name(name: &str) -> Option<KeyStroke> {
        let (key_name, modifier_names) = match name.strip_suffix("++") {
            Some(modifiers) => ("+", modifiers),
            None => match name.rsplit_once('+') {
                Some((modifiers, key)) if !key.is_empty() => (key, modifiers),
                Some(_) | None => (name, ""),
            },
        };
        let modifiers = modifier_names
            .split('+')
            .filter(|part| !part.is_empty())
            .map(|part| {
                MODIFIERS
                    .iter()
                    .find(|(_, name)| *name == part)
                    .map(|(bit, _)| *bit)
            })
            .try_fold(0, |all, bit| bit.map(|bit| all | bit))?;
        let function = key_name
            .strip_prefix('F')
            .and_then(|number| number.parse::<u32>().ok())
            .filter(|number| (1..=35).contains(number));
        let key = match (key_name, function) {
            ("Insert", _) => KEY_INSERT,
            ("Enter", _) => KEY_ENTER,
            ("Return", _) => KEY_RETURN,
            ("Escape", _) => KEY_ESCAPE,
            ("Tab", _) => KEY_TAB,
            ("Backspace", _) => KEY_BACKSPACE,
            ("Delete", _) => KEY_DELETE,
            ("Space", _) => 0x20,
            (_, Some(number)) => KEY_F1 + number - 1,
            (other, None) => match other.strip_prefix("0x") {
                Some(digits) => u32::from_str_radix(digits, 16).ok()?,
                None => {
                    let mut characters = other.chars();
                    let character = characters.next().filter(|_| characters.next().is_none())?;
                    Some(u32::from(character)).filter(|code| (0x21..=0x7e).contains(code))?
                }
            },
        };
        Some(KeyStroke { key, modifiers })
    }
}

/*  * validated        : ✅
 * completeness     : ✅
 * independence     : ✅
 * edge cases       : ✅
 * conforms to doc  : ✅
 * covers bridge    : Workbench::toggled_by_key, Workbench::set_insert_key,
 *                    Workbench::insert_key_name, Workbench::key_name,
 *                    Workbench::reset_insert_key */
#[cfg(test)]
mod tests {
    use super::*;

    const SHIFT: u32 = 0x0200_0000;
    const CONTROL: u32 = 0x0400_0000;
    const KEYPAD: u32 = 0x2000_0000;

    #[test]
    fn the_modifier_of_the_keypad_is_passed_over() {
        // FR-193
        assert_eq!(
            KeyStroke::new(KEY_ENTER, KEYPAD | SHIFT),
            KeyStroke::new(KEY_ENTER, SHIFT)
        );
    }

    #[test]
    fn a_key_event_with_the_same_key_and_modifiers_matches() {
        // FR-193
        let stroke = KeyStroke::new(KEY_INSERT, CONTROL);
        assert_eq!(
            (
                stroke.matches(KEY_INSERT, CONTROL | KEYPAD),
                stroke.matches(KEY_INSERT, 0),
                stroke.matches(KEY_ENTER, CONTROL)
            ),
            (true, false, false)
        );
    }

    #[test]
    fn the_default_insert_key_is_enter_on_macos_and_insert_elsewhere() {
        // FR-198
        let expected = if cfg!(target_os = "macos") {
            "Enter"
        } else {
            "Insert"
        };
        assert_eq!(KeyStroke::default_insert_key().name(), expected);
    }

    #[test]
    fn a_modifier_alone_is_no_key_stroke() {
        // FR-199
        assert_eq!(
            (
                KeyStroke::is_modifier_key(0x0100_0020),
                KeyStroke::is_modifier_key(0x0100_0021),
                KeyStroke::is_modifier_key(KEY_INSERT)
            ),
            (true, true, false)
        );
    }

    #[test]
    fn a_key_stroke_is_named_by_its_modifiers_and_its_key() {
        // FR-199, FR-200
        let names: Vec<String> = [
            KeyStroke::new(KEY_INSERT, 0),
            KeyStroke::new(KEY_ENTER, SHIFT | CONTROL),
            KeyStroke::new(KEY_F1 + 11, 0),
            KeyStroke::new(u32::from('I'), CONTROL),
            KeyStroke::new(0x20, 0),
            KeyStroke::new(u32::from('+'), SHIFT),
            KeyStroke::new(0x0100_0010, 0),
        ]
        .iter()
        .map(KeyStroke::name)
        .collect();
        assert_eq!(
            names,
            vec![
                "Insert",
                "Ctrl+Shift+Enter",
                "F12",
                "Ctrl+I",
                "Space",
                "Shift++",
                "0x1000010"
            ]
        );
    }

    #[test]
    fn every_name_is_read_back_as_its_key_stroke() {
        // FR-200
        let strokes = [
            KeyStroke::new(KEY_INSERT, 0),
            KeyStroke::new(KEY_ENTER, SHIFT | CONTROL),
            KeyStroke::new(KEY_RETURN, 0x0800_0000),
            KeyStroke::new(KEY_F1 + 34, 0x1000_0000),
            KeyStroke::new(u32::from('I'), CONTROL),
            KeyStroke::new(u32::from('F'), 0),
            KeyStroke::new(0x20, 0),
            KeyStroke::new(u32::from('+'), SHIFT),
            KeyStroke::new(u32::from('+'), 0),
            KeyStroke::new(0x0100_0010, 0),
        ];
        let read: Vec<Option<KeyStroke>> = strokes
            .iter()
            .map(|stroke| KeyStroke::of_name(&stroke.name()))
            .collect();
        assert_eq!(read, strokes.iter().copied().map(Some).collect::<Vec<_>>());
    }

    #[test]
    fn a_text_that_is_no_name_is_no_key_stroke() {
        // FR-200
        assert_eq!(
            (
                KeyStroke::of_name(""),
                KeyStroke::of_name("Hyper+Insert"),
                KeyStroke::of_name("Inserts"),
                KeyStroke::of_name("0xZZ"),
                KeyStroke::of_name("F36")
            ),
            (None, None, None, None, None)
        );
    }
}
