//! `scene/resources/shader_include.*`: shader include files.
//!
//! A `ShaderInclude` is just its source text plus the include path it was loaded
//! from and the set of includes it depends on. `set_code` stores the text,
//! `set_include_path` stamps the path and the dependency set drives reloads.
//!
//! The dependency graph here is a simple ordered set of paths (the engine keeps
//! `Ref<ShaderInclude>` objects and emits `changed`); loading/reloading and the
//! change signal are not ported.

#[derive(Clone, Debug, Default)]
pub struct ShaderIncludeData {
    code: String,
    include_path: String,
    dependencies: Vec<String>,
}

impl ShaderIncludeData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_code(&mut self, text: &str) {
        self.code = text.to_string();
    }

    pub fn get_code(&self) -> &str {
        &self.code
    }

    /// Set when the resource is loaded from disk.
    pub fn set_include_path(&mut self, path: &str) {
        self.include_path = path.to_string();
    }

    pub fn get_include_path(&self) -> &str {
        &self.include_path
    }

    /// `_dependency_changed` is driven by this list in the port.
    pub fn add_dependency(&mut self, path: &str) -> bool {
        if self.dependencies.iter().any(|existing| existing == path) {
            return false;
        }
        self.dependencies.push(path.to_string());
        true
    }

    pub fn get_dependencies(&self) -> &[String] {
        &self.dependencies
    }

    pub fn get_size(&self) -> usize {
        self.code.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn code_and_path() {
        let mut s = ShaderIncludeData::new();
        assert_eq!(s.get_code(), "");
        assert_eq!(s.get_include_path(), "");
        assert_eq!(s.get_size(), 0);
        assert!(s.get_dependencies().is_empty());

        s.set_code("#define PI 3.14");
        s.set_include_path("res://shaders/common.gdshaderinc");
        assert_eq!(s.get_code(), "#define PI 3.14");
        assert_eq!(s.get_include_path(), "res://shaders/common.gdshaderinc");
        assert_eq!(s.get_size(), "#define PI 3.14".len());
    }

    #[test]
    fn dependencies_are_unique() {
        let mut s = ShaderIncludeData::new();
        assert!(s.add_dependency("a.gdshaderinc"));
        assert!(!s.add_dependency("a.gdshaderinc"));
        assert!(s.add_dependency("b.gdshaderinc"));
        assert_eq!(s.get_dependencies(), &["a.gdshaderinc", "b.gdshaderinc"]);
    }
}
