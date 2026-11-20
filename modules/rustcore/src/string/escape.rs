//! `core/string/ustring.cpp`: escaping helpers.
//!
//! Ported exactly: `uri_encode` (unreserved = ASCII alphanumerics, `_`, `.`, `-`,
//! `~`; everything else `%XX` with upper-case hex), `uri_decode` (accepts only
//! upper-case hex after `%`, maps `+` to a space, and — matching the engine's
//! control flow — silently drops the `%` when the second hex digit is invalid),
//! `c_escape`/`c_unescape`, `json_escape`, `xml_escape`/`xml_unescape`
//! (named and numeric entities), `property_name_encode` and
//! `get_invalid_node_name_characters`.
//!
//! `c_escape_multiline` is exposed because `property_name_encode` uses it.

/// `is_ascii_identifier_char`: `[A-Za-z0-9_]`.
fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}

/// `String::uri_encode`.
pub fn uri_encode(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for &byte in input.as_bytes() {
        if byte == b'.' || byte == b'-' || byte == b'~' || is_identifier_byte(byte) {
            out.push(byte as char);
        } else {
            out.push('%');
            out.push_str(&format!("{byte:02X}"));
        }
    }
    out
}

fn hex_value(byte: u8) -> Option<u8> {
    match byte {
        b'0'..=b'9' => Some(byte - b'0'),
        b'A'..=b'F' => Some(byte - b'A' + 10),
        _ => None,
    }
}

fn is_accepted_hex(byte: u8) -> bool {
    byte.is_ascii_digit() || byte.is_ascii_uppercase()
}

/// `String::uri_decode`.
pub fn uri_decode(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0usize;
    while i < bytes.len() {
        let byte = bytes[i];
        if byte == b'%' && i + 2 < bytes.len() {
            let first = bytes[i + 1];
            if is_accepted_hex(first) {
                let second = bytes[i + 2];
                if is_accepted_hex(second) {
                    // Both look like hex: `hex_value` is Some for each.
                    let value = hex_value(first).unwrap_or(0) << 4 | hex_value(second).unwrap_or(0);
                    out.push(value);
                    i += 2;
                }
                // Otherwise the engine appends nothing for this byte.
            } else {
                out.push(byte);
            }
        } else if byte == b'+' {
            out.push(b' ');
        } else {
            out.push(byte);
        }
        i += 1;
    }
    String::from_utf8_lossy(&out).into_owned()
}

/// `String::c_escape`.
pub fn c_escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\u{7}' => out.push_str("\\a"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{b}' => out.push_str("\\v"),
            '\'' => out.push_str("\\'"),
            '"' => out.push_str("\\\""),
            other => out.push(other),
        }
    }
    out
}

/// `String::c_unescape`.
pub fn c_unescape(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len());
    let mut i = 0usize;
    while i < chars.len() {
        if chars[i] == '\\' && i + 1 < chars.len() {
            match chars[i + 1] {
                'a' => {
                    out.push('\u{7}');
                    i += 2;
                    continue;
                }
                'b' => {
                    out.push('\u{8}');
                    i += 2;
                    continue;
                }
                'f' => {
                    out.push('\u{c}');
                    i += 2;
                    continue;
                }
                'n' => {
                    out.push('\n');
                    i += 2;
                    continue;
                }
                'r' => {
                    out.push('\r');
                    i += 2;
                    continue;
                }
                't' => {
                    out.push('\t');
                    i += 2;
                    continue;
                }
                'v' => {
                    out.push('\u{b}');
                    i += 2;
                    continue;
                }
                '\'' => {
                    out.push('\'');
                    i += 2;
                    continue;
                }
                '"' => {
                    out.push('"');
                    i += 2;
                    continue;
                }
                '\\' => {
                    out.push('\\');
                    i += 2;
                    continue;
                }
                _ => {}
            }
        }
        out.push(chars[i]);
        i += 1;
    }
    out
}

/// `String::c_escape_multiline`: only the backslash and the double quote.
pub fn c_escape_multiline(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            other => out.push(other),
        }
    }
    out
}

/// `String::json_escape`.
pub fn json_escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '\u{8}' => out.push_str("\\b"),
            '\u{c}' => out.push_str("\\f"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{b}' => out.push_str("\\v"),
            '"' => out.push_str("\\\""),
            other => out.push(other),
        }
    }
    out
}

/// `String::xml_escape`.
pub fn xml_escape(input: &str, escape_quotes: bool) -> String {
    let mut out = String::with_capacity(input.len());
    for c in input.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '\'' if escape_quotes => out.push_str("&apos;"),
            '"' if escape_quotes => out.push_str("&quot;"),
            other => out.push(other),
        }
    }
    out
}

/// `String::xml_unescape`: named entities plus decimal and hex numeric ones.
/// Malformed entities are left untouched.
pub fn xml_unescape(input: &str) -> String {
    let chars: Vec<char> = input.chars().collect();
    let mut out = String::with_capacity(input.len());
    let len = chars.len();
    let mut i = 0usize;

    while i < len {
        if chars[i] != '&' {
            out.push(chars[i]);
            i += 1;
            continue;
        }

        let remaining = len - i;
        let mut eat = 1usize;
        let mut decoded: Option<char> = None;

        if remaining >= 4 && chars[i + 1] == '#' {
            let mut code: u32 = 0;
            let mut overflow = false;
            let mut terminator: Option<usize> = None;

            if chars[i + 2] == 'x' {
                // Hex entity `&#x<num>;`.
                for j in 3..remaining {
                    eat = j + 1;
                    let ct = chars[i + j];
                    if ct == ';' {
                        terminator = Some(j);
                        break;
                    }
                    let digit = if ct.is_ascii_digit() {
                        ct as u32 - '0' as u32
                    } else if ('a'..='f').contains(&ct) {
                        (ct as u32 - 'a' as u32) + 10
                    } else if ('A'..='F').contains(&ct) {
                        (ct as u32 - 'A' as u32) + 10
                    } else {
                        break;
                    };
                    if code > (u32::MAX >> 4) {
                        overflow = true;
                        break;
                    }
                    code <<= 4;
                    code |= digit;
                }
            } else {
                // Decimal entity `&#<num>;`.
                for j in 2..remaining {
                    eat = j + 1;
                    let ct = chars[i + j];
                    if ct == ';' || !ct.is_ascii_digit() {
                        break;
                    }
                }
                if chars[i + eat - 1] == ';' {
                    let digits: String = chars[i + 2..i + eat - 1].iter().collect();
                    match digits.parse::<i64>() {
                        Ok(value) if value > 0 && value <= u32::MAX as i64 => {
                            code = value as u32;
                            terminator = Some(eat - 1);
                        }
                        _ => overflow = true,
                    }
                }
            }

            let ends_with_semicolon =
                i + eat >= 1 && i + eat - 1 < len && chars[i + eat - 1] == ';';
            let _ = terminator;
            if code == 0 || overflow || !ends_with_semicolon {
                eat = 1;
                decoded = Some('&');
            } else {
                decoded = char::from_u32(code).or(Some('\u{fffd}'));
            }
        } else if remaining >= 4 && chars[i + 1] == 'g' && chars[i + 2] == 't' && chars[i + 3] == ';'
        {
            decoded = Some('>');
            eat = 4;
        } else if remaining >= 4 && chars[i + 1] == 'l' && chars[i + 2] == 't' && chars[i + 3] == ';'
        {
            decoded = Some('<');
            eat = 4;
        } else if remaining >= 5
            && chars[i + 1] == 'a'
            && chars[i + 2] == 'm'
            && chars[i + 3] == 'p'
            && chars[i + 4] == ';'
        {
            decoded = Some('&');
            eat = 5;
        } else if remaining >= 6
            && chars[i + 1] == 'q'
            && chars[i + 2] == 'u'
            && chars[i + 3] == 'o'
            && chars[i + 4] == 't'
            && chars[i + 5] == ';'
        {
            decoded = Some('"');
            eat = 6;
        } else if remaining >= 6
            && chars[i + 1] == 'a'
            && chars[i + 2] == 'p'
            && chars[i + 3] == 'o'
            && chars[i + 4] == 's'
            && chars[i + 5] == ';'
        {
            decoded = Some('\'');
            eat = 6;
        }

        if let Some(c) = decoded {
            out.push(c);
        } else {
            out.push('&');
        }
        i += eat;
    }

    out
}

/// `String::property_name_encode`.
pub fn property_name_encode(input: &str) -> String {
    let needs_quotes = input
        .chars()
        .any(|c| matches!(c, '=' | '"' | ';' | '[' | ']') || (c as u32) < 33 || (c as u32) > 126);
    if needs_quotes {
        format!("\"{}\"", c_escape_multiline(input))
    } else {
        input.to_string()
    }
}

/// `String::get_invalid_node_name_characters`.
pub fn get_invalid_node_name_characters(allow_internal: bool) -> String {
    let mut out = String::new();
    for c in ['.', ':', '@', '/', '"', '%'] {
        if allow_internal && c == '@' {
            continue;
        }
        if !out.is_empty() {
            out.push(' ');
        }
        out.push(c);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn uri_encoding() {
        assert_eq!(uri_encode("a b"), "a%20b");
        assert_eq!(uri_encode("a_b.c-d~e"), "a_b.c-d~e");
        assert_eq!(uri_encode("a/b?c"), "a%2Fb%3Fc");
        // UTF-8 bytes are percent-encoded one by one.
        assert_eq!(uri_encode("ñ"), "%C3%B1");
        assert_eq!(uri_encode(""), "");
    }

    #[test]
    fn uri_decoding_quirks() {
        assert_eq!(uri_decode("a%20b"), "a b");
        assert_eq!(uri_decode("a+b"), "a b");
        assert_eq!(uri_decode("%C3%B1"), "ñ");
        assert_eq!(uri_decode("plain"), "plain");

        // Lower-case hex is rejected: the `%` disappears but the digits stay.
        assert_eq!(uri_decode("%2f"), "2f");
        assert_eq!(uri_decode("%2F"), "/");
        // A non-hex first digit keeps the percent sign.
        assert_eq!(uri_decode("%zz"), "%zz");
        // A trailing `%` (no room for two digits) is copied.
        assert_eq!(uri_decode("a%"), "a%");
    }

    #[test]
    fn c_escaping_round_trips() {
        let raw = "a\nb\tc\\d\"e'f\u{7}g\u{8}h\u{c}i\rj\u{b}k";
        let escaped = c_escape(raw);
        assert_eq!(
            escaped,
            "a\\nb\\tc\\\\d\\\"e\\'f\\ag\\bh\\fi\\rj\\vk"
        );
        assert_eq!(c_unescape(&escaped), raw);

        // Unknown escapes stay literal.
        assert_eq!(c_unescape("\\q"), "\\q");
        assert_eq!(c_unescape("a\\"), "a\\");
    }

    #[test]
    fn json_and_multiline_escaping() {
        assert_eq!(json_escape("a\"b\\c\n"), "a\\\"b\\\\c\\n");
        assert_eq!(json_escape("\u{b}"), "\\v");
        assert_eq!(c_escape_multiline("a\"b\\c\n"), "a\\\"b\\\\c\n");
    }

    #[test]
    fn xml_escaping() {
        assert_eq!(
            xml_escape("<a&b>\"'", true),
            "&lt;a&amp;b&gt;&quot;&apos;"
        );
        assert_eq!(xml_escape("<a&b>\"'", false), "&lt;a&amp;b&gt;\"'");
    }

    #[test]
    fn xml_unescaping() {
        assert_eq!(xml_unescape("&lt;a&amp;b&gt;"), "<a&b>");
        assert_eq!(xml_unescape("&quot;x&apos;"), "\"x'");
        assert_eq!(xml_unescape("&#65;&#x42;"), "AB");
        // Round trip with the escaper.
        let raw = "<tag attr=\"v\">a & b</tag>";
        assert_eq!(xml_unescape(&xml_escape(raw, true)), raw);
        // Malformed entities are preserved.
        assert_eq!(xml_unescape("&bogus;"), "&bogus;");
        assert_eq!(xml_unescape("&#;"), "&#;");
        assert_eq!(xml_unescape("a & b"), "a & b");
        assert_eq!(xml_unescape("&#xZZ;"), "&#xZZ;");
        // Zero is invalid and left alone.
        assert_eq!(xml_unescape("&#0;"), "&#0;");
    }

    #[test]
    fn property_names_and_invalid_characters() {
        assert_eq!(property_name_encode("simple"), "simple");
        assert_eq!(property_name_encode("with space"), "\"with space\"");
        assert_eq!(property_name_encode("a=b"), "\"a=b\"");
        // A bare backslash does not force quoting (only the listed characters,
        // spaces and non-ASCII do).
        assert_eq!(property_name_encode("a\\b"), "a\\b");
        // Once quoting is forced, backslashes are escaped.
        assert_eq!(property_name_encode("a b\\c"), "\"a b\\\\c\"");
        // Extended ASCII forces quoting.
        assert_eq!(property_name_encode("ñ"), "\"ñ\"");

        assert_eq!(get_invalid_node_name_characters(false), ". : @ / \" %");
        assert_eq!(get_invalid_node_name_characters(true), ". : / \" %");
    }
}
