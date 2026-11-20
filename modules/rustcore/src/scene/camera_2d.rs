//! `scene/2d/camera_2d.*`: state and the stateless `align()` step.
//!
//! Stored here: anchor mode, offset, zoom (and its reciprocal `zoom_scale`),
//! drag margins/offsets/enabled flags, limits, smoothing speeds, process
//! callback and enabled flag. `align_position` reproduces the engine's
//! `Camera2D::align` math, which is the part that depends only on the node's
//! global position and the screen size.
//!
//! Not ported: the smooth-follow state machine (`first`, `camera_pos`,
//! `smoothed_camera_pos`) and the limit clamping at the end of
//! `get_camera_transform`. Those depend on the viewport and the node tree.

use crate::math::funcs;
use crate::math::vector2::Vector2;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AnchorMode {
    DragCenter,
    FixedTopLeft,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Camera2DProcessCallback {
    Idle,
    Physics,
}

pub const LIMIT_DEFAULT: i32 = 10_000_000;

#[derive(Clone, Copy, Debug)]
pub struct Camera2DData {
    anchor_mode: AnchorMode,
    offset: Vector2,
    zoom: Vector2,
    zoom_scale: Vector2,
    enabled: bool,
    position_smoothing_enabled: bool,
    position_smoothing_speed: f32,
    drag_margin: [f32; 4],
    drag_horizontal_enabled: bool,
    drag_vertical_enabled: bool,
    drag_horizontal_offset: f32,
    drag_vertical_offset: f32,
    limit_left: i32,
    limit_top: i32,
    limit_right: i32,
    limit_bottom: i32,
    process_callback: Camera2DProcessCallback,
}

impl Default for Camera2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl Camera2DData {
    pub fn new() -> Self {
        Self {
            anchor_mode: AnchorMode::DragCenter,
            offset: Vector2::ZERO,
            zoom: Vector2::ONE,
            zoom_scale: Vector2::ONE,
            enabled: true,
            position_smoothing_enabled: false,
            position_smoothing_speed: 5.0,
            drag_margin: [0.2, 0.2, 0.2, 0.2],
            drag_horizontal_enabled: false,
            drag_vertical_enabled: false,
            drag_horizontal_offset: 0.0,
            drag_vertical_offset: 0.0,
            limit_left: -LIMIT_DEFAULT,
            limit_top: -LIMIT_DEFAULT,
            limit_right: LIMIT_DEFAULT,
            limit_bottom: LIMIT_DEFAULT,
            process_callback: Camera2DProcessCallback::Idle,
        }
    }

    pub fn set_anchor_mode(&mut self, mode: AnchorMode) {
        self.anchor_mode = mode;
    }

    pub fn get_anchor_mode(&self) -> AnchorMode {
        self.anchor_mode
    }

    pub fn set_offset(&mut self, offset: Vector2) {
        self.offset = offset;
    }

    pub fn get_offset(&self) -> Vector2 {
        self.offset
    }

    /// `set_zoom`: zero on either axis is rejected (it would break
    /// `affine_invert`), and `zoom_scale` is kept as its reciprocal.
    pub fn set_zoom(&mut self, zoom: Vector2) -> bool {
        if funcs::is_zero_approx(zoom.x) || funcs::is_zero_approx(zoom.y) {
            return false;
        }
        self.zoom = zoom;
        self.zoom_scale = Vector2::new(1.0 / zoom.x, 1.0 / zoom.y);
        true
    }

    pub fn get_zoom(&self) -> Vector2 {
        self.zoom
    }

    pub fn get_zoom_scale(&self) -> Vector2 {
        self.zoom_scale
    }

    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    pub fn set_position_smoothing_enabled(&mut self, enabled: bool) {
        self.position_smoothing_enabled = enabled;
    }

    pub fn is_position_smoothing_enabled(&self) -> bool {
        self.position_smoothing_enabled
    }

    /// `set_position_smoothing_speed` clamps to `>= 0`.
    pub fn set_position_smoothing_speed(&mut self, speed: f32) {
        self.position_smoothing_speed = speed.max(0.0);
    }

    pub fn get_position_smoothing_speed(&self) -> f32 {
        self.position_smoothing_speed
    }

    /// `set_drag_margin(Side, value)`, sides in `[top, left, bottom, right]`
    /// order as the engine's `Side` enum. The engine does not clamp the value.
    pub fn set_drag_margin(&mut self, side: usize, value: f32) -> bool {
        if side >= 4 {
            return false;
        }
        self.drag_margin[side] = value;
        true
    }

    pub fn get_drag_margin(&self, side: usize) -> f32 {
        if side >= 4 {
            return 0.0;
        }
        self.drag_margin[side]
    }

    pub fn set_drag_horizontal_enabled(&mut self, enabled: bool) {
        self.drag_horizontal_enabled = enabled;
    }

    pub fn is_drag_horizontal_enabled(&self) -> bool {
        self.drag_horizontal_enabled
    }

    pub fn set_drag_vertical_enabled(&mut self, enabled: bool) {
        self.drag_vertical_enabled = enabled;
    }

    pub fn is_drag_vertical_enabled(&self) -> bool {
        self.drag_vertical_enabled
    }

    pub fn set_drag_horizontal_offset(&mut self, offset: f32) {
        self.drag_horizontal_offset = offset;
    }

    pub fn get_drag_horizontal_offset(&self) -> f32 {
        self.drag_horizontal_offset
    }

    pub fn set_drag_vertical_offset(&mut self, offset: f32) {
        self.drag_vertical_offset = offset;
    }

    pub fn get_drag_vertical_offset(&self) -> f32 {
        self.drag_vertical_offset
    }

    pub fn set_limit(&mut self, side: usize, value: i32) -> bool {
        match side {
            0 => self.limit_top = value,
            1 => self.limit_left = value,
            2 => self.limit_bottom = value,
            3 => self.limit_right = value,
            _ => return false,
        }
        true
    }

    pub fn get_limit(&self, side: usize) -> i32 {
        match side {
            0 => self.limit_top,
            1 => self.limit_left,
            2 => self.limit_bottom,
            _ => self.limit_right,
        }
    }

    pub fn set_process_callback(&mut self, callback: Camera2DProcessCallback) {
        self.process_callback = callback;
    }

    pub fn get_process_callback(&self) -> Camera2DProcessCallback {
        self.process_callback
    }

    /// `Camera2D::align`: the camera position for a node at `global_position`
    /// and a viewport of `screen_size`.
    pub fn align_position(&self, screen_size: Vector2, global_position: Vector2) -> Vector2 {
        let mut camera_pos = Vector2::ZERO;
        match self.anchor_mode {
            AnchorMode::DragCenter => {
                let horizontal_margin = if self.drag_horizontal_offset < 0.0 {
                    self.drag_margin[3] // SIDE_RIGHT
                } else {
                    self.drag_margin[1] // SIDE_LEFT
                };
                let vertical_margin = if self.drag_vertical_offset < 0.0 {
                    self.drag_margin[0] // SIDE_TOP
                } else {
                    self.drag_margin[2] // SIDE_BOTTOM
                };
                camera_pos.x = global_position.x
                    + screen_size.x * 0.5 * horizontal_margin * self.drag_horizontal_offset;
                camera_pos.y = global_position.y
                    + screen_size.y * 0.5 * vertical_margin * self.drag_vertical_offset;
            }
            AnchorMode::FixedTopLeft => {
                camera_pos = global_position;
            }
        }
        camera_pos
    }

    /// `get_camera_screen_center` for a camera at `camera_pos` (top-left of the
    /// viewport in world space) with a centered anchor.
    pub fn screen_center(&self, screen_size: Vector2, camera_pos: Vector2) -> Vector2 {
        let screen_offset = if self.anchor_mode == AnchorMode::DragCenter {
            screen_size * 0.5
        } else {
            Vector2::ZERO
        };
        camera_pos + screen_offset
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let c = Camera2DData::new();
        assert_eq!(c.get_anchor_mode(), AnchorMode::DragCenter);
        assert_eq!(c.get_zoom(), Vector2::ONE);
        assert_eq!(c.get_zoom_scale(), Vector2::ONE);
        assert!(c.is_enabled());
        assert!(!c.is_position_smoothing_enabled());
        assert!((c.get_position_smoothing_speed() - 5.0).abs() < 1e-6);
        for side in 0..4 {
            assert!((c.get_drag_margin(side) - 0.2).abs() < 1e-6);
        }
        assert_eq!(c.get_limit(0), -LIMIT_DEFAULT);
        assert_eq!(c.get_limit(3), LIMIT_DEFAULT);
        assert_eq!(c.get_process_callback(), Camera2DProcessCallback::Idle);
    }

    #[test]
    fn zero_zoom_is_rejected_and_scale_is_the_reciprocal() {
        let mut c = Camera2DData::new();
        assert!(!c.set_zoom(Vector2::new(0.0, 2.0)));
        assert!(!c.set_zoom(Vector2::new(2.0, 0.0)));
        assert_eq!(c.get_zoom(), Vector2::ONE);

        assert!(c.set_zoom(Vector2::new(2.0, 4.0)));
        assert_eq!(c.get_zoom_scale(), Vector2::new(0.5, 0.25));
    }

    #[test]
    fn smoothing_speed_is_clamped_to_zero() {
        let mut c = Camera2DData::new();
        c.set_position_smoothing_speed(-3.0);
        assert_eq!(c.get_position_smoothing_speed(), 0.0);
        c.set_position_smoothing_speed(2.5);
        assert!((c.get_position_smoothing_speed() - 2.5).abs() < 1e-6);
    }

    #[test]
    fn drag_margins_are_not_clamped_by_the_setter() {
        let mut c = Camera2DData::new();
        assert!(c.set_drag_margin(1, 0.5));
        assert_eq!(c.get_drag_margin(1), 0.5);
        // Out-of-range sides are rejected.
        assert!(!c.set_drag_margin(4, 1.0));
        assert_eq!(c.get_drag_margin(9), 0.0);
    }

    #[test]
    fn align_drag_center_uses_the_signed_offset_and_margin() {
        let mut c = Camera2DData::new();
        c.set_drag_horizontal_offset(0.5);
        c.set_drag_vertical_offset(-0.5);
        let screen = Vector2::new(640.0, 480.0);

        let pos = c.align_position(screen, Vector2::new(100.0, 100.0));
        // x: positive offset uses the left margin (0.2).
        assert!((pos.x - (100.0 + 640.0 * 0.5 * 0.2 * 0.5)).abs() < 1e-4, "{pos:?}");
        // y: negative offset uses the top margin (0.2).
        assert!((pos.y - (100.0 + 480.0 * 0.5 * 0.2 * -0.5)).abs() < 1e-4, "{pos:?}");
    }

    #[test]
    fn align_fixed_top_left_ignores_margins() {
        let mut c = Camera2DData::new();
        c.set_anchor_mode(AnchorMode::FixedTopLeft);
        c.set_drag_horizontal_offset(0.9);
        c.set_drag_margin(1, 0.8);

        let pos = c.align_position(Vector2::new(640.0, 480.0), Vector2::new(7.0, -9.0));
        assert_eq!(pos, Vector2::new(7.0, -9.0));

        // Screen center is the camera position itself for top-left anchors.
        assert_eq!(c.screen_center(Vector2::new(640.0, 480.0), pos), pos);
    }

    #[test]
    fn screen_center_is_half_a_screen_ahead_when_centered() {
        let c = Camera2DData::new();
        let center = c.screen_center(Vector2::new(640.0, 480.0), Vector2::new(10.0, 20.0));
        assert_eq!(center, Vector2::new(330.0, 260.0));
    }
}
