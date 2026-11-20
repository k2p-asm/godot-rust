//! `scene/gui/check_box.*` and `scene/gui/check_button.*` (state only).
//!
//! Both constructors call `set_toggle_mode(true)` and set the text alignment to
//! the left; the rest is `Button` state. `CheckButton` is otherwise identical
//! here (its difference is the drawn on/off icons).

use crate::scene::base_button::{BaseButtonData, HorizontalAlignment};
use crate::scene::base_button::ButtonData;

/// `CheckBox`.
#[derive(Clone, Debug)]
pub struct CheckBoxData {
    pub button: ButtonData,
}

impl Default for CheckBoxData {
    fn default() -> Self {
        Self::new()
    }
}

impl CheckBoxData {
    pub fn new() -> Self {
        let mut button = ButtonData::new();
        button.base.set_toggle_mode(true);
        button.set_alignment(HorizontalAlignment::Left);
        Self { button }
    }

    pub fn set_checked(&mut self, checked: bool) -> bool {
        self.button.base.set_pressed(checked)
    }

    pub fn is_checked(&self) -> bool {
        self.button.base.is_pressed()
    }
}

/// `CheckButton`.
#[derive(Clone, Debug)]
pub struct CheckButtonData {
    pub button: ButtonData,
}

impl Default for CheckButtonData {
    fn default() -> Self {
        Self::new()
    }
}

impl CheckButtonData {
    pub fn new() -> Self {
        let mut button = ButtonData::new();
        button.base.set_toggle_mode(true);
        button.set_alignment(HorizontalAlignment::Left);
        Self { button }
    }

    pub fn set_checked(&mut self, checked: bool) -> bool {
        self.button.base.set_pressed(checked)
    }

    pub fn is_checked(&self) -> bool {
        self.button.base.is_pressed()
    }
}

/// Convenience for callers that only need the shared `BaseButton` knobs.
pub fn toggle_base() -> BaseButtonData {
    let mut base = BaseButtonData::new();
    base.set_toggle_mode(true);
    base
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_box_constructor_state() {
        let mut c = CheckBoxData::new();
        assert!(c.button.base.is_toggle_mode());
        assert_eq!(c.button.get_alignment(), HorizontalAlignment::Left);
        assert!(!c.is_checked());

        assert!(c.set_checked(true));
        assert!(c.is_checked());
        assert!(c.button.base.is_pressed());
        assert!(!c.set_checked(true), "no change, no toggled signal");
        assert!(c.set_checked(false));
        assert!(!c.is_checked());
    }

    #[test]
    fn check_button_matches() {
        let mut c = CheckButtonData::new();
        assert!(c.button.base.is_toggle_mode());
        assert_eq!(c.button.get_alignment(), HorizontalAlignment::Left);
        assert!(c.set_checked(true));
        assert!(c.is_checked());

        // The shared helper produces the same toggle-mode base.
        assert!(toggle_base().is_toggle_mode());
    }
}
