//! `scene/resources/theme.*`: the theme item store.
//!
//! Ported data types: colours, constants and font sizes, which are the ones
//! expressible without the `Font`/`Texture2D`/`StyleBox` object layer. Each type
//! is stored per theme type (`Button`, `LineEdit`, ...) exactly like the
//! engine's nested maps, with the same validation:
//!
//! - type names may be empty but must otherwise be ASCII identifiers
//!   (`[A-Za-z0-9_]`); item names must additionally be non-empty;
//! - `validate_type_name` strips the edges and replaces invalid characters;
//! - `rename_*` fails when the type is missing, the new name exists, or the old
//!   name is missing; `clear_*` fails when either is missing;
//! - font sizes are only returned/has when `> 0`, and fall back to the theme's
//!   default font size and then to `ThemeDB`'s fallback of 16.
//!
//! Type variations are also ported, including the engine's rejections (empty
//! type/base, and a type that is a built-in class) and the recursive variation
//! list.
//!
//! Not ported: icons, styleboxes, fonts (object-backed) and the change signals.

use crate::math::color::Color;
use crate::object::class_db;
use crate::string::string_name::StringName;
use std::collections::{HashMap, HashSet};

/// `ThemeDB`'s fallback font size.
pub const FALLBACK_FONT_SIZE: i32 = 16;

#[inline]
fn is_ascii_identifier_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '_'
}

pub fn is_valid_type_name(name: &str) -> bool {
    name.chars().all(is_ascii_identifier_char)
}

pub fn is_valid_item_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(is_ascii_identifier_char)
}

/// Strips the edges and replaces invalid characters with `_`.
pub fn validate_type_name(name: &str) -> String {
    name.trim()
        .chars()
        .map(|c| if is_ascii_identifier_char(c) { c } else { '_' })
        .collect()
}

#[derive(Debug)]
struct TypeMap<T> {
    map: HashMap<StringName, HashMap<StringName, T>>,
}

impl<T: Copy> Default for TypeMap<T> {
    fn default() -> Self {
        Self {
            map: HashMap::new(),
        }
    }
}

impl<T: Copy> TypeMap<T> {
    fn set(&mut self, name: &str, theme_type: &str, value: T) -> bool {
        if !is_valid_item_name(name) || !is_valid_type_name(theme_type) {
            return false;
        }
        self.map
            .entry(StringName::new(theme_type))
            .or_default()
            .insert(StringName::new(name), value);
        true
    }

    fn get(&self, name: &str, theme_type: &str) -> Option<T> {
        self.map
            .get(&StringName::new(theme_type))
            .and_then(|m| m.get(&StringName::new(name)))
            .copied()
    }

    fn has(&self, name: &str, theme_type: &str) -> bool {
        self.get(name, theme_type).is_some()
    }

    fn rename(&mut self, old_name: &str, new_name: &str, theme_type: &str) -> bool {
        if !is_valid_item_name(new_name) || !is_valid_type_name(theme_type) {
            return false;
        }
        let type_key = StringName::new(theme_type);
        let Some(items) = self.map.get_mut(&type_key) else {
            return false;
        };
        let new_key = StringName::new(new_name);
        if items.contains_key(&new_key) {
            return false;
        }
        let old_key = StringName::new(old_name);
        let Some(value) = items.remove(&old_key) else {
            return false;
        };
        items.insert(new_key, value);
        true
    }

    fn clear(&mut self, name: &str, theme_type: &str) -> bool {
        let type_key = StringName::new(theme_type);
        match self.map.get_mut(&type_key) {
            Some(items) => items.remove(&StringName::new(name)).is_some(),
            None => false,
        }
    }

    fn item_list(&self, theme_type: &str) -> Vec<StringName> {
        let mut names: Vec<StringName> = self
            .map
            .get(&StringName::new(theme_type))
            .map(|m| m.keys().copied().collect())
            .unwrap_or_default();
        names.sort_by(|a, b| a.as_string().cmp(&b.as_string()));
        names
    }

    fn add_type(&mut self, theme_type: &str) -> bool {
        if !is_valid_type_name(theme_type) {
            return false;
        }
        self.map.entry(StringName::new(theme_type)).or_default();
        true
    }

    fn remove_type(&mut self, theme_type: &str) -> bool {
        self.map.remove(&StringName::new(theme_type)).is_some()
    }

    fn rename_type(&mut self, old_type: &str, new_type: &str) -> bool {
        if !is_valid_type_name(new_type) {
            return false;
        }
        let old_key = StringName::new(old_type);
        let new_key = StringName::new(new_type);
        if !self.map.contains_key(&old_key) || self.map.contains_key(&new_key) {
            return false;
        }
        let items = self.map.remove(&old_key).expect("checked above");
        self.map.insert(new_key, items);
        true
    }

    fn type_list(&self) -> Vec<StringName> {
        let mut names: Vec<StringName> = self.map.keys().copied().collect();
        names.sort_by(|a, b| a.as_string().cmp(&b.as_string()));
        names
    }
}

#[derive(Debug, Default)]
pub struct ThemeData {
    colors: TypeMap<Color>,
    constants: TypeMap<i32>,
    font_sizes: TypeMap<i32>,
    variation_map: HashMap<StringName, StringName>,
    variation_base_map: HashMap<StringName, Vec<StringName>>,
    default_base_scale: f32,
    default_font_size: i32,
}

impl ThemeData {
    pub fn new() -> Self {
        Self {
            colors: TypeMap::default(),
            constants: TypeMap::default(),
            font_sizes: TypeMap::default(),
            variation_map: HashMap::new(),
            variation_base_map: HashMap::new(),
            default_base_scale: 0.0,
            default_font_size: -1,
        }
    }

    // -- defaults -----------------------------------------------------------

    pub fn set_default_base_scale(&mut self, base_scale: f32) {
        self.default_base_scale = base_scale;
    }

    pub fn get_default_base_scale(&self) -> f32 {
        self.default_base_scale
    }

    /// Present only when the scale is above zero.
    pub fn has_default_base_scale(&self) -> bool {
        self.default_base_scale > 0.0
    }

    pub fn set_default_font_size(&mut self, font_size: i32) {
        self.default_font_size = font_size;
    }

    pub fn get_default_font_size(&self) -> i32 {
        self.default_font_size
    }

    /// Present only when the size is above zero.
    pub fn has_default_font_size(&self) -> bool {
        self.default_font_size > 0
    }

    // -- colours ------------------------------------------------------------

    pub fn set_color(&mut self, name: &str, theme_type: &str, color: Color) -> bool {
        self.colors.set(name, theme_type, color)
    }

    /// Missing colours read as `Color()`, like the engine.
    pub fn get_color(&self, name: &str, theme_type: &str) -> Color {
        self.colors.get(name, theme_type).unwrap_or(Color::ZERO)
    }

    pub fn has_color(&self, name: &str, theme_type: &str) -> bool {
        self.colors.has(name, theme_type)
    }

    pub fn rename_color(&mut self, old_name: &str, new_name: &str, theme_type: &str) -> bool {
        self.colors.rename(old_name, new_name, theme_type)
    }

    pub fn clear_color(&mut self, name: &str, theme_type: &str) -> bool {
        self.colors.clear(name, theme_type)
    }

    pub fn get_color_list(&self, theme_type: &str) -> Vec<StringName> {
        self.colors.item_list(theme_type)
    }

    pub fn add_color_type(&mut self, theme_type: &str) -> bool {
        self.colors.add_type(theme_type)
    }

    pub fn remove_color_type(&mut self, theme_type: &str) -> bool {
        self.colors.remove_type(theme_type)
    }

    pub fn rename_color_type(&mut self, old_type: &str, new_type: &str) -> bool {
        self.colors.rename_type(old_type, new_type)
    }

    pub fn get_color_type_list(&self) -> Vec<StringName> {
        self.colors.type_list()
    }

    // -- constants ----------------------------------------------------------

    pub fn set_constant(&mut self, name: &str, theme_type: &str, constant: i32) -> bool {
        self.constants.set(name, theme_type, constant)
    }

    /// Missing constants read as 0.
    pub fn get_constant(&self, name: &str, theme_type: &str) -> i32 {
        self.constants.get(name, theme_type).unwrap_or(0)
    }

    pub fn has_constant(&self, name: &str, theme_type: &str) -> bool {
        self.constants.has(name, theme_type)
    }

    pub fn rename_constant(&mut self, old_name: &str, new_name: &str, theme_type: &str) -> bool {
        self.constants.rename(old_name, new_name, theme_type)
    }

    pub fn clear_constant(&mut self, name: &str, theme_type: &str) -> bool {
        self.constants.clear(name, theme_type)
    }

    pub fn get_constant_list(&self, theme_type: &str) -> Vec<StringName> {
        self.constants.item_list(theme_type)
    }

    pub fn add_constant_type(&mut self, theme_type: &str) -> bool {
        self.constants.add_type(theme_type)
    }

    pub fn remove_constant_type(&mut self, theme_type: &str) -> bool {
        self.constants.remove_type(theme_type)
    }

    pub fn rename_constant_type(&mut self, old_type: &str, new_type: &str) -> bool {
        self.constants.rename_type(old_type, new_type)
    }

    pub fn get_constant_type_list(&self) -> Vec<StringName> {
        self.constants.type_list()
    }

    // -- font sizes ---------------------------------------------------------

    pub fn set_font_size(&mut self, name: &str, theme_type: &str, font_size: i32) -> bool {
        self.font_sizes.set(name, theme_type, font_size)
    }

    /// Stored value above zero, then the default, then `FALLBACK_FONT_SIZE`.
    pub fn get_font_size(&self, name: &str, theme_type: &str) -> i32 {
        match self.font_sizes.get(name, theme_type) {
            Some(size) if size > 0 => size,
            _ if self.has_default_font_size() => self.default_font_size,
            _ => FALLBACK_FONT_SIZE,
        }
    }

    /// True when a positive size exists or a default is configured.
    pub fn has_font_size(&self, name: &str, theme_type: &str) -> bool {
        matches!(self.font_sizes.get(name, theme_type), Some(size) if size > 0)
            || self.has_default_font_size()
    }

    /// Presence without the default and without the `> 0` rule.
    pub fn has_font_size_nocheck(&self, name: &str, theme_type: &str) -> bool {
        self.font_sizes.has(name, theme_type)
    }

    pub fn rename_font_size(&mut self, old_name: &str, new_name: &str, theme_type: &str) -> bool {
        self.font_sizes.rename(old_name, new_name, theme_type)
    }

    pub fn clear_font_size(&mut self, name: &str, theme_type: &str) -> bool {
        self.font_sizes.clear(name, theme_type)
    }

    pub fn get_font_size_list(&self, theme_type: &str) -> Vec<StringName> {
        self.font_sizes.item_list(theme_type)
    }

    pub fn add_font_size_type(&mut self, theme_type: &str) -> bool {
        self.font_sizes.add_type(theme_type)
    }

    pub fn remove_font_size_type(&mut self, theme_type: &str) -> bool {
        self.font_sizes.remove_type(theme_type)
    }

    pub fn rename_font_size_type(&mut self, old_type: &str, new_type: &str) -> bool {
        self.font_sizes.rename_type(old_type, new_type)
    }

    pub fn get_font_size_type_list(&self) -> Vec<StringName> {
        self.font_sizes.type_list()
    }

    // -- type variations ----------------------------------------------------

    /// Rejects invalid/empty names, an empty base, and a type that is a
    /// built-in class, like the engine.
    pub fn set_type_variation(&mut self, theme_type: &str, base_type: &str) -> bool {
        if !is_valid_type_name(theme_type) || !is_valid_type_name(base_type) {
            return false;
        }
        if theme_type.is_empty() || base_type.is_empty() {
            return false;
        }
        if class_db::class_exists(theme_type) {
            return false;
        }

        let type_key = StringName::new(theme_type);
        let base_key = StringName::new(base_type);
        if let Some(old_base) = self.variation_map.get(&type_key) {
            if let Some(list) = self.variation_base_map.get_mut(old_base) {
                list.retain(|t| *t != type_key);
            }
        }
        self.variation_map.insert(type_key, base_key);
        self.variation_base_map
            .entry(base_key)
            .or_default()
            .push(type_key);
        true
    }

    /// Direct variation only (`Theme::is_type_variation`).
    pub fn is_type_variation(&self, theme_type: &str, base_type: &str) -> bool {
        self.variation_map
            .get(&StringName::new(theme_type))
            .map(|b| *b == StringName::new(base_type))
            .unwrap_or(false)
    }

    pub fn clear_type_variation(&mut self, theme_type: &str) -> bool {
        let type_key = StringName::new(theme_type);
        let Some(base) = self.variation_map.remove(&type_key) else {
            return false;
        };
        if let Some(list) = self.variation_base_map.get_mut(&base) {
            list.retain(|t| *t != type_key);
        }
        true
    }

    pub fn get_type_variation_base(&self, theme_type: &str) -> Option<StringName> {
        self.variation_map.get(&StringName::new(theme_type)).copied()
    }

    /// Recursive, deduplicated and cycle-safe list of variations of `base_type`.
    pub fn get_type_variation_list(&self, base_type: &str) -> Vec<StringName> {
        fn walk(
            theme: &ThemeData,
            base: &StringName,
            out: &mut Vec<StringName>,
            seen: &mut HashSet<StringName>,
        ) {
            let Some(children) = theme.variation_base_map.get(base) else {
                return;
            };
            for child in children {
                if !seen.insert(*child) {
                    continue;
                }
                out.push(*child);
                walk(theme, child, out, seen);
            }
        }

        let mut out = Vec::new();
        let mut seen = HashSet::new();
        seen.insert(StringName::new(base_type));
        walk(self, &StringName::new(base_type), &mut out, &mut seen);
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn name_validation_and_sanitising() {
        assert!(is_valid_type_name(""));
        assert!(is_valid_type_name("Button"));
        assert!(is_valid_type_name("My_Type2"));
        assert!(!is_valid_type_name("My Type"));
        assert!(!is_valid_type_name("A-B"));

        assert!(!is_valid_item_name(""));
        assert!(is_valid_item_name("font_color"));
        assert!(!is_valid_item_name("font color"));

        assert_eq!(validate_type_name("  My Type!  "), "My_Type_");
        assert_eq!(validate_type_name("OK_1"), "OK_1");
    }

    #[test]
    fn colors_round_trip_and_clear() {
        let mut t = ThemeData::new();
        assert!(!t.has_color("font_color", "Button"));
        assert_eq!(t.get_color("font_color", "Button"), Color::ZERO);

        assert!(t.set_color("font_color", "Button", Color::rgb(1.0, 0.0, 0.0)));
        assert!(t.has_color("font_color", "Button"));
        assert_eq!(t.get_color("font_color", "Button"), Color::rgb(1.0, 0.0, 0.0));

        // Invalid names are rejected and change nothing.
        assert!(!t.set_color("", "Button", Color::ZERO));
        assert!(!t.set_color("bad name", "Button", Color::ZERO));
        assert!(!t.set_color("ok", "bad type", Color::ZERO));

        assert!(t.set_color("font_hover_color", "Button", Color::rgb(0.0, 1.0, 0.0)));
        let list: Vec<String> = t.get_color_list("Button").iter().map(|n| n.as_string()).collect();
        assert_eq!(list, vec!["font_color", "font_hover_color"]);

        assert!(t.clear_color("font_color", "Button"));
        assert!(!t.clear_color("font_color", "Button"));
        assert!(!t.clear_color("x", "Missing"));
        assert!(!t.has_color("font_color", "Button"));
    }

    #[test]
    fn rename_rules() {
        let mut t = ThemeData::new();
        t.set_color("a", "Button", Color::rgb(1.0, 1.0, 1.0));
        t.set_color("b", "Button", Color::ZERO);

        assert!(!t.rename_color("a", "b", "Button"), "target exists");
        assert!(!t.rename_color("missing", "c", "Button"));
        assert!(!t.rename_color("a", "bad name", "Button"));
        assert!(!t.rename_color("a", "c", "Missing"));

        assert!(t.rename_color("a", "c", "Button"));
        assert!(!t.has_color("a", "Button"));
        assert!(t.has_color("c", "Button"));
        assert_eq!(t.get_color("c", "Button"), Color::rgb(1.0, 1.0, 1.0));
    }

    #[test]
    fn type_lists_and_type_renames() {
        let mut t = ThemeData::new();
        assert!(t.add_color_type("Button"));
        assert!(t.add_color_type("Label"));
        assert!(!t.add_color_type("bad type"));
        let types: Vec<String> = t.get_color_type_list().iter().map(|n| n.as_string()).collect();
        assert_eq!(types, vec!["Button", "Label"]);

        assert!(!t.rename_color_type("Button", "Label"), "target exists");
        assert!(t.rename_color_type("Button", "LineEdit"));
        let types: Vec<String> = t.get_color_type_list().iter().map(|n| n.as_string()).collect();
        assert_eq!(types, vec!["Label", "LineEdit"]);

        assert!(t.remove_color_type("Label"));
        assert!(!t.remove_color_type("Label"));
    }

    #[test]
    fn constants_and_font_sizes() {
        let mut t = ThemeData::new();
        assert_eq!(t.get_constant("separation", "BoxContainer"), 0);
        assert!(t.set_constant("separation", "BoxContainer", 4));
        assert_eq!(t.get_constant("separation", "BoxContainer"), 4);
        assert!(t.has_constant("separation", "BoxContainer"));
        assert!(t.rename_constant("separation", "gap", "BoxContainer"));
        assert_eq!(t.get_constant("gap", "BoxContainer"), 4);

        // Font sizes: stored positive value, then default, then fallback 16.
        assert_eq!(t.get_font_size("font_size", "Label"), FALLBACK_FONT_SIZE);
        assert!(!t.has_font_size("font_size", "Label"));
        assert!(t.set_font_size("font_size", "Label", 24));
        assert_eq!(t.get_font_size("font_size", "Label"), 24);
        assert!(t.has_font_size("font_size", "Label"));

        // A zero or negative stored size is ignored.
        assert!(t.set_font_size("font_size", "Label", 0));
        assert!(!t.has_font_size("font_size", "Label"));
        assert_eq!(t.get_font_size("font_size", "Label"), FALLBACK_FONT_SIZE);
        assert!(t.has_font_size_nocheck("font_size", "Label"));

        t.set_default_font_size(12);
        assert!(t.has_default_font_size());
        assert_eq!(t.get_font_size("font_size", "Label"), 12);
        assert!(t.has_font_size("font_size", "Label"));
    }

    #[test]
    fn default_base_scale_presence() {
        let mut t = ThemeData::new();
        assert_eq!(t.get_default_base_scale(), 0.0);
        assert!(!t.has_default_base_scale());
        t.set_default_base_scale(0.5);
        assert!(t.has_default_base_scale());
        t.set_default_base_scale(0.0);
        assert!(!t.has_default_base_scale());
    }

    #[test]
    fn variations_are_validated_and_recursive() {
        let mut t = ThemeData::new();
        assert!(t.set_type_variation("FlatButton", "Button"));
        assert!(t.is_type_variation("FlatButton", "Button"));
        assert!(!t.is_type_variation("Button", "FlatButton"));
        assert_eq!(
            t.get_type_variation_base("FlatButton").map(|n| n.as_string()),
            Some("Button".to_string())
        );

        // Sub-variations are listed recursively.
        assert!(t.set_type_variation("GreenFlatButton", "FlatButton"));
        let list: Vec<String> = t
            .get_type_variation_list("Button")
            .iter()
            .map(|n| n.as_string())
            .collect();
        assert_eq!(list, vec!["FlatButton", "GreenFlatButton"]);

        // Rejections: empty names, empty base, and built-in classes.
        assert!(!t.set_type_variation("", "Button"));
        assert!(!t.set_type_variation("X", ""));
        // The class check uses the port's ClassDB registrations; the engine's
        // built-in classes are not preloaded here, so register one explicitly.
        class_db::register_class("ThemeBuiltIn", None);
        assert!(!t.set_type_variation("ThemeBuiltIn", "Button"));
        assert!(!t.set_type_variation("bad type", "Button"));

        // Reassigning a variation moves it between base lists.
        assert!(t.set_type_variation("GreenFlatButton", "Button"));
        let list: Vec<String> = t
            .get_type_variation_list("FlatButton")
            .iter()
            .map(|n| n.as_string())
            .collect();
        assert!(list.is_empty(), "{list:?}");

        assert!(t.clear_type_variation("FlatButton"));
        assert!(!t.clear_type_variation("FlatButton"));
        assert!(t.get_type_variation_base("FlatButton").is_none());
    }
}
