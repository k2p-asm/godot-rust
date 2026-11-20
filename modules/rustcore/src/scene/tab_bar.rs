//! `scene/gui/tab_bar.*`: tab list and selection state (no drawing).
//!
//! Ported: the tab records (text, tooltip, icon handle, disabled, hidden), the
//! `current`/`previous` selection with the engine's index adjustments on
//! removal and moves, the `_can_deselect` rule (deselection allowed when enabled
//! or when every tab is disabled/hidden), first-tab auto-selection when
//! deselection is off, and the alignment / clip / close-button / switch-on-release
//! settings.
//!
//! The tab bar assumes it is inside the tree (the engine defers selection until
//! then), so there is no `initialized`/`queued_current` staging here.

use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AlignmentMode {
    Left,
    Center,
    Right,
    Max,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum CloseButtonDisplayPolicy {
    ShowNever,
    ShowActiveOnly,
    ShowAlways,
    Max,
}

#[derive(Clone, Debug, Default)]
pub struct TabItem {
    pub text: String,
    pub tooltip: String,
    pub icon: Option<Rid>,
    pub disabled: bool,
    pub hidden: bool,
}

#[derive(Clone, Debug)]
pub struct TabBarData {
    tabs: Vec<TabItem>,
    current: i32,
    previous: i32,
    deselect_enabled: bool,
    switch_on_release: bool,
    tab_alignment: AlignmentMode,
    clip_tabs: bool,
    tab_close_display_policy: CloseButtonDisplayPolicy,
    max_tab_width: i32,
    scrolling_enabled: bool,
}

impl Default for TabBarData {
    fn default() -> Self {
        Self::new()
    }
}

impl TabBarData {
    pub fn new() -> Self {
        Self {
            tabs: Vec::new(),
            current: -1,
            previous: -1,
            deselect_enabled: false,
            switch_on_release: false,
            tab_alignment: AlignmentMode::Left,
            clip_tabs: true,
            tab_close_display_policy: CloseButtonDisplayPolicy::ShowNever,
            max_tab_width: 0,
            scrolling_enabled: true,
        }
    }

    pub fn get_tab_count(&self) -> usize {
        self.tabs.len()
    }

    /// Adds a tab; the first one becomes selected when deselection is off.
    pub fn add_tab(&mut self, text: &str) -> usize {
        self.tabs.push(TabItem {
            text: text.to_string(),
            ..TabItem::default()
        });
        if !self.deselect_enabled && self.tabs.len() == 1 && self.current == -1 {
            self.previous = self.current;
            self.current = 0;
        }
        self.tabs.len() - 1
    }

    pub fn clear_tabs(&mut self) {
        if self.tabs.is_empty() {
            return;
        }
        self.tabs.clear();
        self.current = -1;
        self.previous = -1;
    }

    /// Removes a tab, shifting the selection exactly like the engine.
    pub fn remove_tab(&mut self, index: usize) -> bool {
        if index >= self.tabs.len() {
            return false;
        }
        self.tabs.remove(index);

        let index = index as i32;
        if self.current >= index && self.current > 0 {
            self.current -= 1;
        }
        if self.previous >= index && self.previous > 0 {
            self.previous -= 1;
        }
        if self.tabs.is_empty() {
            self.current = -1;
            self.previous = -1;
        }
        true
    }

    /// Moves a tab and keeps the selection pointing at the same tab.
    pub fn move_tab(&mut self, from: usize, to: usize) -> bool {
        if from == to {
            return true;
        }
        if from >= self.tabs.len() || to >= self.tabs.len() {
            return false;
        }
        let tab = self.tabs.remove(from);
        self.tabs.insert(to, tab);

        let (from, to) = (from as i32, to as i32);
        if self.current == from {
            self.current = to;
        } else if self.current > from && self.current <= to {
            self.current -= 1;
        } else if self.current < from && self.current >= to {
            self.current += 1;
        }
        true
    }

    /// `_can_deselect`: allowed when enabled, or when every tab is disabled or
    /// hidden.
    pub fn can_deselect(&self) -> bool {
        if self.deselect_enabled {
            return true;
        }
        self.tabs.iter().all(|tab| tab.disabled || tab.hidden)
    }

    /// `-1` clears the selection and requires `can_deselect`; any other index
    /// must exist.
    pub fn set_current_tab(&mut self, index: i32) -> bool {
        if index == -1 {
            if !self.can_deselect() {
                return false;
            }
        } else if index < 0 || index as usize >= self.tabs.len() {
            return false;
        }
        self.previous = self.current;
        self.current = index;
        true
    }

    pub fn get_current_tab(&self) -> i32 {
        self.current
    }

    pub fn get_previous_tab(&self) -> i32 {
        self.previous
    }

    pub fn get_tab(&self, index: usize) -> Option<&TabItem> {
        self.tabs.get(index)
    }

    pub fn get_tab_title(&self, index: usize) -> String {
        self.tabs
            .get(index)
            .map(|t| t.text.clone())
            .unwrap_or_default()
    }

    pub fn set_tab_title(&mut self, index: usize, title: &str) -> bool {
        match self.tabs.get_mut(index) {
            Some(t) => {
                t.text = title.to_string();
                true
            }
            None => false,
        }
    }

    pub fn get_tab_tooltip(&self, index: usize) -> String {
        self.tabs
            .get(index)
            .map(|t| t.tooltip.clone())
            .unwrap_or_default()
    }

    pub fn set_tab_tooltip(&mut self, index: usize, tooltip: &str) -> bool {
        match self.tabs.get_mut(index) {
            Some(t) => {
                t.tooltip = tooltip.to_string();
                true
            }
            None => false,
        }
    }

    pub fn get_tab_icon(&self, index: usize) -> Option<Rid> {
        self.tabs.get(index).and_then(|t| t.icon)
    }

    pub fn set_tab_icon(&mut self, index: usize, icon: Option<Rid>) -> bool {
        match self.tabs.get_mut(index) {
            Some(t) => {
                t.icon = icon;
                true
            }
            None => false,
        }
    }

    pub fn is_tab_disabled(&self, index: usize) -> bool {
        self.tabs.get(index).map(|t| t.disabled).unwrap_or(false)
    }

    pub fn set_tab_disabled(&mut self, index: usize, disabled: bool) -> bool {
        match self.tabs.get_mut(index) {
            Some(t) => {
                t.disabled = disabled;
                true
            }
            None => false,
        }
    }

    pub fn is_tab_hidden(&self, index: usize) -> bool {
        self.tabs.get(index).map(|t| t.hidden).unwrap_or(false)
    }

    pub fn set_tab_hidden(&mut self, index: usize, hidden: bool) -> bool {
        match self.tabs.get_mut(index) {
            Some(t) => {
                t.hidden = hidden;
                true
            }
            None => false,
        }
    }

    pub fn set_deselect_enabled(&mut self, enabled: bool) {
        self.deselect_enabled = enabled;
    }

    pub fn get_deselect_enabled(&self) -> bool {
        self.deselect_enabled
    }

    pub fn set_switch_on_release(&mut self, enabled: bool) {
        self.switch_on_release = enabled;
    }

    pub fn get_switch_on_release(&self) -> bool {
        self.switch_on_release
    }

    /// `ALIGNMENT_MAX` is rejected like the engine's index check.
    pub fn set_tab_alignment(&mut self, alignment: AlignmentMode) -> bool {
        if alignment == AlignmentMode::Max {
            return false;
        }
        self.tab_alignment = alignment;
        true
    }

    pub fn get_tab_alignment(&self) -> AlignmentMode {
        self.tab_alignment
    }

    pub fn set_clip_tabs(&mut self, clip: bool) {
        self.clip_tabs = clip;
    }

    pub fn get_clip_tabs(&self) -> bool {
        self.clip_tabs
    }

    /// `CLOSE_BUTTON_MAX` is rejected.
    pub fn set_tab_close_display_policy(&mut self, policy: CloseButtonDisplayPolicy) -> bool {
        if policy == CloseButtonDisplayPolicy::Max {
            return false;
        }
        self.tab_close_display_policy = policy;
        true
    }

    pub fn get_tab_close_display_policy(&self) -> CloseButtonDisplayPolicy {
        self.tab_close_display_policy
    }

    pub fn set_max_tab_width(&mut self, width: i32) {
        self.max_tab_width = width;
    }

    pub fn get_max_tab_width(&self) -> i32 {
        self.max_tab_width
    }

    pub fn set_scrolling_enabled(&mut self, enabled: bool) {
        self.scrolling_enabled = enabled;
    }

    pub fn get_scrolling_enabled(&self) -> bool {
        self.scrolling_enabled
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_first_tab_auto_selection() {
        let mut b = TabBarData::new();
        assert_eq!(b.get_tab_count(), 0);
        assert_eq!(b.get_current_tab(), -1);
        assert_eq!(b.get_previous_tab(), -1);
        assert!(!b.get_deselect_enabled());
        assert_eq!(b.get_tab_alignment(), AlignmentMode::Left);
        assert!(b.get_clip_tabs());
        assert_eq!(
            b.get_tab_close_display_policy(),
            CloseButtonDisplayPolicy::ShowNever
        );
        assert!(b.get_scrolling_enabled());

        assert_eq!(b.add_tab("One"), 0);
        assert_eq!(b.get_current_tab(), 0, "first tab is selected");

        // With deselection enabled the first tab is not auto-selected.
        let mut d = TabBarData::new();
        d.set_deselect_enabled(true);
        d.add_tab("One");
        assert_eq!(d.get_current_tab(), -1);
    }

    #[test]
    fn selection_validation_and_deselection_rule() {
        let mut b = TabBarData::new();
        b.add_tab("One");
        b.add_tab("Two");

        assert!(!b.set_current_tab(-1), "cannot deselect by default");
        assert!(!b.set_current_tab(2));
        assert!(!b.set_current_tab(-5));
        assert!(b.set_current_tab(1));
        assert_eq!(b.get_current_tab(), 1);
        assert_eq!(b.get_previous_tab(), 0);
        assert!(b.set_current_tab(0));
        assert_eq!(b.get_previous_tab(), 1);

        // All tabs disabled or hidden allows deselection.
        assert!(b.set_tab_disabled(0, true));
        assert!(b.set_tab_hidden(1, true));
        assert!(b.can_deselect());
        assert!(b.set_current_tab(-1));
        assert_eq!(b.get_current_tab(), -1);

        // Enabling deselection is enough on its own.
        let mut c = TabBarData::new();
        c.add_tab("One");
        c.set_deselect_enabled(true);
        assert!(c.set_current_tab(-1));
        assert_eq!(c.get_current_tab(), -1);
    }

    #[test]
    fn removal_shifts_the_selection() {
        let mut b = TabBarData::new();
        for name in ["a", "b", "c"] {
            b.add_tab(name);
        }
        assert!(b.set_current_tab(2));
        assert!(b.set_current_tab(1));
        assert_eq!(b.get_previous_tab(), 2);

        // Removing a tab before the selection shifts both indices down.
        assert!(b.remove_tab(0));
        assert_eq!(b.get_tab_count(), 2);
        assert_eq!(b.get_current_tab(), 0);
        assert_eq!(b.get_previous_tab(), 1);

        // Removing at the selection keeps it valid (0 stays 0).
        assert!(b.remove_tab(0));
        assert_eq!(b.get_current_tab(), 0);
        assert_eq!(b.get_tab_count(), 1);

        // Removing the last tab clears the selection.
        assert!(b.remove_tab(0));
        assert_eq!(b.get_tab_count(), 0);
        assert_eq!(b.get_current_tab(), -1);
        assert_eq!(b.get_previous_tab(), -1);
        assert!(!b.remove_tab(0));
    }

    #[test]
    fn moving_follows_the_selected_tab() {
        let mut b = TabBarData::new();
        for name in ["a", "b", "c", "d"] {
            b.add_tab(name);
        }
        assert!(b.set_current_tab(1));

        // Move the selected tab to the end.
        assert!(b.move_tab(1, 3));
        assert_eq!(b.get_current_tab(), 3);
        assert_eq!(b.get_tab_title(3), "b");
        assert_eq!(b.get_tab_title(0), "a");

        // Moving a tab from before the selection shifts it left.
        assert!(b.set_current_tab(2)); // "d"
        assert!(b.move_tab(0, 3));
        assert_eq!(b.get_current_tab(), 1, "same tab, shifted index");
        assert_eq!(b.get_tab_title(1), "d");

        // Moving from after the selection shifts it right.
        assert!(b.set_current_tab(1));
        assert!(b.move_tab(3, 0));
        assert_eq!(b.get_current_tab(), 2);

        assert!(b.move_tab(1, 1), "same index is a no-op success");
        assert!(!b.move_tab(0, 9));
    }

    #[test]
    fn tab_fields_and_settings() {
        let mut b = TabBarData::new();
        b.add_tab("One");
        assert_eq!(b.get_tab_title(0), "One");
        assert_eq!(b.get_tab_tooltip(0), "");
        assert!(b.get_tab_icon(0).is_none());
        assert!(!b.is_tab_disabled(0));
        assert!(!b.is_tab_hidden(0));

        b.set_tab_title(0, "Renamed");
        b.set_tab_tooltip(0, "Tip");
        b.set_tab_icon(0, Some(Rid::from_u64(3)));
        b.set_tab_disabled(0, true);
        b.set_tab_hidden(0, true);
        assert_eq!(b.get_tab_title(0), "Renamed");
        assert_eq!(b.get_tab_tooltip(0), "Tip");
        assert_eq!(b.get_tab_icon(0), Some(Rid::from_u64(3)));
        assert!(b.is_tab_disabled(0));
        assert!(b.is_tab_hidden(0));

        // Out-of-range accessors are safe.
        assert_eq!(b.get_tab_title(5), "");
        assert!(!b.set_tab_title(5, "x"));
        assert!(!b.set_tab_disabled(5, true));
        assert!(!b.set_tab_hidden(5, true));
        assert!(!b.set_tab_icon(5, None));
        assert!(!b.is_tab_disabled(5));

        b.set_switch_on_release(true);
        b.set_clip_tabs(false);
        b.set_max_tab_width(120);
        b.set_scrolling_enabled(false);
        assert!(b.get_switch_on_release());
        assert!(!b.get_clip_tabs());
        assert_eq!(b.get_max_tab_width(), 120);
        assert!(!b.get_scrolling_enabled());

        assert!(b.set_tab_alignment(AlignmentMode::Right));
        assert!(!b.set_tab_alignment(AlignmentMode::Max));
        assert_eq!(b.get_tab_alignment(), AlignmentMode::Right);

        assert!(b.set_tab_close_display_policy(CloseButtonDisplayPolicy::ShowAlways));
        assert!(!b.set_tab_close_display_policy(CloseButtonDisplayPolicy::Max));
        assert_eq!(
            b.get_tab_close_display_policy(),
            CloseButtonDisplayPolicy::ShowAlways
        );

        b.clear_tabs();
        assert_eq!(b.get_tab_count(), 0);
        assert_eq!(b.get_current_tab(), -1);
        // Clearing an empty bar is a no-op.
        b.clear_tabs();

        assert_eq!(AlignmentMode::Left as i32, 0);
        assert_eq!(AlignmentMode::Center as i32, 1);
        assert_eq!(AlignmentMode::Right as i32, 2);
        assert_eq!(AlignmentMode::Max as i32, 3);
        assert_eq!(CloseButtonDisplayPolicy::ShowNever as i32, 0);
        assert_eq!(CloseButtonDisplayPolicy::ShowActiveOnly as i32, 1);
        assert_eq!(CloseButtonDisplayPolicy::ShowAlways as i32, 2);
        assert_eq!(CloseButtonDisplayPolicy::Max as i32, 3);
    }
}
