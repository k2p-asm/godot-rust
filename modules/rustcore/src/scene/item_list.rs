//! `scene/gui/item_list.*`: the item store and selection (no layout/drawing).
//!
//! Items carry text, an icon with region and modulate, the selectable/selected/
//! disabled flags, tooltip enablement and text, metadata, custom foreground and
//! background colours and a column. The constructor defaults: current/hovered/
//! shift-anchor -1, same-column-width off, search on, auto width/height off,
//! wraparound on, single selection, left icons, zero fixed column width, one
//! text line, one maximum column, zero fixed icon size, no right-mouse select,
//! no reselect and icon scale 1.
//!
//! `select`/`deselect` are ported exactly: single selection (or a `single` call)
//! refuses unselectable/disabled items and clears every other flag; multi/toggle
//! selection only marks the item when it is selectable and enabled; `deselect`
//! clears the current index in single mode. `deselect_all` clears everything and
//! `get_selected_items` lists the flagged indices.
//!
//! Drawing, keyboard/mouse input, columns layout, scroll bars and accessibility
//! are not ported.

use crate::math::color::Color;
use crate::math::rect2::Rect2;
use crate::math::vector2i::Vector2i;
use crate::servers::rid::Rid;
use crate::variant::Variant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum SelectMode {
    Single,
    Multi,
    Toggle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum IconMode {
    Left,
    Top,
}

#[derive(Clone, Debug)]
pub struct ItemListItem {
    pub text: String,
    pub icon: Option<Rid>,
    pub icon_region: Rect2,
    pub icon_modulate: Color,
    pub selectable: bool,
    pub selected: bool,
    pub disabled: bool,
    pub tooltip_enabled: bool,
    pub tooltip: String,
    pub metadata: Option<Variant>,
    pub custom_fg: Color,
    pub custom_bg: Color,
    pub column: i32,
}

impl Default for ItemListItem {
    fn default() -> Self {
        Self {
            text: String::new(),
            icon: None,
            icon_region: Rect2::default(),
            icon_modulate: Color::new(1.0, 1.0, 1.0, 1.0),
            selectable: true,
            selected: false,
            disabled: false,
            tooltip_enabled: true,
            tooltip: String::new(),
            metadata: None,
            // `Color()` is opaque black in the engine.
            custom_fg: Color::new(0.0, 0.0, 0.0, 1.0),
            custom_bg: Color::new(0.0, 0.0, 0.0, 0.0),
            column: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ItemListData {
    items: Vec<ItemListItem>,
    current: i32,
    hovered: i32,
    shift_anchor: i32,
    select_mode: SelectMode,
    icon_mode: IconMode,
    same_column_width: bool,
    allow_search: bool,
    allow_rmb_select: bool,
    allow_reselect: bool,
    auto_width: bool,
    auto_width_value: f32,
    auto_height: bool,
    auto_height_value: f32,
    wraparound_items: bool,
    fixed_column_width: i32,
    max_text_lines: i32,
    max_columns: i32,
    fixed_icon_size: Vector2i,
    icon_scale: f32,
}

impl Default for ItemListData {
    fn default() -> Self {
        Self::new()
    }
}

impl ItemListData {
    pub fn new() -> Self {
        Self {
            items: Vec::new(),
            current: -1,
            hovered: -1,
            shift_anchor: -1,
            select_mode: SelectMode::Single,
            icon_mode: IconMode::Left,
            same_column_width: false,
            allow_search: true,
            allow_rmb_select: false,
            allow_reselect: false,
            auto_width: false,
            auto_width_value: 0.0,
            auto_height: false,
            auto_height_value: 0.0,
            wraparound_items: true,
            fixed_column_width: 0,
            max_text_lines: 1,
            max_columns: 1,
            fixed_icon_size: Vector2i::ZERO,
            icon_scale: 1.0,
        }
    }

    pub fn get_item_count(&self) -> usize {
        self.items.len()
    }

    pub fn get_item(&self, index: usize) -> Option<&ItemListItem> {
        self.items.get(index)
    }

    /// `add_item(label, icon, selectable)`.
    pub fn add_item(&mut self, text: &str, icon: Option<Rid>, selectable: bool) -> usize {
        self.items.push(ItemListItem {
            text: text.to_string(),
            icon,
            selectable,
            ..ItemListItem::default()
        });
        self.items.len() - 1
    }

    pub fn add_icon_item(&mut self, icon: Option<Rid>, text: &str, selectable: bool) -> usize {
        self.add_item(text, icon, selectable)
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

    pub fn get_item_icon_region(&self, index: usize) -> Rect2 {
        self.items
            .get(index)
            .map(|item| item.icon_region)
            .unwrap_or_default()
    }

    pub fn set_item_icon_region(&mut self, index: usize, region: Rect2) -> bool {
        match self.items.get_mut(index) {
            Some(item) => {
                item.icon_region = region;
                true
            }
            None => false,
        }
    }

    pub fn get_item_icon_modulate(&self, index: usize) -> Color {
        self.items
            .get(index)
            .map(|item| item.icon_modulate)
            .unwrap_or(Color::new(1.0, 1.0, 1.0, 1.0))
    }

    pub fn set_item_icon_modulate(&mut self, index: usize, color: Color) -> bool {
        match self.items.get_mut(index) {
            Some(item) => {
                item.icon_modulate = color;
                true
            }
            None => false,
        }
    }

    pub fn is_item_selectable(&self, index: usize) -> bool {
        self.items
            .get(index)
            .map(|item| item.selectable)
            .unwrap_or(false)
    }

    pub fn set_item_selectable(&mut self, index: usize, selectable: bool) -> bool {
        match self.items.get_mut(index) {
            Some(item) => {
                item.selectable = selectable;
                true
            }
            None => false,
        }
    }

    pub fn is_item_disabled(&self, index: usize) -> bool {
        self.items
            .get(index)
            .map(|item| item.disabled)
            .unwrap_or(false)
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

    pub fn is_item_tooltip_enabled(&self, index: usize) -> bool {
        self.items
            .get(index)
            .map(|item| item.tooltip_enabled)
            .unwrap_or(false)
    }

    pub fn set_item_tooltip_enabled(&mut self, index: usize, enabled: bool) -> bool {
        match self.items.get_mut(index) {
            Some(item) => {
                item.tooltip_enabled = enabled;
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

    pub fn set_item_tooltip(&mut self, index: usize, tooltip: &str) -> bool {
        match self.items.get_mut(index) {
            Some(item) => {
                item.tooltip = tooltip.to_string();
                true
            }
            None => false,
        }
    }

    pub fn get_item_metadata(&self, index: usize) -> Option<Variant> {
        self.items
            .get(index)
            .and_then(|item| item.metadata.clone())
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

    pub fn get_item_custom_fg_color(&self, index: usize) -> Color {
        self.items
            .get(index)
            .map(|item| item.custom_fg)
            .unwrap_or(Color::new(0.0, 0.0, 0.0, 1.0))
    }

    pub fn set_item_custom_fg_color(&mut self, index: usize, color: Color) -> bool {
        match self.items.get_mut(index) {
            Some(item) => {
                item.custom_fg = color;
                true
            }
            None => false,
        }
    }

    pub fn get_item_custom_bg_color(&self, index: usize) -> Color {
        self.items
            .get(index)
            .map(|item| item.custom_bg)
            .unwrap_or(Color::new(0.0, 0.0, 0.0, 0.0))
    }

    pub fn set_item_custom_bg_color(&mut self, index: usize, color: Color) -> bool {
        match self.items.get_mut(index) {
            Some(item) => {
                item.custom_bg = color;
                true
            }
            None => false,
        }
    }

    pub fn get_item_column(&self, index: usize) -> i32 {
        self.items.get(index).map(|item| item.column).unwrap_or(0)
    }

    pub fn set_item_column(&mut self, index: usize, column: i32) -> bool {
        match self.items.get_mut(index) {
            Some(item) => {
                item.column = column;
                true
            }
            None => false,
        }
    }

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
        if self.hovered == index as i32 {
            self.hovered = -1;
        } else if self.hovered > index as i32 {
            self.hovered -= 1;
        }
        if self.shift_anchor == index as i32 {
            self.shift_anchor = -1;
        } else if self.shift_anchor > index as i32 {
            self.shift_anchor -= 1;
        }
        true
    }

    pub fn clear(&mut self) {
        self.items.clear();
        self.current = -1;
        self.hovered = -1;
        self.shift_anchor = -1;
    }

    // -- selection ----------------------------------------------------------

    /// `select`: single selection replaces the flags; multi/toggle only sets.
    pub fn select(&mut self, index: usize, single: bool) -> bool {
        if index >= self.items.len() {
            return false;
        }
        if single || self.select_mode == SelectMode::Single {
            if !self.items[index].selectable || self.items[index].disabled {
                return false;
            }
            for (i, item) in self.items.iter_mut().enumerate() {
                item.selected = i == index;
            }
            self.current = index as i32;
            return true;
        }
        if self.items[index].selectable && !self.items[index].disabled {
            self.items[index].selected = true;
        }
        true
    }

    /// `deselect`: clears the current index in single mode.
    pub fn deselect(&mut self, index: usize) -> bool {
        if index >= self.items.len() {
            return false;
        }
        self.items[index].selected = false;
        if self.select_mode == SelectMode::Single {
            self.current = -1;
        }
        true
    }

    pub fn deselect_all(&mut self) {
        for item in self.items.iter_mut() {
            item.selected = false;
        }
        self.current = -1;
    }

    pub fn is_selected(&self, index: usize) -> bool {
        self.items
            .get(index)
            .map(|item| item.selected)
            .unwrap_or(false)
    }

    pub fn get_selected_items(&self) -> Vec<i32> {
        self.items
            .iter()
            .enumerate()
            .filter(|(_, item)| item.selected)
            .map(|(index, _)| index as i32)
            .collect()
    }

    pub fn get_current(&self) -> i32 {
        self.current
    }

    pub fn set_current(&mut self, index: i32) {
        self.current = index;
    }

    pub fn get_hovered(&self) -> i32 {
        self.hovered
    }

    pub fn set_hovered(&mut self, index: i32) {
        self.hovered = index;
    }

    /// `ERR_FAIL_INDEX(p_mode, 3)`.
    pub fn set_select_mode(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => SelectMode::Single,
            1 => SelectMode::Multi,
            2 => SelectMode::Toggle,
            _ => return false,
        };
        self.select_mode = parsed;
        true
    }

    pub fn get_select_mode(&self) -> SelectMode {
        self.select_mode
    }

    /// `ERR_FAIL_INDEX(p_mode, 2)`.
    pub fn set_icon_mode(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => IconMode::Left,
            1 => IconMode::Top,
            _ => return false,
        };
        self.icon_mode = parsed;
        true
    }

    pub fn get_icon_mode(&self) -> IconMode {
        self.icon_mode
    }

    pub fn set_same_column_width(&mut self, enabled: bool) {
        self.same_column_width = enabled;
    }

    pub fn is_same_column_width(&self) -> bool {
        self.same_column_width
    }

    pub fn set_allow_search(&mut self, allow: bool) {
        self.allow_search = allow;
    }

    pub fn get_allow_search(&self) -> bool {
        self.allow_search
    }

    pub fn set_allow_rmb_select(&mut self, allow: bool) {
        self.allow_rmb_select = allow;
    }

    pub fn get_allow_rmb_select(&self) -> bool {
        self.allow_rmb_select
    }

    pub fn set_allow_reselect(&mut self, allow: bool) {
        self.allow_reselect = allow;
    }

    pub fn get_allow_reselect(&self) -> bool {
        self.allow_reselect
    }

    pub fn set_auto_width(&mut self, enabled: bool) {
        self.auto_width = enabled;
    }

    pub fn has_auto_width(&self) -> bool {
        self.auto_width
    }

    pub fn set_auto_width_value(&mut self, value: f32) {
        self.auto_width_value = value;
    }

    pub fn get_auto_width_value(&self) -> f32 {
        self.auto_width_value
    }

    pub fn set_auto_height(&mut self, enabled: bool) {
        self.auto_height = enabled;
    }

    pub fn has_auto_height(&self) -> bool {
        self.auto_height
    }

    pub fn set_auto_height_value(&mut self, value: f32) {
        self.auto_height_value = value;
    }

    pub fn get_auto_height_value(&self) -> f32 {
        self.auto_height_value
    }

    pub fn set_wraparound_items(&mut self, enabled: bool) {
        self.wraparound_items = enabled;
    }

    pub fn has_wraparound_items(&self) -> bool {
        self.wraparound_items
    }

    pub fn set_fixed_column_width(&mut self, width: i32) {
        self.fixed_column_width = width;
    }

    pub fn get_fixed_column_width(&self) -> i32 {
        self.fixed_column_width
    }

    pub fn set_max_text_lines(&mut self, lines: i32) {
        self.max_text_lines = lines;
    }

    pub fn get_max_text_lines(&self) -> i32 {
        self.max_text_lines
    }

    pub fn set_max_columns(&mut self, columns: i32) {
        self.max_columns = columns;
    }

    pub fn get_max_columns(&self) -> i32 {
        self.max_columns
    }

    pub fn set_fixed_icon_size(&mut self, size: Vector2i) {
        self.fixed_icon_size = size;
    }

    pub fn get_fixed_icon_size(&self) -> Vector2i {
        self.fixed_icon_size
    }

    pub fn set_icon_scale(&mut self, scale: f32) {
        self.icon_scale = scale;
    }

    pub fn get_icon_scale(&self) -> f32 {
        self.icon_scale
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let l = ItemListData::new();
        assert_eq!(l.get_item_count(), 0);
        assert_eq!(l.get_current(), -1);
        assert_eq!(l.get_hovered(), -1);
        assert_eq!(l.get_select_mode(), SelectMode::Single);
        assert_eq!(l.get_icon_mode(), IconMode::Left);
        assert!(!l.is_same_column_width());
        assert!(l.get_allow_search());
        assert!(!l.get_allow_rmb_select());
        assert!(!l.get_allow_reselect());
        assert!(!l.has_auto_width());
        assert!(!l.has_auto_height());
        assert!(l.has_wraparound_items());
        assert_eq!(l.get_fixed_column_width(), 0);
        assert_eq!(l.get_max_text_lines(), 1);
        assert_eq!(l.get_max_columns(), 1);
        assert_eq!(l.get_fixed_icon_size(), Vector2i::ZERO);
        assert_eq!(l.get_icon_scale(), 1.0);

        assert_eq!(SelectMode::Single as i32, 0);
        assert_eq!(SelectMode::Multi as i32, 1);
        assert_eq!(SelectMode::Toggle as i32, 2);
        assert_eq!(IconMode::Left as i32, 0);
        assert_eq!(IconMode::Top as i32, 1);
    }

    #[test]
    fn item_fields() {
        let mut l = ItemListData::new();
        assert_eq!(l.add_item("One", Some(Rid::from_u64(1)), true), 0);
        assert_eq!(l.add_icon_item(Some(Rid::from_u64(2)), "Two", false), 1);
        assert_eq!(l.get_item_count(), 2);

        assert_eq!(l.get_item_text(0), "One");
        assert_eq!(l.get_item_icon(0), Some(Rid::from_u64(1)));
        assert!(l.is_item_selectable(0));
        assert!(!l.is_item_selectable(1));
        assert!(!l.is_item_disabled(0));
        assert!(l.is_item_tooltip_enabled(0));
        assert_eq!(l.get_item_tooltip(0), "");
        assert_eq!(l.get_item_icon_modulate(0), Color::new(1.0, 1.0, 1.0, 1.0));
        assert_eq!(l.get_item_custom_fg_color(0), Color::new(0.0, 0.0, 0.0, 1.0));
        assert_eq!(l.get_item_custom_bg_color(0), Color::new(0.0, 0.0, 0.0, 0.0));
        assert_eq!(l.get_item_column(0), 0);

        assert!(l.set_item_text(0, "One!"));
        assert!(l.set_item_icon(0, Some(Rid::from_u64(3))));
        assert!(l.set_item_icon_region(0, Rect2::new(crate::math::vector2::Vector2::ZERO, crate::math::vector2::Vector2::ONE)));
        assert!(l.set_item_icon_modulate(0, Color::new(0.5, 0.5, 0.5, 1.0)));
        assert!(l.set_item_selectable(1, true));
        assert!(l.set_item_disabled(0, true));
        assert!(l.set_item_tooltip_enabled(0, false));
        assert!(l.set_item_tooltip(0, "tip"));
        assert!(l.set_item_metadata(0, Variant::Int(5)));
        assert!(l.set_item_custom_fg_color(0, Color::new(1.0, 0.0, 0.0, 1.0)));
        assert!(l.set_item_custom_bg_color(0, Color::new(0.0, 1.0, 0.0, 1.0)));
        assert!(l.set_item_column(0, 2));

        assert_eq!(l.get_item_text(0), "One!");
        assert_eq!(l.get_item_icon(0), Some(Rid::from_u64(3)));
        assert!(l.is_item_selectable(1));
        assert!(l.is_item_disabled(0));
        assert!(!l.is_item_tooltip_enabled(0));
        assert_eq!(l.get_item_tooltip(0), "tip");
        assert_eq!(l.get_item_metadata(0), Some(Variant::Int(5)));
        assert_eq!(l.get_item_custom_fg_color(0), Color::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(l.get_item_custom_bg_color(0), Color::new(0.0, 1.0, 0.0, 1.0));
        assert_eq!(l.get_item_column(0), 2);

        // Out-of-range accessors fail and read defaults.
        assert!(!l.set_item_text(9, "x"));
        assert!(!l.set_item_icon(9, None));
        assert!(!l.set_item_selectable(9, true));
        assert!(!l.set_item_disabled(9, true));
        assert!(!l.set_item_tooltip(9, "x"));
        assert!(!l.set_item_metadata(9, Variant::Nil));
        assert!(!l.set_item_column(9, 1));
        assert_eq!(l.get_item_text(9), "");
        assert_eq!(l.get_item_column(9), 0);
        assert!(!l.is_item_selectable(9));
    }

    #[test]
    fn single_selection_rules() {
        let mut l = ItemListData::new();
        l.add_item("a", None, true);
        l.add_item("b", None, true);
        l.add_item("c", None, true);
        assert!(l.select(1, false));
        assert_eq!(l.get_current(), 1);
        assert_eq!(l.get_selected_items(), vec![1]);

        // Single mode replaces the whole selection.
        assert!(l.select(2, false));
        assert_eq!(l.get_selected_items(), vec![2]);
        assert_eq!(l.get_current(), 2);

        // Unselectable or disabled items are refused in single mode.
        l.set_item_disabled(0, true);
        assert!(!l.select(0, false));
        assert_eq!(l.get_selected_items(), vec![2]);
        l.set_item_selectable(0, false);
        assert!(!l.select(0, false));

        // Deselect in single mode clears the current index.
        assert!(l.deselect(2));
        assert!(!l.is_selected(2));
        assert_eq!(l.get_current(), -1);

        l.select(1, false);
        l.deselect_all();
        assert!(l.get_selected_items().is_empty());
        assert_eq!(l.get_current(), -1);
    }

    #[test]
    fn multi_selection_rules() {
        let mut l = ItemListData::new();
        l.add_item("a", None, true);
        l.add_item("b", None, true);
        l.add_item("c", None, true);
        assert!(l.set_select_mode(1));
        assert_eq!(l.get_select_mode(), SelectMode::Multi);

        assert!(l.select(0, false));
        assert!(l.select(2, false));
        assert_eq!(l.get_selected_items(), vec![0, 2]);

        // An explicit single call still replaces the selection.
        assert!(l.select(1, true));
        assert_eq!(l.get_selected_items(), vec![1]);
        assert_eq!(l.get_current(), 1);

        // In multi mode deselecting does not clear the current index.
        assert!(l.deselect(1));
        assert_eq!(l.get_current(), 1);
        assert!(l.get_selected_items().is_empty());
    }

    #[test]
    fn removal_and_options() {
        let mut l = ItemListData::new();
        l.add_item("a", None, true);
        l.add_item("b", None, true);
        l.add_item("c", None, true);
        l.select(1, false);
        l.set_hovered(2);

        assert!(l.remove_item(0));
        assert_eq!(l.get_item_count(), 2);
        assert_eq!(l.get_current(), 0, "selection follows the item");
        assert_eq!(l.get_hovered(), 1);
        assert!(!l.remove_item(5));

        l.clear();
        assert_eq!(l.get_item_count(), 0);
        assert_eq!(l.get_current(), -1);
        assert_eq!(l.get_hovered(), -1);

        assert!(!l.set_select_mode(3));
        assert!(!l.set_icon_mode(2));
        assert!(l.set_icon_mode(1));
        assert_eq!(l.get_icon_mode(), IconMode::Top);

        l.set_same_column_width(true);
        l.set_allow_search(false);
        l.set_allow_rmb_select(true);
        l.set_allow_reselect(true);
        l.set_auto_width(true);
        l.set_auto_width_value(120.0);
        l.set_auto_height(true);
        l.set_auto_height_value(200.0);
        l.set_wraparound_items(false);
        l.set_fixed_column_width(80);
        l.set_max_text_lines(3);
        l.set_max_columns(4);
        l.set_fixed_icon_size(Vector2i::new(16, 16));
        l.set_icon_scale(2.0);

        assert!(l.is_same_column_width());
        assert!(!l.get_allow_search());
        assert!(l.get_allow_rmb_select());
        assert!(l.get_allow_reselect());
        assert!(l.has_auto_width());
        assert_eq!(l.get_auto_width_value(), 120.0);
        assert!(l.has_auto_height());
        assert_eq!(l.get_auto_height_value(), 200.0);
        assert!(!l.has_wraparound_items());
        assert_eq!(l.get_fixed_column_width(), 80);
        assert_eq!(l.get_max_text_lines(), 3);
        assert_eq!(l.get_max_columns(), 4);
        assert_eq!(l.get_fixed_icon_size(), Vector2i::new(16, 16));
        assert_eq!(l.get_icon_scale(), 2.0);
    }
}
