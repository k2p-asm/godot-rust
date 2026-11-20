//! `core/io/resource_uid.*`: resource UID text form and the id→path table.
//!
//! The text form is base **34**: `a`..`y` (25 letters, values 0..24) followed by
//! `0`..`8` (values 25..33), least-significant digit last, prefixed with
//! `uid://`. `-1` prints as `uid://<invalid>` and parses back to `-1`.
//!
//! `text_to_id` rejects anything that does not start with `uid://`, the
//! `<invalid>` sentinel, and any character outside the two accepted ranges; the
//! parsed value is masked with `0x7FFFFFFFFFFFFFFF`.
//!
//! The id→path table mirrors the engine: `add_id` refuses an existing id,
//! `set_id` refuses a missing one, `remove_id` refuses a missing one, and
//! `get_id_path` returns an empty string for `-1` and for unknown ids.
//!
//! `create_id` takes the random source as an argument instead of using
//! mbedTLS; each candidate is masked and rejected while it collides, exactly
//! like the engine's retry loop.
//!
//! Not ported: the on-disk cache, the reverse cache, and
//! `create_id_for_path`'s MD5-seeded PCG (it needs `FileAccess`).

use std::collections::HashMap;

/// `ResourceUID::INVALID_ID`.
pub const INVALID_ID: i64 = -1;

const CHAR_COUNT: u64 = 25; // 'z' - 'a'
const BASE: u64 = CHAR_COUNT + 9; // '9' - '0'
const UID_PREFIX: &str = "uid://";
const INVALID_TEXT: &str = "uid://<invalid>";

/// `id_to_text`: base-34 text or the `<invalid>` sentinel for negatives.
pub fn id_to_text(id: i64) -> String {
    if id < 0 {
        return INVALID_TEXT.to_string();
    }
    let mut value = id as u64;
    let mut digits = Vec::new();
    loop {
        let digit = (value % BASE) as u8;
        digits.push(char_of(digit));
        value /= BASE;
        if value == 0 {
            break;
        }
    }
    digits.reverse();
    format!("{UID_PREFIX}{}", digits.into_iter().collect::<String>())
}

/// `text_to_id`: `INVALID_ID` for a wrong prefix, the sentinel, or a bad char.
pub fn text_to_id(text: &str) -> i64 {
    let Some(rest) = text.strip_prefix(UID_PREFIX) else {
        return INVALID_ID;
    };
    if text == INVALID_TEXT {
        return INVALID_ID;
    }

    let mut uid: u64 = 0;
    for c in rest.chars() {
        let digit = if c.is_ascii_lowercase() {
            c as u64 - 'a' as u64
        } else if c.is_ascii_digit() {
            c as u64 - '0' as u64 + CHAR_COUNT
        } else {
            return INVALID_ID;
        };
        uid = uid.wrapping_mul(BASE).wrapping_add(digit);
    }
    (uid & 0x7FFF_FFFF_FFFF_FFFF) as i64
}

fn char_of(digit: u8) -> char {
    if (digit as u64) < CHAR_COUNT {
        (b'a' + digit) as char
    } else {
        (b'0' + (digit - CHAR_COUNT as u8)) as char
    }
}

#[derive(Clone, Debug, Default)]
pub struct ResourceUidTable {
    paths: HashMap<i64, String>,
    changed: bool,
}

impl ResourceUidTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_id(&self, id: i64) -> bool {
        self.paths.contains_key(&id)
    }

    /// Fails when the id is already present (`ERR_FAIL_COND(unique_ids.has)`).
    pub fn add_id(&mut self, id: i64, path: &str) -> bool {
        if self.paths.contains_key(&id) {
            return false;
        }
        self.paths.insert(id, path.to_string());
        self.changed = true;
        true
    }

    /// Fails when the id is missing; setting the same path does not mark the
    /// table changed.
    pub fn set_id(&mut self, id: i64, path: &str) -> bool {
        match self.paths.get_mut(&id) {
            Some(existing) => {
                if existing != path {
                    *existing = path.to_string();
                    self.changed = true;
                }
                true
            }
            None => false,
        }
    }

    /// `get_id_path`: empty for `INVALID_ID` and for unknown ids.
    pub fn get_id_path(&self, id: i64) -> String {
        if id == INVALID_ID {
            return String::new();
        }
        self.paths.get(&id).cloned().unwrap_or_default()
    }

    /// Fails when the id is missing.
    pub fn remove_id(&mut self, id: i64) -> bool {
        if self.paths.remove(&id).is_none() {
            return false;
        }
        self.changed = true;
        true
    }

    pub fn clear(&mut self) {
        self.paths.clear();
        self.changed = true;
    }

    pub fn get_id_count(&self) -> usize {
        self.paths.len()
    }

    pub fn is_changed(&self) -> bool {
        self.changed
    }

    pub fn mark_saved(&mut self) {
        self.changed = false;
    }

    /// `uid_to_path`: parse then look up.
    pub fn uid_to_path(&self, uid: &str) -> String {
        self.get_id_path(text_to_id(uid))
    }

    /// `ensure_path`: resolves `uid://` text, otherwise returns the input.
    pub fn ensure_path(&self, uid_or_path: &str) -> String {
        if uid_or_path.starts_with(UID_PREFIX) {
            self.uid_to_path(uid_or_path)
        } else {
            uid_or_path.to_string()
        }
    }

    /// `create_id`: masks every candidate and retries while it is taken.
    pub fn create_id<F: FnMut() -> u64>(&self, mut next: F) -> i64 {
        loop {
            let id = (next() & 0x7FFF_FFFF_FFFF_FFFF) as i64;
            if !self.paths.contains_key(&id) {
                return id;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_ids_print_the_invalid_sentinel() {
        assert_eq!(id_to_text(INVALID_ID), "uid://<invalid>");
        assert_eq!(id_to_text(-1234), "uid://<invalid>");
        assert_eq!(text_to_id("uid://<invalid>"), INVALID_ID);
    }

    #[test]
    fn base_34_round_trips() {
        // Zero is the first character.
        assert_eq!(id_to_text(0), "uid://a");
        assert_eq!(text_to_id("uid://a"), 0);

        // Digit 24 is 'y', digit 25 is '0'.
        assert_eq!(id_to_text(24), "uid://y");
        assert_eq!(id_to_text(25), "uid://0");
        assert_eq!(text_to_id("uid://y"), 24);
        assert_eq!(text_to_id("uid://0"), 25);

        // Two digits: 33*34 + 33 = 1155 -> "88".
        assert_eq!(id_to_text(1155), "uid://88");
        assert_eq!(text_to_id("uid://88"), 1155);

        for id in [0, 1, 33, 34, 1155, 1_000_000, 0x7FFF_FFFF_FFFF_FFFFi64] {
            let text = id_to_text(id);
            assert_eq!(text_to_id(&text), id, "{id} -> {text}");
        }

        // The engine's documented maximum example.
        let max = 0x7FFF_FFFF_FFFF_FFFFi64;
        assert_eq!(id_to_text(max).len(), "uid://".len() + 13);
    }

    #[test]
    fn text_parsing_rejects_malformed_input() {
        assert_eq!(text_to_id(""), INVALID_ID);
        assert_eq!(text_to_id("res://x"), INVALID_ID);
        assert_eq!(text_to_id("uid:/a"), INVALID_ID);
        // The engine only checks `a`..`z` and `0`..`9`, so the two characters
        // above the alphabet are accepted even though `id_to_text` never emits
        // them: 'z' maps to 25 (same as '0') and '9' maps to 34.
        assert_eq!(text_to_id("uid://z"), 25);
        assert_eq!(text_to_id("uid://9"), 34);
        assert_eq!(text_to_id("uid://A"), INVALID_ID);
        assert_eq!(text_to_id("uid://a-b"), INVALID_ID);
        // Empty text after the prefix parses as zero, like the engine loop.
        assert_eq!(text_to_id("uid://"), 0);
    }

    #[test]
    fn table_add_set_remove() {
        let mut t = ResourceUidTable::new();
        assert!(!t.has_id(7));
        assert!(t.add_id(7, "res://a.tres"));
        assert!(t.has_id(7));
        assert_eq!(t.get_id_path(7), "res://a.tres");
        assert!(t.is_changed());
        t.mark_saved();
        assert!(!t.is_changed());

        // Duplicate add fails; set on a missing id fails.
        assert!(!t.add_id(7, "res://b.tres"));
        assert!(!t.set_id(8, "res://b.tres"));

        // Setting a different path updates and marks changed.
        assert!(t.set_id(7, "res://b.tres"));
        assert_eq!(t.get_id_path(7), "res://b.tres");
        assert!(t.is_changed());
        t.mark_saved();
        // Setting the same path is a no-op.
        assert!(t.set_id(7, "res://b.tres"));
        assert!(!t.is_changed());

        assert_eq!(t.get_id_count(), 1);
        assert!(t.remove_id(7));
        assert!(!t.remove_id(7));
        assert_eq!(t.get_id_count(), 0);
        assert_eq!(t.get_id_path(7), "");
    }

    #[test]
    fn invalid_and_unknown_lookups_are_empty() {
        let mut t = ResourceUidTable::new();
        assert_eq!(t.get_id_path(INVALID_ID), "");
        assert_eq!(t.get_id_path(42), "");
        assert!(t.add_id(42, "res://x.tres"));
        assert_eq!(t.get_id_path(INVALID_ID), "");
    }

    #[test]
    fn uid_helpers_and_creation() {
        let mut t = ResourceUidTable::new();
        t.add_id(5, "res://five.tres");
        assert_eq!(t.uid_to_path(&id_to_text(5)), "res://five.tres");
        assert_eq!(t.uid_to_path("uid://<invalid>"), "");
        assert_eq!(t.ensure_path(&id_to_text(5)), "res://five.tres");
        assert_eq!(t.ensure_path("res://plain.tres"), "res://plain.tres");

        // A colliding candidate is skipped, and values are masked.
        let mut candidates = vec![5u64, u64::MAX, 9].into_iter();
        let id = t.create_id(|| candidates.next().unwrap());
        assert_eq!(id, 0x7FFF_FFFF_FFFF_FFFF);

        assert!(t.add_id(id, "res://max.tres"));
        // Next candidate collides with 5, then 9 is free.
        let mut candidates = vec![5u64, 9].into_iter();
        assert_eq!(t.create_id(|| candidates.next().unwrap()), 9);
    }
}
