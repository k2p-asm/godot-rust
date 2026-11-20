//! `scene/gui/tab_container.*`: the container's tab bar placement and options.
//!
//! `TabContainer` owns a `TabBar` (ported as [`TabBarData`]) and a few options of
//! its own. Defaults: tabs visible, tabs on top, hidden tabs excluded from the
//! minimum size, drag-to-rearrange off and the setup selection sentinel `-2`.
//!
//! Ported behaviours: `set_current_tab` is staged into `setup_current_tab` while
//! the container is outside the tree (the engine defers to the internal tab bar
//! until then) and forwarded afterwards; `set_tab_position` rejects
//! `POSITION_MAX` like the engine's index check; the settings are plain
//! assignments.
//!
//! Drawing, the popup button, visibility repainting, rearrange groups' signal
//! plumbing and theme lookups are not ported (`tabs_rearrange_group` is stored
//! and forwarded to nothing).

use crate::scene::tab_bar::TabBarData;
use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum TabPosition {
    Top,
    Bottom,
    Max,
}

#[derive(Clone, Debug)]
pub struct TabContainerData {
    pub tab_bar: TabBarData,
    tabs_visible: bool,
    tabs_position: TabPosition,
    use_hidden_tabs_for_min_size: bool,
    drag_to_rearrange_enabled: bool,
    tabs_rearrange_group: i32,
    popup_button: Option<Rid>,
    in_tree: bool,
    /// `-2` means "no staged selection" (the engine's sentinel).
    setup_current_tab: i32,
}

impl Default for TabContainerData {
    fn default() -> Self {
        Self::new()
    }
}

impl TabContainerData {
    pub fn new() -> Self {
        Self {
            tab_bar: TabBarData::new(),
            tabs_visible: true,
            tabs_position: TabPosition::Top,
            use_hidden_tabs_for_min_size: false,
            drag_to_rearrange_enabled: false,
            tabs_rearrange_group: 0,
            popup_button: None,
            in_tree: false,
            setup_current_tab: -2,
        }
    }

    /// Whether the container is inside the scene tree (drives the staging rule).
    pub fn set_in_tree(&mut self, inside: bool) {
        self.in_tree = inside;
        if inside && self.setup_current_tab != -2 {
            let staged = self.setup_current_tab;
            self.setup_current_tab = -2;
            self.set_current_tab(staged);
        }
    }

    pub fn is_in_tree(&self) -> bool {
        self.in_tree
    }

    pub fn get_setup_current_tab(&self) -> i32 {
        self.setup_current_tab
    }

    /// Staged before entering the tree, forwarded to the tab bar afterwards.
    pub fn set_current_tab(&mut self, current: i32) {
        if !self.in_tree {
            self.setup_current_tab = current;
            return;
        }
        self.tab_bar.set_current_tab(current);
    }

    pub fn get_current_tab(&self) -> i32 {
        self.tab_bar.get_current_tab()
    }

    pub fn get_tab_count(&self) -> usize {
        self.tab_bar.get_tab_count()
    }

    pub fn set_tabs_visible(&mut self, visible: bool) {
        self.tabs_visible = visible;
    }

    pub fn are_tabs_visible(&self) -> bool {
        self.tabs_visible
    }

    /// `ERR_FAIL_INDEX(POSITION_MAX)`.
    pub fn set_tab_position(&mut self, position: TabPosition) -> bool {
        if position == TabPosition::Max {
            return false;
        }
        self.tabs_position = position;
        true
    }

    pub fn get_tab_position(&self) -> TabPosition {
        self.tabs_position
    }

    pub fn set_use_hidden_tabs_for_min_size(&mut self, enabled: bool) {
        self.use_hidden_tabs_for_min_size = enabled;
    }

    pub fn get_use_hidden_tabs_for_min_size(&self) -> bool {
        self.use_hidden_tabs_for_min_size
    }

    pub fn set_drag_to_rearrange_enabled(&mut self, enabled: bool) {
        self.drag_to_rearrange_enabled = enabled;
    }

    pub fn get_drag_to_rearrange_enabled(&self) -> bool {
        self.drag_to_rearrange_enabled
    }

    pub fn set_tabs_rearrange_group(&mut self, group: i32) {
        self.tabs_rearrange_group = group;
    }

    pub fn get_tabs_rearrange_group(&self) -> i32 {
        self.tabs_rearrange_group
    }

    /// The popup button handle (the engine creates it lazily).
    pub fn set_popup_button(&mut self, button: Option<Rid>) {
        self.popup_button = button;
    }

    pub fn get_popup_button(&self) -> Option<Rid> {
        self.popup_button
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let c = TabContainerData::new();
        assert!(c.are_tabs_visible());
        assert_eq!(c.get_tab_position(), TabPosition::Top);
        assert!(!c.get_use_hidden_tabs_for_min_size());
        assert!(!c.get_drag_to_rearrange_enabled());
        assert_eq!(c.get_tabs_rearrange_group(), 0);
        assert!(c.get_popup_button().is_none());
        assert!(!c.is_in_tree());
        assert_eq!(c.get_setup_current_tab(), -2);
        assert_eq!(c.get_tab_count(), 0);
        assert_eq!(c.get_current_tab(), -1);

        assert_eq!(TabPosition::Top as i32, 0);
        assert_eq!(TabPosition::Bottom as i32, 1);
        assert_eq!(TabPosition::Max as i32, 2);
    }

    #[test]
    fn selection_is_staged_outside_the_tree() {
        let mut c = TabContainerData::new();
        c.tab_bar.add_tab("One");
        c.tab_bar.add_tab("Two");

        // Outside the tree: staged, the tab bar keeps its own selection.
        c.set_current_tab(1);
        assert_eq!(c.get_setup_current_tab(), 1);
        assert_eq!(c.get_current_tab(), 0);

        // Entering the tree forwards the staged value and clears it.
        c.set_in_tree(true);
        assert_eq!(c.get_setup_current_tab(), -2);
        assert_eq!(c.get_current_tab(), 1);

        // Inside the tree the call goes straight through.
        c.set_current_tab(0);
        assert_eq!(c.get_current_tab(), 0);
    }

    #[test]
    fn options_and_position_validation() {
        let mut c = TabContainerData::new();
        c.set_tabs_visible(false);
        c.set_use_hidden_tabs_for_min_size(true);
        c.set_drag_to_rearrange_enabled(true);
        c.set_tabs_rearrange_group(7);
        c.set_popup_button(Some(Rid::from_u64(3)));

        assert!(!c.are_tabs_visible());
        assert!(c.get_use_hidden_tabs_for_min_size());
        assert!(c.get_drag_to_rearrange_enabled());
        assert_eq!(c.get_tabs_rearrange_group(), 7);
        assert_eq!(c.get_popup_button(), Some(Rid::from_u64(3)));

        assert!(c.set_tab_position(TabPosition::Bottom));
        assert_eq!(c.get_tab_position(), TabPosition::Bottom);
        assert!(!c.set_tab_position(TabPosition::Max));
        assert_eq!(c.get_tab_position(), TabPosition::Bottom);
    }
}
