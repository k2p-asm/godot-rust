//! `core/io/resource_loader.*`: the format-loader registry (no object loading).
//!
//! The engine's `ResourceLoader` owns a table of `ResourceFormatLoader`s and
//! routes `load`, `exists`, type/extension queries and dependency lookups
//! through them. This port keeps the registry and the routing, with loaders
//! described by plain data (the engine's virtual overrides become fields):
//!
//! - `CacheMode` (`REUSE`, `REPLACE`, `IGNORE`, and the deep variants);
//! - `add_format_loader` up to the engine's 64 entries, `remove_format_loader`;
//! - `get_recognized_extensions_for_type`, `recognize_path`, `handles_type`,
//!   `get_resource_type`, `exists`, `get_dependencies`;
//! - `load` reports [`LoadError::NoLoader`]/[`LoadError::NotSupported`] because
//!   the object layer (and therefore `Resource`) is not ported.
//!
//! Path remapping, import groups, thread loading and the global singleton are
//! not ported.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum CacheMode {
    Ignore,
    Reuse,
    Replace,
    IgnoreDeep,
    ReplaceDeep,
}

impl CacheMode {
    pub fn is_ignore(&self) -> bool {
        matches!(self, CacheMode::Ignore | CacheMode::IgnoreDeep)
    }

    pub fn is_deep(&self) -> bool {
        matches!(self, CacheMode::IgnoreDeep | CacheMode::ReplaceDeep)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LoadError {
    /// `ERR_FILE_NOT_FOUND`: no loader recognizes the path.
    NoLoader,
    /// The object layer is not available in this port.
    NotSupported,
}

/// The data a `ResourceFormatLoader` exposes.
#[derive(Clone, Debug, Default)]
pub struct FormatLoader {
    pub extensions: Vec<String>,
    pub handles_types: Vec<String>,
    /// `path -> resource type`, only for paths this loader owns.
    pub recognize: Vec<(String, String)>,
    pub exists: Vec<String>,
    pub dependencies: Vec<(String, Vec<String>)>,
    /// The type returned for any recognized path when it has no explicit entry.
    pub default_type: String,
}

impl FormatLoader {
    pub fn new(extensions: &[&str], types: &[&str]) -> Self {
        Self {
            extensions: extensions.iter().map(|value| value.to_string()).collect(),
            handles_types: types.iter().map(|value| value.to_string()).collect(),
            ..Self::default()
        }
    }

    /// `recognize_path`: an explicit entry or a matching extension.
    pub fn recognize_path(&self, path: &str, for_type: &str) -> bool {
        if let Some((_, kind)) = self
            .recognize
            .iter()
            .find(|(known, _)| known == path)
        {
            return for_type.is_empty() || kind == for_type;
        }
        let extension = path.rsplit('.').next().unwrap_or("");
        self.extensions.iter().any(|known| known == extension)
    }

    pub fn handles_type(&self, kind: &str) -> bool {
        self.handles_types.iter().any(|known| known == kind)
    }

    pub fn get_resource_type(&self, path: &str) -> String {
        if let Some((_, kind)) = self.recognize.iter().find(|(known, _)| known == path) {
            return kind.clone();
        }
        self.default_type.clone()
    }

    pub fn exists(&self, path: &str) -> bool {
        self.exists.iter().any(|known| known == path)
    }

    pub fn get_dependencies(&self, path: &str) -> Vec<String> {
        self.dependencies
            .iter()
            .find(|(known, _)| known == path)
            .map(|(_, deps)| deps.clone())
            .unwrap_or_default()
    }
}

pub const MAX_LOADERS: usize = 64;

#[derive(Clone, Debug, Default)]
pub struct ResourceLoaderData {
    loaders: Vec<FormatLoader>,
}

impl ResourceLoaderData {
    pub fn new() -> Self {
        Self::default()
    }

    /// `add_resource_format_loader`: refused past the engine's 64 slots.
    pub fn add_format_loader(&mut self, loader: FormatLoader) -> bool {
        if self.loaders.len() >= MAX_LOADERS {
            return false;
        }
        self.loaders.push(loader);
        true
    }

    /// `remove_resource_format_loader`.
    pub fn remove_format_loader(&mut self, index: usize) -> bool {
        if index >= self.loaders.len() {
            return false;
        }
        self.loaders.remove(index);
        true
    }

    pub fn get_loader_count(&self) -> usize {
        self.loaders.len()
    }

    /// `get_recognized_extensions_for_type`.
    pub fn get_recognized_extensions_for_type(&self, kind: &str) -> Vec<String> {
        let mut extensions: Vec<String> = Vec::new();
        for loader in &self.loaders {
            if kind.is_empty() || loader.handles_type(kind) {
                for extension in &loader.extensions {
                    if !extensions.contains(extension) {
                        extensions.push(extension.clone());
                    }
                }
            }
        }
        extensions.sort();
        extensions
    }

    /// `recognize_path`.
    pub fn recognize_path(&self, path: &str, for_type: &str) -> bool {
        self.loaders
            .iter()
            .any(|loader| loader.recognize_path(path, for_type))
    }

    pub fn handles_type(&self, kind: &str) -> bool {
        self.loaders.iter().any(|loader| loader.handles_type(kind))
    }

    /// `get_resource_type`: the first loader that recognizes the path.
    pub fn get_resource_type(&self, path: &str) -> Option<String> {
        self.loaders
            .iter()
            .find(|loader| loader.recognize_path(path, ""))
            .map(|loader| loader.get_resource_type(path))
    }

    /// `exists`.
    pub fn exists(&self, path: &str) -> bool {
        self.loaders.iter().any(|loader| loader.exists(path))
    }

    /// `get_dependencies`.
    pub fn get_dependencies(&self, path: &str) -> Vec<String> {
        self.loaders
            .iter()
            .find(|loader| loader.recognize_path(path, ""))
            .map(|loader| loader.get_dependencies(path))
            .unwrap_or_default()
    }

    /// `load`: recognized paths report `NotSupported` (the object layer is
    /// missing); unknown ones report `NoLoader`.
    pub fn load(&self, path: &str, _cache_mode: CacheMode) -> Result<(), LoadError> {
        if self.recognize_path(path, "") {
            Err(LoadError::NotSupported)
        } else {
            Err(LoadError::NoLoader)
        }
    }

    pub fn clear(&mut self) {
        self.loaders.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn loaders() -> ResourceLoaderData {
        let mut registry = ResourceLoaderData::new();
        let mut text = FormatLoader::new(&["tres", "res"], &["Resource"]);
        text.default_type = "Resource".to_string();
        text.exists
            .push("res://scene.tres".to_string());
        text.dependencies.push((
            "res://scene.tres".to_string(),
            vec!["res://mesh.obj".to_string(), "res://texture.png".to_string()],
        ));
        text.recognize.push((
            "res://special.bin".to_string(),
            "PackedScene".to_string(),
        ));
        assert!(registry.add_format_loader(text));

        let mut texture = FormatLoader::new(&["png"], &["ImageTexture"]);
        texture.default_type = "ImageTexture".to_string();
        texture
            .exists
            .push("res://texture.png".to_string());
        assert!(registry.add_format_loader(texture));
        registry
    }

    #[test]
    fn cache_modes() {
        assert!(CacheMode::Ignore.is_ignore());
        assert!(CacheMode::IgnoreDeep.is_ignore());
        assert!(!CacheMode::Reuse.is_ignore());
        assert!(CacheMode::IgnoreDeep.is_deep());
        assert!(CacheMode::ReplaceDeep.is_deep());
        assert!(!CacheMode::Replace.is_deep());
        assert_eq!(CacheMode::Reuse as i32, 1);
    }

    #[test]
    fn extensions_and_types() {
        let registry = loaders();
        assert_eq!(registry.get_loader_count(), 2);
        assert_eq!(
            registry.get_recognized_extensions_for_type(""),
            vec!["png", "res", "tres"]
        );
        assert_eq!(
            registry.get_recognized_extensions_for_type("Resource"),
            vec!["res", "tres"]
        );
        assert_eq!(
            registry.get_recognized_extensions_for_type("ImageTexture"),
            vec!["png"]
        );
        assert!(registry.get_recognized_extensions_for_type("Missing").is_empty());
        assert!(registry.handles_type("Resource"));
        assert!(!registry.handles_type("Mesh"));
        assert_eq!(registry.get_loader_count(), 2);
    }

    #[test]
    fn path_queries() {
        let registry = loaders();
        assert!(registry.recognize_path("res://scene.tres", ""));
        assert!(registry.recognize_path("res://scene.tres", "Resource"));
        // The default engine implementation ignores the type hint and matches
        // on the extension alone.
        assert!(registry.recognize_path("res://scene.tres", "ImageTexture"));
        assert!(registry.recognize_path("res://special.bin", "PackedScene"));
        assert!(registry.recognize_path("res://anything.png", ""));
        assert!(!registry.recognize_path("res://anything.unknown", ""));

        assert_eq!(
            registry.get_resource_type("res://scene.tres").as_deref(),
            Some("Resource")
        );
        assert_eq!(
            registry.get_resource_type("res://special.bin").as_deref(),
            Some("PackedScene")
        );
        assert_eq!(
            registry.get_resource_type("res://image.png").as_deref(),
            Some("ImageTexture")
        );
        assert!(registry.get_resource_type("res://x.unknown").is_none());

        assert!(registry.exists("res://scene.tres"));
        assert!(!registry.exists("res://other.tres"));
        assert_eq!(
            registry.get_dependencies("res://scene.tres"),
            vec!["res://mesh.obj", "res://texture.png"]
        );
        assert!(registry.get_dependencies("res://image.png").is_empty());
    }

    #[test]
    fn loading_reports_the_engine_errors() {
        let registry = loaders();
        assert_eq!(
            registry.load("res://scene.tres", CacheMode::Reuse),
            Err(LoadError::NotSupported)
        );
        assert_eq!(
            registry.load("res://nope.unknown", CacheMode::Ignore),
            Err(LoadError::NoLoader)
        );
    }

    #[test]
    fn registry_limits_and_removal() {
        let mut registry = ResourceLoaderData::new();
        for index in 0..MAX_LOADERS {
            assert!(registry.add_format_loader(FormatLoader::new(
                &[&format!("e{index}")],
                &[]
            )));
        }
        assert!(!registry.add_format_loader(FormatLoader::new(&["extra"], &[])));
        assert_eq!(registry.get_loader_count(), MAX_LOADERS);
        assert!(registry.remove_format_loader(0));
        assert_eq!(registry.get_loader_count(), MAX_LOADERS - 1);
        assert!(!registry.remove_format_loader(MAX_LOADERS));
        registry.clear();
        assert_eq!(registry.get_loader_count(), 0);
    }
}
