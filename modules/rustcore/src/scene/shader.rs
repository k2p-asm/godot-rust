//! `scene/resources/shader.*`: shader code and default texture parameters.
//!
//! Mode defaults to `Spatial`; code and include path start empty. Default
//! textures are keyed by parameter name and index (0 by default); setting a
//! null texture at index 0 erases the whole parameter, while other indices are
//! stored as given. Compilation, uniforms and the shader RID are not ported.

use std::collections::HashMap;

use crate::servers::rid::Rid;
use crate::string::string_name::StringName;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum ShaderMode {
    Spatial,
    CanvasItem,
    Particles,
    Sky,
    Fog,
    TextureBlit,
    Max,
}

#[derive(Clone, Debug, Default)]
pub struct ShaderData {
    mode: ShaderMode,
    code: String,
    include_path: String,
    default_textures: HashMap<StringName, HashMap<i32, Option<Rid>>>,
}

impl Default for ShaderMode {
    fn default() -> Self {
        ShaderMode::Spatial
    }
}

impl ShaderData {
    pub fn new() -> Self {
        Self {
            mode: ShaderMode::Spatial,
            code: String::new(),
            include_path: String::new(),
            default_textures: HashMap::new(),
        }
    }

    pub fn get_mode(&self) -> ShaderMode {
        self.mode
    }

    /// The engine only exposes the mode getter; subclasses such as `VisualShader`
    /// set it internally.
    pub fn set_mode(&mut self, mode: ShaderMode) {
        self.mode = mode;
    }

    pub fn set_code(&mut self, code: &str) {
        self.code = code.to_string();
    }

    pub fn get_code(&self) -> &str {
        &self.code
    }

    pub fn set_include_path(&mut self, path: &str) {
        self.include_path = path.to_string();
    }

    pub fn get_include_path(&self) -> &str {
        &self.include_path
    }

    /// A null texture at index 0 erases the parameter; other indices keep the
    /// entry with a null texture.
    pub fn set_default_texture_parameter(&mut self, name: &str, texture: Option<Rid>, index: i32) {
        let key = StringName::new(name);
        if texture.is_none() && index == 0 {
            self.default_textures.remove(&key);
            return;
        }
        self.default_textures
            .entry(key)
            .or_default()
            .insert(index, texture);
    }

    pub fn get_default_texture_parameter(&self, name: &str, index: i32) -> Option<Rid> {
        self.default_textures
            .get(&StringName::new(name))
            .and_then(|m| m.get(&index))
            .copied()
            .flatten()
    }

    /// Sorted for determinism.
    pub fn get_default_texture_parameter_list(&self) -> Vec<StringName> {
        let mut names: Vec<StringName> = self.default_textures.keys().copied().collect();
        names.sort_by(|a, b| a.as_string().cmp(&b.as_string()));
        names
    }

    /// Text shaders are the base `Shader`; the engine also has visual shaders.
    pub fn is_text_shader(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let s = ShaderData::new();
        assert_eq!(s.get_mode(), ShaderMode::Spatial);
        assert_eq!(s.get_code(), "");
        assert_eq!(s.get_include_path(), "");
        assert!(s.get_default_texture_parameter_list().is_empty());
        assert!(s.is_text_shader());
    }

    #[test]
    fn code_include_and_mode() {
        let mut s = ShaderData::new();
        s.set_code("shader_type canvas_item;");
        s.set_include_path("res://inc.gdshaderinc");
        s.set_mode(ShaderMode::CanvasItem);
        assert_eq!(s.get_code(), "shader_type canvas_item;");
        assert_eq!(s.get_include_path(), "res://inc.gdshaderinc");
        assert_eq!(s.get_mode(), ShaderMode::CanvasItem);

        assert_eq!(ShaderMode::Spatial as i32, 0);
        assert_eq!(ShaderMode::CanvasItem as i32, 1);
        assert_eq!(ShaderMode::Particles as i32, 2);
        assert_eq!(ShaderMode::Sky as i32, 3);
        assert_eq!(ShaderMode::Fog as i32, 4);
        assert_eq!(ShaderMode::TextureBlit as i32, 5);
        assert_eq!(ShaderMode::Max as i32, 6);
    }

    #[test]
    fn default_texture_parameters() {
        let mut s = ShaderData::new();
        s.set_default_texture_parameter("noise", Some(Rid::from_u64(4)), 0);
        s.set_default_texture_parameter("noise", Some(Rid::from_u64(5)), 2);
        s.set_default_texture_parameter("mask", Some(Rid::from_u64(6)), 0);

        assert_eq!(s.get_default_texture_parameter("noise", 0), Some(Rid::from_u64(4)));
        assert_eq!(s.get_default_texture_parameter("noise", 2), Some(Rid::from_u64(5)));
        assert_eq!(s.get_default_texture_parameter("noise", 1), None);
        let list: Vec<String> = s
            .get_default_texture_parameter_list()
            .iter()
            .map(|n| n.as_string())
            .collect();
        assert_eq!(list, vec!["mask", "noise"]);

        // Null at index 0 removes the whole parameter (including other indices).
        s.set_default_texture_parameter("noise", None, 0);
        assert_eq!(s.get_default_texture_parameter("noise", 0), None);
        assert_eq!(s.get_default_texture_parameter("noise", 2), None);
        let list: Vec<String> = s
            .get_default_texture_parameter_list()
            .iter()
            .map(|n| n.as_string())
            .collect();
        assert_eq!(list, vec!["mask"]);

        // Null at a non-zero index inserts/keeps the entry with a null texture.
        s.set_default_texture_parameter("noise", None, 2);
        assert_eq!(s.get_default_texture_parameter("noise", 2), None);
        assert_eq!(s.get_default_texture_parameter_list().len(), 2);
    }
}
