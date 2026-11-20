//! `scene/gui/scroll_bar.*`: the scroll bar's range, orientation and drag state.
//!
//! A `ScrollBar` is a `Range` plus an orientation, a custom step, the
//! increment/decrement button highlight state and the drag record. Ported:
//! every setter/getter, plus the three derived geometry helpers with the theme
//! metrics passed in as arguments (the port has no theme cache):
//!
//! - `get_grabber_size` = `page / (max - min) * area + grabber_min_size`, zero
//!   for a non-positive range;
//! - `get_area_size` subtracts the style box, both arrow icons and the grabber
//!   minimum from the control size along the orientation axis;
//! - `get_grabber_offset` = `area * as_ratio`.
//!
//! The drag-node machinery, scrolling animation, theme lookups and drawing are
//! not ported; the drag record is exposed so callers can drive it.

use crate::scene::gui_simple::Orientation;
use crate::scene::range::RangeData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum HighlightStatus {
    None,
    Range,
    Incr,
    Decr,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct DragState {
    pub active: bool,
    pub pos_at_click: f32,
    pub value_at_click: f32,
}

#[derive(Clone, Debug)]
pub struct ScrollBarData {
    pub range: RangeData,
    orientation: Orientation,
    custom_step: f32,
    highlight: HighlightStatus,
    incr_active: bool,
    decr_active: bool,
    pub drag: DragState,
    drag_node_enabled: bool,
    scrolling: bool,
    target_scroll: f64,
    smooth_scroll_enabled: bool,
}

impl Default for ScrollBarData {
    fn default() -> Self {
        Self::horizontal()
    }
}

impl ScrollBarData {
    /// `ScrollBar(Orientation)`.
    pub fn new(orientation: Orientation) -> Self {
        Self {
            range: RangeData::new(),
            orientation,
            custom_step: -1.0,
            highlight: HighlightStatus::None,
            incr_active: false,
            decr_active: false,
            drag: DragState::default(),
            drag_node_enabled: true,
            scrolling: false,
            target_scroll: 0.0,
            smooth_scroll_enabled: false,
        }
    }

    pub fn horizontal() -> Self {
        Self::new(Orientation::Horizontal)
    }

    pub fn vertical() -> Self {
        Self::new(Orientation::Vertical)
    }

    pub fn get_orientation(&self) -> Orientation {
        self.orientation
    }

    pub fn set_custom_step(&mut self, step: f32) {
        self.custom_step = step;
    }

    pub fn get_custom_step(&self) -> f32 {
        self.custom_step
    }

    pub fn set_highlight(&mut self, status: HighlightStatus) {
        self.highlight = status;
    }

    pub fn get_highlight(&self) -> HighlightStatus {
        self.highlight
    }

    pub fn set_incr_active(&mut self, active: bool) {
        self.incr_active = active;
    }

    pub fn is_incr_active(&self) -> bool {
        self.incr_active
    }

    pub fn set_decr_active(&mut self, active: bool) {
        self.decr_active = active;
    }

    pub fn is_decr_active(&self) -> bool {
        self.decr_active
    }

    pub fn set_drag_node_enabled(&mut self, enabled: bool) {
        self.drag_node_enabled = enabled;
    }

    pub fn is_drag_node_enabled(&self) -> bool {
        self.drag_node_enabled
    }

    pub fn set_scrolling(&mut self, scrolling: bool) {
        self.scrolling = scrolling;
    }

    pub fn is_scrolling(&self) -> bool {
        self.scrolling
    }

    pub fn set_target_scroll(&mut self, target: f64) {
        self.target_scroll = target;
    }

    pub fn get_target_scroll(&self) -> f64 {
        self.target_scroll
    }

    pub fn set_smooth_scroll_enabled(&mut self, enabled: bool) {
        self.smooth_scroll_enabled = enabled;
    }

    pub fn is_smooth_scroll_enabled(&self) -> bool {
        self.smooth_scroll_enabled
    }

    /// `get_grabber_size`, with the theme metrics supplied by the caller.
    pub fn get_grabber_size(&self, area_size: f64, grabber_min_size: f64) -> f64 {
        let range = self.range.get_max() - self.range.get_min();
        if range <= 0.0 {
            return 0.0;
        }
        let page = if self.range.get_page() > 0.0 {
            self.range.get_page()
        } else {
            0.0
        };
        page / range * area_size + grabber_min_size
    }

    /// `get_area_size`: control size minus the style box, both arrow icons and the
    /// grabber minimum along the orientation axis.
    pub fn get_area_size(
        &self,
        control_size: f64,
        style_min_size: f64,
        increment_icon_size: f64,
        decrement_icon_size: f64,
        grabber_min_size: f64,
    ) -> f64 {
        control_size - style_min_size - increment_icon_size - decrement_icon_size - grabber_min_size
    }

    /// `get_grabber_offset`.
    pub fn get_grabber_offset(&self, area_size: f64) -> f64 {
        area_size * self.range.get_as_ratio()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_orientation() {
        let s = ScrollBarData::horizontal();
        assert_eq!(s.get_orientation(), Orientation::Horizontal);
        assert_eq!(
            ScrollBarData::vertical().get_orientation(),
            Orientation::Vertical
        );
        assert_eq!(s.get_custom_step(), -1.0);
        assert_eq!(s.get_highlight(), HighlightStatus::None);
        assert!(!s.is_incr_active());
        assert!(!s.is_decr_active());
        assert!(!s.drag.active);
        assert!(s.is_drag_node_enabled());
        assert!(!s.is_scrolling());
        assert!(!s.is_smooth_scroll_enabled());
    }

    #[test]
    fn flags_and_drag_record() {
        let mut s = ScrollBarData::vertical();
        s.set_custom_step(0.5);
        s.set_highlight(HighlightStatus::Incr);
        s.set_incr_active(true);
        s.set_decr_active(true);
        s.drag.active = true;
        s.drag.pos_at_click = 3.0;
        s.drag.value_at_click = 1.5;
        s.set_drag_node_enabled(false);
        s.set_scrolling(true);
        s.set_target_scroll(9.0);
        s.set_smooth_scroll_enabled(true);

        assert_eq!(s.get_custom_step(), 0.5);
        assert_eq!(s.get_highlight(), HighlightStatus::Incr);
        assert!(s.is_incr_active() && s.is_decr_active());
        assert!(s.drag.active);
        assert_eq!(s.drag.pos_at_click, 3.0);
        assert_eq!(s.drag.value_at_click, 1.5);
        assert!(!s.is_drag_node_enabled());
        assert!(s.is_scrolling());
        assert_eq!(s.get_target_scroll(), 9.0);
        assert!(s.is_smooth_scroll_enabled());

        assert_eq!(HighlightStatus::None as i32, 0);
        assert_eq!(HighlightStatus::Range as i32, 1);
        assert_eq!(HighlightStatus::Incr as i32, 2);
        assert_eq!(HighlightStatus::Decr as i32, 3);
    }

    #[test]
    fn derived_geometry() {
        let mut s = ScrollBarData::horizontal();
        s.range.set_min(0.0);
        s.range.set_max(100.0);
        s.range.set_page(25.0);

        // page / range * area + grabber minimum.
        assert!((s.get_grabber_size(400.0, 10.0) - (25.0 / 100.0 * 400.0 + 10.0)).abs() < 1e-6);
        // A non-positive range has no grabber.
        s.range.set_max(0.0);
        assert_eq!(s.get_grabber_size(400.0, 10.0), 0.0);

        s.range.set_max(100.0);
        assert_eq!(s.get_area_size(400.0, 4.0, 16.0, 16.0, 10.0), 354.0);

        s.range.set_value(50.0);
        let ratio = s.range.get_as_ratio();
        assert!((s.get_grabber_offset(354.0) - 354.0 * ratio).abs() < 1e-6);
    }
}
