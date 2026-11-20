//! `scene/resources/video_stream.*` and
//! `scene/resources/portable_compressed_texture.*` (state).
//!
//! `VideoStream` keeps the file path and the audio track (0). The
//! `PortableCompressedTexture2D` state mirrors the header defaults:
//! `COMPRESSION_MODE_LOSSLESS`, no kept buffer, zero size/size override, no
//! mipmaps, `FORMAT_L8`, nothing stored yet.
//!
//! Playback instantiation, image compression and the RID upload are not ported.

use crate::math::vector2i::Vector2i;
use crate::servers::rid::Rid;

#[derive(Clone, Debug)]
pub struct VideoStreamData {
    file: String,
    audio_track: i32,
}

impl Default for VideoStreamData {
    fn default() -> Self {
        Self::new()
    }
}

impl VideoStreamData {
    pub fn new() -> Self {
        Self {
            file: String::new(),
            audio_track: 0,
        }
    }

    pub fn set_file(&mut self, file: &str) {
        self.file = file.to_string();
    }

    pub fn get_file(&self) -> &str {
        &self.file
    }

    pub fn set_audio_track(&mut self, track: i32) {
        self.audio_track = track;
    }

    pub fn get_audio_track(&self) -> i32 {
        self.audio_track
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum DataFormat {
    Undefined = 0,
    Image = 1,
    Png = 2,
    Webp = 3,
    BasisUniversal = 4,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum CompressionMode {
    Lossless = 0,
    Lossy = 1,
    BasisUniversal = 2,
    S3tc = 3,
    Etc2 = 4,
    Bptc = 5,
    Astc = 6,
}

#[derive(Clone, Debug)]
pub struct PortableCompressedTexture2DData {
    compression_mode: CompressionMode,
    keep_compressed_buffer: bool,
    size: Vector2i,
    size_override: Vector2i,
    mipmaps: bool,
    format: i32,
    texture: Option<Rid>,
}

impl Default for PortableCompressedTexture2DData {
    fn default() -> Self {
        Self::new()
    }
}

impl PortableCompressedTexture2DData {
    pub fn new() -> Self {
        Self {
            compression_mode: CompressionMode::Lossless,
            keep_compressed_buffer: false,
            size: Vector2i::ZERO,
            size_override: Vector2i::ZERO,
            mipmaps: false,
            // Image::FORMAT_L8
            format: 0,
            texture: None,
        }
    }

    pub fn get_compression_mode(&self) -> CompressionMode {
        self.compression_mode
    }

    pub fn set_compression_mode(&mut self, mode: i32) -> bool {
        let parsed = match mode {
            0 => CompressionMode::Lossless,
            1 => CompressionMode::Lossy,
            2 => CompressionMode::BasisUniversal,
            3 => CompressionMode::S3tc,
            4 => CompressionMode::Etc2,
            5 => CompressionMode::Bptc,
            6 => CompressionMode::Astc,
            _ => return false,
        };
        self.compression_mode = parsed;
        true
    }

    pub fn set_keep_compressed_buffer(&mut self, keep: bool) {
        self.keep_compressed_buffer = keep;
    }

    pub fn is_keeping_compressed_buffer(&self) -> bool {
        self.keep_compressed_buffer
    }

    pub fn set_size_override(&mut self, size: Vector2i) {
        self.size_override = size;
    }

    pub fn get_size_override(&self) -> Vector2i {
        self.size_override
    }

    pub fn set_mipmaps_enabled(&mut self, enabled: bool) {
        self.mipmaps = enabled;
    }

    pub fn are_mipmaps_enabled(&self) -> bool {
        self.mipmaps
    }

    /// The stored image size (from `create_from_image`/`update` in the engine).
    pub fn set_size(&mut self, size: Vector2i) {
        self.size = size;
    }

    pub fn get_size(&self) -> Vector2i {
        self.size
    }

    pub fn get_format(&self) -> i32 {
        self.format
    }

    /// `get_width`/`get_height`: the override wins when set.
    pub fn get_width(&self) -> i32 {
        if self.size_override.x != 0 {
            self.size_override.x
        } else {
            self.size.x
        }
    }

    pub fn get_height(&self) -> i32 {
        if self.size_override.y != 0 {
            self.size_override.y
        } else {
            self.size.y
        }
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
    fn video_stream_state() {
        let mut stream = VideoStreamData::new();
        assert_eq!(stream.get_file(), "");
        assert_eq!(stream.get_audio_track(), 0);
        stream.set_file("res://video.ogv");
        stream.set_audio_track(2);
        assert_eq!(stream.get_file(), "res://video.ogv");
        assert_eq!(stream.get_audio_track(), 2);
    }

    #[test]
    fn portable_texture_state() {
        let mut texture = PortableCompressedTexture2DData::new();
        assert_eq!(
            texture.get_compression_mode(),
            CompressionMode::Lossless
        );
        assert!(!texture.is_keeping_compressed_buffer());
        assert_eq!(texture.get_size(), Vector2i::ZERO);
        assert_eq!(texture.get_size_override(), Vector2i::ZERO);
        assert!(!texture.are_mipmaps_enabled());
        assert_eq!(texture.get_format(), 0);
        assert!(texture.get_texture().is_none());
        assert_eq!(texture.get_width(), 0);
        assert_eq!(texture.get_height(), 0);

        assert!(texture.set_compression_mode(6));
        assert!(!texture.set_compression_mode(7));
        assert_eq!(texture.get_compression_mode(), CompressionMode::Astc);
        texture.set_keep_compressed_buffer(true);
        texture.set_size(Vector2i::new(64, 32));
        assert_eq!(texture.get_width(), 64);
        assert_eq!(texture.get_height(), 32);
        texture.set_size_override(Vector2i::new(16, 8));
        assert_eq!(texture.get_width(), 16);
        assert_eq!(texture.get_height(), 8);
        texture.set_mipmaps_enabled(true);
        assert!(texture.are_mipmaps_enabled());
        texture.set_texture(Some(Rid::from_u64(5)));
        assert_eq!(texture.get_texture(), Some(Rid::from_u64(5)));

        assert_eq!(DataFormat::Image as i32, 1);
        assert_eq!(DataFormat::BasisUniversal as i32, 4);
        assert_eq!(CompressionMode::S3tc as i32, 3);
        assert_eq!(CompressionMode::Bptc as i32, 5);
    }
}
