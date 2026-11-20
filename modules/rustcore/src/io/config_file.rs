//! `core/io/config_file.*`: INI-style configuration with Variant values.
//!
//! `parse` follows the engine's line protocol: `;` comments, `[section]`
//! headers (with `\]` unescaping), `key = value` assignments where the value is
//! a Variant (so arrays may span lines), and section-less keys stored under the
//! empty section name.
//!
//! Semantics ported from `ConfigFile`:
//!
//! - `set_value` with a `Nil` value erases the key and drops the section when it
//!   becomes empty;
//! - `get_value` falls back to the supplied default;
//! - `erase_section`/`erase_section_key` fail on missing sections/keys, with the
//!   key erase dropping an emptied section;
//! - `get_sections`/`get_section_keys` return the stored names.
//!
//! Saving (`encode_to_text`) is not ported.

use std::collections::BTreeMap;

use crate::io::variant_parser::{parse_prefix, ParseError};
use crate::variant::Variant;

#[derive(Clone, Debug, Default)]
pub struct ConfigFileData {
    /// Section name → key → value. The empty section holds keys written before
    /// any header.
    values: BTreeMap<String, BTreeMap<String, Variant>>,
}

impl ConfigFileData {
    pub fn new() -> Self {
        Self::default()
    }

    /// A `Nil` value erases the key (and the section when it becomes empty).
    pub fn set_value(&mut self, section: &str, key: &str, value: Variant) {
        if matches!(value, Variant::Nil) {
            let Some(entries) = self.values.get_mut(section) else {
                return;
            };
            entries.remove(key);
            if entries.is_empty() {
                self.values.remove(section);
            }
            return;
        }
        self.values
            .entry(section.to_string())
            .or_default()
            .insert(key.to_string(), value);
    }

    /// Returns the stored value, or `default` when the section/key is missing.
    pub fn get_value(&self, section: &str, key: &str, default: Variant) -> Variant {
        self.values
            .get(section)
            .and_then(|entries| entries.get(key))
            .cloned()
            .unwrap_or(default)
    }

    pub fn has_section(&self, section: &str) -> bool {
        self.values.contains_key(section)
    }

    pub fn has_section_key(&self, section: &str, key: &str) -> bool {
        self.values
            .get(section)
            .map(|entries| entries.contains_key(key))
            .unwrap_or(false)
    }

    pub fn get_sections(&self) -> Vec<String> {
        self.values.keys().cloned().collect()
    }

    pub fn get_section_keys(&self, section: &str) -> Vec<String> {
        self.values
            .get(section)
            .map(|entries| entries.keys().cloned().collect())
            .unwrap_or_default()
    }

    /// Fails on a missing section.
    pub fn erase_section(&mut self, section: &str) -> bool {
        self.values.remove(section).is_some()
    }

    /// Fails on a missing section or key; drops the section when it empties.
    pub fn erase_section_key(&mut self, section: &str, key: &str) -> bool {
        let Some(entries) = self.values.get_mut(section) else {
            return false;
        };
        if entries.remove(key).is_none() {
            return false;
        }
        if entries.is_empty() {
            self.values.remove(section);
        }
        true
    }

    pub fn clear(&mut self) {
        self.values.clear();
    }

    /// `ConfigFile::parse`: parse `data` into this instance (existing values are
    /// kept, like the engine).
    pub fn parse(&mut self, data: &str) -> Result<(), ParseError> {
        let chars: Vec<char> = data.chars().collect();
        let mut pos = 0usize;
        let mut line = 0usize;
        let mut section = String::new();

        while pos < chars.len() {
            let c = chars[pos];
            match c {
                '\n' => {
                    line += 1;
                    pos += 1;
                }
                c if c.is_whitespace() => pos += 1,
                ';' => {
                    while pos < chars.len() && chars[pos] != '\n' {
                        pos += 1;
                    }
                }
                '[' => {
                    pos += 1;
                    let mut name = String::new();
                    let mut closed = false;
                    while pos < chars.len() {
                        let ch = chars[pos];
                        if ch == '\n' {
                            break;
                        }
                        if ch == '\\' && pos + 1 < chars.len() && chars[pos + 1] == ']' {
                            name.push(']');
                            pos += 2;
                            continue;
                        }
                        if ch == ']' {
                            pos += 1;
                            closed = true;
                            break;
                        }
                        name.push(ch);
                        pos += 1;
                    }
                    if !closed {
                        return Err(ParseError {
                            message: "Expected ']'".to_string(),
                            line,
                        });
                    }
                    section = name;
                }
                _ => {
                    // `key = value` (the key ends at the first '=').
                    let mut key = String::new();
                    while pos < chars.len() {
                        let ch = chars[pos];
                        if ch == '=' {
                            break;
                        }
                        if ch == '\n' {
                            return Err(ParseError {
                                message: "Expected '='".to_string(),
                                line,
                            });
                        }
                        key.push(ch);
                        pos += 1;
                    }
                    if pos >= chars.len() || chars[pos] != '=' {
                        return Err(ParseError {
                            message: "Expected '='".to_string(),
                            line,
                        });
                    }
                    pos += 1; // '='

                    let rest: String = chars[pos..].iter().collect();
                    let (value, consumed) = parse_prefix(&rest)?;
                    line += rest[..consumed.min(rest.len())]
                        .chars()
                        .filter(|&c| c == '\n')
                        .count();
                    pos += consumed;
                    self.set_value(&section, key.trim(), value);
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string::gstring::GoString;

    fn s(text: &str) -> Variant {
        Variant::String(GoString::from_str(text))
    }

    #[test]
    fn sections_and_keys() {
        let mut c = ConfigFileData::new();
        c.parse(
            "; comment\n\
             [player]\n\
             name = \"Hero\"\n\
             lives = 3\n\
             ratio = 0.5\n\
             [input]\n\
             jump = true\n",
        )
        .unwrap();

        assert_eq!(c.get_sections(), vec!["input", "player"]);
        assert_eq!(c.get_section_keys("player"), vec!["lives", "name", "ratio"]);
        assert!(c.has_section("player"));
        assert!(c.has_section_key("player", "lives"));
        assert!(!c.has_section_key("player", "mana"));
        assert_eq!(c.get_value("player", "name", Variant::Nil), s("Hero"));
        assert_eq!(c.get_value("player", "lives", Variant::Nil), Variant::Int(3));
        assert_eq!(c.get_value("player", "ratio", Variant::Nil), Variant::Float(0.5));
        assert_eq!(c.get_value("input", "jump", Variant::Nil), Variant::Bool(true));
    }

    #[test]
    fn missing_values_use_the_default() {
        let mut c = ConfigFileData::new();
        c.parse("[a]\nk = 1\n").unwrap();
        assert_eq!(c.get_value("a", "missing", Variant::Int(9)), Variant::Int(9));
        assert_eq!(
            c.get_value("missing", "k", Variant::Int(4)),
            Variant::Int(4)
        );
        assert_eq!(c.get_value("a", "missing", Variant::Nil), Variant::Nil);
    }

    #[test]
    fn section_less_keys() {
        let mut c = ConfigFileData::new();
        c.parse("first = 1\n[section]\nsecond = 2\n").unwrap();
        assert!(c.has_section(""));
        assert_eq!(c.get_value("", "first", Variant::Nil), Variant::Int(1));
        assert_eq!(c.get_value("section", "second", Variant::Nil), Variant::Int(2));
    }

    #[test]
    fn values_may_span_lines() {
        let mut c = ConfigFileData::new();
        c.parse("[data]\nlist = [\n  1,\n  2,\n]\ndone = true\n")
            .unwrap();
        let value = c.get_value("data", "list", Variant::Nil);
        assert_eq!(value, Variant::Array(vec![Variant::Int(1), Variant::Int(2)]));
        assert_eq!(c.get_value("data", "done", Variant::Nil), Variant::Bool(true));
    }

    #[test]
    fn escaped_closing_bracket_in_section_names() {
        let mut c = ConfigFileData::new();
        c.parse("[weird\\]name]\nk = 1\n").unwrap();
        assert!(c.has_section("weird]name"));
        assert_eq!(c.get_value("weird]name", "k", Variant::Nil), Variant::Int(1));
    }

    #[test]
    fn set_value_erases_with_nil() {
        let mut c = ConfigFileData::new();
        c.set_value("a", "k", Variant::Int(1));
        c.set_value("a", "j", Variant::Int(2));
        assert!(c.has_section("a"));

        c.set_value("a", "k", Variant::Nil);
        assert!(!c.has_section_key("a", "k"));
        assert!(c.has_section("a"));

        c.set_value("a", "j", Variant::Nil);
        assert!(!c.has_section("a"), "emptied section is dropped");
        // Erasing from a missing section is a no-op.
        c.set_value("missing", "k", Variant::Nil);
    }

    #[test]
    fn erase_section_and_key() {
        let mut c = ConfigFileData::new();
        c.set_value("a", "k", Variant::Int(1));
        c.set_value("a", "j", Variant::Int(2));
        c.set_value("b", "k", Variant::Int(3));

        assert!(!c.erase_section("missing"));
        assert!(c.erase_section_key("a", "k"));
        assert!(c.erase_section_key("a", "j"));
        assert!(!c.has_section("a"), "section dropped when it empties");
        assert!(!c.erase_section_key("a", "j"));
        assert!(c.erase_section("b"));
        assert!(!c.has_section("b"));

        c.set_value("c", "x", Variant::Int(1));
        c.clear();
        assert!(c.get_sections().is_empty());
    }

    #[test]
    fn parse_errors() {
        let mut c = ConfigFileData::new();
        assert_eq!(
            c.parse("[unclosed\n").unwrap_err().message,
            "Expected ']'"
        );
        assert_eq!(c.parse("key without equals\n").unwrap_err().message, "Expected '='");
        assert_eq!(
            c.parse("[a]\nk = \n").unwrap_err().message,
            "Expected value, got EOF"
        );
        assert_eq!(
            c.parse("[a]\nk = \"x\n").unwrap_err().message,
            "Unterminated string"
        );
    }

    #[test]
    fn comments_and_whitespace() {
        let mut c = ConfigFileData::new();
        c.parse(
            "\n\n  ; leading comment\n\
             [ a ]  \n\
             k = 1 ; trailing comment\n\
             \n",
        )
        .unwrap();
        // Section names are not trimmed by the engine.
        assert!(!c.has_section("a"));
        assert!(c.has_section(" a "));
        assert_eq!(c.get_value(" a ", "k", Variant::Nil), Variant::Int(1));
    }
}
