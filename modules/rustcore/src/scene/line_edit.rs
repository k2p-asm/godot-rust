//! `scene/gui/line_edit.*`: text, caret and selection state (no input handling).
//!
//! Constructor defaults from the header: left alignment, not editing, editable
//! off, plain text, empty text/placeholder, the bullet `•` as the secret
//! character, `EXPAND_MODE_ORIGINAL_SIZE`, selection/deselect-on-focus-loss/
//! drag-and-drop on, context and emoji menus on, caret not mid-grapheme, caret at
//! 0, no maximum length, auto text direction, the default structured-text parser,
//! expand-to-text off, clear button off, shortcut keys on, virtual keyboard on
//! with `KEYBOARD_TYPE_DEFAULT`, middle-mouse paste on, and a scale-1 right icon.
//!
//! Ported behaviours:
//!
//! - `set_text` clears the selection, truncates through `insert_text_at_caret` and
//!   then resets the caret and scroll to 0;
//! - `insert_text_at_caret` respects `max_length` (rejecting the overflow);
//! - `delete_char` is a no-op with an empty text or caret 0 and removes the
//!   character before the caret (no grapheme clusters here);
//! - `delete_text` refuses inverted/out-of-range columns;
//! - `select` clamps both columns, `(0, 0)` deselects, and `from >= to` is
//!   ignored; `select_all` needs a non-empty text; disabling selection deselects;
//! - `set_caret_column` clamps into `0..=len`;
//! - `set_max_length` rejects negatives and re-applies the text;
//! - the secret character is cut to one character like the engine.
//!
//! Columns are character indices. Shaping, drawing, menus, undo, IME and the
//! virtual keyboard are not ported.

use crate::scene::base_button::HorizontalAlignment;
use crate::servers::rid::Rid;
use crate::variant::Variant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ExpandMode {
    OriginalSize,
    FitToText,
    FitToLineEdit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TextDirection {
    Auto,
    Ltr,
    Rtl,
    Inherited,
}

/// `set_x(bool)` / `is_x()` pairs for the plain boolean flags.
macro_rules! flag_accessors {
    ($($field:ident, $set:ident, $get:ident);* $(;)?) => {
        $(
            pub fn $set(&mut self, enabled: bool) {
                self.$field = enabled;
            }

            pub fn $get(&self) -> bool {
                self.$field
            }
        )*
    };
}

#[derive(Clone, Debug)]
pub struct LineEditData {
    text: Vec<char>,
    placeholder: String,
    editable: bool,
    editing: bool,
    keep_editing_on_text_submit: bool,
    pass: bool,
    secret_character: char,
    alignment: HorizontalAlignment,
    icon_expand_mode: ExpandMode,
    caret_column: usize,
    max_length: i32,
    // Selection as `(begin, end)` columns, only meaningful when `enabled`.
    selection_begin: usize,
    selection_end: usize,
    selection_enabled: bool,
    selecting_enabled: bool,
    deselect_on_focus_loss_enabled: bool,
    drag_and_drop_selection_enabled: bool,
    context_menu_enabled: bool,
    emoji_menu_enabled: bool,
    backspace_deletes_composite_character_enabled: bool,
    caret_mid_grapheme_enabled: bool,
    expand_to_text_length: bool,
    clear_button_enabled: bool,
    shortcut_keys_enabled: bool,
    virtual_keyboard_enabled: bool,
    virtual_keyboard_show_on_focus: bool,
    middle_mouse_paste_enabled: bool,
    text_direction: TextDirection,
    structured_text_parser: i32,
    structured_text_options: Vec<Variant>,
    draw_control_chars: bool,
    language: String,
    right_icon: Option<Rid>,
    flat: bool,
    right_icon_scale: f32,
    draw_caret: bool,
}

impl Default for LineEditData {
    fn default() -> Self {
        Self::new()
    }
}

impl LineEditData {
    pub fn new() -> Self {
        Self {
            text: Vec::new(),
            placeholder: String::new(),
            editable: false,
            editing: false,
            keep_editing_on_text_submit: false,
            pass: false,
            secret_character: '\u{2022}',
            alignment: HorizontalAlignment::Left,
            icon_expand_mode: ExpandMode::OriginalSize,
            caret_column: 0,
            max_length: 0,
            selection_begin: 0,
            selection_end: 0,
            selection_enabled: false,
            selecting_enabled: true,
            deselect_on_focus_loss_enabled: true,
            drag_and_drop_selection_enabled: true,
            context_menu_enabled: true,
            emoji_menu_enabled: true,
            backspace_deletes_composite_character_enabled: false,
            caret_mid_grapheme_enabled: false,
            expand_to_text_length: false,
            clear_button_enabled: false,
            shortcut_keys_enabled: true,
            virtual_keyboard_enabled: true,
            virtual_keyboard_show_on_focus: true,
            middle_mouse_paste_enabled: true,
            text_direction: TextDirection::Auto,
            structured_text_parser: 0,
            structured_text_options: Vec::new(),
            draw_control_chars: false,
            language: String::new(),
            right_icon: None,
            flat: false,
            right_icon_scale: 1.0,
            draw_caret: true,
        }
    }

    // -- text ---------------------------------------------------------------

    pub fn get_text(&self) -> String {
        self.text.iter().collect()
    }

    pub fn get_text_length(&self) -> usize {
        self.text.len()
    }

    /// `set_text`: replace everything, then caret and scroll go to 0.
    pub fn set_text(&mut self, text: &str) {
        self.text.clear();
        self.caret_column = 0;
        self.selection_enabled = false;
        self.insert_text_at_caret(text);
        self.caret_column = 0;
    }

    /// `insert_text_at_caret`; returns the overflow that was rejected.
    pub fn insert_text_at_caret(&mut self, text: &str) -> String {
        let mut insert: String = text.to_string();
        let mut rejected = String::new();
        if self.max_length > 0 {
            let available = self.max_length as usize - self.text.len();
            let insert_len = insert.chars().count();
            if insert_len > available {
                let keep: String = insert.chars().take(available).collect();
                rejected = insert.chars().skip(available).collect();
                insert = keep;
            }
        }
        let caret = self.caret_column.min(self.text.len());
        let mut chars: Vec<char> = insert.chars().collect();
        let inserted = chars.len();
        chars.extend(self.text[caret..].iter());
        self.text.truncate(caret);
        self.text.extend(chars);
        self.set_caret_column((caret + inserted) as i32);
        rejected
    }

    /// `delete_char`: removes the character before the caret.
    pub fn delete_char(&mut self) {
        if self.text.is_empty() || self.caret_column == 0 {
            return;
        }
        let caret = self.caret_column.min(self.text.len());
        self.text.remove(caret - 1);
        self.set_caret_column((caret - 1) as i32);
    }

    /// `delete_text`: refuses inverted or out-of-range columns.
    pub fn delete_text(&mut self, from_column: i32, to_column: i32) -> bool {
        if from_column < 0
            || from_column > to_column
            || to_column as usize > self.text.len()
        {
            return false;
        }
        self.text
            .drain(from_column as usize..to_column as usize);
        self.set_caret_column(from_column);
        true
    }

    pub fn set_placeholder(&mut self, text: &str) {
        self.placeholder = text.to_string();
    }

    pub fn get_placeholder(&self) -> &str {
        &self.placeholder
    }

    pub fn set_editable(&mut self, editable: bool) {
        self.editable = editable;
        if !editable {
            self.editing = false;
        }
    }

    pub fn is_editable(&self) -> bool {
        self.editable
    }

    pub fn is_editing(&self) -> bool {
        self.editing
    }

    pub fn set_editing(&mut self, editing: bool) {
        self.editing = editing;
    }

    pub fn set_keep_editing_on_text_submit(&mut self, enabled: bool) {
        self.keep_editing_on_text_submit = enabled;
    }

    pub fn is_editing_kept_on_text_submit(&self) -> bool {
        self.keep_editing_on_text_submit
    }

    pub fn set_secret(&mut self, enabled: bool) {
        self.pass = enabled;
    }

    pub fn is_secret(&self) -> bool {
        self.pass
    }

    /// `set_secret_character`: only the first character is kept.
    pub fn set_secret_character(&mut self, character: &str) -> bool {
        let mut chars = character.chars();
        let first = match chars.next() {
            Some(c) => c,
            None => return false,
        };
        let truncated = chars.next().is_some();
        self.secret_character = first;
        truncated
    }

    pub fn get_secret_character(&self) -> char {
        self.secret_character
    }

    pub fn set_alignment(&mut self, alignment: HorizontalAlignment) {
        self.alignment = alignment;
    }

    pub fn get_alignment(&self) -> HorizontalAlignment {
        self.alignment
    }

    pub fn set_icon_expand_mode(&mut self, mode: ExpandMode) {
        self.icon_expand_mode = mode;
    }

    pub fn get_icon_expand_mode(&self) -> ExpandMode {
        self.icon_expand_mode
    }

    // -- caret and selection -------------------------------------------------

    pub fn set_caret_column(&mut self, column: i32) {
        let len = self.text.len() as i32;
        self.caret_column = column.clamp(0, len) as usize;
    }

    pub fn get_caret_column(&self) -> i32 {
        self.caret_column as i32
    }

    /// `set_max_length`: negatives are rejected (`ERR_FAIL_COND`).
    pub fn set_max_length(&mut self, max_length: i32) -> bool {
        if max_length < 0 {
            return false;
        }
        self.max_length = max_length;
        // Re-applying the text truncates it to the new limit.
        let text = self.get_text();
        self.set_text(&text);
        true
    }

    pub fn get_max_length(&self) -> i32 {
        self.max_length
    }

    pub fn deselect(&mut self) {
        self.selection_begin = 0;
        self.selection_end = 0;
        self.selection_enabled = false;
    }

    pub fn has_selection(&self) -> bool {
        self.selection_enabled
    }

    pub fn get_selected_text(&self) -> String {
        if !self.selection_enabled {
            return String::new();
        }
        self.text[self.selection_begin..self.selection_end]
            .iter()
            .collect()
    }

    /// `-1` when nothing is selected.
    pub fn get_selection_from_column(&self) -> i32 {
        if !self.selection_enabled {
            return -1;
        }
        self.selection_begin as i32
    }

    pub fn get_selection_to_column(&self) -> i32 {
        if !self.selection_enabled {
            return -1;
        }
        self.selection_end as i32
    }

    pub fn select_all(&mut self) {
        if !self.selecting_enabled {
            return;
        }
        if self.text.is_empty() {
            self.set_caret_column(0);
            return;
        }
        self.selection_begin = 0;
        self.selection_end = self.text.len();
        self.selection_enabled = true;
    }

    /// `select`: `(0, 0)` deselects; both columns are clamped to the text.
    pub fn select(&mut self, mut from: i32, mut to: i32) {
        if !self.selecting_enabled {
            return;
        }
        if from == 0 && to == 0 {
            self.deselect();
            return;
        }
        let len = self.text.len() as i32;
        if from < 0 {
            from = 0;
        }
        if from > len {
            from = len;
        }
        if to < 0 || to > len {
            to = len;
        }
        if from >= to {
            return;
        }
        self.selection_begin = from as usize;
        self.selection_end = to as usize;
        self.selection_enabled = true;
    }

    pub fn selection_delete(&mut self) {
        if self.selection_enabled {
            let (begin, end) = (self.selection_begin as i32, self.selection_end as i32);
            self.delete_text(begin, end);
            self.deselect();
        }
    }

    // -- flags ---------------------------------------------------------------

    flag_accessors! {
        deselect_on_focus_loss_enabled, set_deselect_on_focus_loss_enabled, is_deselect_on_focus_loss_enabled;
        drag_and_drop_selection_enabled, set_drag_and_drop_selection_enabled, is_drag_and_drop_selection_enabled;
        context_menu_enabled, set_context_menu_enabled, is_context_menu_enabled;
        emoji_menu_enabled, set_emoji_menu_enabled, is_emoji_menu_enabled;
        backspace_deletes_composite_character_enabled, set_backspace_deletes_composite_character_enabled, is_backspace_deletes_composite_character_enabled;
        caret_mid_grapheme_enabled, set_caret_mid_grapheme_enabled, is_caret_mid_grapheme_enabled;
        expand_to_text_length, set_expand_to_text_length_enabled, is_expand_to_text_length_enabled;
        clear_button_enabled, set_clear_button_enabled, is_clear_button_enabled;
        shortcut_keys_enabled, set_shortcut_keys_enabled, is_shortcut_keys_enabled;
        virtual_keyboard_enabled, set_virtual_keyboard_enabled, is_virtual_keyboard_enabled;
        virtual_keyboard_show_on_focus, set_virtual_keyboard_show_on_focus, get_virtual_keyboard_show_on_focus;
        middle_mouse_paste_enabled, set_middle_mouse_paste_enabled, is_middle_mouse_paste_enabled;
        draw_control_chars, set_draw_control_chars, is_drawing_control_chars;
        flat, set_flat, is_flat;
    }

    /// Disabling selection also clears it.
    pub fn set_selecting_enabled(&mut self, enabled: bool) {
        self.selecting_enabled = enabled;
        if !enabled {
            self.deselect();
        }
    }

    pub fn is_selecting_enabled(&self) -> bool {
        self.selecting_enabled
    }

    pub fn set_text_direction(&mut self, direction: i32) -> bool {
        let parsed = match direction {
            -1 => TextDirection::Inherited,
            0 => TextDirection::Auto,
            1 => TextDirection::Ltr,
            2 => TextDirection::Rtl,
            3 => TextDirection::Inherited,
            _ => return false,
        };
        self.text_direction = parsed;
        true
    }

    pub fn get_text_direction(&self) -> TextDirection {
        self.text_direction
    }

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

    pub fn set_language(&mut self, language: &str) {
        self.language = language.to_string();
    }

    pub fn get_language(&self) -> &str {
        &self.language
    }

    pub fn set_right_icon(&mut self, icon: Option<Rid>) {
        self.right_icon = icon;
    }

    pub fn get_right_icon(&self) -> Option<Rid> {
        self.right_icon
    }

    pub fn set_right_icon_scale(&mut self, scale: f32) {
        self.right_icon_scale = scale;
    }

    pub fn get_right_icon_scale(&self) -> f32 {
        self.right_icon_scale
    }

    pub fn set_draw_caret(&mut self, draw: bool) {
        self.draw_caret = draw;
    }

    pub fn get_draw_caret(&self) -> bool {
        self.draw_caret
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let l = LineEditData::new();
        assert_eq!(l.get_text(), "");
        assert_eq!(l.get_placeholder(), "");
        assert!(!l.is_editable());
        assert!(!l.is_editing());
        assert!(!l.is_secret());
        assert_eq!(l.get_secret_character(), '\u{2022}');
        assert_eq!(l.get_alignment(), HorizontalAlignment::Left);
        assert_eq!(l.get_icon_expand_mode(), ExpandMode::OriginalSize);
        assert_eq!(l.get_caret_column(), 0);
        assert_eq!(l.get_max_length(), 0);
        assert!(!l.has_selection());
        assert!(l.is_selecting_enabled());
        assert!(l.is_deselect_on_focus_loss_enabled());
        assert!(l.is_drag_and_drop_selection_enabled());
        assert!(l.is_context_menu_enabled());
        assert!(l.is_emoji_menu_enabled());
        assert!(!l.is_backspace_deletes_composite_character_enabled());
        assert!(!l.is_caret_mid_grapheme_enabled());
        assert!(!l.is_expand_to_text_length_enabled());
        assert!(!l.is_clear_button_enabled());
        assert!(l.is_shortcut_keys_enabled());
        assert!(l.is_virtual_keyboard_enabled());
        assert!(l.get_virtual_keyboard_show_on_focus());
        assert!(l.is_middle_mouse_paste_enabled());
        assert_eq!(l.get_text_direction(), TextDirection::Auto);
        assert!(!l.is_drawing_control_chars());
        assert!(!l.is_flat());
        assert_eq!(l.get_right_icon_scale(), 1.0);
        assert!(l.get_right_icon().is_none());
    }

    #[test]
    fn text_insertion_and_truncation() {
        let mut l = LineEditData::new();
        l.set_text("hello");
        assert_eq!(l.get_text(), "hello");
        assert_eq!(l.get_caret_column(), 0, "set_text resets the caret");

        l.set_caret_column(2);
        assert_eq!(l.insert_text_at_caret("XY"), "");
        assert_eq!(l.get_text(), "heXYllo");
        assert_eq!(l.get_caret_column(), 4);

        // max_length truncates the insert and reports the overflow.
        l.set_max_length(6);
        assert_eq!(l.get_text(), "heXYll");
        l.set_caret_column(6);
        assert_eq!(l.insert_text_at_caret("abc"), "abc");
        assert_eq!(l.get_text(), "heXYll");
        // Still full: single characters are rejected too.
        assert_eq!(l.insert_text_at_caret("z"), "z");
        assert_eq!(l.get_text(), "heXYll");

        // Negative limits are refused.
        assert!(!l.set_max_length(-1));
        assert_eq!(l.get_max_length(), 6);
    }

    #[test]
    fn deletion_rules() {
        let mut l = LineEditData::new();
        l.set_text("abc");
        l.delete_char();
        assert_eq!(l.get_text(), "abc", "caret is at 0");

        l.set_caret_column(2);
        l.delete_char();
        assert_eq!(l.get_text(), "ac");
        assert_eq!(l.get_caret_column(), 1);

        assert!(!l.delete_text(-1, 1));
        assert!(!l.delete_text(2, 1));
        assert!(!l.delete_text(0, 9));
        assert!(l.delete_text(0, 2));
        assert_eq!(l.get_text(), "");
        assert_eq!(l.get_caret_column(), 0);
    }

    #[test]
    fn caret_clamping() {
        let mut l = LineEditData::new();
        l.set_text("abcd");
        l.set_caret_column(99);
        assert_eq!(l.get_caret_column(), 4);
        l.set_caret_column(-5);
        assert_eq!(l.get_caret_column(), 0);
        l.set_caret_column(2);
        assert_eq!(l.get_caret_column(), 2);
    }

    #[test]
    fn selection_rules() {
        let mut l = LineEditData::new();
        l.set_text("abcdef");
        assert_eq!(l.get_selection_from_column(), -1);
        assert_eq!(l.get_selection_to_column(), -1);
        assert_eq!(l.get_selected_text(), "");

        l.select(1, 4);
        assert!(l.has_selection());
        assert_eq!(l.get_selected_text(), "bcd");
        assert_eq!(l.get_selection_from_column(), 1);
        assert_eq!(l.get_selection_to_column(), 4);

        // (0, 0) deselects; inverted/empty ranges are ignored.
        l.select(0, 0);
        assert!(!l.has_selection());
        l.select(3, 3);
        assert!(!l.has_selection());
        l.select(5, 1);
        assert!(!l.has_selection());

        // Columns clamp into the text.
        l.select(-5, 99);
        assert_eq!(l.get_selected_text(), "abcdef");

        l.select_all();
        assert_eq!(l.get_selected_text(), "abcdef");
        l.selection_delete();
        assert_eq!(l.get_text(), "");
        assert!(!l.has_selection());
    }

    #[test]
    fn selection_disabled_blocks_selecting() {
        let mut l = LineEditData::new();
        l.set_text("abc");
        l.select(0, 2);
        assert!(l.has_selection());
        l.set_selecting_enabled(false);
        assert!(!l.has_selection(), "disabling selection deselects");
        l.select(0, 2);
        assert!(!l.has_selection());
        l.select_all();
        assert!(!l.has_selection());

        // An empty text cannot be select-all'ed.
        let mut e = LineEditData::new();
        e.select_all();
        assert!(!e.has_selection());
        assert_eq!(e.get_caret_column(), 0);
    }

    #[test]
    fn secret_character_and_directions() {
        let mut l = LineEditData::new();
        l.set_secret(true);
        assert!(l.is_secret());
        // Multi-character input is cut to one character (and reported).
        assert!(l.set_secret_character("***"));
        assert_eq!(l.get_secret_character(), '*');
        assert!(!l.set_secret_character("*"));
        assert!(!l.set_secret_character(""));

        assert!(l.set_text_direction(-1));
        assert_eq!(l.get_text_direction(), TextDirection::Inherited);
        assert!(l.set_text_direction(2));
        assert_eq!(l.get_text_direction(), TextDirection::Rtl);
        assert!(!l.set_text_direction(4));

        l.set_editable(true);
        l.set_editing(true);
        assert!(l.is_editable() && l.is_editing());
        l.set_editable(false);
        assert!(!l.is_editing(), "leaving edit mode clears editing");

        assert_eq!(ExpandMode::OriginalSize as i32, 0);
        assert_eq!(ExpandMode::FitToText as i32, 1);
        assert_eq!(ExpandMode::FitToLineEdit as i32, 2);
    }
}
