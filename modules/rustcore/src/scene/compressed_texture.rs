//! `scene/resources/compressed_texture.*` (state): compressed texture resources.
//!
//! Ported state: the `DataFormat` enum, the `FORMAT_VERSION` and `FormatBits`
//! constants, the load path, the dimensions, the format (default
//! `FORMAT_L8`-equivalent 0), the mipmap flag for layered textures, the layer
//! count and layered type, and the texture RID slots.
//!
//! Loading requires `Image` (and its codecs), which are pending, so `load` stores
//! the path and reports `false`; callers can feed the decoded size through
//! [`CompressedTexture2DData::set_size`] (a port helper).

use crate::servers::rid::Rid;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DataFormat {
    Image = 0,
    Png = 1,
    Webp = 2,
    BasisUniversal = 3,
}

pub const FORMAT_VERSION: u32 = 1;

pub const FORMAT_BIT_STREAM: u32 = 1 << 22;
pub const FORMAT_BIT_HAS_MIPMAPS: u32 = 1 << 23;
pub const FORMAT_BIT_DETECT_3D: u32 = 1 << 24;
pub const FORMAT_BIT_DETECT_NORMAL: u32 = 1 << 26;
pub const FORMAT_BIT_DETECT_ROUGNESS: u32 = 1 << 27;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum LayeredType {
    Layered2DArray = 0,
    Cubemap = 1,
    CubemapArray = 2,
}

/// `CompressedTexture2D`.
#[derive(Clone, Debug, Default)]
pub struct CompressedTexture2DData {
    path_to_file: String,
    format: i32,
    width: i32,
    height: i32,
    texture: Option<Rid>,
}

impl CompressedTexture2DData {
    pub fn new() -> Self {
        Self::default()
    }

    /// `load`: the path is remembered; decoding needs the pending `Image` port.
    pub fn load(&mut self, path: &str) -> bool {
        self.path_to_file = path.to_string();
        false
    }

    pub fn get_load_path(&self) -> &str {
        &self.path_to_file
    }

    pub fn set_path(&mut self, path: &str) {
        self.path_to_file = path.to_string();
    }

    /// Port helper: the decoded size (normally set by `_load_data`).
    pub fn set_size(&mut self, width: i32, height: i32, format: i32) {
        self.width = width;
        self.height = height;
        self.format = format;
    }

    pub fn get_width(&self) -> i32 {
        self.width
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }

    pub fn get_format(&self) -> i32 {
        self.format
    }

    pub fn set_texture(&mut self, texture: Option<Rid>) {
        self.texture = texture;
    }

    pub fn get_texture(&self) -> Option<Rid> {
        self.texture
    }

    pub fn has_alpha(&self) -> bool {
        false
    }
}

/// `CompressedTextureLayered` (2D array / cubemap / cubemap array).
#[derive(Clone, Debug)]
pub struct CompressedTextureLayeredData {
    path_to_file: String,
    format: i32,
    width: i32,
    height: i32,
    layers: i32,
    mipmaps: bool,
    layered_type: LayeredType,
    texture: Option<Rid>,
}

impl Default for CompressedTextureLayeredData {
    fn default() -> Self {
        Self::new(LayeredType::Layered2DArray)
    }
}

impl CompressedTextureLayeredData {
    pub fn new(layered_type: LayeredType) -> Self {
        Self {
            path_to_file: String::new(),
            format: 0,
            width: 0,
            height: 0,
            layers: 0,
            mipmaps: false,
            layered_type,
            texture: None,
        }
    }

    pub fn load(&mut self, path: &str) -> bool {
        self.path_to_file = path.to_string();
        false
    }

    pub fn get_load_path(&self) -> &str {
        &self.path_to_file
    }

    pub fn set_path(&mut self, path: &str) {
        self.path_to_file = path.to_string();
    }

    /// Port helper for the decoded texture description.
    pub fn set_size(&mut self, width: i32, height: i32, layers: i32, mipmaps: bool) {
        self.width = width;
        self.height = height;
        self.layers = layers;
        self.mipmaps = mipmaps;
    }

    pub fn get_format(&self) -> i32 {
        self.format
    }

    pub fn get_width(&self) -> i32 {
        self.width
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }

    pub fn get_layers(&self) -> i32 {
        self.layers
    }

    pub fn has_mipmaps(&self) -> bool {
        self.mipmaps
    }

    pub fn get_layered_type(&self) -> LayeredType {
        self.layered_type
    }

    pub fn set_texture(&mut self, texture: Option<Rid>) {
        self.texture = texture;
    }

    pub fn get_texture(&self) -> Option<Rid> {
        self.texture
    }
}

/// `CompressedTexture3D`.
#[derive(Clone, Debug, Default)]
pub struct CompressedTexture3DData {
    path_to_file: String,
    format: i32,
    width: i32,
    height: i32,
    depth: i32,
    texture: Option<Rid>,
}

impl CompressedTexture3DData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn load(&mut self, path: &str) -> bool {
        self.path_to_file = path.to_string();
        false
    }

    pub fn get_load_path(&self) -> &str {
        &self.path_to_file
    }

    pub fn set_path(&mut self, path: &str) {
        self.path_to_file = path.to_string();
    }

    pub fn set_size(&mut self, width: i32, height: i32, depth: i32) {
        self.width = width;
        self.height = height;
        self.depth = depth;
    }

    pub fn get_width(&self) -> i32 {
        self.width
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }

    pub fn get_depth(&self) -> i32 {
        self.depth
    }

    pub fn get_format(&self) -> i32 {
        self.format
    }

    pub fn set_texture(&mut self, texture: Option<Rid>) {
        self.texture = texture;
    }

    pub fn get_texture(&self) -> Option<Rid> {
        self.texture
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enums_and_format_bits() {
        assert_eq!(DataFormat::Image as i32, 0);
        assert_eq!(DataFormat::Png as i32, 1);
        assert_eq!(DataFormat::Webp as i32, 2);
        assert_eq!(DataFormat::BasisUniversal as i32, 3);
        assert_eq!(FORMAT_VERSION, 1);
        assert_eq!(FORMAT_BIT_STREAM, 1 << 22);
        assert_eq!(FORMAT_BIT_HAS_MIPMAPS, 1 << 23);
        assert_eq!(FORMAT_BIT_DETECT_3D, 1 << 24);
        assert_eq!(FORMAT_BIT_DETECT_NORMAL, 1 << 26);
        assert_eq!(FORMAT_BIT_DETECT_ROUGNESS, 1 << 27);
        assert_eq!(LayeredType::Layered2DArray as i32, 0);
        assert_eq!(LayeredType::Cubemap as i32, 1);
        assert_eq!(LayeredType::CubemapArray as i32, 2);
    }

    #[test]
    fn texture_2d_state() {
        let mut texture = CompressedTexture2DData::new();
        assert_eq!(texture.get_load_path(), "");
        assert_eq!(texture.get_width(), 0);
        assert_eq!(texture.get_height(), 0);
        assert_eq!(texture.get_format(), 0);
        assert!(texture.get_texture().is_none());
        assert!(!texture.has_alpha());

        // `load` remembers the path but needs the pending Image port.
        assert!(!texture.load("res://icon.ctex"));
        assert_eq!(texture.get_load_path(), "res://icon.ctex");
        texture.set_path("res://other.ctex");
        assert_eq!(texture.get_load_path(), "res://other.ctex");

        texture.set_size(64, 32, 5);
        assert_eq!(texture.get_width(), 64);
        assert_eq!(texture.get_height(), 32);
        assert_eq!(texture.get_format(), 5);
        texture.set_texture(Some(Rid::from_u64(7)));
        assert_eq!(texture.get_texture(), Some(Rid::from_u64(7)));
    }

    #[test]
    fn layered_state() {
        let mut texture = CompressedTextureLayeredData::new(LayeredType::Cubemap);
        assert_eq!(texture.get_layered_type(), LayeredType::Cubemap);
        assert_eq!(texture.get_width(), 0);
        assert_eq!(texture.get_layers(), 0);
        assert!(!texture.has_mipmaps());

        assert!(!texture.load("res://sky.ctex"));
        assert_eq!(texture.get_load_path(), "res://sky.ctex");
        texture.set_size(256, 256, 6, true);
        assert_eq!(texture.get_width(), 256);
        assert_eq!(texture.get_height(), 256);
        assert_eq!(texture.get_layers(), 6);
        assert!(texture.has_mipmaps());
        texture.set_texture(Some(Rid::from_u64(9)));
        assert_eq!(texture.get_texture(), Some(Rid::from_u64(9)));
    }

    #[test]
    fn texture_3d_state() {
        let mut texture = CompressedTexture3DData::new();
        assert_eq!(texture.get_load_path(), "");
        assert_eq!(texture.get_depth(), 0);
        assert!(!texture.load("res://volume.ctex"));
        assert_eq!(texture.get_load_path(), "res://volume.ctex");
        texture.set_size(16, 8, 4);
        assert_eq!(texture.get_width(), 16);
        assert_eq!(texture.get_height(), 8);
        assert_eq!(texture.get_depth(), 4);
        texture.set_texture(Some(Rid::from_u64(3)));
        assert_eq!(texture.get_texture(), Some(Rid::from_u64(3)));
    }
}
