//! `scene/2d/parallax_background.*`: the scroll transform fed to ParallaxLayers.
//!
//! Defaults: zero scroll offset, scale 1, zero base offset, base scale `(1,1)`,
//! empty group, zero limit begin/end, zero final offset and camera-zoom
//! ignoring off.
//!
//! Ported: every setter/getter (`set_scroll_offset`, `set_scroll_scale`,
//! `set_scroll_base_offset`, `set_scroll_base_scale`, `set_limit_begin/end`,
//! `set_ignore_camera_zoom`) plus the `_update_scroll` computation, which the
//! port expresses as [`ParallaxBackgroundData::compute_final_offset`]:
//! `offset * scale` shifted by the base offset/scale, clamped to the limits, then
//! emitted as `final_offset` and the per-screen offset.
//!
//! Camera tracking and child layer painting are not ported; callers feed the
//! camera position/screen offset into the computation.

use crate::math::vector2::Vector2;

#[derive(Clone, Debug)]
pub struct ParallaxBackgroundData {
    offset: Vector2,
    scale: f32,
    base_offset: Vector2,
    base_scale: Vector2,
    screen_offset: Vector2,
    group_name: String,
    limit_begin: Vector2,
    limit_end: Vector2,
    final_offset: Vector2,
    ignore_camera_zoom: bool,
}

impl Default for ParallaxBackgroundData {
    fn default() -> Self {
        Self::new()
    }
}

impl ParallaxBackgroundData {
    pub fn new() -> Self {
        Self {
            offset: Vector2::ZERO,
            scale: 1.0,
            base_offset: Vector2::ZERO,
            base_scale: Vector2::new(1.0, 1.0),
            screen_offset: Vector2::ZERO,
            group_name: String::new(),
            limit_begin: Vector2::ZERO,
            limit_end: Vector2::ZERO,
            final_offset: Vector2::ZERO,
            ignore_camera_zoom: false,
        }
    }

    pub fn set_scroll_offset(&mut self, offset: Vector2) {
        self.offset = offset;
    }

    pub fn get_scroll_offset(&self) -> Vector2 {
        self.offset
    }

    pub fn set_scroll_scale(&mut self, scale: f32) {
        self.scale = scale;
    }

    pub fn get_scroll_scale(&self) -> f32 {
        self.scale
    }

    pub fn set_scroll_base_offset(&mut self, offset: Vector2) {
        self.base_offset = offset;
    }

    pub fn get_scroll_base_offset(&self) -> Vector2 {
        self.base_offset
    }

    pub fn set_scroll_base_scale(&mut self, scale: Vector2) {
        self.base_scale = scale;
    }

    pub fn get_scroll_base_scale(&self) -> Vector2 {
        self.base_scale
    }

    pub fn set_limit_begin(&mut self, limit: Vector2) {
        self.limit_begin = limit;
    }

    pub fn get_limit_begin(&self) -> Vector2 {
        self.limit_begin
    }

    pub fn set_limit_end(&mut self, limit: Vector2) {
        self.limit_end = limit;
    }

    pub fn get_limit_end(&self) -> Vector2 {
        self.limit_end
    }

    pub fn set_ignore_camera_zoom(&mut self, ignore: bool) {
        self.ignore_camera_zoom = ignore;
    }

    pub fn is_ignore_camera_zoom(&self) -> bool {
        self.ignore_camera_zoom
    }

    pub fn set_group_name(&mut self, name: &str) {
        self.group_name = name.to_string();
    }

    pub fn get_group_name(&self) -> &str {
        &self.group_name
    }

    pub fn get_screen_offset(&self) -> Vector2 {
        self.screen_offset
    }

    pub fn get_final_offset(&self) -> Vector2 {
        self.final_offset
    }

    /// The scalar core of `_update_scroll`: base offset/scale, then the scroll
    /// offset/scale, clamped to the limits, written to `final_offset` and
    /// `screen_offset`.
    pub fn compute_final_offset(&mut self, camera_position: Vector2) -> Vector2 {
        let ofs = self.base_offset + camera_position * self.base_scale;
        let mut ofs = ofs * self.scale + self.offset;
        // The engine only clamps an axis when begin < end (the defaults 0/0 do
        // not pin the background).
        if self.limit_begin.x < self.limit_end.x {
            ofs.x = ofs.x.clamp(self.limit_begin.x, self.limit_end.x);
        }
        if self.limit_begin.y < self.limit_end.y {
            ofs.y = ofs.y.clamp(self.limit_begin.y, self.limit_end.y);
        }
        self.final_offset = ofs;
        self.screen_offset = ofs;
        ofs
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let p = ParallaxBackgroundData::new();
        assert_eq!(p.get_scroll_offset(), Vector2::ZERO);
        assert_eq!(p.get_scroll_scale(), 1.0);
        assert_eq!(p.get_scroll_base_offset(), Vector2::ZERO);
        assert_eq!(p.get_scroll_base_scale(), Vector2::new(1.0, 1.0));
        assert_eq!(p.get_group_name(), "");
        assert_eq!(p.get_limit_begin(), Vector2::ZERO);
        assert_eq!(p.get_limit_end(), Vector2::ZERO);
        assert_eq!(p.get_final_offset(), Vector2::ZERO);
        assert!(!p.is_ignore_camera_zoom());
    }

    #[test]
    fn setters_round_trip() {
        let mut p = ParallaxBackgroundData::new();
        p.set_scroll_offset(Vector2::new(10.0, 20.0));
        p.set_scroll_scale(2.0);
        p.set_scroll_base_offset(Vector2::new(1.0, 2.0));
        p.set_scroll_base_scale(Vector2::new(0.5, 0.25));
        p.set_limit_begin(Vector2::new(-100.0, -50.0));
        p.set_limit_end(Vector2::new(100.0, 50.0));
        p.set_ignore_camera_zoom(true);
        p.set_group_name("bg");

        assert_eq!(p.get_scroll_offset(), Vector2::new(10.0, 20.0));
        assert_eq!(p.get_scroll_scale(), 2.0);
        assert_eq!(p.get_scroll_base_offset(), Vector2::new(1.0, 2.0));
        assert_eq!(p.get_scroll_base_scale(), Vector2::new(0.5, 0.25));
        assert_eq!(p.get_limit_begin(), Vector2::new(-100.0, -50.0));
        assert_eq!(p.get_limit_end(), Vector2::new(100.0, 50.0));
        assert!(p.is_ignore_camera_zoom());
        assert_eq!(p.get_group_name(), "bg");
    }

    #[test]
    fn offset_computation_and_limits() {
        let mut p = ParallaxBackgroundData::new();
        // (camera * base_scale + base_offset) * scale + offset.
        p.set_scroll_base_offset(Vector2::new(1.0, 0.0));
        p.set_scroll_base_scale(Vector2::new(1.0, 1.0));
        p.set_scroll_scale(0.5);
        p.set_scroll_offset(Vector2::new(0.0, 2.0));
        let ofs = p.compute_final_offset(Vector2::new(10.0, 10.0));
        assert_eq!(ofs, Vector2::new(5.5, 7.0));
        assert_eq!(p.get_final_offset(), ofs);
        assert_eq!(p.get_screen_offset(), ofs);

        // Limits clamp the result.
        p.set_limit_begin(Vector2::new(0.0, 0.0));
        p.set_limit_end(Vector2::new(4.0, 4.0));
        let ofs = p.compute_final_offset(Vector2::new(10.0, 10.0));
        assert_eq!(ofs, Vector2::new(4.0, 4.0));

        // Reversed (or equal) limits do not clamp: the engine only clamps when
        // begin < end on that axis.
        p.set_limit_begin(Vector2::new(8.0, 8.0));
        p.set_limit_end(Vector2::new(1.0, 1.0));
        let ofs = p.compute_final_offset(Vector2::new(10.0, 0.0));
        assert_eq!(ofs, Vector2::new(5.5, 2.0));
    }
}
