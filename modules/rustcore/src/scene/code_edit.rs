//! `scene/gui/code_edit.*` (state): the code editor's configuration.
//!
//! Ported state from `CodeEdit`:
//!
//! - indent management (size 4, `"\t"`, no auto-indent, no spaces) and the
//!   default auto-indent prefixes `: { [ (`;
//! - auto brace completion (off) with the constructor's five pairs and the
//!   engine's insertion rule: keys must be symbols, an open key must be unique,
//!   and pairs are ordered by descending open-key length (equal lengths insert at
//!   the front);
//! - string/comment delimiters with their `line_only` flag;
//! - gutters (name/type/draw/clickable/overwritable, the two constructor gutters
//!   plus the fold gutter) and the gutter limits;
//! - line numbers (digits 1, minimum 3, padding `" "`), breakpoints/bookmarks/
//!   executing-line flags, folding (off, `region`/`endregion` tags), code hints,
//!   code completion flags, symbol lookup/tooltip flags and the line-length
//!   guideline columns.
//!
//! Text buffer editing, drawing, completion popups and the input handling are not
//! ported.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum GutterType {
    String = 0,
    Icon = 1,
    Custom = 2,
}

#[derive(Clone, Debug)]
pub struct Gutter {
    pub name: String,
    pub gutter_type: GutterType,
    pub width: i32,
    pub draw: bool,
    pub clickable: bool,
    pub overwritable: bool,
}

impl Default for Gutter {
    fn default() -> Self {
        Self {
            name: String::new(),
            gutter_type: GutterType::String,
            width: 24,
            draw: false,
            clickable: false,
            overwritable: false,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct BracePair {
    pub open_key: String,
    pub close_key: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DelimiterType {
    String = 0,
    Comment = 1,
    Region = 2,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Delimiter {
    pub start_key: String,
    pub end_key: String,
    pub line_only: bool,
    pub delimiter_type: DelimiterType,
}

fn is_symbol(c: char) -> bool {
    !c.is_alphanumeric() && !c.is_whitespace()
}

#[derive(Clone, Debug)]
pub struct CodeEditData {
    indent_size: i32,
    indent_text: String,
    auto_indent: bool,
    auto_indent_prefixes: Vec<char>,
    indent_using_spaces: bool,
    auto_brace_completion_enabled: bool,
    brace_pairs: Vec<BracePair>,
    delimiters: Vec<Delimiter>,
    gutters: Vec<Gutter>,
    draw_breakpoints: bool,
    draw_bookmarks: bool,
    draw_executing_lines: bool,
    line_number_gutter: i32,
    line_number_digits: i32,
    line_numbers_min_digits: i32,
    line_number_padding: String,
    fold_gutter: i32,
    draw_fold_gutter: bool,
    line_folding_enabled: bool,
    code_region_start_tag: String,
    code_region_end_tag: String,
    code_hint: String,
    code_hint_draw_below: bool,
    code_completion_enabled: bool,
    code_completion_forced: bool,
    highlight_matching_braces_enabled: bool,
    symbol_lookup_on_click_enabled: bool,
    symbol_tooltip_on_hover_enabled: bool,
    line_length_guideline_columns: Vec<i32>,
}

impl Default for CodeEditData {
    fn default() -> Self {
        Self::new()
    }
}

impl CodeEditData {
    pub fn new() -> Self {
        let mut edit = Self {
            indent_size: 4,
            indent_text: "\t".to_string(),
            auto_indent: false,
            auto_indent_prefixes: vec![':', '{', '[', '('],
            indent_using_spaces: false,
            auto_brace_completion_enabled: false,
            brace_pairs: Vec::new(),
            delimiters: Vec::new(),
            gutters: Vec::new(),
            draw_breakpoints: false,
            draw_bookmarks: false,
            draw_executing_lines: false,
            line_number_gutter: 1,
            line_number_digits: 1,
            line_numbers_min_digits: 3,
            line_number_padding: " ".to_string(),
            fold_gutter: 2,
            draw_fold_gutter: false,
            line_folding_enabled: false,
            code_region_start_tag: "region".to_string(),
            code_region_end_tag: "endregion".to_string(),
            code_hint: String::new(),
            code_hint_draw_below: true,
            code_completion_enabled: false,
            code_completion_forced: false,
            // TextEdit's member default.
            highlight_matching_braces_enabled: false,
            symbol_lookup_on_click_enabled: false,
            symbol_tooltip_on_hover_enabled: false,
            line_length_guideline_columns: Vec::new(),
        };
        // Constructor order: main, line numbers, fold gutter.
        edit.add_gutter("main_gutter", GutterType::Custom, false, true);
        edit.add_gutter("line_numbers", GutterType::Custom, false, false);
        edit.add_gutter("fold", GutterType::Icon, false, false);
        edit.fold_gutter = 2;
        edit.line_number_gutter = 1;
        // Default auto brace completion pairs.
        edit.add_auto_brace_completion_pair("(", ")");
        edit.add_auto_brace_completion_pair("{", "}");
        edit.add_auto_brace_completion_pair("[", "]");
        edit.add_auto_brace_completion_pair("\"", "\"");
        edit.add_auto_brace_completion_pair("'", "'");
        edit.add_string_delimiter("\"", "\"", false);
        edit.add_string_delimiter("'", "'", false);
        edit
    }

    // -- indentation --------------------------------------------------------

    pub fn set_indent_size(&mut self, size: i32) {
        self.indent_size = size;
    }

    pub fn get_indent_size(&self) -> i32 {
        self.indent_size
    }

    pub fn set_indent_text(&mut self, text: &str) {
        self.indent_text = text.to_string();
    }

    pub fn get_indent_text(&self) -> &str {
        &self.indent_text
    }

    pub fn set_auto_indent_enabled(&mut self, enabled: bool) {
        self.auto_indent = enabled;
    }

    pub fn is_auto_indent_enabled(&self) -> bool {
        self.auto_indent
    }

    pub fn get_auto_indent_prefixes(&self) -> &[char] {
        &self.auto_indent_prefixes
    }

    pub fn set_indent_using_spaces(&mut self, enabled: bool) {
        self.indent_using_spaces = enabled;
    }

    pub fn is_indent_using_spaces(&self) -> bool {
        self.indent_using_spaces
    }

    // -- brace completion ---------------------------------------------------

    pub fn set_auto_brace_completion_enabled(&mut self, enabled: bool) {
        self.auto_brace_completion_enabled = enabled;
    }

    pub fn is_auto_brace_completion_enabled(&self) -> bool {
        self.auto_brace_completion_enabled
    }

    /// `add_auto_brace_completion_pair`: false for empty/non-symbol keys or a
    /// duplicate open key (the engine logs and returns).
    pub fn add_auto_brace_completion_pair(&mut self, open: &str, close: &str) -> bool {
        if open.is_empty() || close.is_empty() {
            return false;
        }
        if !open.chars().all(is_symbol) || !close.chars().all(is_symbol) {
            return false;
        }
        let mut at = 0usize;
        for pair in &self.brace_pairs {
            if pair.open_key == open {
                return false;
            }
            if open.chars().count() < pair.open_key.chars().count() {
                at += 1;
            }
        }
        self.brace_pairs.insert(
            at,
            BracePair {
                open_key: open.to_string(),
                close_key: close.to_string(),
            },
        );
        true
    }

    pub fn remove_auto_brace_completion_pair(&mut self, open: &str) -> bool {
        match self
            .brace_pairs
            .iter()
            .position(|pair| pair.open_key == open)
        {
            Some(index) => {
                self.brace_pairs.remove(index);
                true
            }
            None => false,
        }
    }

    pub fn has_auto_brace_completion_pair(&self, open: &str) -> bool {
        self.brace_pairs.iter().any(|pair| pair.open_key == open)
    }

    pub fn get_auto_brace_completion_pairs(&self) -> &[BracePair] {
        &self.brace_pairs
    }

    pub fn clear_auto_brace_completion_pairs(&mut self) {
        self.brace_pairs.clear();
    }

    // -- delimiters ---------------------------------------------------------

    pub fn add_string_delimiter(&mut self, start: &str, end: &str, line_only: bool) -> bool {
        self.add_delimiter(start, end, line_only, DelimiterType::String)
    }

    pub fn add_comment_delimiter(&mut self, start: &str, end: &str, line_only: bool) -> bool {
        self.add_delimiter(start, end, line_only, DelimiterType::Comment)
    }

    fn add_delimiter(
        &mut self,
        start: &str,
        end: &str,
        line_only: bool,
        delimiter_type: DelimiterType,
    ) -> bool {
        if start.is_empty() {
            return false;
        }
        if end.is_empty() != line_only {
            return false;
        }
        if self
            .delimiters
            .iter()
            .any(|known| known.start_key == start && known.delimiter_type == delimiter_type)
        {
            return false;
        }
        self.delimiters.push(Delimiter {
            start_key: start.to_string(),
            end_key: end.to_string(),
            line_only,
            delimiter_type,
        });
        true
    }

    pub fn remove_delimiter(&mut self, start: &str, delimiter_type: DelimiterType) -> bool {
        match self
            .delimiters
            .iter()
            .position(|known| known.start_key == start && known.delimiter_type == delimiter_type)
        {
            Some(index) => {
                self.delimiters.remove(index);
                true
            }
            None => false,
        }
    }

    pub fn has_delimiter(&self, start: &str, delimiter_type: DelimiterType) -> bool {
        self.delimiters
            .iter()
            .any(|known| known.start_key == start && known.delimiter_type == delimiter_type)
    }

    pub fn get_delimiters(&self) -> &[Delimiter] {
        &self.delimiters
    }

    pub fn clear_delimiters(&mut self) {
        self.delimiters.clear();
    }

    // -- gutters ------------------------------------------------------------

    /// `add_gutter`: returns the new gutter index.
    pub fn add_gutter(
        &mut self,
        name: &str,
        gutter_type: GutterType,
        clickable: bool,
        overwritable: bool,
    ) -> usize {
        self.gutters.push(Gutter {
            name: name.to_string(),
            gutter_type,
            width: 24,
            draw: false,
            clickable,
            overwritable,
        });
        self.gutters.len() - 1
    }

    pub fn remove_gutter(&mut self, index: usize) -> bool {
        if index >= self.gutters.len() {
            return false;
        }
        self.gutters.remove(index);
        true
    }

    pub fn get_gutter_count(&self) -> usize {
        self.gutters.len()
    }

    pub fn get_gutter(&self, index: usize) -> Option<&Gutter> {
        self.gutters.get(index)
    }

    pub fn get_gutter_mut(&mut self, index: usize) -> Option<&mut Gutter> {
        self.gutters.get_mut(index)
    }

    pub fn clear_gutters(&mut self) {
        self.gutters.clear();
    }

    pub fn set_gutter_name(&mut self, index: usize, name: &str) -> bool {
        match self.gutters.get_mut(index) {
            Some(gutter) => {
                gutter.name = name.to_string();
                true
            }
            None => false,
        }
    }

    pub fn set_gutter_draw(&mut self, index: usize, draw: bool) -> bool {
        match self.gutters.get_mut(index) {
            Some(gutter) => {
                gutter.draw = draw;
                true
            }
            None => false,
        }
    }

    pub fn set_gutter_type(&mut self, index: usize, gutter_type: GutterType) -> bool {
        match self.gutters.get_mut(index) {
            Some(gutter) => {
                gutter.gutter_type = gutter_type;
                true
            }
            None => false,
        }
    }

    pub fn set_gutter_width(&mut self, index: usize, width: i32) -> bool {
        match self.gutters.get_mut(index) {
            Some(gutter) => {
                gutter.width = width;
                true
            }
            None => false,
        }
    }

    pub fn set_gutter_clickable(&mut self, index: usize, clickable: bool) -> bool {
        match self.gutters.get_mut(index) {
            Some(gutter) => {
                gutter.clickable = clickable;
                true
            }
            None => false,
        }
    }

    pub fn set_gutter_overwritable(&mut self, index: usize, overwritable: bool) -> bool {
        match self.gutters.get_mut(index) {
            Some(gutter) => {
                gutter.overwritable = overwritable;
                true
            }
            None => false,
        }
    }

    pub fn set_line_numbers_zero_padded(&mut self, _padded: bool) -> bool {
        // The engine keeps this flag for drawing; the state is not observable
        // without the draw path.
        true
    }

    // -- flags --------------------------------------------------------------

    pub fn set_draw_breakpoints_gutter(&mut self, draw: bool) {
        self.draw_breakpoints = draw;
    }

    pub fn is_drawing_breakpoints_gutter(&self) -> bool {
        self.draw_breakpoints
    }

    pub fn set_draw_bookmarks_gutter(&mut self, draw: bool) {
        self.draw_bookmarks = draw;
    }

    pub fn is_drawing_bookmarks_gutter(&self) -> bool {
        self.draw_bookmarks
    }

    pub fn set_draw_executing_lines_gutter(&mut self, draw: bool) {
        self.draw_executing_lines = draw;
    }

    pub fn is_drawing_executing_lines_gutter(&self) -> bool {
        self.draw_executing_lines
    }

    pub fn get_line_number_gutter(&self) -> i32 {
        self.line_number_gutter
    }

    pub fn get_line_number_digits(&self) -> i32 {
        self.line_number_digits
    }

    pub fn set_line_numbers_min_digits(&mut self, digits: i32) {
        self.line_numbers_min_digits = digits;
    }

    pub fn get_line_numbers_min_digits(&self) -> i32 {
        self.line_numbers_min_digits
    }

    pub fn set_line_number_padding(&mut self, padding: &str) {
        self.line_number_padding = padding.to_string();
    }

    pub fn get_line_number_padding(&self) -> &str {
        &self.line_number_padding
    }

    pub fn get_fold_gutter(&self) -> i32 {
        self.fold_gutter
    }

    pub fn set_draw_fold_gutter(&mut self, draw: bool) {
        self.draw_fold_gutter = draw;
    }

    pub fn is_drawing_fold_gutter(&self) -> bool {
        self.draw_fold_gutter
    }

    pub fn set_line_folding_enabled(&mut self, enabled: bool) {
        self.line_folding_enabled = enabled;
    }

    pub fn is_line_folding_enabled(&self) -> bool {
        self.line_folding_enabled
    }

    pub fn set_code_region_tags(&mut self, start: &str, end: &str) {
        self.code_region_start_tag = start.to_string();
        self.code_region_end_tag = end.to_string();
    }

    pub fn get_code_region_start_tag(&self) -> &str {
        &self.code_region_start_tag
    }

    pub fn get_code_region_end_tag(&self) -> &str {
        &self.code_region_end_tag
    }

    pub fn set_code_hint(&mut self, hint: &str) {
        self.code_hint = hint.to_string();
    }

    pub fn get_code_hint(&self) -> &str {
        &self.code_hint
    }

    pub fn set_code_hint_draw_below(&mut self, below: bool) {
        self.code_hint_draw_below = below;
    }

    pub fn is_code_hint_drawing_below(&self) -> bool {
        self.code_hint_draw_below
    }

    pub fn set_code_completion_enabled(&mut self, enabled: bool) {
        self.code_completion_enabled = enabled;
    }

    pub fn is_code_completion_enabled(&self) -> bool {
        self.code_completion_enabled
    }

    pub fn set_code_completion_forced(&mut self, forced: bool) {
        self.code_completion_forced = forced;
    }

    pub fn is_code_completion_forced(&self) -> bool {
        self.code_completion_forced
    }

    pub fn set_highlight_matching_braces_enabled(&mut self, enabled: bool) {
        self.highlight_matching_braces_enabled = enabled;
    }

    pub fn is_highlight_matching_braces_enabled(&self) -> bool {
        self.highlight_matching_braces_enabled
    }

    pub fn set_symbol_lookup_on_click_enabled(&mut self, enabled: bool) {
        self.symbol_lookup_on_click_enabled = enabled;
    }

    pub fn is_symbol_lookup_on_click_enabled(&self) -> bool {
        self.symbol_lookup_on_click_enabled
    }

    pub fn set_symbol_tooltip_on_hover_enabled(&mut self, enabled: bool) {
        self.symbol_tooltip_on_hover_enabled = enabled;
    }

    pub fn is_symbol_tooltip_on_hover_enabled(&self) -> bool {
        self.symbol_tooltip_on_hover_enabled
    }

    pub fn set_line_length_guidelines(&mut self, columns: Vec<i32>) {
        self.line_length_guideline_columns = columns;
    }

    pub fn get_line_length_guidelines(&self) -> &[i32] {
        &self.line_length_guideline_columns
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indentation_defaults_and_setters() {
        let mut edit = CodeEditData::new();
        assert_eq!(edit.get_indent_size(), 4);
        assert_eq!(edit.get_indent_text(), "\t");
        assert!(!edit.is_auto_indent_enabled());
        assert!(!edit.is_indent_using_spaces());
        assert_eq!(edit.get_auto_indent_prefixes(), &[':', '{', '[', '(']);

        edit.set_indent_size(2);
        edit.set_indent_text("  ");
        edit.set_auto_indent_enabled(true);
        edit.set_indent_using_spaces(true);
        assert_eq!(edit.get_indent_size(), 2);
        assert_eq!(edit.get_indent_text(), "  ");
        assert!(edit.is_auto_indent_enabled());
        assert!(edit.is_indent_using_spaces());
    }

    #[test]
    fn brace_pairs_follow_the_engine_rules() {
        let mut edit = CodeEditData::new();
        assert!(!edit.is_auto_brace_completion_enabled());
        assert_eq!(edit.get_auto_brace_completion_pairs().len(), 5);
        assert!(edit.has_auto_brace_completion_pair("("));
        assert!(edit.has_auto_brace_completion_pair("\""));
        edit.set_auto_brace_completion_enabled(true);

        // Empty and non-symbol keys are refused, duplicates too.
        assert!(!edit.add_auto_brace_completion_pair("", ")"));
        assert!(!edit.add_auto_brace_completion_pair("a", ")"));
        assert!(!edit.add_auto_brace_completion_pair("(", "]"));
        assert!(!edit.add_auto_brace_completion_pair("(", "]"));

        // A longer key is inserted before the shorter ones.
        assert!(edit.add_auto_brace_completion_pair("```", "```"));
        assert_eq!(edit.get_auto_brace_completion_pairs()[0].open_key, "```");
        // A shorter key goes after every longer one (the engine counts longer
        // keys as its insertion offset).
        assert!(edit.add_auto_brace_completion_pair("<", ">"));
        assert_eq!(edit.get_auto_brace_completion_pairs()[0].open_key, "```");
        assert_eq!(edit.get_auto_brace_completion_pairs()[1].open_key, "<");

        assert!(edit.remove_auto_brace_completion_pair("<"));
        assert!(!edit.remove_auto_brace_completion_pair("<"));
        edit.clear_auto_brace_completion_pairs();
        assert!(edit.get_auto_brace_completion_pairs().is_empty());
    }

    #[test]
    fn delimiters() {
        let mut edit = CodeEditData::new();
        // Two string delimiters are added by the constructor.
        assert_eq!(edit.get_delimiters().len(), 2);
        assert!(edit.has_delimiter("\"", DelimiterType::String));
        assert!(edit.has_delimiter("'", DelimiterType::String));

        assert!(edit.add_comment_delimiter("#", "", true));
        assert!(edit.has_delimiter("#", DelimiterType::Comment));
        assert!(!edit.has_delimiter("#", DelimiterType::String));
        // A line-only delimiter must not have an end key, and vice versa.
        assert!(!edit.add_comment_delimiter("//", "", false));
        assert!(!edit.add_comment_delimiter("/*", "*/", true));
        // Duplicates are refused.
        assert!(!edit.add_comment_delimiter("#", "", true));
        assert!(!edit.add_string_delimiter("\"", "\"", false));

        assert!(edit.remove_delimiter("#", DelimiterType::Comment));
        assert!(!edit.remove_delimiter("#", DelimiterType::Comment));
        edit.clear_delimiters();
        assert!(edit.get_delimiters().is_empty());
    }

    #[test]
    fn gutters_and_line_number_state() {
        let mut edit = CodeEditData::new();
        assert_eq!(edit.get_gutter_count(), 3);
        assert_eq!(edit.get_line_number_gutter(), 1);
        assert_eq!(edit.get_fold_gutter(), 2);
        assert_eq!(edit.get_gutter(0).unwrap().name, "main_gutter");
        assert_eq!(edit.get_gutter(1).unwrap().name, "line_numbers");
        assert!(edit.get_gutter(0).unwrap().overwritable);
        assert_eq!(edit.get_gutter(1).unwrap().gutter_type, GutterType::Custom);
        assert_eq!(edit.get_gutter(2).unwrap().gutter_type, GutterType::Icon);

        let index = edit.add_gutter("breakpoints", GutterType::Icon, true, false);
        assert_eq!(index, 3);
        assert_eq!(edit.get_gutter_count(), 4);
        assert!(edit.set_gutter_draw(3, true));
        assert!(edit.get_gutter(3).unwrap().draw);
        assert!(edit.set_gutter_clickable(3, false));
        assert!(!edit.get_gutter(3).unwrap().clickable);
        assert!(edit.set_gutter_width(3, 12));
        assert_eq!(edit.get_gutter(3).unwrap().width, 12);
        assert!(!edit.set_gutter_name(9, "x"));
        assert!(edit.remove_gutter(3));
        assert_eq!(edit.get_gutter_count(), 3);

        assert_eq!(edit.get_line_number_digits(), 1);
        assert_eq!(edit.get_line_numbers_min_digits(), 3);
        assert_eq!(edit.get_line_number_padding(), " ");
        edit.set_line_numbers_min_digits(5);
        edit.set_line_number_padding("0");
        assert_eq!(edit.get_line_numbers_min_digits(), 5);
        assert_eq!(edit.get_line_number_padding(), "0");
        assert_eq!(GutterType::String as i32, 0);
        assert_eq!(GutterType::Custom as i32, 2);
    }

    #[test]
    fn flags_and_guidelines() {
        let mut edit = CodeEditData::new();
        assert!(!edit.is_drawing_breakpoints_gutter());
        assert!(!edit.is_drawing_bookmarks_gutter());
        assert!(!edit.is_drawing_executing_lines_gutter());
        assert!(!edit.is_drawing_fold_gutter());
        assert!(!edit.is_line_folding_enabled());
        assert_eq!(edit.get_code_region_start_tag(), "region");
        assert_eq!(edit.get_code_region_end_tag(), "endregion");
        assert_eq!(edit.get_code_hint(), "");
        assert!(edit.is_code_hint_drawing_below());
        assert!(!edit.is_code_completion_enabled());
        assert!(!edit.is_code_completion_forced());
        assert!(!edit.is_highlight_matching_braces_enabled());
        assert!(!edit.is_symbol_lookup_on_click_enabled());
        assert!(!edit.is_symbol_tooltip_on_hover_enabled());
        assert!(edit.get_line_length_guidelines().is_empty());

        edit.set_draw_breakpoints_gutter(true);
        edit.set_draw_bookmarks_gutter(true);
        edit.set_draw_executing_lines_gutter(true);
        edit.set_draw_fold_gutter(true);
        edit.set_line_folding_enabled(true);
        edit.set_code_region_tags("start", "end");
        edit.set_code_hint("hint");
        edit.set_code_hint_draw_below(false);
        edit.set_code_completion_enabled(true);
        edit.set_code_completion_forced(true);
        edit.set_highlight_matching_braces_enabled(true);
        edit.set_symbol_lookup_on_click_enabled(true);
        edit.set_symbol_tooltip_on_hover_enabled(true);
        edit.set_line_length_guidelines(vec![80, 120]);

        assert!(edit.is_drawing_breakpoints_gutter());
        assert!(edit.is_drawing_bookmarks_gutter());
        assert!(edit.is_drawing_executing_lines_gutter());
        assert!(edit.is_drawing_fold_gutter());
        assert!(edit.is_line_folding_enabled());
        assert_eq!(edit.get_code_region_start_tag(), "start");
        assert_eq!(edit.get_code_region_end_tag(), "end");
        assert_eq!(edit.get_code_hint(), "hint");
        assert!(!edit.is_code_hint_drawing_below());
        assert!(edit.is_code_completion_enabled());
        assert!(edit.is_code_completion_forced());
        assert!(edit.is_highlight_matching_braces_enabled());
        assert!(edit.is_symbol_lookup_on_click_enabled());
        assert!(edit.is_symbol_tooltip_on_hover_enabled());
        assert_eq!(edit.get_line_length_guidelines(), &[80, 120]);
    }
}
