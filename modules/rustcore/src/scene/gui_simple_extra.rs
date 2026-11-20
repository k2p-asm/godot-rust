//! `scene/gui/color_rect.*` and `scene/gui/separator.*`: two small controls.
//!
//! `ColorRect` stores a colour defaulting to opaque white; `Separator` stores an
//! orientation defaulting to horizontal (`Orientation::HORIZONTAL = 0`,
//! `VERTICAL = 1`). `Panel` has no state of its own (only theme styles), so it
//! needs no module.
//!
//! Drawing and minimum-size computation are not ported.

use crate::math::color::Color;

#[derive(Clone, Debug)]
pub struct ColorRectData {
    color: Color,
}

impl Default for ColorRectData {
    fn default() -> Self {
        Self::new()
    }
}

impl ColorRectData {
    pub fn new() -> Self {
        Self {
            color: Color::new(1.0, 1.0, 1.0, 1.0),
        }
    }

    pub fn set_color(&mut self, color: Color) {
        self.color = color;
    }

    pub fn get_color(&self) -> Color {
        self.color
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Orientation {
    Horizontal = 0,
    Vertical = 1,
}

#[derive(Clone, Debug)]
pub struct SeparatorData {
    orientation: Orientation,
}

impl Default for SeparatorData {
    fn default() -> Self {
        Self::new()
    }
}

impl SeparatorData {
    pub fn new() -> Self {
        Self {
            orientation: Orientation::Horizontal,
        }
    }

    pub fn set_orientation(&mut self, orientation: i32) -> bool {
        let parsed = match orientation {
            0 => Orientation::Horizontal,
            1 => Orientation::Vertical,
            _ => return false,
        };
        self.orientation = parsed;
        true
    }

    pub fn get_orientation(&self) -> Orientation {
        self.orientation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_rect_state() {
        let mut rect = ColorRectData::new();
        assert_eq!(rect.get_color(), Color::new(1.0, 1.0, 1.0, 1.0));
        rect.set_color(Color::new(0.25, 0.5, 0.75, 0.5));
        assert_eq!(rect.get_color(), Color::new(0.25, 0.5, 0.75, 0.5));
    }

    #[test]
    fn separator_state() {
        let mut separator = SeparatorData::new();
        assert_eq!(separator.get_orientation(), Orientation::Horizontal);
        assert!(separator.set_orientation(1));
        assert!(!separator.set_orientation(2));
        assert_eq!(separator.get_orientation(), Orientation::Vertical);
        assert_eq!(Orientation::Horizontal as i32, 0);
        assert_eq!(Orientation::Vertical as i32, 1);
    }
}
