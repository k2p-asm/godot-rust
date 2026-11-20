//! `core/io/resource_saver.*`: the format-saver registry (no object writing).
//!
//! `ResourceSaver` keeps a table of `ResourceFormatSaver`s (64 slots) and routes
//! saves, UID writes and extension queries through them. This port keeps the
//! registry and the routing with data-described savers:
//!
//! - the `SaverFlags` bit constants (`FLAG_RELATIVE_PATHS` … `FLAG_REPLACE_-
//!   SUBRESOURCE_PATHS`);
//! - `add_format_saver` (front or back), `remove_format_saver`, the extension
//!   query, `recognize_path` (explicit path or extension) and the max-saver
//!   guard;
//! - `save`/`set_uid` reporting [`SaveError::NotSupported`], because the object
//!   layer (and therefore `Resource`) is not ported;
//! - `set_timestamp_on_save`/`get_timestamp_on_save`.
//!
//! Saving callbacks, custom saver scripts and the singleton are not ported.

use std::collections::BTreeSet;

pub const FLAG_NONE: u32 = 0;
pub const FLAG_RELATIVE_PATHS: u32 = 1;
pub const FLAG_BUNDLE_RESOURCES: u32 = 2;
pub const FLAG_CHANGE_PATH: u32 = 4;
pub const FLAG_OMIT_EDITOR_PROPERTIES: u32 = 8;
pub const FLAG_SAVE_BIG_ENDIAN: u32 = 16;
pub const FLAG_COMPRESS: u32 = 32;
pub const FLAG_REPLACE_SUBRESOURCE_PATHS: u32 = 64;

pub const MAX_SAVERS: usize = 64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SaveError {
    /// `ERR_FILE_UNRECOGNIZED`: no saver handles the path.
    Unrecognized,
    /// `ERR_UNAVAILABLE`: the object layer is not available in this port.
    NotSupported,
}

/// The data a `ResourceFormatSaver` exposes.
#[derive(Clone, Debug, Default)]
pub struct FormatSaver {
    /// Extensions this saver writes (`"tres"`, `"res"`, ...).
    pub extensions: Vec<String>,
    /// Resource types this saver handles (empty accepts everything).
    pub types: Vec<String>,
    /// Paths it explicitly recognizes.
    pub recognized_paths: Vec<String>,
}

impl FormatSaver {
    pub fn new(extensions: &[&str]) -> Self {
        Self {
            extensions: extensions.iter().map(|value| value.to_string()).collect(),
            ..Self::default()
        }
    }

    pub fn for_type(mut self, kind: &str) -> Self {
        self.types.push(kind.to_string());
        self
    }

    /// `_recognize_path`: explicit path or matching extension (the engine's
    /// default ignores the type hint).
    pub fn recognize_path(&self, path: &str) -> bool {
        if self.recognized_paths.iter().any(|known| known == path) {
            return true;
        }
        let extension = path.rsplit('.').next().unwrap_or("");
        self.extensions.iter().any(|known| known == extension)
    }

    pub fn handles_type(&self, kind: &str) -> bool {
        kind.is_empty() || self.types.is_empty() || self.types.iter().any(|known| known == kind)
    }
}

#[derive(Clone, Debug, Default)]
pub struct ResourceSaverData {
    savers: Vec<FormatSaver>,
    timestamp_on_save: bool,
}

impl ResourceSaverData {
    pub fn new() -> Self {
        Self::default()
    }

    /// `add_resource_format_saver`: false when full (the engine's `ERR_FAIL_COND`
    /// on `MAX_SAVERS`).
    pub fn add_format_saver(&mut self, saver: FormatSaver, at_front: bool) -> bool {
        if self.savers.len() >= MAX_SAVERS {
            return false;
        }
        if at_front {
            self.savers.insert(0, saver);
        } else {
            self.savers.push(saver);
        }
        true
    }

    pub fn remove_format_saver(&mut self, index: usize) -> bool {
        if index >= self.savers.len() {
            return false;
        }
        self.savers.remove(index);
        true
    }

    pub fn get_saver_count(&self) -> usize {
        self.savers.len()
    }

    /// `get_recognized_extensions`: the saver that handles the type, or every
    /// saver when the type is empty; sorted and deduplicated.
    pub fn get_recognized_extensions(&self, kind: &str) -> Vec<String> {
        let mut extensions: BTreeSet<String> = BTreeSet::new();
        for saver in &self.savers {
            if kind.is_empty() || saver.types.iter().any(|known| known == kind) {
                for extension in &saver.extensions {
                    extensions.insert(extension.clone());
                }
            }
        }
        extensions.into_iter().collect()
    }

    /// `recognize_path`.
    pub fn recognize_path(&self, path: &str, kind: &str) -> bool {
        self.savers
            .iter()
            .any(|saver| saver.recognize_path(path) && saver.handles_type(kind))
    }

    /// `save`: recognized paths report `NotSupported` without an object layer.
    pub fn save(&self, path: &str, kind: &str, flags: u32) -> Result<(), SaveError> {
        let _ = flags;
        if self.recognize_path(path, kind) {
            Err(SaveError::NotSupported)
        } else {
            Err(SaveError::Unrecognized)
        }
    }

    /// `set_uid`: the UID registry is ported, but writing UIDs requires the
    /// object layer.
    pub fn set_uid(&self, path: &str, uid: i64) -> Result<(), SaveError> {
        let _ = (path, uid);
        Err(SaveError::NotSupported)
    }

    pub fn set_timestamp_on_save(&mut self, timestamp: bool) {
        self.timestamp_on_save = timestamp;
    }

    pub fn get_timestamp_on_save(&self) -> bool {
        self.timestamp_on_save
    }

    pub fn clear(&mut self) {
        self.savers.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flag_values() {
        assert_eq!(FLAG_NONE, 0);
        assert_eq!(FLAG_RELATIVE_PATHS, 1);
        assert_eq!(FLAG_BUNDLE_RESOURCES, 2);
        assert_eq!(FLAG_CHANGE_PATH, 4);
        assert_eq!(FLAG_OMIT_EDITOR_PROPERTIES, 8);
        assert_eq!(FLAG_SAVE_BIG_ENDIAN, 16);
        assert_eq!(FLAG_COMPRESS, 32);
        assert_eq!(FLAG_REPLACE_SUBRESOURCE_PATHS, 64);
        assert_eq!(MAX_SAVERS, 64);
    }

    #[test]
    fn registry_order_and_extensions() {
        let mut registry = ResourceSaverData::new();
        assert!(registry.add_format_saver(
            FormatSaver::new(&["tres", "res"]).for_type("Resource"),
            false
        ));
        assert!(registry.add_format_saver(
            FormatSaver::new(&["png"]).for_type("Image"),
            true
        ));
        assert_eq!(registry.get_saver_count(), 2);
        // The front insertion wins the order.
        assert_eq!(registry.get_recognized_extensions(""), vec!["png", "res", "tres"]);
        assert_eq!(registry.get_recognized_extensions("Resource"), vec!["res", "tres"]);
        assert_eq!(registry.get_recognized_extensions("Image"), vec!["png"]);
        assert!(registry.get_recognized_extensions("Mesh").is_empty());

        assert!(registry.recognize_path("res://a.tres", "Resource"));
        assert!(!registry.recognize_path("res://a.tres", "Image"));
        assert!(registry.recognize_path("res://a.png", ""));
        assert!(!registry.recognize_path("res://a.unknown", ""));

        assert!(registry.remove_format_saver(0));
        assert_eq!(registry.get_recognized_extensions(""), vec!["res", "tres"]);
        assert!(!registry.remove_format_saver(9));
    }

    #[test]
    fn explicit_paths_and_type_filtering() {
        let mut saver = FormatSaver::new(&["bin"]);
        saver.recognized_paths.push("res://special.dat".to_string());
        saver.types.push("PackedScene".to_string());
        let mut registry = ResourceSaverData::new();
        registry.add_format_saver(saver, false);

        assert!(registry.recognize_path("res://special.dat", "PackedScene"));
        assert!(!registry.recognize_path("res://special.dat", "Resource"));
        assert!(registry.recognize_path("res://anything.bin", "PackedScene"));
        // An empty type hint matches any saver that recognizes the extension.
        assert!(registry.recognize_path("res://anything.bin", ""));

        // Savers without a type list accept every type.
        let mut registry = ResourceSaverData::new();
        registry.add_format_saver(FormatSaver::new(&["tres"]), false);
        assert!(registry.recognize_path("a.tres", "Resource"));
        assert!(registry.recognize_path("a.tres", "Whatever"));
    }

    #[test]
    fn saving_reports_the_expected_errors() {
        let mut registry = ResourceSaverData::new();
        registry.add_format_saver(FormatSaver::new(&["tres"]), false);
        assert_eq!(
            registry.save("res://a.tres", "", FLAG_NONE),
            Err(SaveError::NotSupported)
        );
        assert_eq!(
            registry.save("res://a.unknown", "", FLAG_COMPRESS),
            Err(SaveError::Unrecognized)
        );
        assert_eq!(
            registry.set_uid("res://a.tres", 42),
            Err(SaveError::NotSupported)
        );

        assert!(!registry.get_timestamp_on_save());
        registry.set_timestamp_on_save(true);
        assert!(registry.get_timestamp_on_save());
        registry.clear();
        assert_eq!(registry.get_saver_count(), 0);
        assert!(registry.get_recognized_extensions("").is_empty());
    }

    #[test]
    fn saver_limit() {
        let mut registry = ResourceSaverData::new();
        for index in 0..MAX_SAVERS {
            assert!(registry.add_format_saver(
                FormatSaver::new(&[&format!("e{index}")]),
                index % 2 == 0
            ));
        }
        assert!(!registry.add_format_saver(FormatSaver::new(&["extra"]), false));
        assert_eq!(registry.get_saver_count(), MAX_SAVERS);
    }
}
