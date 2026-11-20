//! `scene/gui/aspect_ratio_container.*`: ratio-driven child layout.
//!
//! The child box starts as `(ratio, 1)` and is scaled by:
//!
//! - `WidthControlsHeight`: `size.x / ratio`
//! - `HeightControlsWidth`: `size.y`
//! - `Fit`: `min(size.x / ratio, size.y)`
//! - `Cover`: `max(size.x / ratio, size.y)`
//!
//! then raised to the child's minimum size and placed with the horizontal and
//! vertical alignment fractions (`0`, `0.5`, `1`). The container's own minimum
//! size is the largest child minimum.
//!
//! Not ported: RTL (the engine mirrors the horizontal offset) and the
//! proportional-TextureRect warning/skip.

use crate::math::rect2::Rect2;
use crate::math::vector2::Vector2;
use crate::scene::control::ControlData;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum StretchMode {
    WidthControlsHeight,
    HeightControlsWidth,
    Fit,
    Cover,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum AlignmentMode {
    Begin,
    Center,
    End,
}

impl AlignmentMode {
    fn fraction(self) -> f32 {
        match self {
            AlignmentMode::Begin => 0.0,
            AlignmentMode::Center => 0.5,
            AlignmentMode::End => 1.0,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct AspectRatioContainerData {
    ratio: f32,
    stretch_mode: StretchMode,
    alignment_horizontal: AlignmentMode,
    alignment_vertical: AlignmentMode,
}

impl Default for AspectRatioContainerData {
    fn default() -> Self {
        Self::new()
    }
}

impl AspectRatioContainerData {
    pub fn new() -> Self {
        Self {
            ratio: 1.0,
            stretch_mode: StretchMode::Fit,
            alignment_horizontal: AlignmentMode::Center,
            alignment_vertical: AlignmentMode::Center,
        }
    }

    pub fn set_ratio(&mut self, ratio: f32) {
        self.ratio = ratio;
    }

    pub fn get_ratio(&self) -> f32 {
        self.ratio
    }

    pub fn set_stretch_mode(&mut self, mode: StretchMode) {
        self.stretch_mode = mode;
    }

    pub fn get_stretch_mode(&self) -> StretchMode {
        self.stretch_mode
    }

    pub fn set_alignment_horizontal(&mut self, alignment: AlignmentMode) {
        self.alignment_horizontal = alignment;
    }

    pub fn get_alignment_horizontal(&self) -> AlignmentMode {
        self.alignment_horizontal
    }

    pub fn set_alignment_vertical(&mut self, alignment: AlignmentMode) {
        self.alignment_vertical = alignment;
    }

    pub fn get_alignment_vertical(&self) -> AlignmentMode {
        self.alignment_vertical
    }

    /// `get_minimum_size`: the largest child minimum, with no extra margins.
    pub fn get_minimum_size(child_min: Vector2) -> Vector2 {
        child_min
    }

    /// Sorted child rect for one child (the engine loops all of them).
    pub fn layout(
        &self,
        container: Vector2,
        child: &ControlData,
        child_min: Vector2,
    ) -> Rect2 {
        let base = Vector2::new(self.ratio, 1.0);
        let factor = match self.stretch_mode {
            StretchMode::WidthControlsHeight => container.x / base.x,
            StretchMode::HeightControlsWidth => container.y / base.y,
            StretchMode::Fit => (container.x / base.x).min(container.y / base.y),
            StretchMode::Cover => (container.x / base.x).max(container.y / base.y),
        };

        let mut child_size = base * factor;
        child_size = Vector2::new(
            child_size.x.max(child_min.x),
            child_size.y.max(child_min.y),
        );

        let align = Vector2::new(
            self.alignment_horizontal.fraction(),
            self.alignment_vertical.fraction(),
        );
        let offset = (container - child_size) * align;

        child.layout_in_cell(Rect2::new(offset, child_size), child_min)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const CONTAINER: Vector2 = Vector2 { x: 100.0, y: 100.0 };

    #[test]
    fn defaults() {
        let c = AspectRatioContainerData::new();
        assert_eq!(c.get_ratio(), 1.0);
        assert_eq!(c.get_stretch_mode(), StretchMode::Fit);
        assert_eq!(c.get_alignment_horizontal(), AlignmentMode::Center);
        assert_eq!(c.get_alignment_vertical(), AlignmentMode::Center);
        assert_eq!(AspectRatioContainerData::get_minimum_size(Vector2::new(3.0, 4.0)), Vector2::new(3.0, 4.0));
    }

    #[test]
    fn stretch_modes_scale_the_ratio_box() {
        let child = ControlData::new();
        let mut c = AspectRatioContainerData::new();
        c.set_ratio(2.0);

        // Fit: min(100/2, 100/1) = 50 -> (100, 50), centred vertically.
        let r = c.layout(CONTAINER, &child, Vector2::ZERO);
        assert_eq!(r.size, Vector2::new(100.0, 50.0));
        assert_eq!(r.position, Vector2::new(0.0, 25.0));

        // Cover: max(50, 100) = 100 -> (200, 100), centred horizontally.
        c.set_stretch_mode(StretchMode::Cover);
        let r = c.layout(CONTAINER, &child, Vector2::ZERO);
        assert_eq!(r.size, Vector2::new(200.0, 100.0));
        assert_eq!(r.position, Vector2::new(-50.0, 0.0));

        // Width controls height: 100 / 2 = 50.
        c.set_stretch_mode(StretchMode::WidthControlsHeight);
        let r = c.layout(CONTAINER, &child, Vector2::ZERO);
        assert_eq!(r.size, Vector2::new(100.0, 50.0));

        // Height controls width: 100 / 1 = 100 -> (200, 100).
        c.set_stretch_mode(StretchMode::HeightControlsWidth);
        let r = c.layout(CONTAINER, &child, Vector2::ZERO);
        assert_eq!(r.size, Vector2::new(200.0, 100.0));
    }

    #[test]
    fn alignment_and_child_minimum() {
        let child = ControlData::new();
        let mut c = AspectRatioContainerData::new();
        c.set_ratio(1.0);
        c.set_stretch_mode(StretchMode::WidthControlsHeight);
        c.set_alignment_horizontal(AlignmentMode::Begin);
        c.set_alignment_vertical(AlignmentMode::End);

        // Box (100, 100) in a 100x100 container: only alignment matters.
        let r = c.layout(CONTAINER, &child, Vector2::ZERO);
        assert_eq!(r.position, Vector2::ZERO);

        // Smaller box through Fit with a wide container: 100x20 -> (20,20).
        c.set_stretch_mode(StretchMode::Fit);
        c.set_alignment_horizontal(AlignmentMode::End);
        c.set_alignment_vertical(AlignmentMode::Begin);
        let r = c.layout(Vector2::new(100.0, 20.0), &child, Vector2::ZERO);
        assert_eq!(r.size, Vector2::new(20.0, 20.0));
        assert_eq!(r.position, Vector2::new(80.0, 0.0));

        // The child minimum raises the box after scaling.
        let r = c.layout(Vector2::new(100.0, 20.0), &child, Vector2::new(300.0, 10.0));
        assert_eq!(r.size, Vector2::new(300.0, 20.0));
        assert_eq!(r.position, Vector2::new(-200.0, 0.0));

        assert_eq!(AlignmentMode::Begin as i32, 0);
        assert_eq!(AlignmentMode::Center as i32, 1);
        assert_eq!(AlignmentMode::End as i32, 2);
        assert_eq!(StretchMode::WidthControlsHeight as i32, 0);
        assert_eq!(StretchMode::HeightControlsWidth as i32, 1);
        assert_eq!(StretchMode::Fit as i32, 2);
        assert_eq!(StretchMode::Cover as i32, 3);
    }
}
