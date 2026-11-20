//! `scene/gui/option_button.*` (and the item store it delegates to).
//!
//! The engine's `OptionButton` keeps its items in a `PopupMenu`; this port models
//! the item store directly (`PopupMenuItem`) and reproduces the selection rules:
//!
//! - the first selectable item is selected when it is added to an empty button
//!   (`!has_selectable_items()`), where "selectable" means not disabled and not a
//!   separator;
//! - `select(-1)` (`NONE_SELECTED`) clears the selection, the text and the icon;
//! - selecting the current item is a no-op unless `allow_reselect` is set;
//! - other indices are validated (`ERR_FAIL_INDEX`);
//! - removing the selected item clears the selection.
//!
//! Item ids default to `-1` and are looked up with `get_item_index`. Metadata is
//! a `Variant`. Popups, drawing and the translated-text handling are not ported.

use crate::servers::rid::Rid;
use crate::variant::Variant;

/// `OptionButton::NONE_SELECTED`.
pub const NONE_SELECTED: i32 = -1;

#[derive(Clone, Debug, Default)]
pub struct PopupMenuItem {
    pub text: String,
    pub icon: Option<Rid>,
    pub id: i32,
    pub disabled: bool,
    pub checked: bool,
    pub separator: bool,
    pub radio_check: bool,
    pub checkable: bool,
    /// Submenu name for `add_submenu_item`; `None` for plain items.
    pub submenu: Option<String>,
    pub tooltip: String,
    pub metadata: Option<Variant>,
}

impl PopupMenuItem {
    fn new(text: &str, id: i32) -> Self {
        Self {
            text: text.to_string(),
            icon: None,
            id,
            disabled: false,
            checked: false,
            separator: false,
            radio_check: false,
            checkable: false,
            submenu: None,
            tooltip: String::new(),
            metadata: None,
        }
    }

    /// Selectable means neither disabled nor a separator.
    pub fn is_selectable(&self) -> bool {
        !self.disabled && !self.separator
    }
}

#[derive(Clone, Debug)]
pub struct OptionButtonData {
    items: Vec<PopupMenuItem>,
    current: i32,
    allow_reselect: bool,
    fit_to_longest_item: bool,
}

impl Default for OptionButtonData {
    fn default() -> Self {
        Self::new()
    }
}

impl OptionButtonData {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            current: NONE_SELECTED,
            allow_reselect: false,
            fit_to_longest_item: true,
        }
    }

    pub fn get_item_count(&self) -> usize {
        self.items.len()
    }

    pub fn get_item(&self, index: usize) -> Option<&PopupMenuItem> {
        self.items.get(index)
    }

    pub fn has_selectable_items(&self) -> bool {
        self.items.iter().any(|item| item.is_selectable())
    }

    fn push(&mut self, mut item: PopupMenuItem) {
        let first_selectable = !self.has_selectable_items();
        let index = self.items.len();
        self.items.push(std::mem::take(&mut item));
        // Separators never auto-select; real items do when none was selectable.
        if first_selectable && self.current == NONE_SELECTED && self.items[index].is_selectable() {
            self.select(index as i32);
        }
    }

    /// `add_item`.
    pub fn add_item(&mut self, text: &str, id: i32) {
        self.push(PopupMenuItem::new(text, id));
    }

    /// `add_icon_item`.
    pub fn add_icon_item(&mut self, icon: Option<Rid>, text: &str, id: i32) {
        let mut item = PopupMenuItem::new(text, id);
        item.icon = icon;
        self.push(item);
    }

    /// `add_check_item`.
    pub fn add_check_item(&mut self, text: &str, id: i32) {
        let mut item = PopupMenuItem::new(text, id);
        item.checkable = true;
        self.push(item);
    }

    /// `add_radio_check_item`, the kind `OptionButton` itself uses.
    pub fn add_radio_check_item(&mut self, text: &str, id: i32) {
        let mut item = PopupMenuItem::new(text, id);
        item.checkable = true;
        item.radio_check = true;
        self.push(item);
    }

    /// `add_separator`: a non-selectable item with an empty label.
    pub fn add_separator(&mut self) {
        let mut item = PopupMenuItem::new("", -1);
        item.separator = true;
        item.disabled = true;
        self.push(item);
    }

    /// `add_submenu_item`.
    pub fn add_submenu_item(&mut self, text: &str, submenu: &str, id: i32) {
        let mut item = PopupMenuItem::new(text, id);
        item.submenu = Some(submenu.to_string());
        self.push(item);
    }

    pub fn get_item_text(&self, index: usize) -> String {
        self.items
            .get(index)
            .map(|item| item.text.clone())
            .unwrap_or_default()
    }

    pub fn set_item_text(&mut self, index: usize, text: &str) -> bool {
        match self.items.get_mut(index) {
            Some(item) => {
                item.text = text.to_string();
                true
            }
            None => false,
        }
    }

    pub fn get_item_icon(&self, index: usize) -> Option<Rid> {
        self.items.get(index).and_then(|item| item.icon)
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

    pub fn get_item_id(&self, index: usize) -> i32 {
        self.items.get(index).map(|item| item.id).unwrap_or(-1)
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

    /// First item with that id, or `-1`.
    pub fn get_item_index(&self, id: i32) -> i32 {
        self.items
            .iter()
            .position(|item| item.id == id)
            .map(|index| index as i32)
            .unwrap_or(-1)
    }

    /// Index of the item with the given text, or `-1` (`get_item_index` by text).
    pub fn get_item_index_by_text(&self, text: &str) -> i32 {
        self.items
            .iter()
            .position(|item| item.text == text)
            .map(|index| index as i32)
            .unwrap_or(-1)
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

    pub fn is_item_disabled(&self, index: usize) -> bool {
        self.items.get(index).map(|item| item.disabled).unwrap_or(false)
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

    pub fn is_item_checked(&self, index: usize) -> bool {
        self.items.get(index).map(|item| item.checked).unwrap_or(false)
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

    pub fn get_item_tooltip(&self, index: usize) -> String {
        self.items
            .get(index)
            .map(|item| item.tooltip.clone())
            .unwrap_or_default()
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

    pub fn get_item_metadata(&self, index: usize) -> Option<Variant> {
        self.items.get(index).and_then(|item| item.metadata.clone())
    }

    /// `remove_item`; clears the selection when the removed item was selected.
    pub fn remove_item(&mut self, index: usize) -> bool {
        if index >= self.items.len() {
            return false;
        }
        self.items.remove(index);
        if self.current == index as i32 {
            self.select(NONE_SELECTED);
        } else if self.current > index as i32 {
            // Keep pointing at the same item after the list shifts.
            self.current -= 1;
        }
        true
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.current = NONE_SELECTED;
    }

    pub fn set_allow_reselect(&mut self, allow: bool) {
        self.allow_reselect = allow;
    }

    pub fn get_allow_reselect(&self) -> bool {
        self.allow_reselect
    }

    pub fn set_fit_to_longest_item(&mut self, fit: bool) {
        self.fit_to_longest_item = fit;
    }

    pub fn get_fit_to_longest_item(&self) -> bool {
        self.fit_to_longest_item
    }

    /// `_select`: no-op when already selected unless reselect is allowed.
    pub fn select(&mut self, index: i32) -> bool {
        if index == self.current && !self.allow_reselect {
            return false;
        }
        if index != NONE_SELECTED {
            if index < 0 || index as usize >= self.items.len() {
                return false;
            }
        }
        self.current = index;
        true
    }

    pub fn get_selected(&self) -> i32 {
        self.current
    }

    pub fn get_selected_id(&self) -> i32 {
        if self.current < 0 {
            return -1;
        }
        self.get_item_id(self.current as usize)
    }

    pub fn get_selected_metadata(&self) -> Option<Variant> {
        if self.current < 0 {
            return None;
        }
        self.get_item_metadata(self.current as usize)
    }

    /// The text shown on the button for the current selection.
    pub fn get_selected_text(&self) -> String {
        if self.current < 0 {
            String::new()
        } else {
            self.get_item_text(self.current as usize)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let b = OptionButtonData::new();
        assert_eq!(b.get_item_count(), 0);
        assert_eq!(b.get_selected(), NONE_SELECTED);
        assert_eq!(b.get_selected_id(), -1);
        assert!(b.get_selected_metadata().is_none());
        assert_eq!(b.get_selected_text(), "");
        assert!(!b.get_allow_reselect());
        assert!(b.get_fit_to_longest_item());
        assert!(!b.has_selectable_items());
    }

    #[test]
    fn first_selectable_item_is_auto_selected() {
        let mut b = OptionButtonData::new();
        // A separator is not selectable, so it does not become the selection.
        b.add_separator();
        assert_eq!(b.get_selected(), NONE_SELECTED);
        assert_eq!(b.get_item_count(), 1);

        b.add_item("First", 10);
        assert_eq!(b.get_selected(), 1, "the first real item wins");
        assert_eq!(b.get_selected_id(), 10);
        assert_eq!(b.get_selected_text(), "First");

        b.add_item("Second", 20);
        assert_eq!(b.get_selected(), 1, "later items do not steal the selection");

        // A disabled item is not selectable either. (Disabling the current item
        // does not change the selection, so deselect first.)
        let mut d = OptionButtonData::new();
        d.add_item("disabled", 1);
        d.set_item_disabled(0, true);
        assert!(!d.has_selectable_items());
        assert_eq!(d.get_selected(), 0, "disabling keeps the selection");
        d.select(NONE_SELECTED);
        d.add_item("enabled", 2);
        assert_eq!(d.get_selected(), 1);
        assert_eq!(d.get_selected_id(), 2);
    }

    #[test]
    fn selection_validation_and_reselect() {
        let mut b = OptionButtonData::new();
        b.add_item("a", 1);
        b.add_item("b", 2);
        assert_eq!(b.get_selected(), 0);

        assert!(!b.select(0), "already selected");
        assert!(b.select(1));
        assert_eq!(b.get_selected_id(), 2);
        assert!(!b.select(5));
        assert!(!b.select(-2));
        assert_eq!(b.get_selected(), 1);

        // Deselecting clears the shown text.
        assert!(b.select(NONE_SELECTED));
        assert_eq!(b.get_selected(), -1);
        assert_eq!(b.get_selected_text(), "");
        assert_eq!(b.get_selected_id(), -1);

        // Re-selecting the same item only reports when allowed.
        b.select(0);
        assert!(b.select(0) == false);
        b.set_allow_reselect(true);
        assert!(b.select(0));
        assert_eq!(b.get_selected(), 0);
    }

    #[test]
    fn removing_and_clearing_items() {
        let mut b = OptionButtonData::new();
        b.add_item("a", 1);
        b.add_item("b", 2);
        b.add_item("c", 3);
        b.select(1);

        // Removing before the selection keeps the same item selected.
        assert!(b.remove_item(0));
        assert_eq!(b.get_selected(), 0);
        assert_eq!(b.get_selected_text(), "b");

        // Removing the selected item clears the selection.
        assert!(b.remove_item(0));
        assert_eq!(b.get_selected(), NONE_SELECTED);
        assert_eq!(b.get_item_count(), 1);
        assert!(!b.remove_item(5));

        b.clear();
        assert_eq!(b.get_item_count(), 0);
        assert_eq!(b.get_selected(), NONE_SELECTED);
    }

    #[test]
    fn item_fields_and_lookup() {
        let mut b = OptionButtonData::new();
        b.add_icon_item(Some(Rid::from_u64(7)), "icons", 100);
        b.add_radio_check_item("radio", 200);
        b.add_check_item("check", 300);
        b.add_submenu_item("menu", "Sub", 400);

        assert_eq!(b.get_item_count(), 4);
        assert_eq!(b.get_item_icon(0), Some(Rid::from_u64(7)));
        assert!(b.get_item(1).unwrap().radio_check);
        assert!(b.get_item(2).unwrap().checkable);
        assert_eq!(b.get_item(3).unwrap().submenu.as_deref(), Some("Sub"));

        assert_eq!(b.get_item_index(300), 2);
        assert_eq!(b.get_item_index(999), -1);
        assert_eq!(b.get_item_index_by_text("radio"), 1);
        assert_eq!(b.get_item_index_by_text("missing"), -1);

        assert!(b.set_item_text(1, "renamed"));
        assert_eq!(b.get_item_text(1), "renamed");
        assert!(b.set_item_id(2, 333));
        assert_eq!(b.get_item_index(333), 2);
        assert!(b.set_item_icon(1, Some(Rid::from_u64(8))));
        assert_eq!(b.get_item_icon(1), Some(Rid::from_u64(8)));

        assert!(b.set_item_disabled(3, true));
        assert!(b.is_item_disabled(3));
        assert!(b.set_item_checked(2, true));
        assert!(b.is_item_checked(2));
        assert!(b.set_item_tooltip(0, "tip"));
        assert_eq!(b.get_item_tooltip(0), "tip");
        assert!(b.set_item_metadata(0, Variant::Int(42)));
        assert_eq!(b.get_item_metadata(0), Some(Variant::Int(42)));
        assert!(b.get_item_metadata(9).is_none());

        // Out-of-range setters are refused.
        assert!(!b.set_item_text(9, "x"));
        assert!(!b.set_item_id(9, 1));
        assert!(!b.set_item_icon(9, None));
        assert!(!b.set_item_disabled(9, true));
        assert!(!b.set_item_checked(9, true));
        assert!(!b.set_item_tooltip(9, "x"));
        assert!(!b.set_item_metadata(9, Variant::Nil));
        assert!(b.get_item(9).is_none());
    }

    #[test]
    fn selected_metadata_and_fit_flag() {
        let mut b = OptionButtonData::new();
        b.add_item("a", 1);
        b.add_item("b", 2);
        b.set_item_metadata(1, Variant::String(crate::string::gstring::GoString::from_str(
            "meta",
        )));
        b.select(1);
        assert_eq!(
            b.get_selected_metadata(),
            Some(Variant::String(crate::string::gstring::GoString::from_str(
                "meta"
            )))
        );

        b.set_fit_to_longest_item(false);
        assert!(!b.get_fit_to_longest_item());
    }
}
