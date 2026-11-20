//! `scene/resources/placeholder_textures.*` (3D and layered): size state.
//!
//! `PlaceholderTexture3D` defaults to `(1, 1, 1)`; `PlaceholderTextureLayered`
//! defaults to `1x1` with one layer and the layered type fixed by the concrete
//! subclass (`2D array`, `cubemap`, `cubemap array`). The setters assign without
//! validation, like the C++. Image data/RIDs are not ported.

use crate::math::vector2i::Vector2i;
use crate::math::vector3i::Vector3i;

/// `TextureLayered::LayeredType`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum LayeredType {
    Layered2DArray,
    Cubemap,
    CubemapArray,
}

#[derive(Clone, Copy, Debug)]
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

    pub fn get_width(&self) -> i32 {
        self.size.x
    }

    pub fn get_height(&self) -> i32 {
        self.size.y
    }

    pub fn get_depth(&self) -> i32 {
        self.size.z
    }

    pub fn has_mipmaps(&self) -> bool {
        false
    }
}

#[derive(Clone, Copy, Debug)]
pub struct PlaceholderTextureLayeredData {
    size: Vector2i,
    layers: i32,
    layered_type: LayeredType,
}

impl PlaceholderTextureLayeredData {
    /// The concrete subclass fixes the layered type in its constructor.
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

    pub fn has_mipmaps(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_3d_defaults_and_size() {
        let mut p = PlaceholderTexture3DData::new();
        assert_eq!(p.get_size(), Vector3i::new(1, 1, 1));
        assert_eq!(p.get_width(), 1);
        assert_eq!(p.get_height(), 1);
        assert_eq!(p.get_depth(), 1);
        assert!(!p.has_mipmaps());

        p.set_size(Vector3i::new(4, 5, 6));
        assert_eq!(p.get_size(), Vector3i::new(4, 5, 6));
        assert_eq!((p.get_width(), p.get_height(), p.get_depth()), (4, 5, 6));

        // No validation.
        p.set_size(Vector3i::new(0, -1, 2));
        assert_eq!(p.get_size(), Vector3i::new(0, -1, 2));
    }

    #[test]
    fn placeholder_layered_defaults_and_type() {
        let mut l = PlaceholderTextureLayeredData::new(LayeredType::CubemapArray);
        assert_eq!(l.get_size(), Vector2i::new(1, 1));
        assert_eq!(l.get_layers(), 1);
        assert_eq!(l.get_layered_type(), LayeredType::CubemapArray);
        assert!(!l.has_mipmaps());

        l.set_size(Vector2i::new(8, 16));
        l.set_layers(6);
        assert_eq!(l.get_width(), 8);
        assert_eq!(l.get_height(), 16);
        assert_eq!(l.get_layers(), 6);

        assert_eq!(LayeredType::Layered2DArray as i32, 0);
        assert_eq!(LayeredType::Cubemap as i32, 1);
        assert_eq!(LayeredType::CubemapArray as i32, 2);
    }
}
