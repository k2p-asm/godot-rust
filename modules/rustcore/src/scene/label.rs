//! `scene/gui/label.*`: visibility and text-shaping state (no rendering).
//!
//! Defaults from the header: paragraph separator `"\\n"` (a backslash followed by
//! `n`), auto text direction, the default structured-text parser, the
//! before-shaping visible-character behaviour, all characters visible
//! (`visible_chars = -1`, ratio 1), no skipped lines (`0`) and no line limit
//! (`-1`).
//!
//! `set_visible_characters` and `set_visible_ratio` are coupled through the total
//! character count, which the engine obtains from the shaped text; here it is an
//! explicit argument. `set_lines_skipped` rejects negatives; `max_lines_visible`
//! has no validation.
//!
//! Shaping, drawing and the theme cache are not ported.

use crate::variant::Variant;

/// `TextServer::Direction`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum TextDirection {
    Auto,
    Ltr,
    Rtl,
    Inherited,
}

/// `TextServer::VisibleCharactersBehavior`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum VisibleCharactersBehavior {
    CharsBeforeShaping,
    CharsAfterShaping,
    GlyphsAuto,
    GlyphsLtr,
    GlyphsRtl,
}

#[derive(Clone, Debug)]
pub struct LabelData {
    paragraph_separator: String,
    language: String,
    text_direction: TextDirection,
    structured_text_parser: i32,
    structured_text_options: Vec<Variant>,
    visible_chars_behavior: VisibleCharactersBehavior,
    visible_chars: i32,
    visible_ratio: f32,
    lines_skipped: i32,
    max_lines_visible: i32,
    tab_stops: Vec<f32>,
}

impl Default for LabelData {
    fn default() -> Self {
        Self::new()
    }
}

impl LabelData {
    pub fn new() -> Self {
        Self {
            // The header writes "\\n", i.e. a backslash and an `n` character.
            paragraph_separator: "\\n".to_string(),
            language: String::new(),
            text_direction: TextDirection::Auto,
            structured_text_parser: 0,
            structured_text_options: Vec::new(),
            visible_chars_behavior: VisibleCharactersBehavior::CharsBeforeShaping,
            visible_chars: -1,
            visible_ratio: 1.0,
            lines_skipped: 0,
            max_lines_visible: -1,
            tab_stops: Vec::new(),
        }
    }

    pub fn set_paragraph_separator(&mut self, separator: &str) {
        self.paragraph_separator = separator.to_string();
    }

    pub fn get_paragraph_separator(&self) -> &str {
        &self.paragraph_separator
    }

    pub fn set_language(&mut self, language: &str) {
        self.language = language.to_string();
    }

    pub fn get_language(&self) -> &str {
        &self.language
    }

    pub fn set_text_direction(&mut self, direction: TextDirection) {
        self.text_direction = direction;
    }

    pub fn get_text_direction(&self) -> TextDirection {
        self.text_direction
    }

    /// `StructuredTextParser` discriminator; 0 is `STRUCTURED_TEXT_DEFAULT`.
    pub fn set_structured_text_bidi_override(&mut self, parser: i32) {
        self.structured_text_parser = parser;
    }

    pub fn get_structured_text_bidi_override(&self) -> i32 {
        self.structured_text_parser
    }

    pub fn set_structured_text_bidi_override_options(&mut self, options: Vec<Variant>) {
        self.structured_text_options = options;
    }

    pub fn get_structured_text_bidi_override_options(&self) -> &[Variant] {
        &self.structured_text_options
    }

    pub fn set_visible_characters_behavior(&mut self, behavior: VisibleCharactersBehavior) {
        self.visible_chars_behavior = behavior;
    }

    pub fn get_visible_characters_behavior(&self) -> VisibleCharactersBehavior {
        self.visible_chars_behavior
    }

    /// Sets the count and recomputes the ratio from `total_characters`.
    /// `-1` (or an empty text) means fully visible.
    pub fn set_visible_characters(&mut self, amount: i32, total_characters: i32) {
        if self.visible_chars == amount {
            return;
        }
        self.visible_chars = amount;
        if amount == -1 || total_characters == 0 {
            self.visible_ratio = 1.0;
        } else {
            self.visible_ratio = amount as f32 / total_characters as f32;
        }
    }

    pub fn get_visible_characters(&self) -> i32 {
        self.visible_chars
    }

    /// Sets the ratio and recomputes the count (`-1` for a full ratio, `0` for a
    /// negative one).
    pub fn set_visible_ratio(&mut self, ratio: f32, total_characters: i32) {
        if self.visible_ratio == ratio {
            return;
        }
        self.visible_ratio = ratio;
        if ratio >= 1.0 {
            self.visible_chars = -1;
            self.visible_ratio = 1.0;
        } else if ratio < 0.0 {
            self.visible_chars = 0;
            self.visible_ratio = 0.0;
        } else {
            self.visible_chars = (total_characters as f32 * ratio) as i32;
        }
    }

    pub fn get_visible_ratio(&self) -> f32 {
        self.visible_ratio
    }

    /// `ERR_FAIL_COND(lines < 0)`.
    pub fn set_lines_skipped(&mut self, lines: i32) -> bool {
        if lines < 0 {
            return false;
        }
        self.lines_skipped = lines;
        true
    }

    pub fn get_lines_skipped(&self) -> i32 {
        self.lines_skipped
    }

    pub fn set_max_lines_visible(&mut self, lines: i32) {
        self.max_lines_visible = lines;
    }

    pub fn get_max_lines_visible(&self) -> i32 {
        self.max_lines_visible
    }

    pub fn set_tab_stops(&mut self, stops: Vec<f32>) {
        self.tab_stops = stops;
    }

    pub fn get_tab_stops(&self) -> &[f32] {
        &self.tab_stops
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let l = LabelData::new();
        assert_eq!(l.get_paragraph_separator(), "\\n");
        assert_eq!(l.get_language(), "");
        assert_eq!(l.get_text_direction(), TextDirection::Auto);
        assert_eq!(l.get_structured_text_bidi_override(), 0);
        assert!(l.get_structured_text_bidi_override_options().is_empty());
        assert_eq!(
            l.get_visible_characters_behavior(),
            VisibleCharactersBehavior::CharsBeforeShaping
        );
        assert_eq!(l.get_visible_characters(), -1);
        assert_eq!(l.get_visible_ratio(), 1.0);
        assert_eq!(l.get_lines_skipped(), 0);
        assert_eq!(l.get_max_lines_visible(), -1);
        assert!(l.get_tab_stops().is_empty());
    }

    #[test]
    fn visible_characters_recompute_the_ratio() {
        let mut l = LabelData::new();
        l.set_visible_characters(5, 10);
        assert_eq!(l.get_visible_characters(), 5);
        assert_eq!(l.get_visible_ratio(), 0.5);

        // -1 means fully visible.
        l.set_visible_characters(-1, 10);
        assert_eq!(l.get_visible_characters(), -1);
        assert_eq!(l.get_visible_ratio(), 1.0);

        // No characters at all also means fully visible.
        l.set_visible_characters(3, 0);
        assert_eq!(l.get_visible_characters(), 3);
        assert_eq!(l.get_visible_ratio(), 1.0);

        // Setting the same value again is a no-op.
        l.set_visible_characters(3, 0);
        assert_eq!(l.get_visible_ratio(), 1.0);
    }

    #[test]
    fn visible_ratio_recomputes_the_count() {
        let mut l = LabelData::new();
        l.set_visible_ratio(0.25, 40);
        assert_eq!(l.get_visible_characters(), 10);
        assert_eq!(l.get_visible_ratio(), 0.25);

        // Ratios at or above 1 pin both to "all visible".
        l.set_visible_ratio(1.5, 40);
        assert_eq!(l.get_visible_characters(), -1);
        assert_eq!(l.get_visible_ratio(), 1.0);

        // Negative ratios pin both to zero.
        l.set_visible_ratio(-0.5, 40);
        assert_eq!(l.get_visible_characters(), 0);
        assert_eq!(l.get_visible_ratio(), 0.0);

        // Truncation, not rounding, for fractional characters.
        l.set_visible_ratio(0.333, 10);
        assert_eq!(l.get_visible_characters(), 3);
    }

    #[test]
    fn lines_and_settings() {
        let mut l = LabelData::new();
        assert!(!l.set_lines_skipped(-1));
        assert!(l.set_lines_skipped(2));
        assert_eq!(l.get_lines_skipped(), 2);

        l.set_max_lines_visible(3);
        assert_eq!(l.get_max_lines_visible(), 3);
        // Negative limits are accepted by the engine.
        l.set_max_lines_visible(-4);
        assert_eq!(l.get_max_lines_visible(), -4);

        l.set_paragraph_separator("|");
        l.set_language("es");
        l.set_text_direction(TextDirection::Rtl);
        l.set_structured_text_bidi_override(3);
        l.set_structured_text_bidi_override_options(vec![Variant::Int(1)]);
        l.set_visible_characters_behavior(VisibleCharactersBehavior::GlyphsAuto);
        l.set_tab_stops(vec![8.0, 16.0]);

        assert_eq!(l.get_paragraph_separator(), "|");
        assert_eq!(l.get_language(), "es");
        assert_eq!(l.get_text_direction(), TextDirection::Rtl);
        assert_eq!(l.get_structured_text_bidi_override(), 3);
        assert_eq!(l.get_structured_text_bidi_override_options(), &[Variant::Int(1)]);
        assert_eq!(
            l.get_visible_characters_behavior(),
            VisibleCharactersBehavior::GlyphsAuto
        );
        assert_eq!(l.get_tab_stops(), &[8.0, 16.0]);

        assert_eq!(TextDirection::Auto as i32, 0);
        assert_eq!(TextDirection::Ltr as i32, 1);
        assert_eq!(TextDirection::Rtl as i32, 2);
        assert_eq!(TextDirection::Inherited as i32, 3);
        assert_eq!(VisibleCharactersBehavior::CharsBeforeShaping as i32, 0);
        assert_eq!(VisibleCharactersBehavior::CharsAfterShaping as i32, 1);
        assert_eq!(VisibleCharactersBehavior::GlyphsAuto as i32, 2);
        assert_eq!(VisibleCharactersBehavior::GlyphsLtr as i32, 3);
        assert_eq!(VisibleCharactersBehavior::GlyphsRtl as i32, 4);
    }
}
