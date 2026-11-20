//! `scene/2d/canvas_modulate.*`: the canvas-wide tint.
//!
//! Default colour is white. The node tracks whether it is inside a canvas and
//! visible, and the engine lets exactly one visible `CanvasModulate` per canvas
//! be the active one; that bookkeeping is modelled with the three flags so
//! callers can drive it.

use crate::math::color::Color;

#[derive(Clone, Copy, Debug)]
pub struct CanvasModulateData {
    color: Color,
    is_in_canvas: bool,
    was_visible_in_tree: bool,
    is_active: bool,
}

impl Default for CanvasModulateData {
    fn default() -> Self {
        Self::new()
    }
}

impl CanvasModulateData {
    pub fn new() -> Self {
        Self {
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            is_in_canvas: false,
            was_visible_in_tree: false,
            is_active: false,
        }
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn get_color(&self) -> Color {
        self.color
    }

    /// Entering/leaving the canvas (the engine's notification handling).
    pub fn set_in_canvas(&mut self, inside: bool) {
        self.is_in_canvas = inside;
        if !inside {
            self.is_active = false;
            self.was_visible_in_tree = false;
        }
    }

    pub fn is_in_canvas(&self) -> bool {
        self.is_in_canvas
    }

    pub fn set_was_visible_in_tree(&mut self, visible: bool) {
        self.was_visible_in_tree = visible;
    }

    pub fn was_visible_in_tree(&self) -> bool {
        self.was_visible_in_tree
    }

    /// A canvas only accepts one active modulate; the caller arbitrates.
    pub fn set_active(&mut self, active: bool) {
        self.is_active = active;
    }

    pub fn is_active(&self) -> bool {
        self.is_active
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_color() {
        let mut c = CanvasModulateData::new();
        assert_eq!(c.get_color(), Color::new(1.0, 1.0, 1.0, 1.0));
        assert!(!c.is_in_canvas());
        assert!(!c.was_visible_in_tree());
        assert!(!c.is_active());

        c.set_color(Color::new(0.25, 0.5, 0.75, 0.5));
        assert_eq!(c.get_color(), Color::new(0.25, 0.5, 0.75, 0.5));
    }

    #[test]
    fn canvas_membership_resets_activity() {
        let mut c = CanvasModulateData::new();
        c.set_in_canvas(true);
        c.set_was_visible_in_tree(true);
        c.set_active(true);
        assert!(c.is_in_canvas() && c.is_active() && c.was_visible_in_tree());

        c.set_in_canvas(false);
        assert!(!c.is_in_canvas());
        assert!(!c.is_active(), "leaving the canvas deactivates");
        assert!(!c.was_visible_in_tree());
    }
}
