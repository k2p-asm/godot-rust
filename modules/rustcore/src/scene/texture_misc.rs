//! `scene/resources/curve_texture.*` and `placeholder_textures.*`.
//!
//! `CurveTexture` and `CurveXYZTexture` validate their width into `32..=4096`
//! and provide the engine's `ensure_default_setup`, which builds a flat `1.0`
//! curve when none is assigned. `PlaceholderTexture2D` stores a `Size2`
//! (default `1x1`) with no validation, exactly like the C++.
//!
//! The image/RID generation is not ported.

use crate::math::vector2::Vector2;
use crate::scene::curve::Curve;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum TextureMode {
    Rgb,
    Red,
}

#[derive(Debug)]
pub struct CurveTexture {
    curve: Option<Curve>,
    width: i32,
    texture_mode: TextureMode,
}

impl Default for CurveTexture {
    fn default() -> Self {
        Self::new()
    }
}

impl CurveTexture {
    pub fn new() -> Self {
        Self {
            curve: None,
            width: 256,
            texture_mode: TextureMode::Rgb,
        }
    }

    /// `ERR_FAIL_COND(p_width < 32 || p_width > 4096)`.
    pub fn set_width(&mut self, width: i32) -> bool {
        if width < 32 || width > 4096 {
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

    pub fn has_alpha(&self) -> bool {
        false
    }

    pub fn set_texture_mode(&mut self, mode: TextureMode) {
        self.texture_mode = mode;
    }

    pub fn get_texture_mode(&self) -> TextureMode {
        self.texture_mode
    }

    /// Creates a flat curve at `1.0` between `min` and `max` when none is set.
    pub fn ensure_default_setup(&mut self, min: f32, max: f32) {
        if self.curve.is_none() {
            let mut curve = Curve::new();
            curve.add_point(Vector2::new(0.0, 1.0));
            curve.add_point(Vector2::new(1.0, 1.0));
            curve.set_min_value(min);
            curve.set_max_value(max);
            self.curve = Some(curve);
        }
    }

    pub fn set_curve(&mut self, curve: Option<Curve>) {
        self.curve = curve;
    }

    pub fn get_curve(&self) -> Option<&Curve> {
        self.curve.as_ref()
    }
}

#[derive(Debug)]
pub struct CurveXYZTexture {
    curve_x: Option<Curve>,
    curve_y: Option<Curve>,
    curve_z: Option<Curve>,
    width: i32,
}

impl Default for CurveXYZTexture {
    fn default() -> Self {
        Self::new()
    }
}

impl CurveXYZTexture {
    pub fn new() -> Self {
        Self {
            curve_x: None,
            curve_y: None,
            curve_z: None,
            width: 256,
        }
    }

    /// Same `32..=4096` range as `CurveTexture`.
    pub fn set_width(&mut self, width: i32) -> bool {
        if width < 32 || width > 4096 {
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

    pub fn has_alpha(&self) -> bool {
        false
    }

    /// Fills each missing curve independently, leaving the assigned ones alone.
    pub fn ensure_default_setup(&mut self, min: f32, max: f32) {
        fn flat(min: f32, max: f32) -> Curve {
            let mut curve = Curve::new();
            curve.add_point(Vector2::new(0.0, 1.0));
            curve.add_point(Vector2::new(1.0, 1.0));
            curve.set_min_value(min);
            curve.set_max_value(max);
            curve
        }
        if self.curve_x.is_none() {
            self.curve_x = Some(flat(min, max));
        }
        if self.curve_y.is_none() {
            self.curve_y = Some(flat(min, max));
        }
        if self.curve_z.is_none() {
            self.curve_z = Some(flat(min, max));
        }
    }

    pub fn set_curve_x(&mut self, curve: Option<Curve>) {
        self.curve_x = curve;
    }

    pub fn get_curve_x(&self) -> Option<&Curve> {
        self.curve_x.as_ref()
    }

    pub fn set_curve_y(&mut self, curve: Option<Curve>) {
        self.curve_y = curve;
    }

    pub fn get_curve_y(&self) -> Option<&Curve> {
        self.curve_y.as_ref()
    }

    pub fn set_curve_z(&mut self, curve: Option<Curve>) {
        self.curve_z = curve;
    }

    pub fn get_curve_z(&self) -> Option<&Curve> {
        self.curve_z.as_ref()
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PlaceholderTexture2D {
    size: Vector2,
}

impl Default for PlaceholderTexture2D {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaceholderTexture2D {
    pub fn new() -> Self {
        Self {
            size: Vector2::new(1.0, 1.0),
        }
    }

    pub fn set_size(&mut self, size: Vector2) {
        self.size = size;
    }

    pub fn get_size(&self) -> Vector2 {
        self.size
    }

    pub fn get_width(&self) -> i32 {
        self.size.x as i32
    }

    pub fn get_height(&self) -> i32 {
        self.size.y as i32
    }

    pub fn has_alpha(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curve_texture_defaults_and_width_range() {
        let mut t = CurveTexture::new();
        assert_eq!(t.get_width(), 256);
        assert_eq!(t.get_height(), 1);
        assert!(!t.has_alpha());
        assert_eq!(t.get_texture_mode(), TextureMode::Rgb);
        assert!(t.get_curve().is_none());

        assert!(!t.set_width(31));
        assert!(!t.set_width(4097));
        assert_eq!(t.get_width(), 256);
        assert!(t.set_width(32));
        assert!(t.set_width(4096));
        assert_eq!(t.get_width(), 4096);

        t.set_texture_mode(TextureMode::Red);
        assert_eq!(t.get_texture_mode(), TextureMode::Red);
        assert_eq!(TextureMode::Rgb as i32, 0);
        assert_eq!(TextureMode::Red as i32, 1);
    }

    #[test]
    fn curve_texture_default_setup() {
        let mut t = CurveTexture::new();
        t.ensure_default_setup(0.0, 1.0);
        let curve = t.get_curve().expect("curve created");
        assert_eq!(curve.get_point_count(), 2);
        assert_eq!(curve.get_point(0).unwrap().position, Vector2::new(0.0, 1.0));
        assert_eq!(curve.get_point(1).unwrap().position, Vector2::new(1.0, 1.0));
        assert_eq!(curve.get_min_value(), 0.0);
        assert_eq!(curve.get_max_value(), 1.0);

        // A second call leaves the existing curve untouched.
        t.ensure_default_setup(5.0, 10.0);
        assert_eq!(t.get_curve().unwrap().get_point_count(), 2);
        assert_eq!(t.get_curve().unwrap().get_max_value(), 1.0);
    }

    #[test]
    fn curve_xyz_texture_defaults() {
        let mut t = CurveXYZTexture::new();
        assert_eq!(t.get_width(), 256);
        assert_eq!(t.get_height(), 1);
        assert!(!t.has_alpha());
        assert!(t.get_curve_x().is_none());
        assert!(!t.set_width(10));

        // An assigned curve survives the default setup.
        let mut custom = Curve::new();
        custom.add_point(Vector2::new(0.0, 0.5));
        t.set_curve_x(Some(custom));

        t.ensure_default_setup(-1.0, 2.0);
        assert_eq!(t.get_curve_x().unwrap().get_point_count(), 1);
        assert_eq!(t.get_curve_y().unwrap().get_point_count(), 2);
        assert_eq!(t.get_curve_z().unwrap().get_point_count(), 2);
        assert_eq!(t.get_curve_y().unwrap().get_min_value(), -1.0);
        assert_eq!(t.get_curve_z().unwrap().get_max_value(), 2.0);
    }

    #[test]
    fn placeholder_texture_2d() {
        let mut p = PlaceholderTexture2D::new();
        assert_eq!(p.get_size(), Vector2::new(1.0, 1.0));
        assert_eq!(p.get_width(), 1);
        assert_eq!(p.get_height(), 1);
        assert!(!p.has_alpha());

        // No validation, like the engine setter.
        p.set_size(Vector2::new(4.0, 7.0));
        assert_eq!(p.get_width(), 4);
        assert_eq!(p.get_height(), 7);
        p.set_size(Vector2::ZERO);
        assert_eq!(p.get_width(), 0);
    }
}
