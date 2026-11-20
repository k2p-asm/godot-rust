//! `scene/gui/menu_button.*` and `scene/gui/link_button.*`: the two specialised
//! buttons.
//!
//! `MenuButton`'s constructor makes the button flat and toggled, switches it to
//! press-mode actions and disables shortcut handling; `switch_on_hover` starts
//! off. The popup itself is a handle.
//!
//! `LinkButton` starts with underline-always, no URI, auto text direction, the
//! default structured-text parser, no overrun trimming and the `…` ellipsis
//! character.
//!
//! Drawing, popup behaviour and text shaping are not ported.

use crate::scene::base_button::{ActionMode, BaseButtonData, HorizontalAlignment, VerticalAlignment};
use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum UnderlineMode {
    Always,
    OnHover,
    Never,
}

#[derive(Clone, Debug)]
pub struct MenuButtonData {
    pub base: BaseButtonData,
    switch_on_hover: bool,
    disable_shortcuts: bool,
    popup: Option<Rid>,
    // Button-side text state (the engine's Button owns it).
    text: String,
}

impl Default for MenuButtonData {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuButtonData {
    pub fn new() -> Self {
        let mut base = BaseButtonData::new();
        // `MenuButton::MenuButton`: flat, toggle mode, press action.
        base.set_toggle_mode(true);
        base.set_action_mode(ActionMode::Press);
        Self {
            base,
            switch_on_hover: false,
            disable_shortcuts: false,
            popup: None,
            text: String::new(),
        }
    }

    /// Flat is a `Button` property; exposed for parity with the constructor.
    pub fn is_flat(&self) -> bool {
        true
    }

    pub fn set_switch_on_hover(&mut self, enabled: bool) {
        self.switch_on_hover = enabled;
    }

    pub fn is_switch_on_hover(&self) -> bool {
        self.switch_on_hover
    }

    pub fn set_disable_shortcuts(&mut self, disabled: bool) {
        self.disable_shortcuts = disabled;
    }

    pub fn is_disabling_shortcuts(&self) -> bool {
        self.disable_shortcuts
    }

    pub fn set_popup(&mut self, popup: Option<Rid>) {
        self.popup = popup;
    }

    pub fn get_popup(&self) -> Option<Rid> {
        self.popup
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }
}

#[derive(Clone, Debug)]
pub struct LinkButtonData {
    pub base: BaseButtonData,
    text: String,
    xl_text: String,
    underline_mode: UnderlineMode,
    uri: String,
    language: String,
    text_direction: i32,
    structured_text_parser: i32,
    overrun_behavior: i32,
    el_char: String,
    alignment: HorizontalAlignment,
    vertical_alignment: VerticalAlignment,
}

impl Default for LinkButtonData {
    fn default() -> Self {
        Self::new()
    }
}

impl LinkButtonData {
    pub fn new() -> Self {
        Self {
            base: BaseButtonData::new(),
            text: String::new(),
            xl_text: String::new(),
            underline_mode: UnderlineMode::Always,
            uri: String::new(),
            language: String::new(),
            text_direction: 0,
            structured_text_parser: 0,
            overrun_behavior: 0,
            el_char: "…".to_string(),
            alignment: HorizontalAlignment::Left,
            vertical_alignment: VerticalAlignment::Center,
        }
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    pub fn set_xl_text(&mut self, text: &str) {
        self.xl_text = text.to_string();
    }

    pub fn get_xl_text(&self) -> &str {
        &self.xl_text
    }

    pub fn set_underline_mode(&mut self, mode: UnderlineMode) {
        self.underline_mode = mode;
    }

    pub fn get_underline_mode(&self) -> UnderlineMode {
        self.underline_mode
    }

    pub fn set_uri(&mut self, uri: &str) {
        self.uri = uri.to_string();
    }

    pub fn get_uri(&self) -> &str {
        &self.uri
    }

    pub fn set_language(&mut self, language: &str) {
        self.language = language.to_string();
    }

    pub fn get_language(&self) -> &str {
        &self.language
    }

    pub fn set_text_direction(&mut self, direction: i32) {
        self.text_direction = direction;
    }

    pub fn get_text_direction(&self) -> i32 {
        self.text_direction
    }

    pub fn set_structured_text_bidi_override(&mut self, parser: i32) {
        self.structured_text_parser = parser;
    }

    pub fn get_structured_text_bidi_override(&self) -> i32 {
        self.structured_text_parser
    }

    pub fn set_text_overrun_behavior(&mut self, behavior: i32) {
        self.overrun_behavior = behavior;
    }

    pub fn get_text_overrun_behavior(&self) -> i32 {
        self.overrun_behavior
    }

    /// The ellipsis used when the text does not fit (`…` by default).
    pub fn set_ellipsis_char(&mut self, character: &str) {
        self.el_char = character.to_string();
    }

    pub fn get_ellipsis_char(&self) -> &str {
        &self.el_char
    }

    pub fn set_alignment(&mut self, alignment: HorizontalAlignment) {
        self.alignment = alignment;
    }

    pub fn get_alignment(&self) -> HorizontalAlignment {
        self.alignment
    }

    pub fn set_vertical_alignment(&mut self, alignment: VerticalAlignment) {
        self.vertical_alignment = alignment;
    }

    pub fn get_vertical_alignment(&self) -> VerticalAlignment {
        self.vertical_alignment
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn menu_button_constructor_state() {
        let m = MenuButtonData::new();
        // Flat, toggle mode and press-mode actions come from the constructor.
        assert!(m.is_flat());
        assert!(m.base.is_toggle_mode());
        assert_eq!(m.base.get_action_mode(), ActionMode::Press);
        assert!(!m.is_switch_on_hover());
        assert!(!m.is_disabling_shortcuts());
        assert!(m.get_popup().is_none());
        assert_eq!(m.get_text(), "");

        // Press-mode actions matter: `is_pressed` reports the press attempt.
        assert!(!m.base.is_pressed());
    }

    #[test]
    fn menu_button_setters() {
        let mut m = MenuButtonData::new();
        m.set_switch_on_hover(true);
        m.set_disable_shortcuts(true);
        m.set_popup(Some(Rid::from_u64(2)));
        m.set_text("File");
        assert!(m.is_switch_on_hover());
        assert!(m.is_disabling_shortcuts());
        assert_eq!(m.get_popup(), Some(Rid::from_u64(2)));
        assert_eq!(m.get_text(), "File");
    }

    #[test]
    fn link_button_defaults() {
        let l = LinkButtonData::new();
        assert_eq!(l.get_text(), "");
        assert_eq!(l.get_xl_text(), "");
        assert_eq!(l.get_underline_mode(), UnderlineMode::Always);
        assert_eq!(l.get_uri(), "");
        assert_eq!(l.get_language(), "");
        assert_eq!(l.get_text_direction(), 0);
        assert_eq!(l.get_structured_text_bidi_override(), 0);
        assert_eq!(l.get_text_overrun_behavior(), 0);
        assert_eq!(l.get_ellipsis_char(), "…");
        assert_eq!(l.get_alignment(), HorizontalAlignment::Left);
        assert_eq!(l.get_vertical_alignment(), VerticalAlignment::Center);
        // A plain BaseButton: no toggle mode.
        assert!(!l.base.is_toggle_mode());
    }

    #[test]
    fn link_button_setters_and_enum_values() {
        let mut l = LinkButtonData::new();
        l.set_text("Docs");
        l.set_xl_text("DOCS");
        l.set_underline_mode(UnderlineMode::OnHover);
        l.set_uri("https://example.com");
        l.set_language("en");
        l.set_text_direction(2);
        l.set_structured_text_bidi_override(1);
        l.set_text_overrun_behavior(3);
        l.set_ellipsis_char("...");
        l.set_alignment(HorizontalAlignment::Center);
        l.set_vertical_alignment(VerticalAlignment::Top);

        assert_eq!(l.get_text(), "Docs");
        assert_eq!(l.get_xl_text(), "DOCS");
        assert_eq!(l.get_underline_mode(), UnderlineMode::OnHover);
        assert_eq!(l.get_uri(), "https://example.com");
        assert_eq!(l.get_language(), "en");
        assert_eq!(l.get_text_direction(), 2);
        assert_eq!(l.get_structured_text_bidi_override(), 1);
        assert_eq!(l.get_text_overrun_behavior(), 3);
        assert_eq!(l.get_ellipsis_char(), "...");
        assert_eq!(l.get_alignment(), HorizontalAlignment::Center);
        assert_eq!(l.get_vertical_alignment(), VerticalAlignment::Top);

        assert_eq!(UnderlineMode::Always as i32, 0);
        assert_eq!(UnderlineMode::OnHover as i32, 1);
        assert_eq!(UnderlineMode::Never as i32, 2);
    }
}
