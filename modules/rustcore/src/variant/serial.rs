//! Minimal `VariantWriter` / `VariantParser` for the POD payloads.
//!
//! The engine's serializers live in `core/variant/variant_parser.cpp` and depend
//! on `String`, so they cannot be linked into the parity harness. This module is
//! **non-differential**: it defines a small, self-consistent text format and
//! proves round-trips with unit tests.
//!
//! Format (one value per document):
//! `null`, `true`, `false`, `42`, `1.5`, `"text"`, `Vector2(1, 2)`,
//! `Vector3(1, 2, 3)`, `Color(1, 0.5, 0.25, 1)`, `[a, b]`, `{key: value, ...}`.

use std::collections::HashMap;

use crate::math::color::Color;
use crate::math::vector2::Vector2;
use crate::math::vector3::Vector3;
use crate::string::gstring::GoString;
use crate::string::string_name::StringName;
use crate::variant::Variant;

/// Serializes a variant to the text format above.
pub fn to_text(v: &Variant) -> String {
    match v {
        Variant::Nil => "null".to_string(),
        Variant::Bool(b) => if *b { "true" } else { "false" }.to_string(),
        Variant::Int(i) => i.to_string(),
        Variant::Float(f) => format!("{}", *f as f32),
        Variant::String(s) => {
            let mut out = String::from("\"");
            for c in s.as_string().chars() {
                match c {
                    '"' => out.push_str("\\\""),
                    '\\' => out.push_str("\\\\"),
                    '\n' => out.push_str("\\n"),
                    _ => out.push(c),
                }
            }
            out.push('"');
            out
        }
        Variant::Vector2(v2) => format!("Vector2({}, {})", v2.x as f32, v2.y as f32),
        Variant::Vector3(v3) => format!("Vector3({}, {}, {})", v3.x as f32, v3.y as f32, v3.z as f32),
        Variant::Color(c) => format!(
            "Color({}, {}, {}, {})",
            c.r as f32, c.g as f32, c.b as f32, c.a as f32
        ),
        Variant::Array(a) => {
            let parts: Vec<String> = a.iter().map(to_text).collect();
            format!("[{}]", parts.join(", "))
        }
        Variant::Dictionary(d) => {
            let mut keys: Vec<&StringName> = d.keys().collect();
            keys.sort_by_key(|k| k.as_string());
            let parts: Vec<String> = keys
                .iter()
                .map(|k| format!("{}: {}", k.as_string(), to_text(&d[*k])))
                .collect();
            format!("{{{}}}", parts.join(", "))
        }
        // Unsupported payloads serialize to null rather than erroring.
        _ => "null".to_string(),
    }
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

impl<'a> Parser<'a> {
    fn skip_ws(&mut self) {
        while self.i < self.s.len() && (self.s[self.i] as char).is_whitespace() {
            self.i += 1;
        }
    }

    fn eat(&mut self, c: u8) -> bool {
        self.skip_ws();
        if self.i < self.s.len() && self.s[self.i] == c {
            self.i += 1;
            true
        } else {
            false
        }
    }

    fn ident(&mut self) -> String {
        self.skip_ws();
        let start = self.i;
        while self.i < self.s.len() {
            let c = self.s[self.i] as char;
            if c.is_ascii_alphanumeric() || c == '_' {
                self.i += 1;
            } else {
                break;
            }
        }
        String::from_utf8_lossy(&self.s[start..self.i]).to_string()
    }

    fn number(&mut self) -> f64 {
        self.skip_ws();
        let start = self.i;
        while self.i < self.s.len() {
            let c = self.s[self.i] as char;
            if c.is_ascii_digit() || c == '-' || c == '+' || c == '.' || c == 'e' || c == 'E' {
                self.i += 1;
            } else {
                break;
            }
        }
        String::from_utf8_lossy(&self.s[start..self.i])
            .parse()
            .unwrap_or(0.0)
    }

    fn string(&mut self) -> String {
        // Assumes the opening quote was consumed.
        let mut out = String::new();
        while self.i < self.s.len() {
            let b = self.s[self.i];
            self.i += 1;
            match b {
                b'"' => break,
                b'\\' if self.i < self.s.len() => {
                    let esc = self.s[self.i];
                    self.i += 1;
                    out.push(match esc {
                        b'n' => '\n',
                        b'\\' => '\\',
                        b'"' => '"',
                        other => other as char,
                    });
                }
                other => out.push(other as char),
            }
        }
        out
    }

    fn value(&mut self) -> Variant {
        self.skip_ws();
        if self.i >= self.s.len() {
            return Variant::Nil;
        }

        match self.s[self.i] {
            b'"' => {
                self.i += 1;
                Variant::String(GoString::from_str(&self.string()))
            }
            b'[' => {
                self.i += 1;
                let mut items = Vec::new();
                if !self.eat(b']') {
                    loop {
                        items.push(self.value());
                        if self.eat(b',') {
                            continue;
                        }
                        self.eat(b']');
                        break;
                    }
                }
                Variant::Array(items)
            }
            b'{' => {
                self.i += 1;
                let mut map = HashMap::new();
                if !self.eat(b'}') {
                    loop {
                        let key = self.ident();
                        self.eat(b':');
                        let val = self.value();
                        map.insert(StringName::new(&key), val);
                        if self.eat(b',') {
                            continue;
                        }
                        self.eat(b'}');
                        break;
                    }
                }
                Variant::Dictionary(map)
            }
            c if c == b'-' || c.is_ascii_digit() => {
                let n = self.number();
                if n.fract() == 0.0 && !self.slice_has_dot() {
                    Variant::Int(n as i64)
                } else {
                    Variant::Float(n)
                }
            }
            _ => {
                let word = self.ident();
                match word.as_str() {
                    "null" => Variant::Nil,
                    "true" => Variant::Bool(true),
                    "false" => Variant::Bool(false),
                    "Vector2" | "Vector3" | "Color" => {
                        self.eat(b'(');
                        let mut nums = Vec::new();
                        loop {
                            nums.push(self.number());
                            if self.eat(b',') {
                                continue;
                            }
                            self.eat(b')');
                            break;
                        }
                        match word.as_str() {
                            "Vector2" => Variant::Vector2(Vector2::new(
                                nums.first().copied().unwrap_or(0.0) as f32,
                                nums.get(1).copied().unwrap_or(0.0) as f32,
                            )),
                            "Vector3" => Variant::Vector3(Vector3::new(
                                nums.first().copied().unwrap_or(0.0) as f32,
                                nums.get(1).copied().unwrap_or(0.0) as f32,
                                nums.get(2).copied().unwrap_or(0.0) as f32,
                            )),
                            _ => Variant::Color(Color::new(
                                nums.first().copied().unwrap_or(0.0) as f32,
                                nums.get(1).copied().unwrap_or(0.0) as f32,
                                nums.get(2).copied().unwrap_or(0.0) as f32,
                                nums.get(3).copied().unwrap_or(0.0) as f32,
                            )),
                        }
                    }
                    _ => Variant::Nil,
                }
            }
        }
    }

    /// True when the number just parsed contained a decimal point.
    fn slice_has_dot(&self) -> bool {
        let mut j = self.i;
        while j > 0 {
            j -= 1;
            match self.s[j] {
                b'.' => return true,
                b',' | b']' | b'}' | b'(' | b')' | b' ' => break,
                _ => {}
            }
        }
        false
    }
}

/// Parses the text format above; unknown syntax yields `Variant::Nil`.
pub fn from_text(text: &str) -> Variant {
    let mut p = Parser { s: text.as_bytes(), i: 0 };
    p.value()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn roundtrip(v: Variant) {
        let text = to_text(&v);
        let back = from_text(&text);
        assert_eq!(back, v, "text was: {text}");
    }

    #[test]
    fn scalars_roundtrip() {
        roundtrip(Variant::Nil);
        roundtrip(Variant::Bool(true));
        roundtrip(Variant::Bool(false));
        roundtrip(Variant::Int(42));
        roundtrip(Variant::Int(-7));
        roundtrip(Variant::Float(1.5));
        roundtrip(Variant::String(GoString::from_str("hello")));
    }

    #[test]
    fn strings_escape_specials() {
        let s = Variant::String(GoString::from_str("a\"b\\c\nd"));
        let text = to_text(&s);
        assert!(text.contains("\\\""));
        roundtrip(s);
    }

    #[test]
    fn math_types_roundtrip() {
        roundtrip(Variant::Vector2(Vector2::new(1.5, -2.25)));
        roundtrip(Variant::Vector3(Vector3::new(1.0, 2.0, 3.0)));
        roundtrip(Variant::Color(Color::new(1.0, 0.5, 0.25, 1.0)));
    }

    #[test]
    fn nested_containers_roundtrip() {
        let mut dict = HashMap::new();
        dict.insert(StringName::new("name"), Variant::String(GoString::from_str("hero")));
        dict.insert(StringName::new("hp"), Variant::Int(100));
        dict.insert(
            StringName::new("pos"),
            Variant::Vector3(Vector3::new(1.0, 0.0, -1.0)),
        );

        let v = Variant::Array(vec![
            Variant::Dictionary(dict),
            Variant::Array(vec![Variant::Int(1), Variant::Bool(true)]),
            Variant::Float(0.25),
        ]);
        roundtrip(v);
    }

    #[test]
    fn unknown_payloads_become_null() {
        let text = to_text(&Variant::Basis(crate::math::basis::Basis::IDENTITY));
        assert_eq!(text, "null");
        assert_eq!(from_text("nonsense"), Variant::Nil);
    }
}
