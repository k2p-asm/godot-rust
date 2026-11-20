//! `scene/resources/placeholder_textures.*`: placeholder texture state.
//!
//! Defaults and accessors only: `PlaceholderTexture2D` size `(1,1)` (width and
//! height derive from it, no alpha, no image), `PlaceholderTexture3D` size
//! `(1,1,1)` with `FORMAT_RGB8`, and the layered base (size `(1,1)`, one layer)
//! with the three fixed layered types.
//!
//! RIDs, `emit_changed` and rendering-server creation are not ported.

use crate::math::vector2i::Vector2i;
use crate::math::vector3i::Vector3i;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum LayeredType {
    Layered2DArray,
    Cubemap,
    CubemapArray,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ImageFormat {
    Rgb8,
}

#[derive(Clone, Debug)]
pub struct PlaceholderTexture2DData {
    size: Vector2i,
}

impl Default for PlaceholderTexture2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaceholderTexture2DData {
    pub fn new() -> Self {
        Self {
            size: Vector2i::new(1, 1),
        }
    }

    pub fn set_size(&mut self, size: Vector2i) {
        self.size = size;
    }

    pub fn get_size(&self) -> Vector2i {
        self.size
    }

    pub fn get_width(&self) -> i32 {
        self.size.x
    }

    pub fn get_height(&self) -> i32 {
        self.size.y
    }

    pub fn has_alpha(&self) -> bool {
        false
    }

    pub fn has_image(&self) -> bool {
        false
    }
}

#[derive(Clone, Debug)]
pub struct PlaceholderTexture3DData {
    size: Vector3i,
}

impl Default for PlaceholderTexture3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl PlaceholderTexture3DData {
    pub fn new() -> Self {
        Self {
            size: Vector3i::new(1, 1, 1),
        }
    }

    pub fn set_size(&mut self, size: Vector3i) {
        self.size = size;
    }

    pub fn get_size(&self) -> Vector3i {
        self.size
    }

    pub fn get_format(&self) -> ImageFormat {
        ImageFormat::Rgb8
    }

    pub fn get_width(&self) -> i32 {
        self.size.x
    }

    pub fn get_height(&self) -> i32 {
        self.size.y
    }

    pub fn get_depth(&self) -> i32 {
        self.size.z
    }
}

#[derive(Clone, Debug)]
pub struct PlaceholderTextureLayeredData {
    size: Vector2i,
    layers: i32,
    layered_type: LayeredType,
}

impl Default for PlaceholderTextureLayeredData {
    fn default() -> Self {
        Self::new(LayeredType::Layered2DArray)
    }
}

impl PlaceholderTextureLayeredData {
    pub fn new(layered_type: LayeredType) -> Self {
        Self {
            size: Vector2i::new(1, 1),
            layers: 1,
            layered_type,
        }
    }

    pub fn set_size(&mut self, size: Vector2i) {
        self.size = size;
    }

    pub fn get_size(&self) -> Vector2i {
        self.size
    }

    pub fn set_layers(&mut self, layers: i32) {
        self.layers = layers;
    }

    pub fn get_layers(&self) -> i32 {
        self.layers
    }

    pub fn get_layered_type(&self) -> LayeredType {
        self.layered_type
    }

    pub fn get_width(&self) -> i32 {
        self.size.x
    }

    pub fn get_height(&self) -> i32 {
        self.size.y
    }
}

pub fn placeholder_texture_2d_array() -> PlaceholderTextureLayeredData {
    PlaceholderTextureLayeredData::new(LayeredType::Layered2DArray)
}

pub fn placeholder_cubemap() -> PlaceholderTextureLayeredData {
    PlaceholderTextureLayeredData::new(LayeredType::Cubemap)
}

pub fn placeholder_cubemap_array() -> PlaceholderTextureLayeredData {
    PlaceholderTextureLayeredData::new(LayeredType::CubemapArray)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn texture_2d_defaults() {
        let mut texture = PlaceholderTexture2DData::new();
        assert_eq!(texture.get_size(), Vector2i::new(1, 1));
        assert_eq!(texture.get_width(), 1);
        assert_eq!(texture.get_height(), 1);
        assert!(!texture.has_alpha());
        assert!(!texture.has_image());

        texture.set_size(Vector2i::new(64, 32));
        assert_eq!(texture.get_size(), Vector2i::new(64, 32));
        assert_eq!(texture.get_width(), 64);
        assert_eq!(texture.get_height(), 32);
    }

    #[test]
    fn texture_3d_defaults() {
        let mut texture = PlaceholderTexture3DData::new();
        assert_eq!(texture.get_size(), Vector3i::new(1, 1, 1));
        assert_eq!(texture.get_format(), ImageFormat::Rgb8);
        assert_eq!(texture.get_width(), 1);
        assert_eq!(texture.get_height(), 1);
        assert_eq!(texture.get_depth(), 1);

        texture.set_size(Vector3i::new(4, 8, 16));
        assert_eq!(texture.get_width(), 4);
        assert_eq!(texture.get_height(), 8);
        assert_eq!(texture.get_depth(), 16);
    }

    #[test]
    fn layered_variants() {
        let mut array = placeholder_texture_2d_array();
        assert_eq!(array.get_layered_type(), LayeredType::Layered2DArray);
        assert_eq!(array.get_size(), Vector2i::new(1, 1));
        assert_eq!(array.get_layers(), 1);
        array.set_size(Vector2i::new(32, 32));
        array.set_layers(6);
        assert_eq!(array.get_size(), Vector2i::new(32, 32));
        assert_eq!(array.get_layers(), 6);
        assert_eq!(array.get_width(), 32);
        assert_eq!(array.get_height(), 32);

        assert_eq!(
            placeholder_cubemap().get_layered_type(),
            LayeredType::Cubemap
        );
        assert_eq!(
            placeholder_cubemap_array().get_layered_type(),
            LayeredType::CubemapArray
        );
        assert_eq!(LayeredType::Layered2DArray as i32, 0);
        assert_eq!(LayeredType::Cubemap as i32, 1);
        assert_eq!(LayeredType::CubemapArray as i32, 2);
    }
}
