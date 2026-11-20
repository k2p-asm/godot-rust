//! `scene/gui/progress_bar.*`: fill direction and the display flags.
//!
//! Defaults: begin-to-end fill, percentage shown, neither indeterminate nor
//! previewing the indeterminate state, and a 200 px/s indeterminate speed.
//!
//! `set_fill_mode` rejects `FILL_MODE_MAX` (and negatives) with `ERR_FAIL_INDEX`
//! and resets the indeterminate animation progress, like the C++. The percentage
//! the bar displays is `Range::get_as_ratio()`; `get_percentage` mirrors the
//! engine's convenience accessor.
//!
//! Drawing, the font/theme and the animation timer are not ported.

use crate::scene::range::RangeData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum FillMode {
    BeginToEnd,
    EndToBegin,
    TopToBottom,
    BottomToTop,
    Max,
}

pub const FILL_MODE_MAX: i32 = 4;

#[derive(Clone, Debug)]
pub struct ProgressBarData {
    pub range: RangeData,
    mode: FillMode,
    show_percentage: bool,
    indeterminate: bool,
    editor_preview_indeterminate: bool,
    indeterminate_min_speed: f64,
    indeterminate_fill_progress: f64,
}

impl Default for ProgressBarData {
    fn default() -> Self {
        Self::new()
    }
}

impl ProgressBarData {
    pub fn new() -> Self {
        Self {
            range: RangeData::new(),
            mode: FillMode::BeginToEnd,
            show_percentage: true,
            indeterminate: false,
            editor_preview_indeterminate: false,
            indeterminate_min_speed: 200.0,
            indeterminate_fill_progress: 0.0,
        }
    }

    /// `ERR_FAIL_INDEX(p_fill, FILL_MODE_MAX)`; resets the animation progress.
    pub fn set_fill_mode(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => FillMode::BeginToEnd,
            1 => FillMode::EndToBegin,
            2 => FillMode::TopToBottom,
            3 => FillMode::BottomToTop,
            _ => return false,
        };
        self.mode = parsed;
        self.indeterminate_fill_progress = 0.0;
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

    pub fn set_indeterminate_min_speed(&mut self, speed: f64) {
        self.indeterminate_min_speed = speed;
    }

    pub fn get_indeterminate_min_speed(&self) -> f64 {
        self.indeterminate_min_speed
    }

    pub fn get_indeterminate_fill_progress(&self) -> f64 {
        self.indeterminate_fill_progress
    }

    /// Advances the indeterminate stripe by `delta` seconds, wrapping at 1 like
    /// the engine's draw step.
    pub fn advance_indeterminate(&mut self, delta: f64, available_width: f64) {
        if available_width <= 0.0 || self.indeterminate_min_speed <= 0.0 {
            return;
        }
        self.indeterminate_fill_progress +=
            delta * self.indeterminate_min_speed / available_width;
        self.indeterminate_fill_progress %= 1.0;
    }

    /// The percentage text value the bar shows (`get_as_ratio()`).
    pub fn get_percentage(&self) -> f64 {
        self.range.get_as_ratio()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let p = ProgressBarData::new();
        assert_eq!(p.get_fill_mode(), FillMode::BeginToEnd);
        assert!(p.is_percentage_shown());
        assert!(!p.is_indeterminate());
        assert!(!p.is_editor_preview_indeterminate_enabled());
        assert_eq!(p.get_indeterminate_min_speed(), 200.0);
        assert_eq!(p.get_indeterminate_fill_progress(), 0.0);
        assert_eq!(p.get_percentage(), 0.0);
    }

    #[test]
    fn fill_modes_are_validated() {
        let mut p = ProgressBarData::new();
        assert!(!p.set_fill_mode(-1));
        assert!(!p.set_fill_mode(FILL_MODE_MAX));
        assert_eq!(p.get_fill_mode(), FillMode::BeginToEnd);

        for mode in 0..FILL_MODE_MAX {
            assert!(p.set_fill_mode(mode), "{mode}");
        }
        assert_eq!(p.get_fill_mode(), FillMode::BottomToTop);

        assert_eq!(FillMode::BeginToEnd as i32, 0);
        assert_eq!(FillMode::EndToBegin as i32, 1);
        assert_eq!(FillMode::TopToBottom as i32, 2);
        assert_eq!(FillMode::BottomToTop as i32, 3);
        assert_eq!(FillMode::Max as i32, 4);
    }

    #[test]
    fn flags_and_animation() {
        let mut p = ProgressBarData::new();
        p.set_show_percentage(false);
        p.set_indeterminate(true);
        p.set_editor_preview_indeterminate(true);
        p.set_indeterminate_min_speed(100.0);
        assert!(!p.is_percentage_shown());
        assert!(p.is_indeterminate());
        assert!(p.is_editor_preview_indeterminate_enabled());

        // 100 px/s over a 200 px span advances half a stripe per second.
        p.advance_indeterminate(1.0, 200.0);
        assert!((p.get_indeterminate_fill_progress() - 0.5).abs() < 1e-9);
        p.advance_indeterminate(1.0, 200.0);
        assert!((p.get_indeterminate_fill_progress() - 0.0).abs() < 1e-9);

        // A zero span or speed does nothing.
        p.advance_indeterminate(1.0, 0.0);
        p.set_indeterminate_min_speed(0.0);
        p.advance_indeterminate(1.0, 200.0);
        assert_eq!(p.get_indeterminate_fill_progress(), 0.0);

        // Changing the fill mode resets the stripe.
        p.set_indeterminate_min_speed(100.0);
        p.advance_indeterminate(1.0, 400.0);
        assert!(p.get_indeterminate_fill_progress() > 0.0);
        p.set_fill_mode(2);
        assert_eq!(p.get_indeterminate_fill_progress(), 0.0);
    }

    #[test]
    fn percentage_follows_the_range() {
        let mut p = ProgressBarData::new();
        p.range.set_min(0.0);
        p.range.set_max(200.0);
        p.range.set_value(50.0);
        assert!((p.get_percentage() - 0.25).abs() < 1e-9);
    }
}
