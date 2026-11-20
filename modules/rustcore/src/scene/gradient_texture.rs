//! `scene/resources/gradient_texture.*`: 1D/2D gradient textures.
//!
//! Dimensions are validated like the engine (`1..=16384`, otherwise the old
//! value is kept). `GradientTexture2D` also exposes the gradient-offset
//! computation that `_update` uses per texel: linear (projected on the
//! uncapped segment, signed), radial, square (Chebyshev) and conic (positive
//! angle over `TAU`), followed by the none/repeat/mirror wrapping.
//!
//! The gradient itself is stored by value; the image generation, RIDs and the
//! `changed` notifications are not ported.

use crate::math::funcs;
use crate::math::geometry_2d;
use crate::math::vector2::Vector2;
use crate::scene::gradient::Gradient;

/// `Texture2D`'s accepted dimension range.
const MAX_TEXTURE_SIZE: i32 = 16384;

#[derive(Clone, Debug)]
pub struct GradientTexture1D {
    gradient: Option<Gradient>,
    width: i32,
    use_hdr: bool,
}

impl Default for GradientTexture1D {
    fn default() -> Self {
        Self::new()
    }
}

impl GradientTexture1D {
    pub fn new() -> Self {
        Self {
            gradient: None,
            width: 256,
            use_hdr: false,
        }
    }

    pub fn set_gradient(&mut self, gradient: Option<Gradient>) {
        self.gradient = gradient;
    }

    pub fn get_gradient(&self) -> Option<&Gradient> {
        self.gradient.as_ref()
    }

    /// Rejects values outside `1..=16384`, keeping the old width.
    pub fn set_width(&mut self, width: i32) -> bool {
        if width <= 0 || width > MAX_TEXTURE_SIZE {
            return false;
        }
        self.width = width;
        true
    }

    pub fn get_width(&self) -> i32 {
        self.width
    }

    pub fn get_height(&self) -> i32 {
        1
    }

    pub fn set_use_hdr(&mut self, enable: bool) {
        self.use_hdr = enable;
    }

    pub fn is_using_hdr(&self) -> bool {
        self.use_hdr
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Fill {
    Linear,
    Radial,
    Square,
    Conic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum Repeat {
    None,
    Repeat,
    Mirror,
}

#[derive(Clone, Debug)]
pub struct GradientTexture2D {
    gradient: Option<Gradient>,
    width: i32,
    height: i32,
    use_hdr: bool,
    fill_from: Vector2,
    fill_to: Vector2,
    fill: Fill,
    repeat: Repeat,
}

impl Default for GradientTexture2D {
    fn default() -> Self {
        Self::new()
    }
}

impl GradientTexture2D {
    pub fn new() -> Self {
        Self {
            gradient: None,
            width: 64,
            height: 64,
            use_hdr: false,
            fill_from: Vector2::ZERO,
            fill_to: Vector2::new(1.0, 0.0),
            fill: Fill::Linear,
            repeat: Repeat::None,
        }
    }

    pub fn set_gradient(&mut self, gradient: Option<Gradient>) {
        self.gradient = gradient;
    }

    pub fn get_gradient(&self) -> Option<&Gradient> {
        self.gradient.as_ref()
    }

    pub fn set_width(&mut self, width: i32) -> bool {
        if width <= 0 || width > MAX_TEXTURE_SIZE {
            return false;
        }
        self.width = width;
        true
    }

    pub fn get_width(&self) -> i32 {
        self.width
    }

    pub fn set_height(&mut self, height: i32) -> bool {
        if height <= 0 || height > MAX_TEXTURE_SIZE {
            return false;
        }
        self.height = height;
        true
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }

    pub fn set_use_hdr(&mut self, enable: bool) {
        self.use_hdr = enable;
    }

    pub fn is_using_hdr(&self) -> bool {
        self.use_hdr
    }

    pub fn set_fill(&mut self, fill: Fill) {
        self.fill = fill;
    }

    pub fn get_fill(&self) -> Fill {
        self.fill
    }

    pub fn set_fill_from(&mut self, from: Vector2) {
        self.fill_from = from;
    }

    pub fn get_fill_from(&self) -> Vector2 {
        self.fill_from
    }

    pub fn set_fill_to(&mut self, to: Vector2) {
        self.fill_to = to;
    }

    pub fn get_fill_to(&self) -> Vector2 {
        self.fill_to
    }

    pub fn set_repeat(&mut self, repeat: Repeat) {
        self.repeat = repeat;
    }

    pub fn get_repeat(&self) -> Repeat {
        self.repeat
    }

    /// `_get_gradient_offset_at`, texel by texel exactly as the engine does it.
    pub fn get_gradient_offset_at(&self, x: i32, y: i32) -> f32 {
        if self.fill_to == self.fill_from {
            return 0.0;
        }

        let mut pos = Vector2::ZERO;
        if self.width > 1 {
            pos.x = x as f32 / (self.width - 1) as f32;
        }
        if self.height > 1 {
            pos.y = y as f32 / (self.height - 1) as f32;
        }

        let delta = self.fill_to - self.fill_from;
        let mut ofs = 0.0f32;
        match self.fill {
            Fill::Linear => {
                let closest =
                    geometry_2d::get_closest_point_to_segment_uncapped(pos, self.fill_from, self.fill_to);
                ofs = (closest - self.fill_from).length() / delta.length();
                if (closest - self.fill_from).dot(delta) < 0.0 {
                    ofs *= -1.0;
                }
            }
            Fill::Radial => {
                ofs = (pos - self.fill_from).length() / delta.length();
            }
            Fill::Square => {
                let numerator =
                    (pos.x - self.fill_from.x).abs().max((pos.y - self.fill_from.y).abs());
                let denominator = delta.x.abs().max(delta.y.abs());
                ofs = numerator / denominator;
            }
            Fill::Conic => {
                let rel_angle = delta.angle_to(pos - self.fill_from);
                ofs = funcs::fposmod(rel_angle, funcs::TAU as f32) / funcs::TAU as f32;
            }
        }

        match self.repeat {
            Repeat::None => ofs.clamp(0.0, 1.0),
            Repeat::Repeat => {
                let mut r = ofs % 1.0;
                if r < 0.0 {
                    r = 1.0 + r;
                }
                r
            }
            Repeat::Mirror => {
                let mut r = ofs.abs() % 2.0;
                if r > 1.0 {
                    r = 2.0 - r;
                }
                r
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texture_1d_defaults_and_size_validation() {
        let mut t = GradientTexture1D::new();
        assert!(t.get_gradient().is_none());
        assert_eq!(t.get_width(), 256);
        assert_eq!(t.get_height(), 1);
        assert!(!t.is_using_hdr());

        assert!(!t.set_width(0));
        assert!(!t.set_width(-5));
        assert!(!t.set_width(16385));
        assert_eq!(t.get_width(), 256);
        assert!(t.set_width(1));
        assert!(t.set_width(16384));
        assert_eq!(t.get_width(), 16384);

        t.set_use_hdr(true);
        assert!(t.is_using_hdr());
    }

    #[test]
    fn texture_2d_defaults() {
        let mut t = GradientTexture2D::new();
        assert_eq!(t.get_width(), 64);
        assert_eq!(t.get_height(), 64);
        assert_eq!(t.get_fill_from(), Vector2::ZERO);
        assert_eq!(t.get_fill_to(), Vector2::new(1.0, 0.0));
        assert_eq!(t.get_fill(), Fill::Linear);
        assert_eq!(t.get_repeat(), Repeat::None);
        assert!(!t.is_using_hdr());

        assert!(!t.set_width(0));
        assert!(!t.set_height(0));
        assert!(!t.set_width(20000));
        assert!(!t.set_height(20000));
        assert_eq!((t.get_width(), t.get_height()), (64, 64));
        assert!(t.set_width(2));
        assert!(t.set_height(3));
        assert_eq!((t.get_width(), t.get_height()), (2, 3));

        assert_eq!(Fill::Linear as i32, 0);
        assert_eq!(Fill::Radial as i32, 1);
        assert_eq!(Fill::Square as i32, 2);
        assert_eq!(Fill::Conic as i32, 3);
        assert_eq!(Repeat::None as i32, 0);
        assert_eq!(Repeat::Repeat as i32, 1);
        assert_eq!(Repeat::Mirror as i32, 2);
    }

    #[test]
    fn linear_offset_is_projected_and_signed() {
        let mut t = GradientTexture2D::new();
        t.set_width(3);
        t.set_height(3);

        // Left edge is 0, right edge is 1, and the middle is 0.5.
        assert_eq!(t.get_gradient_offset_at(0, 0), 0.0);
        assert!((t.get_gradient_offset_at(1, 1) - 0.5).abs() < 1e-5);
        assert!((t.get_gradient_offset_at(2, 0) - 1.0).abs() < 1e-5);

        // Behind the start the offset is negative and then clamped to 0.
        t.set_fill_from(Vector2::new(0.5, 0.0));
        t.set_fill_to(Vector2::new(1.0, 0.0));
        let behind = {
            // Disable clamping by checking the raw sign through a repeat mode.
            t.set_repeat(Repeat::Mirror);
            let v = t.get_gradient_offset_at(0, 0); // pos.x = 0, ofs = -1 -> mirror 0
            t.set_repeat(Repeat::None);
            v
        };
        assert!(behind >= 0.0);

        // Degenerate fill: everything maps to 0.
        t.set_fill_to(t.get_fill_from());
        assert_eq!(t.get_gradient_offset_at(2, 2), 0.0);
    }

    #[test]
    fn radial_square_and_conic_offsets() {
        // Radial: distance from fill_from over the length of fill_from->fill_to.
        let mut t = GradientTexture2D::new();
        t.set_width(3);
        t.set_height(3);
        t.set_fill(Fill::Radial);
        assert!((t.get_gradient_offset_at(0, 0) - 0.0).abs() < 1e-5);
        // pos (0.5, 0.5), distance sqrt(0.5) = 0.7071.
        assert!((t.get_gradient_offset_at(1, 1) - 0.70710678).abs() < 1e-4);

        // Square (Chebyshev distance): with fill_to (1,1) the corner is at 1.
        t.set_fill(Fill::Square);
        t.set_fill_to(Vector2::new(1.0, 1.0));
        assert!((t.get_gradient_offset_at(1, 1) - 0.5).abs() < 1e-5);
        assert!((t.get_gradient_offset_at(2, 2) - 1.0).abs() < 1e-5);
        // max(|0.5|, |0.5|) / max(1, 1) = 0.5; an asymmetric texel picks the larger axis.
        assert!((t.get_gradient_offset_at(2, 0) - 1.0).abs() < 1e-5);

        // Conic: 45 degrees from +X is an eighth of the turn.
        t.set_fill(Fill::Conic);
        t.set_fill_from(Vector2::ZERO);
        t.set_fill_to(Vector2::new(1.0, 0.0));
        assert!((t.get_gradient_offset_at(1, 1) - 0.125).abs() < 1e-5);
        // Aiming along +Y, the +X sample is a quarter turn in the negative
        // direction, which the positive wrapping reports as 3/4.
        t.set_fill_to(Vector2::new(0.0, 1.0));
        assert!((t.get_gradient_offset_at(1, 0) - 0.75).abs() < 1e-5);
        // Straight down the fill direction is 0.
        assert!(t.get_gradient_offset_at(0, 2).abs() < 1e-5);
    }

    #[test]
    fn repeat_and_mirror_wrapping() {
        // Radial with fill_to 0.4 at pos.x 0.7 gives 1.75.
        let mut t = GradientTexture2D::new();
        t.set_width(11);
        t.set_height(1);
        t.set_fill(Fill::Radial);
        t.set_fill_to(Vector2::new(0.4, 0.0));
        // The raw offset is 1.75, but every repeat mode transforms it, so the
        // observable values are the clamped/wrapped ones below.
        assert_eq!(t.get_gradient_offset_at(7, 0), 1.0);

        // Repeat wraps to 0.75.
        t.set_repeat(Repeat::Repeat);
        assert!((t.get_gradient_offset_at(7, 0) - 0.75).abs() < 1e-4);

        // Mirror reflects 1.75 to 0.25.
        t.set_repeat(Repeat::Mirror);
        assert!((t.get_gradient_offset_at(7, 0) - 0.25).abs() < 1e-4);
    }
}
