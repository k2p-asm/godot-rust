//! `scene/gui/slider.*`: slider ticks, drag record and edit flags.
//!
//! A `Slider` is a `Range` with tick marks, a grab record and two interaction
//! flags. Defaults: no ticks at the bottom-right, editable and scrollable, and a
//! custom step of -1.
//!
//! Ported: every setter/getter and `set_editable`, which clears the active grab
//! like the engine. Drawing, input and the theme are not ported.

use crate::scene::gui_simple::Orientation;
use crate::scene::range::RangeData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum TickPosition {
    BottomRight,
    TopLeft,
    Both,
    Center,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct GrabState {
    pub pos: i32,
    pub uvalue: f64,
    pub value_before_dragging: f64,
    pub active: bool,
}

#[derive(Clone, Debug)]
pub struct SliderData {
    pub range: RangeData,
    orientation: Orientation,
    ticks: i32,
    ticks_position: TickPosition,
    mouse_inside: bool,
    custom_step: f64,
    editable: bool,
    scrollable: bool,
    pub grab: GrabState,
}

impl Default for SliderData {
    fn default() -> Self {
        Self::new(Orientation::Horizontal)
    }
}

impl SliderData {
    pub fn new(orientation: Orientation) -> Self {
        Self {
            range: RangeData::new(),
            orientation,
            ticks: 0,
            ticks_position: TickPosition::BottomRight,
            mouse_inside: false,
            custom_step: -1.0,
            editable: true,
            scrollable: true,
            grab: GrabState::default(),
        }
    }

    pub fn get_orientation(&self) -> Orientation {
        self.orientation
    }

    pub fn set_ticks(&mut self, count: i32) {
        self.ticks = count;
    }

    pub fn get_ticks(&self) -> i32 {
        self.ticks
    }

    /// `TICK_POSITION_*` are plain enum values; out-of-range values are rejected
    /// by the property binder, so they are accepted here like any enum cast.
    pub fn set_ticks_position(&mut self, position: TickPosition) {
        self.ticks_position = position;
    }

    pub fn get_ticks_position(&self) -> TickPosition {
        self.ticks_position
    }

    pub fn set_mouse_inside(&mut self, inside: bool) {
        self.mouse_inside = inside;
    }

    pub fn is_mouse_inside(&self) -> bool {
        self.mouse_inside
    }

    /// `set_editable` also cancels an active grab.
    pub fn set_editable(&mut self, editable: bool) {
        self.grab.active = false;
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

    pub fn set_custom_step(&mut self, step: f64) {
        self.custom_step = step;
    }

    pub fn get_custom_step(&self) -> f64 {
        self.custom_step
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let s = SliderData::default();
        assert_eq!(s.get_orientation(), Orientation::Horizontal);
        assert_eq!(s.get_ticks(), 0);
        assert_eq!(s.get_ticks_position(), TickPosition::BottomRight);
        assert!(!s.is_mouse_inside());
        assert_eq!(s.get_custom_step(), -1.0);
        assert!(s.is_editable());
        assert!(s.is_scrollable());
        assert!(!s.grab.active);
    }

    #[test]
    fn setters_and_grab_cancel() {
        let mut s = SliderData::new(Orientation::Vertical);
        s.set_ticks(5);
        s.set_ticks_position(TickPosition::Center);
        s.set_mouse_inside(true);
        s.set_scrollable(false);
        s.set_custom_step(0.25);
        s.grab.active = true;
        s.grab.pos = 3;
        s.grab.uvalue = 0.5;
        s.grab.value_before_dragging = 10.0;

        assert_eq!(s.get_orientation(), Orientation::Vertical);
        assert_eq!(s.get_ticks(), 5);
        assert_eq!(s.get_ticks_position(), TickPosition::Center);
        assert!(s.is_mouse_inside());
        assert!(!s.is_scrollable());
        assert_eq!(s.get_custom_step(), 0.25);
        assert!(s.grab.active);
        assert_eq!(s.grab.pos, 3);

        // Toggling editability clears the grab.
        s.set_editable(false);
        assert!(!s.grab.active);
        assert!(!s.is_editable());
        s.set_editable(true);
        assert!(s.is_editable());

        assert_eq!(TickPosition::BottomRight as i32, 0);
        assert_eq!(TickPosition::TopLeft as i32, 1);
        assert_eq!(TickPosition::Both as i32, 2);
        assert_eq!(TickPosition::Center as i32, 3);
    }
}
