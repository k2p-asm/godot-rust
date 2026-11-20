//! `core/io/json.*`: the JSON parser and stringifier.
//!
//! Parsing follows `JSON::_get_token` / `_parse_value`: numbers always become
//! `Float`, object keys are strings, the four `\uXXXX` escapes (including
//! surrogate pairs) are decoded, and the error message/line pairs match the
//! engine's strings. Recursion is capped at `Variant::MAX_RECURSION_DEPTH`
//! (1024).
//!
//! Stringifying follows `JSON::_stringify`: no indentation by default, keys
//! sorted, floats printed with `max(1, 14 - floor(log10(|v|)))` decimals (so
//! `1.0` is `1.00000000000000`), `0.0` special-cased to `0.0`, infinities to
//! `1e99999`/`-1e99999` and NaN to `null`.
//!
//! Not ported: the `Resource` wrapper, `from_native`/`to_native`, circular
//! markers (the port's containers cannot form cycles) and `keep_text`.

use crate::string::gstring::GoString;
use crate::string::num::num_float;
use crate::string::string_name::StringName;
use crate::variant::compare;
use crate::variant::Variant;
use std::collections::HashMap;

/// `Variant::MAX_RECURSION_DEPTH`.
pub const MAX_RECURSION_DEPTH: usize = 1024;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct JsonError {
    pub message: String,
    pub line: usize,
}

impl std::fmt::Display for JsonError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} (line {})", self.message, self.line)
    }
}

struct Parser {
    chars: Vec<char>,
    pos: usize,
    line: usize,
}

impl Parser {
    fn error<T>(&self, message: impl Into<String>) -> Result<T, JsonError> {
        Err(JsonError {
            message: message.into(),
            line: self.line,
        })
    }

    fn peek(&self) -> Option<char> {
        self.chars.get(self.pos).copied()
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.peek()?;
        self.pos += 1;
        if c == '\n' {
            self.line += 1;
        }
        Some(c)
    }

    fn skip_whitespace(&mut self) {
        while let Some(c) = self.peek() {
            if c == ' ' || c == '\t' || c == '\r' || c == '\n' {
                self.bump();
            } else {
                break;
            }
        }
    }

    fn parse_value(&mut self, depth: usize) -> Result<Variant, JsonError> {
        if depth > MAX_RECURSION_DEPTH {
            return self.error("JSON structure is too deep");
        }
        self.skip_whitespace();
        let Some(c) = self.peek() else {
            return self.error("Expected value, got 'EOF'");
        };
        match c {
            '{' => self.parse_object(depth),
            '[' => self.parse_array(depth),
            '"' => {
                let s = self.parse_string()?;
                Ok(Variant::String(s))
            }
            't' | 'f' | 'n' => {
                let start = self.pos;
                while matches!(self.peek(), Some(ch) if ch.is_ascii_alphabetic()) {
                    self.bump();
                }
                let word: String = self.chars[start..self.pos].iter().collect();
                match word.as_str() {
                    "true" => Ok(Variant::Bool(true)),
                    "false" => Ok(Variant::Bool(false)),
                    "null" => Ok(Variant::Nil),
                    _ => self.error(format!("Expected 'true', 'false', or 'null', got '{word}'")),
                }
            }
            '-' | '0'..='9' => self.parse_number(),
            other => self.error(format!("Expected value, got '{other}'")),
        }
    }

    fn parse_number(&mut self) -> Result<Variant, JsonError> {
        let start = self.pos;
        if self.peek() == Some('-') {
            self.bump();
        }
        let mut digits = 0;
        while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
            self.bump();
            digits += 1;
        }
        if self.peek() == Some('.') {
            self.bump();
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.bump();
                digits += 1;
            }
        }
        if matches!(self.peek(), Some('e') | Some('E')) {
            self.bump();
            if matches!(self.peek(), Some('+') | Some('-')) {
                self.bump();
            }
            let mut exp_digits = 0;
            while matches!(self.peek(), Some(c) if c.is_ascii_digit()) {
                self.bump();
                exp_digits += 1;
            }
            if exp_digits == 0 {
                return self.error("Expected value, got 'number'");
            }
        }
        if digits == 0 {
            return self.error("Expected value, got 'number'");
        }
        let text: String = self.chars[start..self.pos].iter().collect();
        match text.parse::<f64>() {
            Ok(value) => Ok(Variant::Float(value)),
            Err(_) => self.error("Expected value, got 'number'"),
        }
    }

    fn parse_string(&mut self) -> Result<GoString, JsonError> {
        debug_assert_eq!(self.peek(), Some('"'));
        self.bump();

        let mut out = String::new();
        loop {
            let Some(c) = self.peek() else {
                return self.error("Unterminated string");
            };
            if c == '"' {
                self.bump();
                break;
            }
            if c == '\\' {
                self.bump();
                let Some(next) = self.peek() else {
                    return self.error("Unterminated string");
                };
                self.bump();
                match next {
                    'b' => out.push('\u{8}'),
                    't' => out.push('\t'),
                    'n' => out.push('\n'),
                    'f' => out.push('\u{c}'),
                    'r' => out.push('\r'),
                    '"' => out.push('"'),
                    '\\' => out.push('\\'),
                    '/' => out.push('/'),
                    'u' => {
                        let mut res = self.parse_hex4()?;
                        if (res & 0xfffffc00) == 0xd800 {
                            // Lead surrogate: a trail escape must follow.
                            if self.peek() != Some('\\') {
                                return self
                                    .error("Invalid UTF-16 sequence in string, unpaired lead surrogate");
                            }
                            self.bump();
                            if self.peek() != Some('u') {
                                return self
                                    .error("Invalid UTF-16 sequence in string, unpaired lead surrogate");
                            }
                            self.bump();
                            let trail = self.parse_hex4()?;
                            if (trail & 0xfffffc00) == 0xdc00 {
                                res = (res << 10) + trail
                                    - ((0xd800u32 << 10) + 0xdc00 - 0x10000);
                            } else {
                                return self
                                    .error("Invalid UTF-16 sequence in string, unpaired lead surrogate");
                            }
                        } else if (res & 0xfffffc00) == 0xdc00 {
                            return self
                                .error("Invalid UTF-16 sequence in string, unpaired trail surrogate");
                        }
                        match char::from_u32(res) {
                            Some(ch) => out.push(ch),
                            None => out.push('\u{fffd}'),
                        }
                    }
                    _ => return self.error("Invalid escape sequence"),
                }
            } else {
                self.bump();
                out.push(c);
            }
        }
        Ok(GoString::from_str(&out))
    }

    fn parse_hex4(&mut self) -> Result<u32, JsonError> {
        let mut res = 0u32;
        for _ in 0..4 {
            let Some(c) = self.peek() else {
                return self.error("Unterminated string");
            };
            if !c.is_ascii_hexdigit() {
                return self.error("Malformed hex constant in string");
            }
            self.bump();
            res <<= 4;
            res |= c.to_digit(16).unwrap_or(0);
        }
        Ok(res)
    }

    fn parse_array(&mut self, depth: usize) -> Result<Variant, JsonError> {
        self.bump(); // '['
        let mut values = Vec::new();
        self.skip_whitespace();
        if self.peek() == Some(']') {
            self.bump();
            return Ok(Variant::Array(values));
        }
        loop {
            values.push(self.parse_value(depth + 1)?);
            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.bump();
                }
                Some(']') => {
                    self.bump();
                    break;
                }
                _ => return self.error("Expected ','"),
            }
        }
        Ok(Variant::Array(values))
    }

    fn parse_object(&mut self, depth: usize) -> Result<Variant, JsonError> {
        self.bump(); // '{'
        let mut map: HashMap<StringName, Variant> = HashMap::new();
        self.skip_whitespace();
        if self.peek() == Some('}') {
            self.bump();
            return Ok(Variant::Dictionary(map));
        }
        loop {
            self.skip_whitespace();
            if self.peek() != Some('"') {
                return self.error("Expected key");
            }
            let key = self.parse_string()?;
            self.skip_whitespace();
            if self.peek() != Some(':') {
                return self.error("Expected ':'");
            }
            self.bump();
            let value = self.parse_value(depth + 1)?;
            map.insert(StringName::new(&key.as_string()), value);
            self.skip_whitespace();
            match self.peek() {
                Some(',') => {
                    self.bump();
                }
                Some('}') => {
                    self.bump();
                    break;
                }
                _ => return self.error("Expected '}' or ','"),
            }
        }
        Ok(Variant::Dictionary(map))
    }
}

/// `JSON::parse_string`, returning the error instead of a null variant.
pub fn parse(text: &str) -> Result<Variant, JsonError> {
    let mut parser = Parser {
        chars: text.chars().collect(),
        pos: 0,
        line: 0,
    };
    let value = parser.parse_value(0)?;
    parser.skip_whitespace();
    if parser.peek().is_some() {
        return Err(JsonError {
            message: "Expected 'EOF'".to_string(),
            line: parser.line,
        });
    }
    Ok(value)
}

fn json_escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
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

fn add_indent(out: &mut String, indent: &str, level: usize) {
    for _ in 0..level {
        out.push_str(indent);
    }
}

fn number_to_json(num: f64) -> String {
    if !num.is_finite() {
        if num == f64::INFINITY {
            return "1e99999".to_string();
        }
        if num == f64::NEG_INFINITY {
            return "-1e99999".to_string();
        }
        return "null".to_string();
    }
    if num == 0.0 {
        return "0.0".to_string();
    }

    let magnitude = num.abs().log10().floor() as i32;
    let precision = (14 - magnitude).max(1).min(32) as usize;
    // The engine routes through `String::num(num, precision)`, a fixed-point
    // format that keeps trailing zeros.
    num_float(num, precision as i32).as_string()
}

fn stringify_value(
    out: &mut String,
    value: &Variant,
    indent: &str,
    level: usize,
    sort_keys: bool,
) {
    let colon = if indent.is_empty() { ":" } else { ": " };
    let end_statement = if indent.is_empty() { "" } else { "\n" };

    match value {
        Variant::Nil => out.push_str("null"),
        Variant::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
        Variant::Int(i) => out.push_str(&i.to_string()),
        Variant::Float(f) => out.push_str(&number_to_json(*f)),
        Variant::String(s) => {
            out.push('"');
            out.push_str(&json_escape(&s.as_string()));
            out.push('"');
        }
        Variant::Array(values) => {
            if values.is_empty() {
                out.push_str("[]");
                return;
            }
            out.push('[');
            out.push_str(end_statement);
            for (index, v) in values.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                    out.push_str(end_statement);
                }
                add_indent(out, indent, level + 1);
                stringify_value(out, v, indent, level + 1, sort_keys);
            }
            out.push_str(end_statement);
            add_indent(out, indent, level);
            out.push(']');
        }
        Variant::Dictionary(map) => {
            if map.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push('{');
            out.push_str(end_statement);

            let mut keys: Vec<&StringName> = map.keys().collect();
            if sort_keys {
                keys.sort_by(|a, b| a.as_string().cmp(&b.as_string()));
            }

            for (index, key) in keys.iter().enumerate() {
                if index > 0 {
                    out.push(',');
                    out.push_str(end_statement);
                }
                add_indent(out, indent, level + 1);
                out.push('"');
                out.push_str(&json_escape(&key.as_string()));
                out.push('"');
                out.push_str(colon);
                stringify_value(out, &map[*key], indent, level + 1, sort_keys);
            }
            out.push_str(end_statement);
            add_indent(out, indent, level);
            out.push('}');
        }
        // `JSON::_stringify`'s default branch: stringify then quote/escape.
        other => {
            out.push('"');
            out.push_str(&json_escape(&compare::stringify(other).as_string()));
            out.push('"');
        }
    }
}

/// `JSON::stringify` with its defaults (empty indent, sorted keys).
pub fn stringify(value: &Variant) -> String {
    stringify_with(value, "", true)
}

pub fn stringify_with(value: &Variant, indent: &str, sort_keys: bool) -> String {
    let mut out = String::new();
    stringify_value(&mut out, value, indent, 0, sort_keys);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(text: &str) -> Variant {
        Variant::String(GoString::from_str(text))
    }

    fn obj(pairs: &[(&str, Variant)]) -> Variant {
        let mut map = HashMap::new();
        for (k, v) in pairs {
            map.insert(StringName::new(k), v.clone());
        }
        Variant::Dictionary(map)
    }

    #[test]
    fn parses_scalars_arrays_and_objects() {
        assert_eq!(parse("null").unwrap(), Variant::Nil);
        assert_eq!(parse(" true ").unwrap(), Variant::Bool(true));
        assert_eq!(parse("false").unwrap(), Variant::Bool(false));
        // Numbers are always floats in Godot's JSON.
        assert_eq!(parse("42").unwrap(), Variant::Float(42.0));
        assert_eq!(parse("-1.5e2").unwrap(), Variant::Float(-150.0));
        assert_eq!(parse("\"hi\"").unwrap(), s("hi"));

        let array = parse("[1, [true], \"x\"]").unwrap();
        let Variant::Array(values) = &array else {
            panic!("not an array")
        };
        assert_eq!(values.len(), 3);
        assert_eq!(values[0], Variant::Float(1.0));
        assert_eq!(values[2], s("x"));

        let object = parse("{\"b\": 2, \"a\": [1]}").unwrap();
        assert_eq!(object.dict_get("b"), Some(&Variant::Float(2.0)));
        let Variant::Array(a) = object.dict_get("a").unwrap() else {
            panic!("not an array")
        };
        assert_eq!(a[0], Variant::Float(1.0));
    }

    #[test]
    fn string_escapes_and_surrogates() {
        assert_eq!(parse(r#""a\nb\t\"c\\d""#).unwrap(), s("a\nb\t\"c\\d"));
        assert_eq!(parse(r#""\u00e9""#).unwrap(), s("é"));
        // U+1F600 as a surrogate pair.
        assert_eq!(parse(r#""\ud83d\ude00""#).unwrap(), s("😀"));

        let err = parse(r#""\ud83d""#).unwrap_err();
        assert_eq!(err.message, "Invalid UTF-16 sequence in string, unpaired lead surrogate");
        let err = parse(r#""\ude00""#).unwrap_err();
        assert_eq!(err.message, "Invalid UTF-16 sequence in string, unpaired trail surrogate");
        let err = parse(r#""\u12g4""#).unwrap_err();
        assert_eq!(err.message, "Malformed hex constant in string");
        let err = parse(r#""abc"#).unwrap_err();
        assert_eq!(err.message, "Unterminated string");
        let err = parse(r#""\q""#).unwrap_err();
        assert_eq!(err.message, "Invalid escape sequence");
    }

    #[test]
    fn syntax_errors_carry_the_line() {
        let e = parse("{\n \"a\" 1}").unwrap_err();
        assert_eq!(e.message, "Expected ':'");
        assert_eq!(e.line, 1);

        assert_eq!(parse("[1 2]").unwrap_err().message, "Expected ','");
        assert_eq!(parse("{\"a\":1").unwrap_err().message, "Expected '}' or ','");
        assert_eq!(parse("{1:2}").unwrap_err().message, "Expected key");
        assert_eq!(parse("tru").unwrap_err().message, "Expected 'true', 'false', or 'null', got 'tru'");
        assert_eq!(parse("x").unwrap_err().message, "Expected value, got 'x'");
        assert_eq!(parse("[1] x").unwrap_err().message, "Expected 'EOF'");
        assert_eq!(parse("").unwrap_err().message, "Expected value, got 'EOF'");
    }

    #[test]
    fn recursion_is_capped() {
        let deep = "[".repeat(MAX_RECURSION_DEPTH + 2);
        let err = parse(&deep).unwrap_err();
        assert_eq!(err.message, "JSON structure is too deep");
    }

    #[test]
    fn stringify_scalars() {
        assert_eq!(stringify(&Variant::Nil), "null");
        assert_eq!(stringify(&Variant::Bool(true)), "true");
        assert_eq!(stringify(&Variant::Int(-7)), "-7");
        assert_eq!(stringify(&Variant::Float(0.0)), "0.0");
        assert_eq!(stringify(&Variant::Float(1.0)), "1.00000000000000");
        assert_eq!(stringify(&Variant::Float(1.5)), "1.50000000000000");
        assert_eq!(stringify(&Variant::Float(f64::INFINITY)), "1e99999");
        assert_eq!(stringify(&Variant::Float(f64::NEG_INFINITY)), "-1e99999");
        assert_eq!(stringify(&Variant::Float(f64::NAN)), "null");
        assert_eq!(stringify(&s("a\"b\\c\n")), "\"a\\\"b\\\\c\\n\"");
        // Non-JSON variants fall back to their string form, quoted.
        assert_eq!(stringify(&Variant::Vector2(crate::math::vector2::Vector2::new(1.0, 2.0))), "\"(1, 2)\"");
    }

    #[test]
    fn stringify_collections_sorted_and_indented() {
        let value = obj(&[
            ("b", Variant::Float(2.0)),
            ("a", Variant::Array(vec![Variant::Int(1), Variant::Bool(true)])),
        ]);

        // Compact, keys sorted a before b.
        let compact = stringify(&value);
        assert_eq!(
            compact,
            "{\"a\":[1,true],\"b\":2.00000000000000}"
        );

        // Unsorted keeps insertion order of the HashMap, so sort only when asked;
        // the two-key case is checked through the sorted path above.
        let indented = stringify_with(&value, "  ", true);
        assert_eq!(
            indented,
            "{\n  \"a\": [\n    1,\n    true\n  ],\n  \"b\": 2.00000000000000\n}"
        );

        assert_eq!(stringify(&Variant::Array(vec![])), "[]");
        assert_eq!(stringify(&obj(&[])), "{}");
    }

    #[test]
    fn round_trip() {
        let text = "{\"a\":[1,-2.5,true,null,\"x\\ny\"],\"b\":{\"c\":0}}";
        let value = parse(text).unwrap();
        let again = parse(&stringify(&value)).unwrap();
        assert_eq!(value, again);
    }
}
