//! `scene/gui/menu_bar.*`: the menu list and bar options (no native menus).
//!
//! Defaults from the header: switch on hover on, shortcut handling enabled,
//! native menus preferred, not flat, `start_index` -1, and no focused/selected/
//! active menu.
//!
//! Each menu entry keeps its title, tooltip, hidden/disabled flags and the popup
//! handle. `add_menu`/`remove_menu` maintain the list and shift `start_index`;
//! `clear` drops everything. `set_current_menu` accepts `-1` or a valid index.
//!
//! The native (global) menu integration, popups' contents, drawing and input are
//! not ported.

use crate::servers::rid::Rid;

#[derive(Clone, Debug, Default)]
pub struct MenuEntry {
    pub name: String,
    pub tooltip: String,
    pub hidden: bool,
    pub disabled: bool,
    pub popup: Option<Rid>,
}

#[derive(Clone, Debug)]
pub struct MenuBarData {
    menus: Vec<MenuEntry>,
    switch_on_hover: bool,
    disable_shortcuts: bool,
    prefer_native: bool,
    flat: bool,
    start_index: i32,
    focused_menu: i32,
    selected_menu: i32,
    active_menu: i32,
}

impl Default for MenuBarData {
    fn default() -> Self {
        Self::new()
    }
}

impl MenuBarData {
    pub fn new() -> Self {
        Self {
            menus: Vec::new(),
            switch_on_hover: true,
            disable_shortcuts: false,
            prefer_native: true,
            flat: false,
            start_index: -1,
            focused_menu: -1,
            selected_menu: -1,
            active_menu: -1,
        }
    }

    /// `add_menu`: appends a menu and returns its index.
    pub fn add_menu(&mut self, name: &str) -> usize {
        self.menus.push(MenuEntry {
            name: name.to_string(),
            ..MenuEntry::default()
        });
        self.menus.len() - 1
    }

    pub fn get_menu_count(&self) -> usize {
        self.menus.len()
    }

    pub fn get_menu(&self, index: usize) -> Option<&MenuEntry> {
        self.menus.get(index)
    }

    /// `ERR_FAIL_INDEX`; false for bad indices.
    pub fn set_menu_title(&mut self, index: usize, title: &str) -> bool {
        match self.menus.get_mut(index) {
            Some(menu) => {
                menu.name = title.to_string();
                true
            }
            None => false,
        }
    }

    pub fn get_menu_title(&self, index: usize) -> String {
        self.menus
            .get(index)
            .map(|menu| menu.name.clone())
            .unwrap_or_default()
    }

    pub fn set_menu_tooltip(&mut self, index: usize, tooltip: &str) -> bool {
        match self.menus.get_mut(index) {
            Some(menu) => {
                menu.tooltip = tooltip.to_string();
                true
            }
            None => false,
        }
    }

    pub fn get_menu_tooltip(&self, index: usize) -> String {
        self.menus
            .get(index)
            .map(|menu| menu.tooltip.clone())
            .unwrap_or_default()
    }

    pub fn set_menu_disabled(&mut self, index: usize, disabled: bool) -> bool {
        match self.menus.get_mut(index) {
            Some(menu) => {
                menu.disabled = disabled;
                true
            }
            None => false,
        }
    }

    pub fn is_menu_disabled(&self, index: usize) -> bool {
        self.menus
            .get(index)
            .map(|menu| menu.disabled)
            .unwrap_or(false)
    }

    pub fn set_menu_hidden(&mut self, index: usize, hidden: bool) -> bool {
        match self.menus.get_mut(index) {
            Some(menu) => {
                menu.hidden = hidden;
                true
            }
            None => false,
        }
    }

    pub fn is_menu_hidden(&self, index: usize) -> bool {
        self.menus
            .get(index)
            .map(|menu| menu.hidden)
            .unwrap_or(false)
    }

    /// The popup handle (`get_menu_popup`), stored by the caller.
    pub fn set_menu_popup(&mut self, index: usize, popup: Option<Rid>) -> bool {
        match self.menus.get_mut(index) {
            Some(menu) => {
                menu.popup = popup;
                true
            }
            None => false,
        }
    }

    pub fn get_menu_popup(&self, index: usize) -> Option<Rid> {
        self.menus.get(index).and_then(|menu| menu.popup)
    }

    /// `remove_menu`: shifts `start_index` and clears stale focus/selection.
    pub fn remove_menu(&mut self, index: usize) -> bool {
        if index >= self.menus.len() {
            return false;
        }
        self.menus.remove(index);
        let index = index as i32;
        if self.start_index == index {
            self.start_index = -1;
        } else if self.start_index > index {
            self.start_index -= 1;
        }
        if self.focused_menu == index {
            self.focused_menu = -1;
        } else if self.focused_menu > index {
            self.focused_menu -= 1;
        }
        if self.selected_menu == index {
            self.selected_menu = -1;
        } else if self.selected_menu > index {
            self.selected_menu -= 1;
        }
        if self.active_menu == index {
            self.active_menu = -1;
        } else if self.active_menu > index {
            self.active_menu -= 1;
        }
        true
    }

    pub fn clear(&mut self) {
        self.menus.clear();
        self.start_index = -1;
        self.focused_menu = -1;
        self.selected_menu = -1;
        self.active_menu = -1;
    }

    /// `set_current_menu`: `-1` clears, otherwise the index must exist.
    pub fn set_current_menu(&mut self, index: i32) -> bool {
        if index != -1 && (index < 0 || index as usize >= self.menus.len()) {
            return false;
        }
        self.selected_menu = index;
        true
    }

    pub fn get_current_menu(&self) -> i32 {
        self.selected_menu
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

    pub fn set_prefer_global_menu(&mut self, prefer: bool) {
        self.prefer_native = prefer;
    }

    pub fn is_prefer_global_menu(&self) -> bool {
        self.prefer_native
    }

    pub fn set_flat(&mut self, flat: bool) {
        self.flat = flat;
    }

    pub fn is_flat(&self) -> bool {
        self.flat
    }

    /// `start_index` marks the first menu shown on the bar (-1 for all).
    pub fn set_start_index(&mut self, index: i32) {
        self.start_index = index;
    }

    pub fn get_start_index(&self) -> i32 {
        self.start_index
    }

    pub fn set_focused_menu(&mut self, index: i32) {
        self.focused_menu = index;
    }

    pub fn get_focused_menu(&self) -> i32 {
        self.focused_menu
    }

    pub fn set_active_menu(&mut self, index: i32) {
        self.active_menu = index;
    }

    pub fn get_active_menu(&self) -> i32 {
        self.active_menu
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let m = MenuBarData::new();
        assert_eq!(m.get_menu_count(), 0);
        assert!(m.is_switch_on_hover());
        assert!(!m.is_shortcuts_disabled());
        assert!(m.is_prefer_global_menu());
        assert!(!m.is_flat());
        assert_eq!(m.get_start_index(), -1);
        assert_eq!(m.get_current_menu(), -1);
        assert_eq!(m.get_focused_menu(), -1);
        assert_eq!(m.get_active_menu(), -1);
    }

    #[test]
    fn menus_and_fields() {
        let mut m = MenuBarData::new();
        assert_eq!(m.add_menu("File"), 0);
        assert_eq!(m.add_menu("Edit"), 1);
        assert_eq!(m.get_menu_count(), 2);
        assert_eq!(m.get_menu_title(0), "File");
        assert_eq!(m.get_menu_tooltip(0), "");
        assert!(m.get_menu_popup(0).is_none());

        assert!(m.set_menu_title(0, "Project"));
        assert!(m.set_menu_tooltip(0, "Project actions"));
        assert!(m.set_menu_disabled(1, true));
        assert!(m.set_menu_hidden(1, true));
        assert!(m.set_menu_popup(0, Some(Rid::from_u64(4))));

        assert_eq!(m.get_menu_title(0), "Project");
        assert_eq!(m.get_menu_tooltip(0), "Project actions");
        assert!(m.is_menu_disabled(1));
        assert!(m.is_menu_hidden(1));
        assert_eq!(m.get_menu_popup(0), Some(Rid::from_u64(4)));
        assert!(m.get_menu(0).is_some());

        // Bad indices fail and read defaults.
        assert!(!m.set_menu_title(5, "x"));
        assert!(!m.set_menu_disabled(5, true));
        assert!(!m.set_menu_hidden(5, true));
        assert!(!m.set_menu_popup(5, None));
        assert_eq!(m.get_menu_title(5), "");
        assert_eq!(m.get_menu_popup(5), None);
        assert!(m.get_menu(5).is_none());
        assert!(!m.is_menu_disabled(5));
    }

    #[test]
    fn current_menu_and_options() {
        let mut m = MenuBarData::new();
        m.add_menu("a");
        m.add_menu("b");
        assert!(!m.set_current_menu(2));
        assert!(m.set_current_menu(1));
        assert_eq!(m.get_current_menu(), 1);
        assert!(m.set_current_menu(-1));
        assert_eq!(m.get_current_menu(), -1);

        m.set_switch_on_hover(false);
        m.set_disable_shortcuts(true);
        m.set_prefer_global_menu(false);
        m.set_flat(true);
        m.set_start_index(1);
        assert!(!m.is_switch_on_hover());
        assert!(m.is_shortcuts_disabled());
        assert!(!m.is_prefer_global_menu());
        assert!(m.is_flat());
        assert_eq!(m.get_start_index(), 1);

        m.set_focused_menu(0);
        m.set_active_menu(1);
        assert_eq!(m.get_focused_menu(), 0);
        assert_eq!(m.get_active_menu(), 1);
    }

    #[test]
    fn removal_shifts_state() {
        let mut m = MenuBarData::new();
        m.add_menu("a");
        m.add_menu("b");
        m.add_menu("c");
        m.set_start_index(2);
        m.set_current_menu(2);
        m.set_focused_menu(2);
        m.set_active_menu(2);

        assert!(m.remove_menu(0));
        assert_eq!(m.get_menu_count(), 2);
        assert_eq!(m.get_start_index(), 1);
        assert_eq!(m.get_current_menu(), 1);
        assert_eq!(m.get_focused_menu(), 1);
        assert_eq!(m.get_active_menu(), 1);
        assert_eq!(m.get_menu_title(0), "b");

        // Removing the selected menu clears the selection.
        assert!(m.remove_menu(1));
        assert_eq!(m.get_current_menu(), -1);
        assert_eq!(m.get_focused_menu(), -1);
        assert_eq!(m.get_active_menu(), -1);
        assert_eq!(m.get_start_index(), -1, "removing the start menu clears it");
        assert!(!m.remove_menu(9));

        m.clear();
        assert_eq!(m.get_menu_count(), 0);
        assert_eq!(m.get_start_index(), -1);
        assert_eq!(m.get_current_menu(), -1);
    }
}
