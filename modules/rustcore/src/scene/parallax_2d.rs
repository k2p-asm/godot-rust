//! `scene/2d/parallax_2d.*` (the modern `Parallax2D`): scroll state.
//!
//! Defaults come from the header: scale `(1, 1)`, repeat times 1, limits
//! `±10000000`, following the viewport on, camera-scroll ignoring off.
//! `set_repeat_size` clamps to `>= 0` (`maxf(0)`) and `set_repeat_times` to
//! `>= 1`; every other setter assigns directly.
//!
//! The camera-moved scroll/repeat computation is not ported (it needs the
//! viewport transform and the node tree).

use crate::math::vector2::Vector2;

/// `Parallax2D::DEFAULT_LIMIT`.
pub const DEFAULT_LIMIT: f32 = 10_000_000.0;

#[derive(Clone, Debug)]
pub struct Parallax2DData {
    group_name: String,
    scroll_scale: Vector2,
    scroll_offset: Vector2,
    screen_offset: Vector2,
    repeat_size: Vector2,
    repeat_times: i32,
    limit_begin: Vector2,
    limit_end: Vector2,
    autoscroll: Vector2,
    follow_viewport: bool,
    ignore_camera_scroll: bool,
}

impl Default for Parallax2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl Parallax2DData {
    pub fn new() -> Self {
        Self {
            group_name: String::new(),
            scroll_scale: Vector2::new(1.0, 1.0),
            scroll_offset: Vector2::ZERO,
            screen_offset: Vector2::ZERO,
            repeat_size: Vector2::ZERO,
            repeat_times: 1,
            limit_begin: Vector2::new(-DEFAULT_LIMIT, -DEFAULT_LIMIT),
            limit_end: Vector2::new(DEFAULT_LIMIT, DEFAULT_LIMIT),
            autoscroll: Vector2::ZERO,
            follow_viewport: true,
            ignore_camera_scroll: false,
        }
    }

    pub fn set_scroll_scale(&mut self, scale: Vector2) {
        self.scroll_scale = scale;
    }

    pub fn get_scroll_scale(&self) -> Vector2 {
        self.scroll_scale
    }

    pub fn set_scroll_offset(&mut self, offset: Vector2) {
        self.scroll_offset = offset;
    }

    pub fn get_scroll_offset(&self) -> Vector2 {
        self.scroll_offset
    }

    pub fn set_screen_offset(&mut self, offset: Vector2) {
        self.screen_offset = offset;
    }

    pub fn get_screen_offset(&self) -> Vector2 {
        self.screen_offset
    }

    /// Negative components are clamped to zero (`maxf(0)`).
    pub fn set_repeat_size(&mut self, size: Vector2) {
        self.repeat_size = Vector2::new(size.x.max(0.0), size.y.max(0.0));
    }

    pub fn get_repeat_size(&self) -> Vector2 {
        self.repeat_size
    }

    /// Values below 1 are clamped up to 1.
    pub fn set_repeat_times(&mut self, times: i32) {
        self.repeat_times = times.max(1);
    }

    pub fn get_repeat_times(&self) -> i32 {
        self.repeat_times
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

    pub fn set_autoscroll(&mut self, autoscroll: Vector2) {
        self.autoscroll = autoscroll;
    }

    pub fn get_autoscroll(&self) -> Vector2 {
        self.autoscroll
    }

    pub fn set_follow_viewport(&mut self, follow: bool) {
        self.follow_viewport = follow;
    }

    pub fn get_follow_viewport(&self) -> bool {
        self.follow_viewport
    }

    pub fn set_ignore_camera_scroll(&mut self, ignore: bool) {
        self.ignore_camera_scroll = ignore;
    }

    pub fn is_ignore_camera_scroll(&self) -> bool {
        self.ignore_camera_scroll
    }

    pub fn set_group_name(&mut self, name: &str) {
        self.group_name = name.to_string();
    }

    pub fn get_group_name(&self) -> &str {
        &self.group_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_match_the_header() {
        let p = Parallax2DData::new();
        assert_eq!(p.get_group_name(), "");
        assert_eq!(p.get_scroll_scale(), Vector2::new(1.0, 1.0));
        assert_eq!(p.get_scroll_offset(), Vector2::ZERO);
        assert_eq!(p.get_screen_offset(), Vector2::ZERO);
        assert_eq!(p.get_repeat_size(), Vector2::ZERO);
        assert_eq!(p.get_repeat_times(), 1);
        assert_eq!(p.get_limit_begin(), Vector2::new(-DEFAULT_LIMIT, -DEFAULT_LIMIT));
        assert_eq!(p.get_limit_end(), Vector2::new(DEFAULT_LIMIT, DEFAULT_LIMIT));
        assert_eq!(p.get_autoscroll(), Vector2::ZERO);
        assert!(p.get_follow_viewport());
        assert!(!p.is_ignore_camera_scroll());
    }

    #[test]
    fn repeat_values_are_clamped() {
        let mut p = Parallax2DData::new();
        p.set_repeat_size(Vector2::new(10.0, -4.0));
        assert_eq!(p.get_repeat_size(), Vector2::new(10.0, 0.0));

        p.set_repeat_times(0);
        assert_eq!(p.get_repeat_times(), 1);
        p.set_repeat_times(-5);
        assert_eq!(p.get_repeat_times(), 1);
        p.set_repeat_times(3);
        assert_eq!(p.get_repeat_times(), 3);
    }

    #[test]
    fn setters_round_trip() {
        let mut p = Parallax2DData::new();
        p.set_scroll_scale(Vector2::new(0.5, 2.0));
        p.set_scroll_offset(Vector2::new(3.0, -1.0));
        p.set_screen_offset(Vector2::new(-2.0, 4.0));
        p.set_limit_begin(Vector2::new(-100.0, -50.0));
        p.set_limit_end(Vector2::new(100.0, 50.0));
        p.set_autoscroll(Vector2::new(1.0, 0.0));
        p.set_follow_viewport(false);
        p.set_ignore_camera_scroll(true);
        p.set_group_name("foreground");

        assert_eq!(p.get_scroll_scale(), Vector2::new(0.5, 2.0));
        assert_eq!(p.get_scroll_offset(), Vector2::new(3.0, -1.0));
        assert_eq!(p.get_screen_offset(), Vector2::new(-2.0, 4.0));
        assert_eq!(p.get_limit_begin(), Vector2::new(-100.0, -50.0));
        assert_eq!(p.get_limit_end(), Vector2::new(100.0, 50.0));
        assert_eq!(p.get_autoscroll(), Vector2::new(1.0, 0.0));
        assert!(!p.get_follow_viewport());
        assert!(p.is_ignore_camera_scroll());
        assert_eq!(p.get_group_name(), "foreground");
    }
}
