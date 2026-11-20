//! First unit of the `String` port: the copy-on-write buffer plus the core
//! operations that do not depend on the rest of the engine.
//!
//! Godot's `String` holds UTF-32 (`char32_t`) in a refcounted `CowData` buffer:
//! copies share storage until the first write, which unshares. In Rust that is
//! exactly `Arc<Vec<u32>>` + `Arc::make_mut`, so the observable behaviour is the
//! same and no unsafe refcounting is needed.
//!
//! Not ported yet (they need the engine tables or are behind `ustring.cpp`):
//! `String::num` (grisu2), locale-aware case conversion, and the `%` format
//! specifiers.
//!
//! NOTE: this file is ASCII-only on purpose. The source is UTF-8 and a
//! PowerShell `Set-Content` round-trip silently replaced non-ASCII literals,
//! so every non-ASCII character in tests is written as a `\u{...}` escape.

use std::sync::Arc;

use crate::templates::hashfuncs;

#[derive(Clone, Debug, Default)]
pub struct GoString {
    data: Arc<Vec<u32>>,
}

impl GoString {
    #[inline]
    pub fn new() -> Self {
        Self { data: Arc::new(Vec::new()) }
    }

    /// Builds from UTF-8 text.
    pub fn from_str(s: &str) -> Self {
        Self { data: Arc::new(s.chars().map(|c| c as u32).collect()) }
    }

    pub fn as_string(&self) -> String {
        self.data
            .iter()
            .map(|c| char::from_u32(*c).unwrap_or(char::REPLACEMENT_CHARACTER))
            .collect()
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.data.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    #[inline]
    pub fn get(&self, idx: usize) -> u32 {
        self.data[idx]
    }

    /// True when two handles share the same buffer, like equal `ptr()` values.
    #[inline]
    pub fn shares_buffer_with(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.data, &other.data)
    }

    #[inline]
    pub fn refcount(&self) -> usize {
        Arc::strong_count(&self.data)
    }

    /// Mutable access that unshares first, mirroring `ptrw()`.
    #[inline]
    fn data_mut(&mut self) -> &mut Vec<u32> {
        Arc::make_mut(&mut self.data)
    }

    #[inline]
    pub fn push(&mut self, c: u32) {
        self.data_mut().push(c);
    }

    pub fn append(&mut self, other: &Self) {
        let extra = other.data.clone();
        self.data_mut().extend_from_slice(&extra);
    }

    #[inline]
    pub fn concatenated(&self, other: &Self) -> Self {
        let mut out = self.clone();
        out.append(other);
        out
    }

    pub fn find(&self, needle: &Self, from: usize) -> isize {
        let n = needle.len();
        if n == 0 {
            return -1;
        }
        if n > self.len() || from > self.len() - n {
            return -1;
        }
        for i in from..=(self.len() - n) {
            if self.data[i..i + n] == needle.data[..] {
                return i as isize;
            }
        }
        -1
    }

    pub fn rfind(&self, needle: &Self, from: isize) -> isize {
        let n = needle.len();
        if n == 0 || n > self.len() {
            return -1;
        }
        let start = if from < 0 {
            self.len() - n
        } else {
            (from as usize).min(self.len() - n)
        };
        for i in (0..=start).rev() {
            if self.data[i..i + n] == needle.data[..] {
                return i as isize;
            }
        }
        -1
    }

    /// `substr(from, len)` with the engine's clamping (negative runs to the end).
    pub fn substr(&self, from: usize, len: isize) -> Self {
        if from >= self.len() {
            return Self::new();
        }
        let end = if len < 0 {
            self.len()
        } else {
            (from + len as usize).min(self.len())
        };
        Self { data: Arc::new(self.data[from..end].to_vec()) }
    }

    #[inline]
    pub fn begins_with(&self, other: &Self) -> bool {
        other.len() <= self.len() && self.data[..other.len()] == other.data[..]
    }

    #[inline]
    pub fn ends_with(&self, other: &Self) -> bool {
        other.len() <= self.len() && self.data[self.len() - other.len()..] == other.data[..]
    }

    /// ASCII-only case folding, as in `char_utils`' ASCII helpers.
    pub fn to_upper_ascii(&self) -> Self {
        Self {
            data: Arc::new(
                self.data
                    .iter()
                    .map(|c| if (0x61..=0x7a).contains(c) { c - 32 } else { *c })
                    .collect(),
            ),
        }
    }

    pub fn to_lower_ascii(&self) -> Self {
        Self {
            data: Arc::new(
                self.data
                    .iter()
                    .map(|c| if (0x41..=0x5a).contains(c) { c + 32 } else { *c })
                    .collect(),
            ),
        }
    }

    /// Builds from UTF-32 units, as the engine stores them.
    pub fn from_units(units: &[u32]) -> Self {
        Self { data: Arc::new(units.to_vec()) }
    }

    /// Unicode-aware upper case (`String::to_upper`).
    ///
    /// The engine routes this through its locale/ICU path; here it uses the
    /// Unicode mappings compiled into `char`, which agree on the standard
    /// (locale-independent) mappings. **Non-differential**: `ustring.cpp`
    /// cannot be linked in the harness. Locale-specific behaviour (e.g. the
    /// Turkish dotted/dotless i) is not modelled.
    ///
    /// Note that some mappings change length: "strasse" stays 7 units but
    /// "strasse" with U+00DF becomes "STRASSE" (8).
    pub fn to_upper(&self) -> Self {
        let mut out = Vec::with_capacity(self.data.len());
        for c in self.data.iter() {
            match char::from_u32(*c) {
                Some(ch) => out.extend(ch.to_uppercase().map(|u| u as u32)),
                None => out.push(*c),
            }
        }
        Self::from_units(&out)
    }

    /// Unicode-aware lower case (`String::to_lower`), same caveats as
    /// `to_upper`.
    pub fn to_lower(&self) -> Self {
        let mut out = Vec::with_capacity(self.data.len());
        for c in self.data.iter() {
            match char::from_u32(*c) {
                Some(ch) => out.extend(ch.to_lowercase().map(|u| u as u32)),
                None => out.push(*c),
            }
        }
        Self::from_units(&out)
    }

    /// UTF-8 bytes of the string.
    ///
    /// Units outside the Unicode scalar range (surrogates) become U+FFFD here;
    /// the engine's exact `append_utf8` fallback is still pending verification.
    pub fn to_utf8(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.data.len());
        for c in self.data.iter() {
            match char::from_u32(*c) {
                Some(ch) => {
                    let mut buf = [0u8; 4];
                    out.extend_from_slice(ch.encode_utf8(&mut buf).as_bytes());
                }
                None => out.extend_from_slice("\u{FFFD}".as_bytes()),
            }
        }
        out
    }

    /// Builds from UTF-8 bytes (`String::utf8`); invalid input is replaced.
    pub fn from_utf8(bytes: &[u8]) -> Self {
        match std::str::from_utf8(bytes) {
            Ok(s) => Self::from_str(s),
            Err(_) => Self::from_str(&String::from_utf8_lossy(bytes)),
        }
    }

    /// UTF-16 code units, surrogate pairs included.
    pub fn to_utf16(&self) -> Vec<u16> {
        let mut out = Vec::with_capacity(self.data.len());
        for c in self.data.iter() {
            match char::from_u32(*c) {
                Some(ch) => {
                    let mut buf = [0u16; 2];
                    out.extend_from_slice(ch.encode_utf16(&mut buf));
                }
                None => out.push(0xFFFD),
            }
        }
        out
    }

    pub fn from_utf16(units: &[u16]) -> Self {
        Self::from_str(&String::from_utf16_lossy(units))
    }

    /// `String::strip_edges`: removes leading/trailing whitespace.
    pub fn strip_edges(&self) -> Self {
        let is_ws = |c: u32| {
            crate::string::char_utils::is_whitespace(c) || crate::string::char_utils::is_linebreak(c)
        };
        let start = self.data.iter().position(|c| !is_ws(*c));
        let end = self.data.iter().rposition(|c| !is_ws(*c));
        match (start, end) {
            (Some(s), Some(e)) => Self::from_units(&self.data[s..=e]),
            _ => Self::new(),
        }
    }

    /// `String::split`: empty separators yield the whole string.
    pub fn split(&self, sep: &Self) -> Vec<Self> {
        if sep.is_empty() {
            return vec![self.clone()];
        }
        let mut out = Vec::new();
        let n = sep.len();
        let mut start = 0usize;
        let mut i = 0usize;
        while i + n <= self.len() {
            if self.data[i..i + n] == sep.data[..] {
                out.push(Self::from_units(&self.data[start..i]));
                start = i + n;
                i = start;
            } else {
                i += 1;
            }
        }
        out.push(Self::from_units(&self.data[start..]));
        out
    }

    /// `String::replace(old, new)` with non-empty `old`.
    pub fn replace(&self, old: &Self, new: &Self) -> Self {
        if old.is_empty() {
            return self.clone();
        }
        let mut out: Vec<u32> = Vec::new();
        let n = old.len();
        let mut i = 0usize;
        while i < self.len() {
            if i + n <= self.len() && self.data[i..i + n] == old.data[..] {
                out.extend_from_slice(&new.data);
                i += n;
            } else {
                out.push(self.data[i]);
                i += 1;
            }
        }
        Self::from_units(&out)
    }

    /// Number of non-overlapping occurrences.
    pub fn count(&self, needle: &Self) -> usize {
        if needle.is_empty() {
            return 0;
        }
        let mut total = 0;
        let mut i = 0usize;
        let n = needle.len();
        while i + n <= self.len() {
            if self.data[i..i + n] == needle.data[..] {
                total += 1;
                i += n;
            } else {
                i += 1;
            }
        }
        total
    }

    /// `String::to_int`: parses the leading integer, 0 when there is none.
    pub fn to_int(&self) -> i64 {
        let text = self.as_string();
        let digits: String = text
            .chars()
            .take_while(|c| c.is_ascii_digit() || *c == '-' || *c == '+')
            .collect();
        digits.parse().unwrap_or(0)
    }

    /// `String::is_valid_int`.
    pub fn is_valid_int(&self) -> bool {
        let text = self.as_string();
        let t = text.trim();
        !t.is_empty() && t.parse::<i64>().is_ok()
    }

    /// `String::join`: concatenates `parts` with this string as separator.
    pub fn join(&self, parts: &[Self]) -> Self {
        let mut out = Vec::new();
        for (i, p) in parts.iter().enumerate() {
            if i > 0 {
                out.extend_from_slice(&self.data);
            }
            out.extend_from_slice(&p.data);
        }
        Self::from_units(&out)
    }

    /// `String::is_valid_float`.
    pub fn is_valid_float(&self) -> bool {
        let text = self.as_string();
        let t = text.trim();
        !t.is_empty() && t.parse::<f64>().is_ok()
    }

    /// `String::to_float`: parses the longest numeric prefix, 0 when none.
    pub fn to_float(&self) -> f64 {
        let text = self.as_string();
        let mut end = 0usize;
        let mut seen_dot = false;
        for (i, c) in text.char_indices() {
            let ok = match c {
                '-' | '+' => i == 0,
                '.' => !seen_dot,
                _ => c.is_ascii_digit(),
            };
            if !ok {
                break;
            }
            if c == '.' {
                seen_dot = true;
            }
            end = i + c.len_utf8();
        }
        text[..end].parse().unwrap_or(0.0)
    }

    /// `find_char`: index of the first occurrence of `c`, -1 when absent.
    pub fn find_char(&self, c: char) -> isize {
        let needle = c as u32;
        match self.data.iter().position(|&u| u == needle) {
            Some(i) => i as isize,
            None => -1,
        }
    }

    /// `insert(pos, text)`, with `pos` clamped to the string bounds.
    pub fn insert(&self, pos: usize, text: &Self) -> Self {
        let pos = pos.min(self.len());
        let mut data = Vec::with_capacity(self.len() + text.len());
        data.extend_from_slice(&self.data[..pos]);
        data.extend_from_slice(&text.data);
        data.extend_from_slice(&self.data[pos..]);
        Self { data: Arc::new(data) }
    }

    /// `repeat(count)`: empty for `count <= 0`.
    pub fn repeat(&self, count: i32) -> Self {
        if count <= 0 || self.is_empty() {
            return Self::new();
        }
        let mut data = Vec::with_capacity(self.len() * count as usize);
        for _ in 0..count {
            data.extend_from_slice(&self.data);
        }
        Self { data: Arc::new(data) }
    }

    /// `pad_decimals`: truncate or zero-pad the fractional part to `digits`.
    pub fn pad_decimals(&self, digits: i32) -> Self {
        let s = self.clone();
        let mut c = s.find_char('.');

        if c == -1 {
            if digits <= 0 {
                return s;
            }
            let s = s.concatenated(&Self::from_str("."));
            c = s.len() as isize - 1;
            return pad_decimals_at(&s, c, digits);
        }

        if digits <= 0 {
            return s.substr(0, c);
        }
        pad_decimals_at(&s, c, digits)
    }

    /// `pad_zeros`: zero-pad the integer part to `digits` digits, keeping the
    /// sign and any fractional part in place.
    pub fn pad_zeros(&self, digits: i32) -> Self {
        let s = self.clone();
        let mut end = s.find_char('.');
        if end == -1 {
            end = s.len() as isize;
        }
        if end == 0 {
            return s;
        }

        let mut begin = 0isize;
        while begin < end && !(0x30..=0x39).contains(&s.data[begin as usize]) {
            begin += 1;
        }

        let zeros = digits - (end - begin) as i32;
        if zeros <= 0 {
            s
        } else {
            s.insert(begin as usize, &Self::from_str("0").repeat(zeros))
        }
    }

    /// `lpad`: left-pad with `character` (repeated) up to `min_length`.
    pub fn lpad(&self, min_length: i32, character: &Self) -> Self {
        let padding = min_length - self.len() as i32;
        if padding > 0 {
            character.repeat(padding).concatenated(self)
        } else {
            self.clone()
        }
    }

    /// `rpad`: right-pad with `character` (repeated) up to `min_length`.
    pub fn rpad(&self, min_length: i32, character: &Self) -> Self {
        let padding = min_length - self.len() as i32;
        if padding > 0 {
            self.concatenated(&character.repeat(padding))
        } else {
            self.clone()
        }
    }

    /// `String::hash()` -- MurmurHash3 over the UTF-32 units, little-endian.
    pub fn hash(&self) -> u32 {
        let mut bytes = Vec::with_capacity(self.data.len() * 4);
        for c in self.data.iter() {
            bytes.extend_from_slice(&c.to_le_bytes());
        }
        hashfuncs::hash_murmur3_buffer(&bytes, hashfuncs::HASH_MURMUR3_SEED)
    }
}

impl PartialEq for GoString {
    fn eq(&self, other: &Self) -> bool {
        self.data == other.data
    }
}
impl Eq for GoString {}

impl std::fmt::Display for GoString {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_string())
    }
}

/// The shared tail of `pad_decimals`, operating on a string that already has a
/// decimal point at index `c`.
fn pad_decimals_at(s: &GoString, c: isize, digits: i32) -> GoString {
    let decimals = s.len() as i32 - (c as i32 + 1);
    if decimals > digits {
        s.substr(0, c + digits as isize + 1)
    } else {
        s.concatenated(&GoString::from_str("0").repeat(digits - decimals))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(v: &str) -> GoString {
        GoString::from_str(v)
    }

    #[test]
    fn padding_helpers() {
        assert_eq!(s("3.1").pad_decimals(3).as_string(), "3.100");
        assert_eq!(s("3.14159").pad_decimals(2).as_string(), "3.14");
        assert_eq!(s("3.14159").pad_decimals(0).as_string(), "3");
        // No dot and no decimals requested: unchanged.
        assert_eq!(s("7").pad_decimals(0).as_string(), "7");
        // No dot but decimals requested: the dot is added first.
        assert_eq!(s("7").pad_decimals(2).as_string(), "7.00");

        assert_eq!(s("42").pad_zeros(5).as_string(), "00042");
        // Sign and fractional part stay in place.
        assert_eq!(s("-42").pad_zeros(5).as_string(), "-00042");
        assert_eq!(s("3.5").pad_zeros(4).as_string(), "0003.5");
        // Already long enough: unchanged.
        assert_eq!(s("12345").pad_zeros(3).as_string(), "12345");

        assert_eq!(s("ab").lpad(5, &s("0")).as_string(), "000ab");
        assert_eq!(s("ab").rpad(5, &s("-")).as_string(), "ab---");
        assert_eq!(s("abcdef").lpad(3, &s("0")).as_string(), "abcdef");
        // The engine repeats the padding string `min_length - len` times, so a
        // multi-character pad overshoots the requested length.
        assert_eq!(s("x").rpad(5, &s("ab")).as_string(), "xabababab");
        // Negative counts give an empty repeat, never a panic.
        assert_eq!(s("").repeat(-3).as_string(), "");
        assert_eq!(s("ab").repeat(0).as_string(), "");
    }

    #[test]
    fn copy_on_write_shares_then_unshares() {
        let a = s("hello");
        let mut b = a.clone();
        assert!(a.shares_buffer_with(&b));
        assert_eq!(a.refcount(), 2);

        b.push(0x21); // '!'
        assert!(!a.shares_buffer_with(&b));
        assert_eq!(a.as_string(), "hello");
        assert_eq!(b.as_string(), "hello!");
    }

    #[test]
    fn append_to_shared_does_not_alias() {
        let base = s("abc");
        let mut x = base.clone();
        let y = base.clone();
        x.append(&y);
        assert_eq!(x.as_string(), "abcabc");
        assert_eq!(y.as_string(), "abc");
        assert_eq!(base.as_string(), "abc");
    }

    #[test]
    fn search_and_slices() {
        let text = s("godot_rust_port");
        assert_eq!(text.find(&s("rust"), 0), 6);
        assert_eq!(text.find(&s("rust"), 7), -1);
        assert_eq!(text.rfind(&s("o"), -1), 12); // last "o" of "port"
        assert_eq!(text.substr(6, 4).as_string(), "rust");
        assert_eq!(text.substr(6, -1).as_string(), "rust_port");
        assert_eq!(text.substr(999, 2).as_string(), "");
        assert!(text.begins_with(&s("godot")));
        assert!(text.ends_with(&s("port")));
        assert!(!text.begins_with(&s("rust")));
    }

    #[test]
    fn unicode_case_mapping() {
        // U+00DF (sharp s) maps to "SS": length can grow.
        let sharp = s("\u{DF}");
        assert_eq!(sharp.to_upper().len(), 2);
        assert_eq!(sharp.to_upper().as_string(), "SS");

        // U+00E9 -> U+00C9, U+00FC -> U+00DC.
        assert_eq!(s("caf\u{E9}").to_upper().as_string(), "CAF\u{C9}");
        assert_eq!(s("M\u{DC}NCHEN").to_lower().as_string(), "m\u{FC}nchen");

        // Non-BMP: U+10428 (deseret small) -> U+10400.
        let d = s("\u{10428}");
        assert_eq!(d.to_upper().len(), 1);
        assert_eq!(d.to_upper().get(0), 0x10400);

        // Digestive symbols unaffected by case.
        assert_eq!(s("123!?").to_upper().as_string(), "123!?");
        assert!(GoString::new().to_upper().is_empty());
    }

    #[test]
    fn ascii_case_leaves_unicode_alone() {
        // U+00D1 = N with tilde, U+00FA = u with acute.
        let mixed = s("Godot \u{D1}and\u{FA}");
        assert_eq!(mixed.to_upper_ascii().as_string(), "GODOT \u{D1}AND\u{FA}");
        assert_eq!(mixed.to_lower_ascii().as_string(), "godot \u{D1}and\u{FA}");
    }

    #[test]
    fn utf_roundtrips_and_boundaries() {
        for text in ["", "abc", "\u{F1}and\u{FA}", "\u{65E5}\u{672C}\u{8A9E}", "\u{1F3AE}", "\u{10FFFF}"] {
            let v = GoString::from_str(text);
            assert_eq!(GoString::from_utf8(&v.to_utf8()), v, "utf8 {text}");
            assert_eq!(GoString::from_utf16(&v.to_utf16()), v, "utf16 {text}");
        }

        // A non-BMP character takes 2 UTF-16 units and 4 UTF-8 bytes.
        assert_eq!(GoString::from_str("\u{1F3AE}").to_utf16().len(), 2);
        assert_eq!(GoString::from_str("\u{1F3AE}").to_utf8().len(), 4);
        assert_eq!(GoString::from_str("\u{65E5}").to_utf16().len(), 1);

        // U+10FFFF is the last valid scalar.
        let last = GoString::from_str("\u{10FFFF}");
        assert_eq!(last.to_utf8(), vec![0xF4, 0x8F, 0xBF, 0xBF]);
        assert_eq!(last.to_utf16().len(), 2);

        // Invalid UTF-8 must not panic.
        let bad = GoString::from_utf8(&[0xFF, 0xFE]);
        assert_eq!(bad.len(), 2);
    }

    #[test]
    fn from_units_roundtrip() {
        let units = [0x41u32, 0x1F3AE, 0x10FFFF];
        let s = GoString::from_units(&units);
        assert_eq!(s.len(), 3);
        assert_eq!(s.get(1), 0x1F3AE);
    }

    #[test]
    fn empty_and_length() {
        let e = GoString::new();
        assert!(e.is_empty());
        assert_eq!(e.find(&s("a"), 0), -1);
        assert_eq!(e.substr(0, 5), GoString::new());
        assert_eq!(s("\u{E1}\u{E9}\u{ED}").len(), 3);
    }

    #[test]
    fn strip_split_replace_join() {
        let padded = s("  \t hello \n ");
        assert_eq!(padded.strip_edges().as_string(), "hello");
        assert_eq!(s("   ").strip_edges(), GoString::new());

        let csv = s("a,bb,,c");
        let parts: Vec<String> = csv.split(&s(",")).iter().map(|p| p.as_string()).collect();
        assert_eq!(parts, vec!["a", "bb", "", "c"]);
        // Empty separator yields the whole string.
        assert_eq!(csv.split(&GoString::new()).len(), 1);

        assert_eq!(s("a-b-c").replace(&s("-"), &s("+")).as_string(), "a+b+c");
        assert_eq!(s("aaa").replace(&s("aa"), &s("b")).as_string(), "ba");
        assert_eq!(s("aaa").count(&s("a")), 3);
        assert_eq!(s("aaaa").count(&s("aa")), 2); // Non-overlapping.
        assert_eq!(s("x").count(&GoString::new()), 0);

        let joined = s("-").join(&[s("a"), s("b"), s("c")]);
        assert_eq!(joined.as_string(), "a-b-c");
    }

    #[test]
    fn int_parsing() {
        assert_eq!(s("42").to_int(), 42);
        assert_eq!(s("-7abc").to_int(), -7);
        assert_eq!(s("abc").to_int(), 0);
        assert_eq!(s("").to_int(), 0);

        assert!(s(" 123 ").is_valid_int());
        assert!(s("-9").is_valid_int());
        assert!(!s("12.5").is_valid_int());
        assert!(!s("").is_valid_int());
        assert!(!s("abc").is_valid_int());

        assert!((s("3.5").to_float() - 3.5).abs() < 1e-9);
        assert!((s("-2.25rest").to_float() + 2.25).abs() < 1e-9);
        assert_eq!(s("abc").to_float(), 0.0);
        assert!(s(" 1.5 ").is_valid_float());
        assert!(!s("1.2.3").is_valid_float());
        assert!(!s("").is_valid_float());
    }

    #[test]
    fn hash_is_stable_and_distinguishes() {
        assert_eq!(s("abc").hash(), s("abc").hash());
        assert_ne!(s("abc").hash(), s("abd").hash());
        assert_ne!(s("abc").hash(), 0);
    }
}
