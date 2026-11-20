//! `scene/gui/{scroll_bar,slider,progress_bar}.*`: the Range subclasses.
//!
//! The value logic lives in `RangeData`; these add each control's own state and
//! constructor defaults:
//!
//! - `ScrollBar` sets `step = 0` in its constructor, `custom_step = -1` and
//!   smooth scrolling off.
//! - `Slider` defaults to no ticks, bottom-right ticks, editable and scrollable.
//! - `ProgressBar` sets `step = 0.01` in its constructor, shows the percentage,
//!   starts determinate, and its fill mode rejects `FILL_MODE_MAX` and below
//!   zero (`ERR_FAIL_INDEX`).
//!
//! Drawing, input and the drag-node helpers are not ported.

use crate::scene::gui_misc::Orientation;
use crate::scene::range::RangeData;

#[derive(Clone, Copy, Debug)]
pub struct ScrollBarData {
    pub range: RangeData,
    custom_step: f32,
    smooth_scroll_enabled: bool,
}

impl Default for ScrollBarData {
    fn default() -> Self {
        Self::new(Orientation::Horizontal)
    }
}

impl ScrollBarData {
    pub fn new(_orientation: Orientation) -> Self {
        let mut range = RangeData::new();
        // `ScrollBar::ScrollBar` calls `set_step(0)`.
        range.set_step(0.0);
        Self {
            range,
            custom_step: -1.0,
            smooth_scroll_enabled: false,
        }
    }

    pub fn set_custom_step(&mut self, step: f32) {
        self.custom_step = step;
    }

    pub fn get_custom_step(&self) -> f32 {
        self.custom_step
    }

    pub fn set_smooth_scroll_enabled(&mut self, enable: bool) {
        self.smooth_scroll_enabled = enable;
    }

    pub fn is_smooth_scroll_enabled(&self) -> bool {
        self.smooth_scroll_enabled
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum TickPosition {
    BottomRight,
    TopLeft,
    Both,
    Center,
}

#[derive(Clone, Copy, Debug)]
pub struct SliderData {
    pub range: RangeData,
    ticks: i32,
    ticks_position: TickPosition,
    ticks_on_borders: bool,
    editable: bool,
    scrollable: bool,
}

impl Default for SliderData {
    fn default() -> Self {
        Self::new(Orientation::Vertical)
    }
}

impl SliderData {
    pub fn new(_orientation: Orientation) -> Self {
        Self {
            range: RangeData::new(),
            ticks: 0,
            ticks_position: TickPosition::BottomRight,
            ticks_on_borders: false,
            editable: true,
            scrollable: true,
        }
    }

    pub fn set_ticks(&mut self, count: i32) {
        self.ticks = count;
    }

    pub fn get_ticks(&self) -> i32 {
        self.ticks
    }

    pub fn set_ticks_on_borders(&mut self, on_borders: bool) {
        self.ticks_on_borders = on_borders;
    }

    pub fn get_ticks_on_borders(&self) -> bool {
        self.ticks_on_borders
    }

    pub fn set_ticks_position(&mut self, position: TickPosition) {
        self.ticks_position = position;
    }

    pub fn get_ticks_position(&self) -> TickPosition {
        self.ticks_position
    }

    pub fn set_editable(&mut self, editable: bool) {
        self.editable = editable;
    }

    pub fn is_editable(&self) -> bool {
        self.editable
    }

    pub fn set_scrollable(&mut self, scrollable: bool) {
        self.scrollable = scrollable;
    }

    pub fn is_scrollable(&self) -> bool {
        self.scrollable
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum FillMode {
    BeginToEnd,
    EndToBegin,
    TopToBottom,
    BottomToTop,
    Max,
}

#[derive(Clone, Copy, Debug)]
pub struct ProgressBarData {
    pub range: RangeData,
    mode: FillMode,
    show_percentage: bool,
    indeterminate: bool,
    editor_preview_indeterminate: bool,
}

impl Default for ProgressBarData {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressBarData {
    pub fn new() -> Self {
        let mut range = RangeData::new();
        // `ProgressBar::ProgressBar` calls `set_step(0.01)`.
        range.set_step(0.01);
        Self {
            range,
            mode: FillMode::BeginToEnd,
            show_percentage: true,
            indeterminate: false,
            editor_preview_indeterminate: false,
        }
    }

    /// `ERR_FAIL_INDEX(p_fill, FILL_MODE_MAX)`; negatives are rejected too.
    pub fn set_fill_mode(&mut self, fill: i32) -> bool {
        if fill < 0 || fill as usize >= FillMode::Max as usize {
            return false;
        }
        self.mode = match fill {
            0 => FillMode::BeginToEnd,
            1 => FillMode::EndToBegin,
            2 => FillMode::TopToBottom,
            _ => FillMode::BottomToTop,
        };
        true
    }

    pub fn get_fill_mode(&self) -> FillMode {
        self.mode
    }

    pub fn set_show_percentage(&mut self, visible: bool) {
        self.show_percentage = visible;
    }

    pub fn is_percentage_shown(&self) -> bool {
        self.show_percentage
    }

    pub fn set_indeterminate(&mut self, indeterminate: bool) {
        self.indeterminate = indeterminate;
    }

    pub fn is_indeterminate(&self) -> bool {
        self.indeterminate
    }

    pub fn set_editor_preview_indeterminate(&mut self, preview: bool) {
        self.editor_preview_indeterminate = preview;
    }

    pub fn is_editor_preview_indeterminate_enabled(&self) -> bool {
        self.editor_preview_indeterminate
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scroll_bar_constructor_disables_snapping() {
        let mut b = ScrollBarData::new(Orientation::Vertical);
        assert_eq!(b.range.get_step(), 0.0);
        assert_eq!(b.get_custom_step(), -1.0);
        assert!(!b.is_smooth_scroll_enabled());

        // step 0 means the value is kept as given.
        b.range.set_value(2.5);
        assert_eq!(b.range.get_value(), 2.5);

        b.set_custom_step(0.25);
        b.set_smooth_scroll_enabled(true);
        assert_eq!(b.get_custom_step(), 0.25);
        assert!(b.is_smooth_scroll_enabled());
    }

    #[test]
    fn slider_defaults_and_flags() {
        let mut s = SliderData::new(Orientation::Horizontal);
        assert_eq!(s.range.get_step(), 1.0);
        assert_eq!(s.get_ticks(), 0);
        assert_eq!(s.get_ticks_position(), TickPosition::BottomRight);
        assert!(!s.get_ticks_on_borders());
        assert!(s.is_editable());
        assert!(s.is_scrollable());

        s.set_ticks(5);
        s.set_ticks_on_borders(true);
        s.set_ticks_position(TickPosition::Both);
        s.set_editable(false);
        s.set_scrollable(false);
        assert_eq!(s.get_ticks(), 5);
        assert!(s.get_ticks_on_borders());
        assert_eq!(s.get_ticks_position(), TickPosition::Both);
        assert!(!s.is_editable());
        assert!(!s.is_scrollable());

        assert_eq!(TickPosition::BottomRight as i32, 0);
        assert_eq!(TickPosition::TopLeft as i32, 1);
        assert_eq!(TickPosition::Both as i32, 2);
        assert_eq!(TickPosition::Center as i32, 3);
    }

    #[test]
    fn progress_bar_defaults_and_fill_modes() {
        let mut p = ProgressBarData::new();
        assert_eq!(p.range.get_step(), 0.01);
        assert_eq!(p.get_fill_mode(), FillMode::BeginToEnd);
        assert!(p.is_percentage_shown());
        assert!(!p.is_indeterminate());
        assert!(!p.is_editor_preview_indeterminate_enabled());

        // 0.01 snapping: 0.333 becomes 0.33.
        p.range.set_value(0.333);
        assert!((p.range.get_value() - 0.33).abs() < 1e-9, "{}", p.range.get_value());

        assert!(p.set_fill_mode(3));
        assert_eq!(p.get_fill_mode(), FillMode::BottomToTop);
        assert!(!p.set_fill_mode(4));
        assert!(!p.set_fill_mode(-1));
        assert_eq!(p.get_fill_mode(), FillMode::BottomToTop);

        p.set_show_percentage(false);
        p.set_indeterminate(true);
        p.set_editor_preview_indeterminate(true);
        assert!(!p.is_percentage_shown());
        assert!(p.is_indeterminate());
        assert!(p.is_editor_preview_indeterminate_enabled());

        assert_eq!(FillMode::BeginToEnd as i32, 0);
        assert_eq!(FillMode::EndToBegin as i32, 1);
        assert_eq!(FillMode::TopToBottom as i32, 2);
        assert_eq!(FillMode::BottomToTop as i32, 3);
        assert_eq!(FillMode::Max as i32, 4);
    }
}
