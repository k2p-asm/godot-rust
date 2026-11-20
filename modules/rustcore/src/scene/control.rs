//! `scene/gui/control.*`: anchors, offsets and the layout rect.
//!
//! A control's rect is derived from its anchors and offsets relative to the
//! parent:
//!
//! `left = parent.x * anchor_left + offset_left`, and the same for the other
//! three sides. `set_size` solves that equation for the right/bottom offsets.
//!
//! The engine enforces `custom_minimum_size` by adjusting offsets; here the
//! returned rect is grown instead. Presets only set the anchors (the engine's
//! `keep_offsets` variant is not ported).
//!
//! Not ported: containers, focus, themes, and drawing.

use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LayoutPreset {
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
    CenterLeft,
    CenterTop,
    CenterRight,
    CenterBottom,
    Center,
    LeftWide,
    TopWide,
    RightWide,
    BottomWide,
    FullRect,
}

#[derive(Clone, Copy, Debug)]
pub struct ControlData {
    /// `[left, top, right, bottom]`, in 0..1 of the parent.
    anchors: [f32; 4],
    /// `[left, top, right, bottom]` in pixels.
    offsets: [f32; 4],
    custom_min_size: Vector2,
}

impl Default for ControlData {
    fn default() -> Self {
        Self::new()
    }
}

impl ControlData {
    pub fn new() -> Self {
        Self {
            anchors: [0.0, 0.0, 0.0, 0.0],
            offsets: [0.0, 0.0, 0.0, 0.0],
            custom_min_size: Vector2::ZERO,
        }
    }

    /// Sides are `[left, top, right, bottom]`.
    pub fn set_anchor(&mut self, side: usize, value: f32) -> bool {
        if side >= 4 {
            return false;
        }
        self.anchors[side] = value;
        true
    }

    pub fn get_anchor(&self, side: usize) -> f32 {
        self.anchors.get(side).copied().unwrap_or(0.0)
    }

    pub fn set_offset(&mut self, side: usize, value: f32) -> bool {
        if side >= 4 {
            return false;
        }
        self.offsets[side] = value;
        true
    }

    pub fn get_offset(&self, side: usize) -> f32 {
        self.offsets.get(side).copied().unwrap_or(0.0)
    }

    pub fn set_anchors_preset(&mut self, preset: LayoutPreset) {
        let (l, t, r, b) = match preset {
            LayoutPreset::TopLeft => (0.0, 0.0, 0.0, 0.0),
            LayoutPreset::TopRight => (1.0, 0.0, 1.0, 0.0),
            LayoutPreset::BottomLeft => (0.0, 1.0, 0.0, 1.0),
            LayoutPreset::BottomRight => (1.0, 1.0, 1.0, 1.0),
            LayoutPreset::CenterLeft => (0.0, 0.5, 0.0, 0.5),
            LayoutPreset::CenterTop => (0.5, 0.0, 0.5, 0.0),
            LayoutPreset::CenterRight => (1.0, 0.5, 1.0, 0.5),
            LayoutPreset::CenterBottom => (0.5, 1.0, 0.5, 1.0),
            LayoutPreset::Center => (0.5, 0.5, 0.5, 0.5),
            LayoutPreset::LeftWide => (0.0, 0.0, 0.0, 1.0),
            LayoutPreset::TopWide => (0.0, 0.0, 1.0, 0.0),
            LayoutPreset::RightWide => (1.0, 0.0, 1.0, 1.0),
            LayoutPreset::BottomWide => (0.0, 1.0, 1.0, 1.0),
            LayoutPreset::FullRect => (0.0, 0.0, 1.0, 1.0),
        };
        self.anchors = [l, t, r, b];
    }

    pub fn set_custom_minimum_size(&mut self, size: Vector2) {
        self.custom_min_size = Vector2::new(size.x.max(0.0), size.y.max(0.0));
    }

    pub fn get_custom_minimum_size(&self) -> Vector2 {
        self.custom_min_size
    }

    /// `get_rect`, with the minimum size enforced on the result.
    pub fn get_rect(&self, parent_size: Vector2) -> Rect2 {
        let left = parent_size.x * self.anchors[0] + self.offsets[0];
        let top = parent_size.y * self.anchors[1] + self.offsets[1];
        let right = parent_size.x * self.anchors[2] + self.offsets[2];
        let bottom = parent_size.y * self.anchors[3] + self.offsets[3];

        let mut size = Vector2::new(right - left, bottom - top);
        if size.x < self.custom_min_size.x {
            size.x = self.custom_min_size.x;
        }
        if size.y < self.custom_min_size.y {
            size.y = self.custom_min_size.y;
        }
        Rect2::new(Vector2::new(left, top), size)
    }

    /// `set_position`: shifts the offsets so the rect starts at `position`.
    pub fn set_position(&mut self, position: Vector2, parent_size: Vector2) {
        let left = parent_size.x * self.anchors[0] + self.offsets[0];
        let top = parent_size.y * self.anchors[1] + self.offsets[1];
        self.offsets[0] += position.x - left;
        self.offsets[1] += position.y - top;
        self.offsets[2] += position.x - left;
        self.offsets[3] += position.y - top;
    }

    /// `set_size`: solves `size = (anchor2 - anchor1) * parent + (offset2 - offset1)`
    /// for the right/bottom offsets, keeping the left/top corner fixed.
    pub fn set_size(&mut self, size: Vector2, parent_size: Vector2) {
        self.offsets[2] = self.offsets[0] + size.x
            - parent_size.x * (self.anchors[2] - self.anchors[0]);
        self.offsets[3] = self.offsets[1] + size.y
            - parent_size.y * (self.anchors[3] - self.anchors[1]);
    }

    pub fn get_position(&self, parent_size: Vector2) -> Vector2 {
        Vector2::new(
            parent_size.x * self.anchors[0] + self.offsets[0],
            parent_size.y * self.anchors[1] + self.offsets[1],
        )
    }

    pub fn get_size(&self, parent_size: Vector2) -> Vector2 {
        self.get_rect(parent_size).size
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const PARENT: Vector2 = Vector2 { x: 200.0, y: 100.0 };

    #[test]
    fn defaults_are_a_zero_rect_at_the_origin() {
        let c = ControlData::new();
        assert_eq!(c.get_rect(PARENT), Rect2::new(Vector2::ZERO, Vector2::ZERO));
        assert_eq!(c.get_position(PARENT), Vector2::ZERO);
    }

    #[test]
    fn full_rect_covers_the_parent() {
        let mut c = ControlData::new();
        c.set_anchors_preset(LayoutPreset::FullRect);
        let r = c.get_rect(PARENT);
        assert_eq!(r.position, Vector2::ZERO);
        assert_eq!(r.size, PARENT);
    }

    #[test]
    fn anchors_with_offsets_inset_the_rect() {
        let mut c = ControlData::new();
        c.set_anchors_preset(LayoutPreset::FullRect);
        c.set_offset(0, 10.0); // left
        c.set_offset(1, 5.0); // top
        c.set_offset(2, -10.0); // right
        c.set_offset(3, -5.0); // bottom
        let r = c.get_rect(PARENT);
        assert_eq!(r.position, Vector2::new(10.0, 5.0));
        assert_eq!(r.size, Vector2::new(180.0, 90.0));
    }

    #[test]
    fn centred_preset_uses_half_the_parent() {
        let mut c = ControlData::new();
        c.set_anchors_preset(LayoutPreset::Center);
        c.set_offset(0, -25.0);
        c.set_offset(2, 25.0);
        c.set_offset(1, -10.0);
        c.set_offset(3, 10.0);
        let r = c.get_rect(PARENT);
        assert_eq!(r.position, Vector2::new(75.0, 40.0));
        assert_eq!(r.size, Vector2::new(50.0, 20.0));
    }

    #[test]
    fn set_size_and_position_drive_the_offsets() {
        let mut c = ControlData::new();
        c.set_anchors_preset(LayoutPreset::TopLeft);
        c.set_size(Vector2::new(80.0, 30.0), PARENT);
        assert_eq!(c.get_size(PARENT), Vector2::new(80.0, 30.0));

        c.set_position(Vector2::new(15.0, 20.0), PARENT);
        let r = c.get_rect(PARENT);
        assert_eq!(r.position, Vector2::new(15.0, 20.0));
        assert_eq!(r.size, Vector2::new(80.0, 30.0));

        // With a full-rect anchor pair, setting size adjusts for the anchor span.
        let mut f = ControlData::new();
        f.set_anchors_preset(LayoutPreset::FullRect);
        f.set_size(Vector2::new(50.0, 40.0), PARENT);
        assert_eq!(f.get_size(PARENT), Vector2::new(50.0, 40.0));
    }

    #[test]
    fn minimum_size_grows_the_rect() {
        let mut c = ControlData::new();
        c.set_anchors_preset(LayoutPreset::TopLeft);
        c.set_size(Vector2::new(5.0, 5.0), PARENT);
        c.set_custom_minimum_size(Vector2::new(20.0, 12.0));

        let r = c.get_rect(PARENT);
        assert_eq!(r.size, Vector2::new(20.0, 12.0));
        // Negative minimums are clamped at zero.
        c.set_custom_minimum_size(Vector2::new(-5.0, -5.0));
        assert_eq!(c.get_custom_minimum_size(), Vector2::ZERO);
    }

    #[test]
    fn invalid_sides_are_rejected() {
        let mut c = ControlData::new();
        assert!(!c.set_anchor(4, 0.5));
        assert!(!c.set_offset(9, 1.0));
        assert_eq!(c.get_anchor(4), 0.0);
        assert_eq!(c.get_offset(9), 0.0);
    }
}
