//! `scene/resources/curve_texture.*`: the curve texture resources' state.
//!
//! `CurveTexture` defaults: width 256, `TEXTURE_MODE_RGB`, no curve, height 1 and
//! no alpha. `CurveXYZTexture` keeps three curves (X/Y/Z) and the same width
//! default. Widths are accepted in the engine's `32..=4096` range.
//!
//! The texture baking (`_update` sampling the curve into an image and creating the
//! RID), `get_rid` and `ensure_default_setup` are not ported.

use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum TextureMode {
    Rgb = 0,
    Red = 1,
}

#[derive(Clone, Debug)]
pub struct CurveTextureData {
    curve: Option<Rid>,
    width: i32,
    texture_mode: TextureMode,
    /// The width/mode of the last upload (the engine's cache markers).
    uploaded_width: i32,
    uploaded_mode: TextureMode,
}

impl Default for CurveTextureData {
    fn default() -> Self {
        Self::new()
    }
}

impl CurveTextureData {
    pub fn new() -> Self {
        Self {
            curve: None,
            width: 256,
            texture_mode: TextureMode::Rgb,
            uploaded_width: 0,
            uploaded_mode: TextureMode::Rgb,
        }
    }

    /// `set_width`: `ERR_FAIL_COND(p_width < 32 || p_width > 4096)`.
    pub fn set_width(&mut self, width: i32) -> bool {
        if !(32..=4096).contains(&width) {
            return false;
        }
        if self.width == width {
            return true;
        }
        self.width = width;
        true
    }

    pub fn get_width(&self) -> i32 {
        self.width
    }

    pub fn set_texture_mode(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => TextureMode::Rgb,
            1 => TextureMode::Red,
            _ => return false,
        };
        self.texture_mode = parsed;
        true
    }

    pub fn get_texture_mode(&self) -> TextureMode {
        self.texture_mode
    }

    pub fn set_curve(&mut self, curve: Option<Rid>) {
        self.curve = curve;
    }

    pub fn get_curve(&self) -> Option<Rid> {
        self.curve
    }

    pub fn get_height(&self) -> i32 {
        1
    }

    pub fn has_alpha(&self) -> bool {
        false
    }

    pub fn get_uploaded_width(&self) -> i32 {
        self.uploaded_width
    }

    pub fn get_uploaded_mode(&self) -> TextureMode {
        self.uploaded_mode
    }
}

#[derive(Clone, Debug)]
pub struct CurveXyzTextureData {
    curve_x: Option<Rid>,
    curve_y: Option<Rid>,
    curve_z: Option<Rid>,
    width: i32,
    uploaded_width: i32,
}

impl Default for CurveXyzTextureData {
    fn default() -> Self {
        Self::new()
    }
}

impl CurveXyzTextureData {
    pub fn new() -> Self {
        Self {
            curve_x: None,
            curve_y: None,
            curve_z: None,
            width: 256,
            uploaded_width: 0,
        }
    }

    pub fn set_width(&mut self, width: i32) -> bool {
        if !(32..=4096).contains(&width) {
            return false;
        }
        self.width = width;
        true
    }

    pub fn get_width(&self) -> i32 {
        self.width
    }

    pub fn set_curve_x(&mut self, curve: Option<Rid>) {
        self.curve_x = curve;
    }

    pub fn get_curve_x(&self) -> Option<Rid> {
        self.curve_x
    }

    pub fn set_curve_y(&mut self, curve: Option<Rid>) {
        self.curve_y = curve;
    }

    pub fn get_curve_y(&self) -> Option<Rid> {
        self.curve_y
    }

    pub fn set_curve_z(&mut self, curve: Option<Rid>) {
        self.curve_z = curve;
    }

    pub fn get_curve_z(&self) -> Option<Rid> {
        self.curve_z
    }

    pub fn get_height(&self) -> i32 {
        1
    }

    pub fn has_alpha(&self) -> bool {
        false
    }

    pub fn get_uploaded_width(&self) -> i32 {
        self.uploaded_width
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn curve_texture_defaults_and_guards() {
        let mut texture = CurveTextureData::new();
        assert!(texture.get_curve().is_none());
        assert_eq!(texture.get_width(), 256);
        assert_eq!(texture.get_texture_mode(), TextureMode::Rgb);
        assert_eq!(texture.get_height(), 1);
        assert!(!texture.has_alpha());
        assert_eq!(texture.get_uploaded_width(), 0);

        assert!(!texture.set_width(31));
        assert!(!texture.set_width(4097));
        assert!(texture.set_width(32));
        assert!(texture.set_width(4096));
        assert_eq!(texture.get_width(), 4096);

        assert!(texture.set_texture_mode(1));
        assert!(!texture.set_texture_mode(2));
        assert_eq!(texture.get_texture_mode(), TextureMode::Red);
        texture.set_texture_mode(0);
        assert_eq!(texture.get_texture_mode(), TextureMode::Rgb);

        texture.set_curve(Some(Rid::from_u64(3)));
        assert_eq!(texture.get_curve(), Some(Rid::from_u64(3)));
        texture.set_curve(None);
        assert!(texture.get_curve().is_none());
        assert_eq!(TextureMode::Rgb as i32, 0);
        assert_eq!(TextureMode::Red as i32, 1);
    }

    #[test]
    fn curve_xyz_texture_state() {
        let mut texture = CurveXyzTextureData::new();
        assert!(texture.get_curve_x().is_none());
        assert!(texture.get_curve_y().is_none());
        assert!(texture.get_curve_z().is_none());
        assert_eq!(texture.get_width(), 256);
        assert_eq!(texture.get_height(), 1);
        assert!(!texture.has_alpha());

        assert!(!texture.set_width(16));
        assert!(texture.set_width(512));
        assert_eq!(texture.get_width(), 512);

        texture.set_curve_x(Some(Rid::from_u64(1)));
        texture.set_curve_y(Some(Rid::from_u64(2)));
        texture.set_curve_z(Some(Rid::from_u64(3)));
        assert_eq!(texture.get_curve_x(), Some(Rid::from_u64(1)));
        assert_eq!(texture.get_curve_y(), Some(Rid::from_u64(2)));
        assert_eq!(texture.get_curve_z(), Some(Rid::from_u64(3)));
        texture.set_curve_y(None);
        assert!(texture.get_curve_y().is_none());
    }
}
