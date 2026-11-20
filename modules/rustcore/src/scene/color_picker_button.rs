//! `scene/gui/color_picker.*` (`ColorPickerButton` state).
//!
//! Defaults from the engine: the picker colour is `Color()` (opaque black),
//! alpha and intensity editing are on, and the popup was never open. The popup
//! and the embedded `ColorPicker` objects are not ported; the button state and
//! its accessors are.
//!
//! Drawing, the popup lifecycle and the deferred picker creation are not ported.

use crate::math::color::Color;

#[derive(Clone, Debug)]
pub struct ColorPickerButtonData {
    color: Color,
    edit_alpha: bool,
    edit_intensity: bool,
    popup_was_open: bool,
}

impl Default for ColorPickerButtonData {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorPickerButtonData {
    pub fn new() -> Self {
        Self {
            color: Color::new(0.0, 0.0, 0.0, 1.0),
            edit_alpha: true,
            edit_intensity: true,
            popup_was_open: false,
        }
    }

    pub fn set_pick_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn get_pick_color(&self) -> Color {
        self.color
    }

    pub fn set_edit_alpha(&mut self, enabled: bool) {
        self.edit_alpha = enabled;
    }

    pub fn is_editing_alpha(&self) -> bool {
        self.edit_alpha
    }

    pub fn set_edit_intensity(&mut self, enabled: bool) {
        self.edit_intensity = enabled;
    }

    pub fn is_editing_intensity(&self) -> bool {
        self.edit_intensity
    }

    pub fn was_popup_open(&self) -> bool {
        self.popup_was_open
    }

    /// Port helper mirroring the engine's `popup_was_open` bookkeeping.
    pub fn set_popup_was_open(&mut self, open: bool) {
        self.popup_was_open = open;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let button = ColorPickerButtonData::new();
        assert_eq!(button.get_pick_color(), Color::new(0.0, 0.0, 0.0, 1.0));
        assert!(button.is_editing_alpha());
        assert!(button.is_editing_intensity());
        assert!(!button.was_popup_open());
    }

    #[test]
    fn setters() {
        let mut button = ColorPickerButtonData::new();
        button.set_pick_color(Color::new(0.2, 0.4, 0.6, 0.8));
        button.set_edit_alpha(false);
        button.set_edit_intensity(false);
        button.set_popup_was_open(true);
        assert_eq!(button.get_pick_color(), Color::new(0.2, 0.4, 0.6, 0.8));
        assert!(!button.is_editing_alpha());
        assert!(!button.is_editing_intensity());
        assert!(button.was_popup_open());
    }
}
