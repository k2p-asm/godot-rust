//! `scene/resources/material.*`: base material and `ShaderMaterial` state.
//!
//! `Material` keeps the next pass and the render priority (default 0, validated
//! into `-128..=127`). `ShaderMaterial` adds the shader `Rid` and a parameter
//! map: setting a parameter to `null` erases it, any other value stores a copy;
//! missing parameters read back as `null`. The RenderingServer side and the
//! shader compilation are not ported.

use std::collections::HashMap;

use crate::servers::rid::Rid;
use crate::string::string_name::StringName;
use crate::variant::Variant;

/// `RenderingServer::MATERIAL_RENDER_PRIORITY_MIN/MAX`.
pub const RENDER_PRIORITY_MIN: i32 = -128;
pub const RENDER_PRIORITY_MAX: i32 = 127;

#[derive(Clone, Debug, Default)]
pub struct MaterialData {
    next_pass: Option<Rid>,
    render_priority: i32,
}

impl MaterialData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_next_pass(&mut self, next_pass: Option<Rid>) {
        self.next_pass = next_pass;
    }

    pub fn get_next_pass(&self) -> Option<Rid> {
        self.next_pass
    }

    /// Rejects values outside `RENDER_PRIORITY_MIN..=RENDER_PRIORITY_MAX`.
    pub fn set_render_priority(&mut self, priority: i32) -> bool {
        if !(RENDER_PRIORITY_MIN..=RENDER_PRIORITY_MAX).contains(&priority) {
            return false;
        }
        self.render_priority = priority;
        true
    }

    pub fn get_render_priority(&self) -> i32 {
        self.render_priority
    }
}

#[derive(Clone, Debug, Default)]
pub struct ShaderMaterialData {
    pub material: MaterialData,
    shader: Option<Rid>,
    params: HashMap<StringName, Variant>,
}

impl ShaderMaterialData {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set_shader(&mut self, shader: Option<Rid>) {
        self.shader = shader;
    }

    pub fn get_shader(&self) -> Option<Rid> {
        self.shader
    }

    /// A `Nil` value removes the parameter, like the engine's `param_cache.erase`.
    pub fn set_shader_parameter(&mut self, name: &str, value: Variant) {
        let key = StringName::new(name);
        if matches!(value, Variant::Nil) {
            self.params.remove(&key);
        } else {
            self.params.insert(key, value);
        }
    }

    pub fn get_shader_parameter(&self, name: &str) -> Option<Variant> {
        self.params.get(&StringName::new(name)).cloned()
    }

    /// Sorted for deterministic output; the engine returns its map keys.
    pub fn get_shader_parameter_list(&self) -> Vec<StringName> {
        let mut keys: Vec<StringName> = self.params.keys().copied().collect();
        keys.sort_by(|a, b| a.as_string().cmp(&b.as_string()));
        keys
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn material_defaults_and_priority_bounds() {
        let mut m = MaterialData::new();
        assert!(m.get_next_pass().is_none());
        assert_eq!(m.get_render_priority(), 0);

        assert!(!m.set_render_priority(RENDER_PRIORITY_MIN - 1));
        assert!(!m.set_render_priority(RENDER_PRIORITY_MAX + 1));
        assert_eq!(m.get_render_priority(), 0);
        assert!(m.set_render_priority(RENDER_PRIORITY_MIN));
        assert_eq!(m.get_render_priority(), -128);
        assert!(m.set_render_priority(RENDER_PRIORITY_MAX));
        assert_eq!(m.get_render_priority(), 127);

        m.set_next_pass(Some(Rid::from_u64(3)));
        assert_eq!(m.get_next_pass(), Some(Rid::from_u64(3)));
        m.set_next_pass(None);
        assert!(m.get_next_pass().is_none());
    }

    #[test]
    fn shader_material_parameters() {
        let mut s = ShaderMaterialData::new();
        assert!(s.get_shader().is_none());
        assert!(s.get_shader_parameter("tint").is_none());
        assert!(s.get_shader_parameter_list().is_empty());

        s.set_shader(Some(Rid::from_u64(9)));
        assert_eq!(s.get_shader(), Some(Rid::from_u64(9)));

        s.set_shader_parameter("tint", Variant::Float(0.5));
        s.set_shader_parameter("enabled", Variant::Bool(true));
        assert_eq!(s.get_shader_parameter("tint"), Some(Variant::Float(0.5)));
        assert_eq!(s.get_shader_parameter("enabled"), Some(Variant::Bool(true)));
        let list: Vec<String> = s
            .get_shader_parameter_list()
            .iter()
            .map(|n| n.as_string())
            .collect();
        assert_eq!(list, vec!["enabled", "tint"]);

        // Overwriting keeps one entry.
        s.set_shader_parameter("tint", Variant::Float(1.5));
        assert_eq!(s.get_shader_parameter("tint"), Some(Variant::Float(1.5)));
        assert_eq!(s.get_shader_parameter_list().len(), 2);

        // Nil removes the entry.
        s.set_shader_parameter("tint", Variant::Nil);
        assert!(s.get_shader_parameter("tint").is_none());
        assert_eq!(s.get_shader_parameter_list().len(), 1);

        // The base material state is still available.
        assert!(s.material.set_render_priority(5));
        assert_eq!(s.material.get_render_priority(), 5);
    }
}
