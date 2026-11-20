//! Small node/resource states: `MissingNode`, `LightmapProbe`, `TextFile`,
//! `ExternalTexture`, `CameraTexture` and `Texture2DRD`.
//!
//! State and accessors only:
//!
//! - `MissingNode`: original class/scene, the property/signal recording flags and
//!   the captured property map;
//! - `LightmapProbe`: an empty `Node3D` marker;
//! - `TextFile`: the text, the file path and `has_text` (`load_text` needs the
//!   pending resource-path layer, so it records the path and reports false);
//! - `ExternalTexture`: 256×256 default size, the external buffer id, the
//!   placeholder flag and the texture RID;
//! - `CameraTexture`: feed id (0) and feed image (RGBA) with the RID;
//! - `Texture2DRD`: the RD texture RID and the reported size.

use crate::math::vector2::Vector2;
use crate::math::vector2i::Vector2i;
use crate::servers::rid::Rid;
use crate::variant::Variant;

// -- MissingNode ------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct MissingNodeData {
    original_class: String,
    original_scene: String,
    recording_properties: bool,
    recording_signals: bool,
    properties: Vec<(String, Variant)>,
}

impl MissingNodeData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_original_class(&mut self, class: &str) {
        self.original_class = class.to_string();
    }

    pub fn get_original_class(&self) -> &str {
        &self.original_class
    }

    pub fn set_original_scene(&mut self, scene: &str) {
        self.original_scene = scene.to_string();
    }

    pub fn get_original_scene(&self) -> &str {
        &self.original_scene
    }

    pub fn set_recording_properties(&mut self, enabled: bool) {
        self.recording_properties = enabled;
    }

    pub fn is_recording_properties(&self) -> bool {
        self.recording_properties
    }

    pub fn set_recording_signals(&mut self, enabled: bool) {
        self.recording_signals = enabled;
    }

    pub fn is_recording_signals(&self) -> bool {
        self.recording_signals
    }

    /// `_set`: records the property (last write wins).
    pub fn set_property(&mut self, name: &str, value: Variant) {
        if let Some(entry) = self
            .properties
            .iter_mut()
            .find(|(known, _)| known == name)
        {
            entry.1 = value;
        } else {
            self.properties.push((name.to_string(), value));
        }
    }

    pub fn get_property(&self, name: &str) -> Option<&Variant> {
        self.properties
            .iter()
            .find(|(known, _)| known == name)
            .map(|(_, value)| value)
    }

    pub fn get_property_count(&self) -> usize {
        self.properties.len()
    }
}

// -- LightmapProbe ----------------------------------------------------------

/// `LightmapProbe` has no state of its own.
#[derive(Clone, Debug, Default)]
pub struct LightmapProbeData;

impl LightmapProbeData {
    pub fn new() -> Self {
        Self
    }
}

// -- TextFile ---------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct TextFileData {
    text: String,
    path: String,
}

impl TextFileData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn has_text(&self) -> bool {
        !self.text.is_empty()
    }

    pub fn get_text(&self) -> &str {
        &self.text
    }

    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();
    }

    pub fn set_file_path(&mut self, path: &str) {
        self.path = path.to_string();
    }

    pub fn get_file_path(&self) -> &str {
        &self.path
    }

    /// `load_text`: the path is recorded; reading needs the pending resource
    /// path layer, so the call reports false.
    pub fn load_text(&mut self, path: &str) -> bool {
        self.path = path.to_string();
        false
    }
}

// -- ExternalTexture --------------------------------------------------------

#[derive(Clone, Debug)]
pub struct ExternalTextureData {
    size: Vector2,
    external_buffer: u64,
    using_placeholder: bool,
    texture: Option<Rid>,
}

impl Default for ExternalTextureData {
    fn default() -> Self {
        Self::new()
    }
}

impl ExternalTextureData {
    pub fn new() -> Self {
        Self {
            size: Vector2::new(256.0, 256.0),
            external_buffer: 0,
            using_placeholder: false,
            texture: None,
        }
    }

    pub fn get_size(&self) -> Vector2 {
        self.size
    }

    pub fn set_size(&mut self, size: Vector2) {
        self.size = size;
    }

    pub fn get_width(&self) -> i32 {
        self.size.x as i32
    }

    pub fn get_height(&self) -> i32 {
        self.size.y as i32
    }

    pub fn set_external_buffer_id(&mut self, buffer: u64) {
        self.external_buffer = buffer;
    }

    /// The external id this port exposes is the stored buffer id.
    pub fn get_external_texture_id(&self) -> u64 {
        self.external_buffer
    }

    pub fn set_texture(&mut self, texture: Option<Rid>) {
        self.texture = texture;
    }

    pub fn get_texture(&self) -> Option<Rid> {
        self.texture
    }

    pub fn is_using_placeholder(&self) -> bool {
        self.using_placeholder
    }

    pub fn set_using_placeholder(&mut self, using: bool) {
        self.using_placeholder = using;
    }

    pub fn has_alpha(&self) -> bool {
        false
    }
}

// -- CameraTexture ----------------------------------------------------------

/// `CameraServer::FeedImage` values used by `CameraTexture`.
pub const FEED_RGBA_IMAGE: i32 = 0;
pub const FEED_YCBCR_IMAGE: i32 = 1;
pub const FEED_Y_IMAGE: i32 = 2;
pub const FEED_CBCR_IMAGE: i32 = 3;

#[derive(Clone, Debug)]
pub struct CameraTextureData {
    camera_feed_id: i32,
    which_feed: i32,
    texture: Option<Rid>,
}

impl Default for CameraTextureData {
    fn default() -> Self {
        Self::new()
    }
}

impl CameraTextureData {
    pub fn new() -> Self {
        Self {
            camera_feed_id: 0,
            which_feed: FEED_RGBA_IMAGE,
            texture: None,
        }
    }

    pub fn set_camera_feed_id(&mut self, id: i32) {
        self.camera_feed_id = id;
    }

    pub fn get_camera_feed_id(&self) -> i32 {
        self.camera_feed_id
    }

    pub fn set_which_feed(&mut self, feed: i32) {
        self.which_feed = feed;
    }

    pub fn get_which_feed(&self) -> i32 {
        self.which_feed
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

// -- Texture2DRD ------------------------------------------------------------

#[derive(Clone, Debug, Default)]
pub struct Texture2DRDData {
    texture_rid: Option<Rid>,
    texture_rd_rid: Option<Rid>,
    size: Vector2i,
}

impl Texture2DRDData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_texture_rd_rid(&mut self, rid: Option<Rid>) {
        self.texture_rd_rid = rid;
    }

    pub fn get_texture_rd_rid(&self) -> Option<Rid> {
        self.texture_rd_rid
    }

    pub fn set_texture(&mut self, rid: Option<Rid>) {
        self.texture_rid = rid;
    }

    pub fn get_texture(&self) -> Option<Rid> {
        self.texture_rid
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
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string::gstring::GoString;

    #[test]
    fn missing_node_state() {
        let mut node = MissingNodeData::new();
        assert_eq!(node.get_original_class(), "");
        assert_eq!(node.get_original_scene(), "");
        assert!(!node.is_recording_properties());
        assert!(!node.is_recording_signals());
        assert_eq!(node.get_property_count(), 0);

        node.set_original_class("Sprite2D");
        node.set_original_scene("res://scene.tscn");
        node.set_recording_properties(true);
        node.set_recording_signals(true);
        node.set_property("position", Variant::Vector2(Vector2::new(1.0, 2.0)));
        node.set_property("visible", Variant::Bool(false));
        node.set_property("position", Variant::Vector2(Vector2::new(3.0, 4.0)));

        assert_eq!(node.get_original_class(), "Sprite2D");
        assert_eq!(node.get_original_scene(), "res://scene.tscn");
        assert!(node.is_recording_properties());
        assert!(node.is_recording_signals());
        assert_eq!(node.get_property_count(), 2);
        assert_eq!(
            node.get_property("position"),
            Some(&Variant::Vector2(Vector2::new(3.0, 4.0)))
        );
        assert!(node.get_property("missing").is_none());
    }

    #[test]
    fn text_file_state() {
        let mut file = TextFileData::new();
        assert!(!file.has_text());
        assert_eq!(file.get_text(), "");
        assert_eq!(file.get_file_path(), "");

        file.set_text("hello");
        assert!(file.has_text());
        assert_eq!(file.get_text(), "hello");

        assert!(!file.load_text("res://notes.txt"));
        assert_eq!(file.get_file_path(), "res://notes.txt");
        file.set_file_path("res://other.txt");
        assert_eq!(file.get_file_path(), "res://other.txt");
        // Loading does not clear the text.
        file.set_text("");
        assert!(!file.has_text());
    }

    #[test]
    fn external_texture_state() {
        let mut texture = ExternalTextureData::new();
        assert_eq!(texture.get_size(), Vector2::new(256.0, 256.0));
        assert_eq!(texture.get_width(), 256);
        assert_eq!(texture.get_height(), 256);
        assert_eq!(texture.get_external_texture_id(), 0);
        assert!(!texture.is_using_placeholder());
        assert!(!texture.has_alpha());

        texture.set_size(Vector2::new(64.0, 32.0));
        texture.set_external_buffer_id(42);
        texture.set_using_placeholder(true);
        texture.set_texture(Some(Rid::from_u64(5)));
        assert_eq!(texture.get_width(), 64);
        assert_eq!(texture.get_height(), 32);
        assert_eq!(texture.get_external_texture_id(), 42);
        assert!(texture.is_using_placeholder());
        assert_eq!(texture.get_texture(), Some(Rid::from_u64(5)));
    }

    #[test]
    fn camera_and_rd_textures() {
        let mut camera = CameraTextureData::new();
        assert_eq!(camera.get_camera_feed_id(), 0);
        assert_eq!(camera.get_which_feed(), FEED_RGBA_IMAGE);
        camera.set_camera_feed_id(3);
        camera.set_which_feed(FEED_Y_IMAGE);
        assert_eq!(camera.get_camera_feed_id(), 3);
        assert_eq!(camera.get_which_feed(), FEED_Y_IMAGE);
        camera.set_texture(Some(Rid::from_u64(1)));
        assert_eq!(camera.get_texture(), Some(Rid::from_u64(1)));
        assert_eq!(FEED_YCBCR_IMAGE, 1);
        assert_eq!(FEED_CBCR_IMAGE, 3);

        let mut rd = Texture2DRDData::new();
        assert_eq!(rd.get_size(), Vector2i::ZERO);
        assert_eq!(rd.get_width(), 0);
        rd.set_size(Vector2i::new(128, 64));
        rd.set_texture_rd_rid(Some(Rid::from_u64(9)));
        rd.set_texture(Some(Rid::from_u64(8)));
        assert_eq!(rd.get_width(), 128);
        assert_eq!(rd.get_height(), 64);
        assert_eq!(rd.get_texture_rd_rid(), Some(Rid::from_u64(9)));
        assert_eq!(rd.get_texture(), Some(Rid::from_u64(8)));
        assert!(!rd.has_alpha());
    }

    #[test]
    fn lightmap_probe_and_text_round_trip() {
        let probe = LightmapProbeData::new();
        let _ = probe;

        let mut file = TextFileData::new();
        file.set_text(&format!("line {}", 1));
        assert_eq!(file.get_text(), "line 1");
        file.set_text("");
        file.set_text("áéí");
        assert_eq!(file.get_text(), "áéí");
        let _ = GoString::from_str("x");
    }
}
