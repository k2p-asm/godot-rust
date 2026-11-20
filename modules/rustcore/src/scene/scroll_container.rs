//! `scene/gui/scroll_container.*`: scroll offsets and content clamping.
//!
//! The scrollable range is `max(0, content - container)` on each axis, forced
//! to zero when that axis is disabled. `clamp_scroll` keeps the stored offsets
//! inside that range and `layout` offsets the content by the negative scroll.
//!
//! Not ported: scrollbars, wheel input, `follow_focus`, and the `RESERVE` mode.

use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;
use crate::math::vector2i::Vector2i;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ScrollMode {
    Disabled,
    Auto,
    Always,
    ShowNever,
}

pub struct ScrollContainerData {
    h_mode: ScrollMode,
    v_mode: ScrollMode,
    scroll_horizontal: i32,
    scroll_vertical: i32,
    deadzone: i32,
}

impl Default for ScrollContainerData {
    fn default() -> Self {
        Self::new()
    }
}

impl ScrollContainerData {
    pub fn new() -> Self {
        Self {
            h_mode: ScrollMode::Auto,
            v_mode: ScrollMode::Auto,
            scroll_horizontal: 0,
            scroll_vertical: 0,
            deadzone: 0,
        }
    }

    pub fn set_h_scroll_mode(&mut self, mode: ScrollMode) {
        self.h_mode = mode;
    }

    pub fn get_h_scroll_mode(&self) -> ScrollMode {
        self.h_mode
    }

    pub fn set_v_scroll_mode(&mut self, mode: ScrollMode) {
        self.v_mode = mode;
    }

    pub fn get_v_scroll_mode(&self) -> ScrollMode {
        self.v_mode
    }

    pub fn set_scroll_horizontal(&mut self, value: i32) {
        self.scroll_horizontal = value.max(0);
    }

    pub fn get_scroll_horizontal(&self) -> i32 {
        self.scroll_horizontal
    }

    pub fn set_scroll_vertical(&mut self, value: i32) {
        self.scroll_vertical = value.max(0);
    }

    pub fn get_scroll_vertical(&self) -> i32 {
        self.scroll_vertical
    }

    pub fn set_deadzone(&mut self, deadzone: i32) {
        self.deadzone = deadzone.max(0);
    }

    pub fn get_deadzone(&self) -> i32 {
        self.deadzone
    }

    /// `get_max_scroll`-style bounds for a container/content pair.
    pub fn max_scroll(&self, container: Vector2, content: Vector2) -> Vector2i {
        let max_x = if self.h_mode == ScrollMode::Disabled {
            0
        } else {
            (content.x - container.x).max(0.0) as i32
        };
        let max_y = if self.v_mode == ScrollMode::Disabled {
            0
        } else {
            (content.y - container.y).max(0.0) as i32
        };
        Vector2i::new(max_x, max_y)
    }

    /// Clamps the stored offsets into `[0, max]` and returns them.
    pub fn clamp_scroll(&mut self, container: Vector2, content: Vector2) -> Vector2i {
        let max = self.max_scroll(container, content);
        self.scroll_horizontal = self.scroll_horizontal.clamp(0, max.x);
        self.scroll_vertical = self.scroll_vertical.clamp(0, max.y);
        Vector2i::new(self.scroll_horizontal, self.scroll_vertical)
    }

    /// Content rect: shifted by `-scroll` and never smaller than the container
    /// on the scrollable axes.
    pub fn layout(&self, container: Vector2, content: Vector2) -> Rect2 {
        let width = if self.h_mode == ScrollMode::Disabled {
            content.x
        } else {
            content.x.max(container.x)
        };
        let height = if self.v_mode == ScrollMode::Disabled {
            content.y
        } else {
            content.y.max(container.y)
        };

        Rect2::new(
            Vector2::new(-(self.scroll_horizontal as f32), -(self.scroll_vertical as f32)),
            Vector2::new(width, height),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const VIEW: Vector2 = Vector2 { x: 100.0, y: 80.0 };

    #[test]
    fn defaults() {
        let s = ScrollContainerData::new();
        assert_eq!(s.get_h_scroll_mode(), ScrollMode::Auto);
        assert_eq!(s.get_v_scroll_mode(), ScrollMode::Auto);
        assert_eq!(s.get_scroll_horizontal(), 0);
        assert_eq!(s.get_scroll_vertical(), 0);
        assert_eq!(s.get_deadzone(), 0);
    }

    #[test]
    fn max_scroll_is_content_minus_view() {
        let s = ScrollContainerData::new();
        let max = s.max_scroll(VIEW, Vector2::new(250.0, 200.0));
        assert_eq!(max, Vector2i::new(150, 120));

        // Content smaller than the view: no scrolling.
        let max = s.max_scroll(VIEW, Vector2::new(50.0, 40.0));
        assert_eq!(max, Vector2i::new(0, 0));
    }

    #[test]
    fn disabled_axis_never_scrolls() {
        let mut s = ScrollContainerData::new();
        s.set_v_scroll_mode(ScrollMode::Disabled);
        let max = s.max_scroll(VIEW, Vector2::new(250.0, 200.0));
        assert_eq!(max.x, 150);
        assert_eq!(max.y, 0);
    }

    #[test]
    fn scroll_values_clamp_to_the_bounds() {
        let mut s = ScrollContainerData::new();
        s.set_scroll_horizontal(500);
        s.set_scroll_vertical(500);
        let clamped = s.clamp_scroll(VIEW, Vector2::new(250.0, 200.0));
        assert_eq!(clamped, Vector2i::new(150, 120));
        assert_eq!(s.get_scroll_horizontal(), 150);

        // Negative input is rejected by the setter.
        s.set_scroll_horizontal(-10);
        assert_eq!(s.get_scroll_horizontal(), 0);

        // Growing the view shrinks the range and clamps again.
        s.set_scroll_horizontal(500);
        s.clamp_scroll(Vector2::new(300.0, 80.0), Vector2::new(250.0, 200.0));
        assert_eq!(s.get_scroll_horizontal(), 0);
    }

    #[test]
    fn layout_offsets_the_content() {
        let mut s = ScrollContainerData::new();
        s.set_scroll_horizontal(20);
        s.set_scroll_vertical(10);
        let r = s.layout(VIEW, Vector2::new(250.0, 200.0));
        assert_eq!(r.position, Vector2::new(-20.0, -10.0));
        assert_eq!(r.size, Vector2::new(250.0, 200.0));

        // Small content still fills the container on scrollable axes.
        let r = s.layout(VIEW, Vector2::new(50.0, 40.0));
        assert_eq!(r.size, VIEW);
    }

    #[test]
    fn deadzone_is_non_negative() {
        let mut s = ScrollContainerData::new();
        s.set_deadzone(-5);
        assert_eq!(s.get_deadzone(), 0);
        s.set_deadzone(8);
        assert_eq!(s.get_deadzone(), 8);
    }
}
