//! `scene/gui/popup_menu.*`: the item store, selection and popup options.
//!
//! Items carry the fields the engine's `Item` struct does: text/xl-text, icon
//! with max width and modulate, checkable type (none/check box/radio button) with
//! `checked`, multistate count/state, visibility, separator/disabled flags, id,
//! metadata, submenu name, tooltip and indent.
//!
//! Ported: every add variant (`add_item`, `add_icon_item`, `add_check_item`,
//! `add_radio_check_item`, `add_separator`, `add_submenu_item`), the id/index
//! lookups, the per-field accessors with the engine's empty returns for bad
//! indices, `remove_item` with the selection adjustment, `clear`, the current and
//! focused indices, and the popup options: the three hide-on-selection flags,
//! `allow_search`, `prefer_native_menu`, the system-menu handle and the search
//! bar settings.
//!
//! Shortcuts, native menus, text shaping, drawing and activation signals are not
//! ported.

use crate::math::color::Color;
use crate::servers::rid::Rid;
use crate::variant::Variant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum CheckableType {
    None,
    CheckBox,
    RadioButton,
}

#[derive(Clone, Debug)]
pub struct PopupMenuItem {
    pub text: String,
    pub xl_text: String,
    pub icon: Option<Rid>,
    pub icon_max_width: i32,
    pub icon_modulate: Color,
    pub checked: bool,
    pub visible: bool,
    pub checkable_type: CheckableType,
    pub max_states: i32,
    pub state: i32,
    pub separator: bool,
    pub disabled: bool,
    pub id: i32,
    pub metadata: Option<Variant>,
    pub submenu: String,
    pub tooltip: String,
    pub indent: i32,
}

impl Default for PopupMenuItem {
    fn default() -> Self {
        Self {
            text: String::new(),
            xl_text: String::new(),
            icon: None,
            icon_max_width: 0,
            icon_modulate: Color::new(1.0, 1.0, 1.0, 1.0),
            checked: false,
            visible: true,
            checkable_type: CheckableType::None,
            max_states: 0,
            state: 0,
            separator: false,
            disabled: false,
            id: 0,
            metadata: None,
            submenu: String::new(),
            tooltip: String::new(),
            indent: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct PopupMenuData {
    items: Vec<PopupMenuItem>,
    current: i32,
    focused: i32,
    hide_on_item_selection: bool,
    hide_on_checkable_item_selection: bool,
    hide_on_multistate_item_selection: bool,
    allow_search: bool,
    prefer_native_menu: bool,
    system_menu: Option<Rid>,
    search_bar_enabled: bool,
    search_bar_min_item_count: i32,
    search_bar_fuzzy_search_enabled: bool,
}

impl Default for PopupMenuData {
    fn default() -> Self {
        Self::new()
    }
}

impl PopupMenuData {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            current: -1,
            focused: -1,
            hide_on_item_selection: true,
            hide_on_checkable_item_selection: true,
            hide_on_multistate_item_selection: false,
            allow_search: true,
            prefer_native_menu: false,
            system_menu: None,
            search_bar_enabled: false,
            search_bar_min_item_count: 0,
            search_bar_fuzzy_search_enabled: true,
        }
    }

    fn push(&mut self, item: PopupMenuItem) -> usize {
        self.items.push(item);
        self.items.len() - 1
    }

    // -- adding -------------------------------------------------------------

    /// `add_item(label, id = 0, accel = 0)`.
    pub fn add_item(&mut self, label: &str, id: i32) -> usize {
        let mut item = PopupMenuItem {
            text: label.to_string(),
            xl_text: label.to_string(),
            id,
            ..PopupMenuItem::default()
        };
        item.icon_modulate = Color::new(1.0, 1.0, 1.0, 1.0);
        self.push(item)
    }

    pub fn add_icon_item(&mut self, icon: Option<Rid>, label: &str, id: i32) -> usize {
        let mut item = PopupMenuItem {
            text: label.to_string(),
            xl_text: label.to_string(),
            icon,
            id,
            ..PopupMenuItem::default()
        };
        item.icon_modulate = Color::new(1.0, 1.0, 1.0, 1.0);
        self.push(item)
    }

    pub fn add_check_item(&mut self, label: &str, id: i32) -> usize {
        let mut item = self.add_item(label, id);
        self.items[item].checkable_type = CheckableType::CheckBox;
        item
    }

    pub fn add_radio_check_item(&mut self, label: &str, id: i32) -> usize {
        let mut item = self.add_item(label, id);
        self.items[item].checkable_type = CheckableType::RadioButton;
        item
    }

    pub fn add_icon_check_item(&mut self, icon: Option<Rid>, label: &str, id: i32) -> usize {
        let mut item = self.add_icon_item(icon, label, id);
        self.items[item].checkable_type = CheckableType::CheckBox;
        item
    }

    pub fn add_icon_radio_check_item(&mut self, icon: Option<Rid>, label: &str, id: i32) -> usize {
        let mut item = self.add_icon_item(icon, label, id);
        self.items[item].checkable_type = CheckableType::RadioButton;
        item
    }

    /// `add_separator`: an empty, disabled, non-checkable item.
    pub fn add_separator(&mut self, label: &str) -> usize {
        let item = PopupMenuItem {
            text: label.to_string(),
            xl_text: label.to_string(),
            separator: true,
            disabled: true,
            id: 0,
            ..PopupMenuItem::default()
        };
        self.push(item)
    }

    pub fn add_submenu_item(&mut self, label: &str, submenu: &str, id: i32) -> usize {
        let item = PopupMenuItem {
            text: label.to_string(),
            xl_text: label.to_string(),
            submenu: submenu.to_string(),
            id,
            ..PopupMenuItem::default()
        };
        self.push(item)
    }

    // -- counts and lookup --------------------------------------------------

    pub fn get_item_count(&self) -> usize {
        self.items.len()
    }

    pub fn get_item(&self, index: usize) -> Option<&PopupMenuItem> {
        self.items.get(index)
    }

    /// `get_item_index(id)`, first match or -1.
    pub fn get_item_index(&self, id: i32) -> i32 {
        self.items
            .iter()
            .position(|item| item.id == id)
            .map(|index| index as i32)
            .unwrap_or(-1)
    }

    /// `get_item_index_by_text`, first match or -1.
    pub fn get_item_index_by_text(&self, text: &str) -> i32 {
        self.items
            .iter()
            .position(|item| item.text == text)
            .map(|index| index as i32)
            .unwrap_or(-1)
    }

    // -- per-item accessors -------------------------------------------------

    pub fn get_item_text(&self, index: usize) -> String {
        self.items
            .get(index)
            .map(|i| i.text.clone())
            .unwrap_or_default()
    }

    pub fn set_item_text(&mut self, index: usize, text: &str) -> bool {
        match self.items.get_mut(index) {
            Some(item) => {
                item.text = text.to_string();
                item.xl_text = text.to_string();
                true
            }
            None => false,
        }
    }

    pub fn set_item_xl_text(&mut self, index: usize, text: &str) -> bool {
        match self.items.get_mut(index) {
            Some(item) => {
                item.xl_text = text.to_string();
                true
            }
            None => false,
        }
    }

    pub fn get_item_xl_text(&self, index: usize) -> String {
        self.items
            .get(index)
            .map(|i| i.xl_text.clone())
            .unwrap_or_default()
    }

    pub fn get_item_icon(&self, index: usize) -> Option<Rid> {
        self.items.get(index).and_then(|i| i.icon)
    }

    pub fn set_item_icon(&mut self, index: usize, icon: Option<Rid>) -> bool {
        match self.items.get_mut(index) {
            Some(item) => {
                item.icon = icon;
                true
            }
            None => false,
        }
    }

    pub fn get_item_icon_max_width(&self, index: usize) -> i32 {
        self.items.get(index).map(|i| i.icon_max_width).unwrap_or(0)
    }

    pub fn set_item_icon_max_width(&mut self, index: usize, width: i32) -> bool {
        match self.items.get_mut(index) {
            Some(item) => {
                item.icon_max_width = width;
                true
            }
            None => false,
        }
    }

    pub fn get_item_icon_modulate(&self, index: usize) -> Color {
        self.items
            .get(index)
            .map(|i| i.icon_modulate)
            .unwrap_or(Color::new(1.0, 1.0, 1.0, 1.0))
    }

    pub fn set_item_icon_modulate(&mut self, index: usize, modulate: Color) -> bool {
        match self.items.get_mut(index) {
            Some(item) => {
                item.icon_modulate = modulate;
                true
            }
            None => false,
        }
    }

    pub fn get_item_id(&self, index: usize) -> i32 {
        self.items.get(index).map(|i| i.id).unwrap_or(0)
    }

    pub fn set_item_id(&mut self, index: usize, id: i32) -> bool {
        match self.items.get_mut(index) {
            Some(item) => {
                item.id = id;
                true
            }
            None => false,
        }
    }

    pub fn is_item_checked(&self, index: usize) -> bool {
        self.items.get(index).map(|i| i.checked).unwrap_or(false)
    }

    pub fn set_item_checked(&mut self, index: usize, checked: bool) -> bool {
        match self.items.get_mut(index) {
            Some(item) => {
                item.checked = checked;
                true
            }
            None => false,
        }
    }

    pub fn is_item_checkable(&self, index: usize) -> bool {
        self.items
            .get(index)
            .map(|i| i.checkable_type != CheckableType::None)
            .unwrap_or(false)
    }

    pub fn is_item_radio_checkable(&self, index: usize) -> bool {
        self.items
            .get(index)
            .map(|i| i.checkable_type == CheckableType::RadioButton)
            .unwrap_or(false)
    }

    pub fn is_item_disabled(&self, index: usize) -> bool {
        self.items.get(index).map(|i| i.disabled).unwrap_or(false)
    }

    pub fn set_item_disabled(&mut self, index: usize, disabled: bool) -> bool {
        match self.items.get_mut(index) {
            Some(item) => {
                item.disabled = disabled;
                true
            }
            None => false,
        }
    }

    pub fn is_item_separator(&self, index: usize) -> bool {
        self.items.get(index).map(|i| i.separator).unwrap_or(false)
    }

    pub fn is_item_visible(&self, index: usize) -> bool {
        self.items.get(index).map(|i| i.visible).unwrap_or(false)
    }

    pub fn set_item_visible(&mut self, index: usize, visible: bool) -> bool {
        match self.items.get_mut(index) {
            Some(item) => {
                item.visible = visible;
                true
            }
            None => false,
        }
    }

    /// `set_item_multistate`: number of states (0 disables the multistate mode).
    pub fn set_item_multistate(&mut self, index: usize, states: i32) -> bool {
        match self.items.get_mut(index) {
            Some(item) => {
                item.max_states = states;
                true
            }
            None => false,
        }
    }

    pub fn get_item_state(&self, index: usize) -> i32 {
        self.items.get(index).map(|i| i.state).unwrap_or(0)
    }

    pub fn set_item_state(&mut self, index: usize, state: i32) -> bool {
        match self.items.get_mut(index) {
            Some(item) => {
                item.state = state;
                true
            }
            None => false,
        }
    }

    pub fn get_item_submenu(&self, index: usize) -> String {
        self.items
            .get(index)
            .map(|i| i.submenu.clone())
            .unwrap_or_default()
    }

    pub fn set_item_submenu(&mut self, index: usize, submenu: &str) -> bool {
        match self.items.get_mut(index) {
            Some(item) => {
                item.submenu = submenu.to_string();
                true
            }
            None => false,
        }
    }

    pub fn get_item_tooltip(&self, index: usize) -> String {
        self.items
            .get(index)
            .map(|i| i.tooltip.clone())
            .unwrap_or_default()
    }

    pub fn set_item_tooltip(&mut self, index: usize, tooltip: &str) -> bool {
        match self.items.get_mut(index) {
            Some(item) => {
                item.tooltip = tooltip.to_string();
                true
            }
            None => false,
        }
    }

    pub fn get_item_indent(&self, index: usize) -> i32 {
        self.items.get(index).map(|i| i.indent).unwrap_or(0)
    }

    pub fn set_item_indent(&mut self, index: usize, indent: i32) -> bool {
        match self.items.get_mut(index) {
            Some(item) => {
                item.indent = indent;
                true
            }
            None => false,
        }
    }

    pub fn get_item_metadata(&self, index: usize) -> Option<Variant> {
        self.items.get(index).and_then(|i| i.metadata.clone())
    }

    pub fn set_item_metadata(&mut self, index: usize, metadata: Variant) -> bool {
        match self.items.get_mut(index) {
            Some(item) => {
                item.metadata = Some(metadata);
                true
            }
            None => false,
        }
    }

    // -- removal ------------------------------------------------------------

    /// `remove_item`; shifts the current/focused indices like the engine.
    pub fn remove_item(&mut self, index: usize) -> bool {
        if index >= self.items.len() {
            return false;
        }
        self.items.remove(index);
        if self.current == index as i32 {
            self.current = -1;
        } else if self.current > index as i32 {
            self.current -= 1;
        }
        if self.focused == index as i32 {
            self.focused = -1;
        } else if self.focused > index as i32 {
            self.focused -= 1;
        }
        true
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.current = -1;
        self.focused = -1;
    }

    // -- selection ----------------------------------------------------------

    pub fn set_current_index(&mut self, index: i32) {
        self.current = index;
    }

    pub fn get_current_index(&self) -> i32 {
        self.current
    }

    pub fn set_focused_item(&mut self, index: i32) {
        self.focused = index;
    }

    pub fn get_focused_item(&self) -> i32 {
        self.focused
    }

    // -- options ------------------------------------------------------------

    pub fn set_hide_on_item_selection(&mut self, enable: bool) {
        self.hide_on_item_selection = enable;
    }

    pub fn is_hide_on_item_selection(&self) -> bool {
        self.hide_on_item_selection
    }

    pub fn set_hide_on_checkable_item_selection(&mut self, enable: bool) {
        self.hide_on_checkable_item_selection = enable;
    }

    pub fn is_hide_on_checkable_item_selection(&self) -> bool {
        self.hide_on_checkable_item_selection
    }

    pub fn set_hide_on_multistate_item_selection(&mut self, enable: bool) {
        self.hide_on_multistate_item_selection = enable;
    }

    pub fn is_hide_on_multistate_item_selection(&self) -> bool {
        self.hide_on_multistate_item_selection
    }

    pub fn set_allow_search(&mut self, allow: bool) {
        self.allow_search = allow;
    }

    pub fn get_allow_search(&self) -> bool {
        self.allow_search
    }

    pub fn set_prefer_native_menu(&mut self, prefer: bool) {
        self.prefer_native_menu = prefer;
    }

    pub fn is_prefer_native_menu(&self) -> bool {
        self.prefer_native_menu
    }

    pub fn set_system_menu(&mut self, menu: Option<Rid>) {
        self.system_menu = menu;
    }

    pub fn get_system_menu(&self) -> Option<Rid> {
        self.system_menu
    }

    pub fn set_search_bar_enabled(&mut self, enabled: bool) {
        self.search_bar_enabled = enabled;
    }

    pub fn is_search_bar_enabled(&self) -> bool {
        self.search_bar_enabled
    }

    pub fn set_search_bar_min_item_count(&mut self, count: i32) {
        self.search_bar_min_item_count = count;
    }

    pub fn get_search_bar_min_item_count(&self) -> i32 {
        self.search_bar_min_item_count
    }

    pub fn set_search_bar_fuzzy_search_enabled(&mut self, enabled: bool) {
        self.search_bar_fuzzy_search_enabled = enabled;
    }

    pub fn is_search_bar_fuzzy_search_enabled(&self) -> bool {
        self.search_bar_fuzzy_search_enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let p = PopupMenuData::new();
        assert_eq!(p.get_item_count(), 0);
        assert_eq!(p.get_current_index(), -1);
        assert_eq!(p.get_focused_item(), -1);
        assert!(p.is_hide_on_item_selection());
        assert!(p.is_hide_on_checkable_item_selection());
        assert!(!p.is_hide_on_multistate_item_selection());
        assert!(p.get_allow_search());
        assert!(!p.is_prefer_native_menu());
        assert!(p.get_system_menu().is_none());
        assert!(!p.is_search_bar_enabled());
        assert_eq!(p.get_search_bar_min_item_count(), 0);
        assert!(p.is_search_bar_fuzzy_search_enabled());
    }

    #[test]
    fn item_variants_and_fields() {
        let mut p = PopupMenuData::new();
        assert_eq!(p.add_item("Open", 10), 0);
        assert_eq!(p.add_icon_item(Some(Rid::from_u64(1)), "Save", 20), 1);
        assert_eq!(p.add_check_item("Check", 30), 2);
        assert_eq!(p.add_radio_check_item("Radio", 40), 3);
        assert_eq!(p.add_separator(""), 4);
        assert_eq!(p.add_submenu_item("Sub", "SubMenu", 50), 5);
        assert_eq!(p.get_item_count(), 6);

        assert_eq!(p.get_item_text(0), "Open");
        assert_eq!(p.get_item_icon(1), Some(Rid::from_u64(1)));
        assert!(!p.is_item_checkable(0));
        assert!(p.is_item_checkable(2));
        assert!(p.is_item_radio_checkable(3));
        assert!(!p.is_item_radio_checkable(2));
        assert!(p.is_item_separator(4));
        assert_eq!(p.get_item_submenu(5), "SubMenu");
        assert_eq!(p.get_item_id(5), 50);
        assert_eq!(p.get_item_index(30), 2);
        assert_eq!(p.get_item_index(999), -1);
        assert_eq!(p.get_item_index_by_text("Save"), 1);
        assert_eq!(p.get_item_index_by_text("Nope"), -1);

        assert!(p.set_item_text(0, "Close"));
        assert_eq!(p.get_item_text(0), "Close");
        assert!(p.set_item_checked(2, true));
        assert!(p.is_item_checked(2));
        assert!(p.set_item_disabled(1, true));
        assert!(p.is_item_disabled(1));
        assert!(p.set_item_visible(0, false));
        assert!(!p.is_item_visible(0));
        assert!(p.set_item_multistate(2, 3));
        assert!(p.set_item_state(2, 2));
        assert_eq!(p.get_item_state(2), 2);
        assert!(p.set_item_tooltip(0, "tip"));
        assert_eq!(p.get_item_tooltip(0), "tip");
        assert!(p.set_item_indent(0, 2));
        assert_eq!(p.get_item_indent(0), 2);
        assert!(p.set_item_metadata(0, Variant::Int(7)));
        assert_eq!(p.get_item_metadata(0), Some(Variant::Int(7)));

        // Out-of-range setters fail and getters return defaults.
        assert!(!p.set_item_text(9, "x"));
        assert!(!p.set_item_icon(9, None));
        assert!(!p.set_item_checked(9, true));
        assert!(!p.set_item_disabled(9, true));
        assert!(!p.set_item_visible(9, true));
        assert!(!p.set_item_multistate(9, 1));
        assert!(!p.set_item_state(9, 1));
        assert!(!p.set_item_tooltip(9, "x"));
        assert!(!p.set_item_indent(9, 1));
        assert!(!p.set_item_metadata(9, Variant::Nil));
        assert!(p.get_item(9).is_none());
        assert_eq!(p.get_item_text(9), "");
        assert_eq!(p.get_item_id(9), 0);
        assert!(p.get_item_metadata(9).is_none());
        assert!(!p.is_item_separator(9));
        assert!(!p.is_item_checked(9));
    }

    #[test]
    fn removal_and_clear() {
        let mut p = PopupMenuData::new();
        p.add_item("a", 1);
        p.add_item("b", 2);
        p.add_item("c", 3);
        p.set_current_index(1);
        p.set_focused_item(2);

        assert!(p.remove_item(0));
        assert_eq!(p.get_item_count(), 2);
        assert_eq!(p.get_current_index(), 0, "selection follows the same item");
        assert_eq!(p.get_focused_item(), 1);

        // Removing the selected item clears the selection.
        assert!(p.remove_item(0));
        assert_eq!(p.get_item_count(), 1);
        assert_eq!(p.get_current_index(), -1);
        assert_eq!(p.get_focused_item(), 0, "focused 1 shifted to 0");
        assert!(!p.remove_item(5));

        p.clear();
        assert_eq!(p.get_item_count(), 0);
        assert_eq!(p.get_current_index(), -1);
        assert_eq!(p.get_focused_item(), -1);
    }

    #[test]
    fn options_round_trip() {
        let mut p = PopupMenuData::new();
        p.set_hide_on_item_selection(false);
        p.set_hide_on_checkable_item_selection(false);
        p.set_hide_on_multistate_item_selection(true);
        p.set_allow_search(false);
        p.set_prefer_native_menu(true);
        p.set_system_menu(Some(Rid::from_u64(9)));
        p.set_search_bar_enabled(true);
        p.set_search_bar_min_item_count(8);
        p.set_search_bar_fuzzy_search_enabled(false);

        assert!(!p.is_hide_on_item_selection());
        assert!(!p.is_hide_on_checkable_item_selection());
        assert!(p.is_hide_on_multistate_item_selection());
        assert!(!p.get_allow_search());
        assert!(p.is_prefer_native_menu());
        assert_eq!(p.get_system_menu(), Some(Rid::from_u64(9)));
        assert!(p.is_search_bar_enabled());
        assert_eq!(p.get_search_bar_min_item_count(), 8);
        assert!(!p.is_search_bar_fuzzy_search_enabled());

        assert_eq!(CheckableType::None as i32, 0);
        assert_eq!(CheckableType::CheckBox as i32, 1);
        assert_eq!(CheckableType::RadioButton as i32, 2);
    }
}
