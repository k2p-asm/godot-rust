//! `scene/gui/menu_button.*`: the menu button's state.
//!
//! Defaults: `switch_on_hover` off and `disable_shortcuts` off; the popup itself
//! is an object owned by the button (not ported). Drawing, shortcut dispatch and
//! popup handling are not ported.

#[derive(Clone, Debug, Default)]
pub struct MenuButtonData {
    switch_on_hover: bool,
    disable_shortcuts: bool,
}

impl MenuButtonData {
    pub fn new() -> Self {
        Self::default()
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

    pub fn is_shortcuts_disabled(&self) -> bool {
        self.disable_shortcuts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_setters() {
        let mut menu = MenuButtonData::new();
        assert!(!menu.is_switch_on_hover());
        assert!(!menu.is_shortcuts_disabled());
        menu.set_switch_on_hover(true);
        menu.set_disable_shortcuts(true);
        assert!(menu.is_switch_on_hover());
        assert!(menu.is_shortcuts_disabled());
    }
}
