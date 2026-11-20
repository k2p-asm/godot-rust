//! `scene/resources/syntax_highlighter.*`: `CodeHighlighter`'s state.
//!
//! Ported from `CodeHighlighter`: the keyword and member-keyword colour maps
//! (set/remove/query/clear), the colour regions (`start_key`, optional
//! `end_key`, `line_only`, colour) with the engine's symbol validation,
//! duplicate-start rejection and longest-start-first ordering, the five base
//! colours (font/member/function/symbol/number, defaulting to opaque black like
//! `Color()`), and the `uint_suffix` flag.
//!
//! The line tokenizer (`_get_line_syntax_highlighting_impl`), the highlighting
//! cache and the `TextEdit` binding are not ported.

use std::collections::BTreeMap;

use crate::math::color::Color;

/// `SyntaxHighlighter`'s cache invalidation counter (the engine calls
/// `clear_highlighting_cache()` after every mutation; the port keeps a version
/// so callers can detect that a re-highlight is needed).
pub const HIGHLIGHTING_CACHE_CLEARED: u64 = 0;

#[derive(Clone, Debug, PartialEq)]
pub struct ColorRegion {
    pub color: Color,
    pub start_key: String,
    pub end_key: String,
    pub line_only: bool,
}

impl ColorRegion {
    /// `get_color_regions`' dictionary key: `start` or `start end`.
    pub fn key(&self) -> String {
        if self.end_key.is_empty() {
            self.start_key.clone()
        } else {
            format!("{} {}", self.start_key, self.end_key)
        }
    }
}

/// The engine requires colour-region keys to be made of symbols.
fn is_symbol_char(c: char) -> bool {
    !c.is_alphanumeric() && !c.is_whitespace()
}

fn all_symbols(text: &str) -> bool {
    !text.is_empty() && text.chars().all(is_symbol_char)
}

#[derive(Clone, Debug, Default)]
pub struct CodeHighlighterData {
    keywords: BTreeMap<String, Color>,
    member_keywords: BTreeMap<String, Color>,
    color_regions: Vec<ColorRegion>,
    font_color: Color,
    member_color: Color,
    function_color: Color,
    symbol_color: Color,
    number_color: Color,
    uint_suffix_enabled: bool,
    cache_version: u64,
}

impl CodeHighlighterData {
    pub fn new() -> Self {
        // In-class `Color` members default-construct to opaque black.
        Self {
            font_color: Color::new(0.0, 0.0, 0.0, 1.0),
            member_color: Color::new(0.0, 0.0, 0.0, 1.0),
            function_color: Color::new(0.0, 0.0, 0.0, 1.0),
            symbol_color: Color::new(0.0, 0.0, 0.0, 1.0),
            number_color: Color::new(0.0, 0.0, 0.0, 1.0),
            ..Self::default()
        }
    }

    /// `clear_highlighting_cache`: bumps the port's version counter.
    fn invalidate(&mut self) {
        self.cache_version += 1;
    }

    /// Callers can cache line results until this changes.
    pub fn get_cache_version(&self) -> u64 {
        self.cache_version
    }

    // -- keyword colours ----------------------------------------------------

    pub fn add_keyword_color(&mut self, keyword: &str, color: Color) {
        self.keywords.insert(keyword.to_string(), color);
        self.invalidate();
    }

    pub fn remove_keyword_color(&mut self, keyword: &str) {
        self.keywords.remove(keyword);
        self.invalidate();
    }

    pub fn has_keyword_color(&self, keyword: &str) -> bool {
        self.keywords.contains_key(keyword)
    }

    /// `get_keyword_color`: `None` where the engine logs `ERR_FAIL_COND_V` and
    /// returns `Color()`.
    pub fn get_keyword_color(&self, keyword: &str) -> Option<Color> {
        self.keywords.get(keyword).copied()
    }

    pub fn set_keyword_colors(&mut self, colors: &[(&str, Color)]) {
        self.keywords.clear();
        for (keyword, color) in colors {
            self.keywords.insert(keyword.to_string(), *color);
        }
        self.invalidate();
    }

    pub fn clear_keyword_colors(&mut self) {
        self.keywords.clear();
        self.invalidate();
    }

    pub fn get_keyword_colors(&self) -> Vec<(String, Color)> {
        self.keywords
            .iter()
            .map(|(keyword, color)| (keyword.clone(), *color))
            .collect()
    }

    pub fn get_keyword_count(&self) -> usize {
        self.keywords.len()
    }

    // -- member keyword colours ---------------------------------------------

    pub fn add_member_keyword_color(&mut self, keyword: &str, color: Color) {
        self.member_keywords.insert(keyword.to_string(), color);
        self.invalidate();
    }

    pub fn remove_member_keyword_color(&mut self, keyword: &str) {
        self.member_keywords.remove(keyword);
        self.invalidate();
    }

    pub fn has_member_keyword_color(&self, keyword: &str) -> bool {
        self.member_keywords.contains_key(keyword)
    }

    pub fn get_member_keyword_color(&self, keyword: &str) -> Option<Color> {
        self.member_keywords.get(keyword).copied()
    }

    pub fn set_member_keyword_colors(&mut self, colors: &[(&str, Color)]) {
        self.member_keywords.clear();
        for (keyword, color) in colors {
            self.member_keywords.insert(keyword.to_string(), *color);
        }
        self.invalidate();
    }

    pub fn clear_member_keyword_colors(&mut self) {
        self.member_keywords.clear();
        self.invalidate();
    }

    pub fn get_member_keyword_count(&self) -> usize {
        self.member_keywords.len()
    }

    // -- colour regions -----------------------------------------------------

    /// `add_color_region`: false when a key is not all symbols or the start key
    /// already exists.
    pub fn add_color_region(
        &mut self,
        start_key: &str,
        end_key: &str,
        color: Color,
        line_only: bool,
    ) -> bool {
        if !all_symbols(start_key) {
            return false;
        }
        if !end_key.is_empty() && !all_symbols(end_key) {
            return false;
        }
        if self
            .color_regions
            .iter()
            .any(|region| region.start_key == start_key)
        {
            return false;
        }
        // Longest start keys first, matching the engine's insertion rule.
        let mut at = 0usize;
        for region in &self.color_regions {
            if start_key.len() < region.start_key.len() {
                at += 1;
            }
        }
        self.color_regions.insert(
            at,
            ColorRegion {
                color,
                start_key: start_key.to_string(),
                end_key: end_key.to_string(),
                line_only,
            },
        );
        self.invalidate();
        true
    }

    /// `remove_color_region`.
    pub fn remove_color_region(&mut self, start_key: &str) -> bool {
        match self
            .color_regions
            .iter()
            .position(|region| region.start_key == start_key)
        {
            Some(index) => {
                self.color_regions.remove(index);
                self.invalidate();
                true
            }
            None => false,
        }
    }

    pub fn has_color_region(&self, start_key: &str) -> bool {
        self.color_regions
            .iter()
            .any(|region| region.start_key == start_key)
    }

    pub fn clear_color_regions(&mut self) {
        self.color_regions.clear();
        self.invalidate();
    }

    pub fn get_color_regions(&self) -> &[ColorRegion] {
        &self.color_regions
    }

    /// `get_color_regions` as the engine's dictionary (key -> colour).
    pub fn get_color_regions_map(&self) -> Vec<(String, Color)> {
        self.color_regions
            .iter()
            .map(|region| (region.key(), region.color))
            .collect()
    }

    // -- base colours -------------------------------------------------------

    pub fn set_font_color(&mut self, color: Color) {
        self.font_color = color;
        self.invalidate();
    }

    pub fn get_font_color(&self) -> Color {
        self.font_color
    }

    pub fn set_member_variable_color(&mut self, color: Color) {
        self.member_color = color;
        self.invalidate();
    }

    pub fn get_member_variable_color(&self) -> Color {
        self.member_color
    }

    pub fn set_function_color(&mut self, color: Color) {
        self.function_color = color;
        self.invalidate();
    }

    pub fn get_function_color(&self) -> Color {
        self.function_color
    }

    pub fn set_symbol_color(&mut self, color: Color) {
        self.symbol_color = color;
        self.invalidate();
    }

    pub fn get_symbol_color(&self) -> Color {
        self.symbol_color
    }

    pub fn set_number_color(&mut self, color: Color) {
        self.number_color = color;
        self.invalidate();
    }

    pub fn get_number_color(&self) -> Color {
        self.number_color
    }

    /// `set_uint_suffix_enabled` (the engine does not invalidate the cache here).
    pub fn set_uint_suffix_enabled(&mut self, enabled: bool) {
        self.uint_suffix_enabled = enabled;
    }

    pub fn is_uint_suffix_enabled(&self) -> bool {
        self.uint_suffix_enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn red() -> Color {
        Color::new(1.0, 0.0, 0.0, 1.0)
    }

    fn blue() -> Color {
        Color::new(0.0, 0.0, 1.0, 1.0)
    }

    #[test]
    fn keywords_and_members() {
        let mut highlighter = CodeHighlighterData::new();
        assert_eq!(highlighter.get_keyword_count(), 0);
        assert!(!highlighter.has_keyword_color("if"));
        assert_eq!(highlighter.get_keyword_color("if"), None);

        highlighter.add_keyword_color("if", red());
        assert!(highlighter.has_keyword_color("if"));
        assert_eq!(highlighter.get_keyword_color("if"), Some(red()));
        highlighter.add_keyword_color("if", blue());
        assert_eq!(highlighter.get_keyword_color("if"), Some(blue()), "overwrites");

        highlighter.add_keyword_color("else", red());
        assert_eq!(highlighter.get_keyword_count(), 2);
        highlighter.remove_keyword_color("if");
        assert!(!highlighter.has_keyword_color("if"));
        highlighter.remove_keyword_color("missing");
        assert_eq!(highlighter.get_keyword_count(), 1);

        highlighter.set_keyword_colors(&[("for", red()), ("while", blue())]);
        assert_eq!(highlighter.get_keyword_count(), 2);
        assert!(!highlighter.has_keyword_color("else"));
        let colors = highlighter.get_keyword_colors();
        assert_eq!(colors[0].0, "for");
        assert_eq!(colors[0].1, red());
        highlighter.clear_keyword_colors();
        assert_eq!(highlighter.get_keyword_count(), 0);

        highlighter.add_member_keyword_color("position", blue());
        assert!(highlighter.has_member_keyword_color("position"));
        assert_eq!(highlighter.get_member_keyword_color("position"), Some(blue()));
        assert_eq!(highlighter.get_member_keyword_count(), 1);
        highlighter.remove_member_keyword_color("position");
        assert_eq!(highlighter.get_member_keyword_count(), 0);
        highlighter.set_member_keyword_colors(&[("x", red()), ("y", blue())]);
        assert_eq!(highlighter.get_member_keyword_count(), 2);
        highlighter.clear_member_keyword_colors();
        assert_eq!(highlighter.get_member_keyword_count(), 0);
    }

    #[test]
    fn colour_regions() {
        let mut highlighter = CodeHighlighterData::new();
        assert!(highlighter.get_color_regions().is_empty());

        // Keys must be symbols.
        assert!(!highlighter.add_color_region("abc", "\"", red(), false));
        assert!(!highlighter.add_color_region("\"", "abc", red(), false));
        assert!(!highlighter.add_color_region("", "\"", red(), false));

        assert!(highlighter.add_color_region("#", "", red(), true));
        assert!(highlighter.add_color_region("\"\"\"", "\"\"\"", blue(), false));
        assert!(highlighter.add_color_region("\"", "\"", red(), false));
        assert!(!highlighter.add_color_region("\"", "x", blue(), false), "duplicate start");
        assert!(!highlighter.add_color_region("\"", "x", blue(), false), "invalid end");

        // The engine inserts a new region before every longer start key, so the
        // order ends up: """ (len 3), " (len 1, inserted after """), # (len 1).
        let regions = highlighter.get_color_regions();
        assert_eq!(regions.len(), 3);
        assert_eq!(regions[0].start_key, "\"\"\"");
        assert_eq!(regions[1].start_key, "\"");
        assert_eq!(regions[2].start_key, "#");
        assert!(regions[2].line_only);
        assert_eq!(regions[0].key(), "\"\"\" \"\"\"");
        assert_eq!(regions[1].key(), "\" \"");
        assert_eq!(regions[2].key(), "#");

        assert!(highlighter.has_color_region("#"));
        assert!(highlighter.remove_color_region("\"\"\""));
        assert!(!highlighter.remove_color_region("\"\"\""));
        assert_eq!(highlighter.get_color_regions().len(), 2);
        let map = highlighter.get_color_regions_map();
        assert_eq!(map.len(), 2);
        highlighter.clear_color_regions();
        assert!(highlighter.get_color_regions().is_empty());
    }

    #[test]
    fn base_colours_and_cache_versions() {
        let mut highlighter = CodeHighlighterData::new();
        assert_eq!(highlighter.get_font_color(), Color::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(highlighter.get_member_variable_color(), Color::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(highlighter.get_function_color(), Color::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(highlighter.get_symbol_color(), Color::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(highlighter.get_number_color(), Color::new(0.0, 0.0, 0.0, 1.0));
        assert!(!highlighter.is_uint_suffix_enabled());

        let version = highlighter.get_cache_version();
        highlighter.set_font_color(red());
        highlighter.set_member_variable_color(blue());
        highlighter.set_function_color(red());
        highlighter.set_symbol_color(blue());
        highlighter.set_number_color(red());
        assert!(highlighter.get_cache_version() > version);
        assert_eq!(highlighter.get_font_color(), red());
        assert_eq!(highlighter.get_number_color(), red());

        highlighter.set_uint_suffix_enabled(true);
        assert!(highlighter.is_uint_suffix_enabled());
        // The uint suffix flag does not invalidate the cache (engine behaviour).
        let version = highlighter.get_cache_version();
        highlighter.set_uint_suffix_enabled(false);
        assert_eq!(highlighter.get_cache_version(), version);
        assert_eq!(HIGHLIGHTING_CACHE_CLEARED, 0);
    }
}
