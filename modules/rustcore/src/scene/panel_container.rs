//! `scene/gui/panel_container.*`: the style box inset container.
//!
//! The engine reads the theme's panel style box and:
//!
//! - reports `child_min + style_min` as its minimum size;
//! - sorts each child inside `Rect2(style_offset, size - style_min)`, letting
//!   `fit_child_in_rect` apply the child's size flags.
//!
//! Both pieces are reproduced here with the style box's minimum size and offset
//! passed explicitly, since the port has no theme lookup on controls yet.

use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;
use crate::scene::control::ControlData;

#[derive(Clone, Copy, Debug, Default)]
pub struct PanelContainerData {
    _private: (),
}

impl PanelContainerData {
    pub fn new() -> Self {
        Self::default()
    }

    /// `child_min + style_min` (the engine adds the style box minimum).
    pub fn get_minimum_size(&self, child_min: Vector2, style_min: Vector2) -> Vector2 {
        child_min + style_min
    }

    /// Child rect: the style box's offset, with the style box minimum removed
    /// from the available size, then the child's own size flags applied.
    pub fn layout(
        &self,
        container: Vector2,
        style_min: Vector2,
        style_offset: Vector2,
        child: &ControlData,
        child_min: Vector2,
    ) -> Rect2 {
        let cell = Rect2::new(style_offset, container - style_min);
        child.layout_in_cell(cell, child_min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::scene::control::{size_flags, LayoutPreset};

    const CONTAINER: Vector2 = Vector2 { x: 100.0, y: 80.0 };

    #[test]
    fn minimum_size_adds_the_style_box() {
        let p = PanelContainerData::new();
        assert_eq!(
            p.get_minimum_size(Vector2::new(30.0, 20.0), Vector2::new(10.0, 6.0)),
            Vector2::new(40.0, 26.0)
        );
        // No style box: the child minimum passes through.
        assert_eq!(
            p.get_minimum_size(Vector2::new(30.0, 20.0), Vector2::ZERO),
            Vector2::new(30.0, 20.0)
        );
    }

    #[test]
    fn fill_child_takes_the_inset_cell() {
        let p = PanelContainerData::new();
        let child = ControlData::new();
        let rect = p.layout(
            CONTAINER,
            Vector2::new(20.0, 10.0),
            Vector2::new(10.0, 5.0),
            &child,
            Vector2::ZERO,
        );
        assert_eq!(rect.position, Vector2::new(10.0, 5.0));
        assert_eq!(rect.size, Vector2::new(80.0, 70.0));
    }

    #[test]
    fn child_flags_apply_inside_the_cell() {
        let p = PanelContainerData::new();
        let mut child = ControlData::new();
        child.set_size_flags_horizontal(size_flags::SHRINK_CENTER);
        child.set_size_flags_vertical(size_flags::SHRINK_END);

        let rect = p.layout(
            CONTAINER,
            Vector2::new(20.0, 10.0),
            Vector2::new(10.0, 5.0),
            &child,
            Vector2::new(20.0, 10.0),
        );
        // Cell: (10,5) 80x70. Centred horizontally: 10 + (80-20)/2 = 40.
        // Shrink-end vertically: 5 + (70-10) = 65.
        assert_eq!(rect.position, Vector2::new(40.0, 65.0));
        assert_eq!(rect.size, Vector2::new(20.0, 10.0));

        // A zero style box behaves like a plain control parent.
        let mut top_left = ControlData::new();
        top_left.set_anchors_preset(LayoutPreset::TopLeft);
        let rect = p.layout(CONTAINER, Vector2::ZERO, Vector2::ZERO, &top_left, Vector2::new(5.0, 5.0));
        assert_eq!(rect, Rect2::new(Vector2::ZERO, CONTAINER));
    }
}
