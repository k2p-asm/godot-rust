//! Port of `core/string/char_utils.h` — the character classification layer that
//! `String` sits on. This is the first slice of Phase 2.
//!
//! Ported: every table-free predicate (`is_digit`, `is_hex_digit`, `is_symbol`,
//! `is_whitespace`, `is_punct`, ...).
//! Pending: `is_unicode_identifier_start/continue`, `is_unicode_upper_case`,
//! `is_unicode_lower_case`, `is_unicode_letter`. Those are binary searches over
//! generated range tables in `char_utils.cpp` (thousands of entries); they must
//! be ported with the table data, not by hand.

#[inline]
pub const fn is_ascii_upper_case(c: u32) -> bool {
    c >= b'A' as u32 && c <= b'Z' as u32
}

#[inline]
pub const fn is_ascii_lower_case(c: u32) -> bool {
    c >= b'a' as u32 && c <= b'z' as u32
}

#[inline]
pub const fn is_digit(c: u32) -> bool {
    c >= b'0' as u32 && c <= b'9' as u32
}

#[inline]
pub const fn is_hex_digit(c: u32) -> bool {
    is_digit(c) || (c >= b'a' as u32 && c <= b'f' as u32) || (c >= b'A' as u32 && c <= b'F' as u32)
}

#[inline]
pub const fn is_binary_digit(c: u32) -> bool {
    c == b'0' as u32 || c == b'1' as u32
}

#[inline]
pub const fn is_ascii_alphabet_char(c: u32) -> bool {
    is_ascii_lower_case(c) || is_ascii_upper_case(c)
}

#[inline]
pub const fn is_ascii_alphanumeric_char(c: u32) -> bool {
    is_ascii_alphabet_char(c) || is_digit(c)
}

#[inline]
pub const fn is_ascii_identifier_char(c: u32) -> bool {
    is_ascii_alphanumeric_char(c) || c == b'_' as u32
}

#[inline]
pub const fn is_symbol(c: u32) -> bool {
    c != b'_' as u32
        && ((c >= b'!' as u32 && c <= b'/' as u32)
            || (c >= b':' as u32 && c <= b'@' as u32)
            || (c >= b'[' as u32 && c <= b'`' as u32)
            || (c >= b'{' as u32 && c <= b'~' as u32)
            || c == 0x09
            || c == 0x20)
}

#[inline]
pub const fn is_control(c: u32) -> bool {
    c <= 0x001f || (c >= 0x007f && c <= 0x009f)
}

#[inline]
pub const fn is_whitespace(c: u32) -> bool {
    c == 0x20
        || c == 0x00a0
        || c == 0x1680
        || (c >= 0x2000 && c <= 0x200b)
        || c == 0x202f
        || c == 0x205f
        || c == 0x3000
        || c == 0x2028
        || c == 0x2029
        || (c >= 0x0009 && c <= 0x000d)
        || c == 0x0085
}

#[inline]
pub const fn is_linebreak(c: u32) -> bool {
    (c >= 0x000a && c <= 0x000d) || c == 0x0085 || c == 0x2028 || c == 0x2029
}

#[inline]
pub const fn is_punct(c: u32) -> bool {
    (c >= 0x20 && c <= 0x2f)
        || (c >= 0x3a && c <= 0x40)
        || (c >= 0x5b && c <= 0x5e)
        || c == 0x60
        || (c >= 0x7b && c <= 0x7e)
        || (c >= 0x2000 && c <= 0x206f)
        || (c >= 0x3000 && c <= 0x303f)
}

#[inline]
pub const fn is_underscore(c: u32) -> bool {
    c == b'_' as u32
}

#[inline]
pub const fn is_hyphen(c: u32) -> bool {
    c == b'-' as u32 || c == 0x2010 || c == 0x2011
}

/// Binary search over a sorted range table, matching `BSEARCH_CHAR_RANGE`.
#[inline]
fn in_ranges(table: &[(u32, u32)], c: u32) -> bool {
    let mut low: i64 = 0;
    let mut high: i64 = table.len() as i64 - 1;
    let mut middle = (low + high) / 2;

    while low <= high {
        let (start, end) = table[middle as usize];
        if c < start {
            high = middle - 1;
        } else if c > end {
            low = middle + 1;
        } else {
            return true;
        }
        middle = (low + high) / 2;
    }

    false
}

#[inline]
pub fn is_unicode_identifier_start(c: u32) -> bool {
    in_ranges(&crate::string::char_range::XID_START, c)
}

#[inline]
pub fn is_unicode_identifier_continue(c: u32) -> bool {
    in_ranges(&crate::string::char_range::XID_CONTINUE, c)
}

#[inline]
pub fn is_unicode_upper_case(c: u32) -> bool {
    in_ranges(&crate::string::char_range::UPPERCASE_LETTER, c)
}

#[inline]
pub fn is_unicode_lower_case(c: u32) -> bool {
    in_ranges(&crate::string::char_range::LOWERCASE_LETTER, c)
}

#[inline]
pub fn is_unicode_letter(c: u32) -> bool {
    in_ranges(&crate::string::char_range::UNICODE_LETTER, c)
}
pub const HEX_CHAR_TABLE_UPPER: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'A', 'B', 'C', 'D', 'E', 'F',
];
pub const HEX_CHAR_TABLE_LOWER: [char; 16] = [
    '0', '1', '2', '3', '4', '5', '6', '7', '8', '9', 'a', 'b', 'c', 'd', 'e', 'f',
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ascii_classes() {
        assert!(is_digit(b'7' as u32));
        assert!(!is_digit(b'a' as u32));
        assert!(is_hex_digit(b'F' as u32));
        assert!(is_hex_digit(b'f' as u32));
        assert!(!is_hex_digit(b'g' as u32));
        assert!(is_ascii_identifier_char(b'_' as u32));
        assert!(is_ascii_alphanumeric_char(b'9' as u32));
        assert!(!is_symbol(b'_' as u32));
        assert!(is_symbol(b'@' as u32));
        assert!(is_symbol(b' ' as u32));
    }

    #[test]
    fn unicode_classes() {
        assert!(is_whitespace(0x00a0));
        assert!(is_whitespace(0x3000));
        assert!(!is_whitespace(0x0041));
        assert!(is_linebreak(0x2028));
        assert!(!is_linebreak(0x0020));
        assert!(is_control(0x001f));
        assert!(is_control(0x0085));
        assert!(!is_control(0x0020));
        assert!(is_punct(0x2014)); // em dash
        assert!(is_hyphen(0x2010));
        assert!(is_underscore(b'_' as u32));
    }
}
