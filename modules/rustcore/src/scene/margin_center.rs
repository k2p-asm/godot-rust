//! `scene/gui/margin_container.*` and `scene/gui/center_container.*`.
//!
//! Two small layout containers over `Control`:
//!
//! - `MarginContainer` insets its child by four margins and never gives it less
//!   than its minimum size.
//! - `CenterContainer` places the child at the centre unless `use_top_left`.
//!
//! In the engine both read margins/flags from theme and properties; here they
//! are plain setters, and the child rect is computed directly.

use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;

#[derive(Clone, Copy, Debug, Default)]
pub struct MarginContainerData {
    /// `[left, top, right, bottom]` in pixels.
    margins: [i32; 4],
}

impl MarginContainerData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_margin(&mut self, side: usize, value: i32) -> bool {
        if side >= 4 {
            return false;
        }
        self.margins[side] = value;
        true
    }

    pub fn get_margin(&self, side: usize) -> i32 {
        self.margins.get(side).copied().unwrap_or(0)
    }

    /// Child rect inside the margins, at least `child_min` in size.
    pub fn layout(&self, container: Vector2, child_min: Vector2) -> Rect2 {
        let left = self.margins[0] as f32;
        let top = self.margins[1] as f32;
        let right = self.margins[2] as f32;
        let bottom = self.margins[3] as f32;

        let width = (container.x - left - right).max(child_min.x);
        let height = (container.y - top - bottom).max(child_min.y);

        Rect2::new(Vector2::new(left, top), Vector2::new(width, height))
    }
}

#[derive(Clone, Copy, Debug, Default)]
pub struct CenterContainerData {
    use_top_left: bool,
}

impl CenterContainerData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_use_top_left(&mut self, use_top_left: bool) {
        self.use_top_left = use_top_left;
    }

    pub fn is_using_top_left(&self) -> bool {
        self.use_top_left
    }

    /// Child rect: centred with its minimum size, or at the origin when
    /// `use_top_left` is set.
    pub fn layout(&self, container: Vector2, child_min: Vector2) -> Rect2 {
        if self.use_top_left {
            return Rect2::new(Vector2::ZERO, child_min);
        }
        let position = Vector2::new(
            (container.x - child_min.x) / 2.0,
            (container.y - child_min.y) / 2.0,
        );
        Rect2::new(position, child_min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTAINER: Vector2 = Vector2 { x: 100.0, y: 80.0 };

    #[test]
    fn margins_inset_the_child() {
        let mut m = MarginContainerData::new();
        m.set_margin(0, 10);
        m.set_margin(1, 5);
        m.set_margin(2, 20);
        m.set_margin(3, 5);

        let r = m.layout(CONTAINER, Vector2::new(10.0, 10.0));
        assert_eq!(r.position, Vector2::new(10.0, 5.0));
        assert_eq!(r.size, Vector2::new(70.0, 70.0));
        assert!(!m.set_margin(4, 1));
        assert_eq!(m.get_margin(9), 0);
    }

    #[test]
    fn margins_never_shrink_below_the_child_minimum() {
        let mut m = MarginContainerData::new();
        for side in 0..4 {
            m.set_margin(side, 60);
        }
        let r = m.layout(CONTAINER, Vector2::new(30.0, 20.0));
        assert_eq!(r.size, Vector2::new(30.0, 20.0));
    }

    #[test]
    fn center_places_the_child_in_the_middle() {
        let c = CenterContainerData::new();
        let r = c.layout(CONTAINER, Vector2::new(40.0, 20.0));
        assert_eq!(r.position, Vector2::new(30.0, 30.0));
        assert_eq!(r.size, Vector2::new(40.0, 20.0));
    }

    #[test]
    fn top_left_flag_puts_the_child_at_the_origin() {
        let mut c = CenterContainerData::new();
        c.set_use_top_left(true);
        assert!(c.is_using_top_left());
        let r = c.layout(CONTAINER, Vector2::new(40.0, 20.0));
        assert_eq!(r.position, Vector2::ZERO);
        assert_eq!(r.size, Vector2::new(40.0, 20.0));
    }

    #[test]
    fn oversized_child_centres_with_a_negative_offset() {
        let c = CenterContainerData::new();
        let r = c.layout(Vector2::new(10.0, 10.0), Vector2::new(30.0, 10.0));
        // (10 - 30) / 2 = -10
        assert_eq!(r.position, Vector2::new(-10.0, 0.0));
    }
}
