//! `scene/3d/visual_instance_3d.*`: visibility and geometry instance state.
//!
//! `VisualInstance3D`: layer mask (default 1, layer numbers 1..20), sorting
//! offset (0) and AABB-centre sorting (on).
//!
//! `GeometryInstance3D`: shadow casting (on), material override/overlay
//! handles, visibility range begin/end and margins (0), fade mode (disabled),
//! transparency (clamped to `0..=1`), LOD bias (1, negatives rejected),
//! per-instance shader parameters (`null` erases), extra cull margin, custom
//! AABB, lightmap texel scale (1), GI mode (static) and occlusion-culling
//! ignore.
//!
//! The RIDs and RenderingServer updates are not ported.

use std::collections::HashMap;

use crate::math::aabb::Aabb;
use crate::servers::rid::Rid;
use crate::string::string_name::StringName;
use crate::variant::Variant;

pub use crate::scene::mesh_library::ShadowCastingSetting;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum GIMode {
    Disabled,
    Static,
    Dynamic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum LightmapScale {
    Scale1X,
    Scale2X,
    Scale4X,
    Scale8X,
    Max,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum VisibilityRangeFadeMode {
    Disabled,
    SelfOnly,
    Dependencies,
}

impl Default for VisibilityRangeFadeMode {
    fn default() -> Self {
        VisibilityRangeFadeMode::Disabled
    }
}

#[derive(Clone, Copy, Debug)]
pub struct VisualInstance3DData {
    layers: u32,
    sorting_offset: f32,
    sorting_use_aabb_center: bool,
}

impl Default for VisualInstance3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl VisualInstance3DData {
    pub fn new() -> Self {
        Self {
            layers: 1,
            sorting_offset: 0.0,
            sorting_use_aabb_center: true,
        }
    }

    pub fn set_layer_mask(&mut self, mask: u32) {
        self.layers = mask;
    }

    pub fn get_layer_mask(&self) -> u32 {
        self.layers
    }

    /// Layer numbers are `1..=20`.
    pub fn set_layer_mask_value(&mut self, layer: i32, enabled: bool) -> bool {
        if !(1..=20).contains(&layer) {
            return false;
        }
        let bit = 1u32 << (layer - 1);
        if enabled {
            self.layers |= bit;
        } else {
            self.layers &= !bit;
        }
        true
    }

    pub fn get_layer_mask_value(&self, layer: i32) -> Option<bool> {
        if !(1..=20).contains(&layer) {
            return None;
        }
        Some((self.layers & (1u32 << (layer - 1))) != 0)
    }

    pub fn set_sorting_offset(&mut self, offset: f32) {
        self.sorting_offset = offset;
    }

    pub fn get_sorting_offset(&self) -> f32 {
        self.sorting_offset
    }

    pub fn set_sorting_use_aabb_center(&mut self, enabled: bool) {
        self.sorting_use_aabb_center = enabled;
    }

    pub fn is_sorting_use_aabb_center(&self) -> bool {
        self.sorting_use_aabb_center
    }
}

#[derive(Clone, Debug)]
pub struct GeometryInstance3DData {
    pub visual: VisualInstance3DData,
    shadow_casting_setting: ShadowCastingSetting,
    material_override: Option<Rid>,
    material_overlay: Option<Rid>,
    visibility_range_begin: f32,
    visibility_range_end: f32,
    visibility_range_begin_margin: f32,
    visibility_range_end_margin: f32,
    visibility_range_fade_mode: VisibilityRangeFadeMode,
    transparency: f32,
    lod_bias: f32,
    instance_shader_parameters: HashMap<StringName, Variant>,
    extra_cull_margin: f32,
    custom_aabb: Aabb,
    lightmap_texel_scale: f32,
    gi_mode: GIMode,
    ignore_occlusion_culling: bool,
}

impl Default for GeometryInstance3DData {
    fn default() -> Self {
        Self::new()
    }
}

impl GeometryInstance3DData {
    pub fn new() -> Self {
        Self {
            visual: VisualInstance3DData::new(),
            shadow_casting_setting: ShadowCastingSetting::On,
            material_override: None,
            material_overlay: None,
            visibility_range_begin: 0.0,
            visibility_range_end: 0.0,
            visibility_range_begin_margin: 0.0,
            visibility_range_end_margin: 0.0,
            visibility_range_fade_mode: VisibilityRangeFadeMode::Disabled,
            transparency: 0.0,
            lod_bias: 1.0,
            instance_shader_parameters: HashMap::new(),
            extra_cull_margin: 0.0,
            custom_aabb: Aabb::default(),
            lightmap_texel_scale: 1.0,
            gi_mode: GIMode::Static,
            ignore_occlusion_culling: false,
        }
    }

    pub fn set_cast_shadows_setting(&mut self, setting: ShadowCastingSetting) {
        self.shadow_casting_setting = setting;
    }

    pub fn get_cast_shadows_setting(&self) -> ShadowCastingSetting {
        self.shadow_casting_setting
    }

    pub fn set_material_override(&mut self, material: Option<Rid>) {
        self.material_override = material;
    }

    pub fn get_material_override(&self) -> Option<Rid> {
        self.material_override
    }

    pub fn set_material_overlay(&mut self, material: Option<Rid>) {
        self.material_overlay = material;
    }

    pub fn get_material_overlay(&self) -> Option<Rid> {
        self.material_overlay
    }

    pub fn set_visibility_range_begin(&mut self, distance: f32) {
        self.visibility_range_begin = distance;
    }

    pub fn get_visibility_range_begin(&self) -> f32 {
        self.visibility_range_begin
    }

    pub fn set_visibility_range_end(&mut self, distance: f32) {
        self.visibility_range_end = distance;
    }

    pub fn get_visibility_range_end(&self) -> f32 {
        self.visibility_range_end
    }

    pub fn set_visibility_range_begin_margin(&mut self, distance: f32) {
        self.visibility_range_begin_margin = distance;
    }

    pub fn get_visibility_range_begin_margin(&self) -> f32 {
        self.visibility_range_begin_margin
    }

    pub fn set_visibility_range_end_margin(&mut self, distance: f32) {
        self.visibility_range_end_margin = distance;
    }

    pub fn get_visibility_range_end_margin(&self) -> f32 {
        self.visibility_range_end_margin
    }

    pub fn set_visibility_range_fade_mode(&mut self, mode: VisibilityRangeFadeMode) {
        self.visibility_range_fade_mode = mode;
    }

    pub fn get_visibility_range_fade_mode(&self) -> VisibilityRangeFadeMode {
        self.visibility_range_fade_mode
    }

    /// Clamped to `0..=1`, like `CLAMP(p_transparency, 0.0f, 1.0f)`.
    pub fn set_transparency(&mut self, transparency: f32) {
        self.transparency = transparency.clamp(0.0, 1.0);
    }

    pub fn get_transparency(&self) -> f32 {
        self.transparency
    }

    /// Negative LOD biases are rejected.
    pub fn set_lod_bias(&mut self, bias: f32) -> bool {
        if bias < 0.0 {
            return false;
        }
        self.lod_bias = bias;
        true
    }

    pub fn get_lod_bias(&self) -> f32 {
        self.lod_bias
    }

    /// `null` removes the parameter (the server then uses the shader default).
    pub fn set_instance_shader_parameter(&mut self, name: &str, value: Variant) {
        let key = StringName::new(name);
        if matches!(value, Variant::Nil) {
            self.instance_shader_parameters.remove(&key);
        } else {
            self.instance_shader_parameters.insert(key, value);
        }
    }

    pub fn get_instance_shader_parameter(&self, name: &str) -> Option<Variant> {
        self.instance_shader_parameters
            .get(&StringName::new(name))
            .cloned()
    }

    /// Sorted for determinism.
    pub fn get_instance_shader_parameter_list(&self) -> Vec<StringName> {
        let mut names: Vec<StringName> = self.instance_shader_parameters.keys().copied().collect();
        names.sort_by(|a, b| a.as_string().cmp(&b.as_string()));
        names
    }

    pub fn set_extra_cull_margin(&mut self, margin: f32) {
        self.extra_cull_margin = margin;
    }

    pub fn get_extra_cull_margin(&self) -> f32 {
        self.extra_cull_margin
    }

    pub fn set_custom_aabb(&mut self, aabb: Aabb) {
        self.custom_aabb = aabb;
    }

    pub fn get_custom_aabb(&self) -> Aabb {
        self.custom_aabb
    }

    pub fn set_lightmap_texel_scale(&mut self, scale: f32) {
        self.lightmap_texel_scale = scale;
    }

    pub fn get_lightmap_texel_scale(&self) -> f32 {
        self.lightmap_texel_scale
    }

    pub fn set_gi_mode(&mut self, mode: GIMode) {
        self.gi_mode = mode;
    }

    pub fn get_gi_mode(&self) -> GIMode {
        self.gi_mode
    }

    pub fn set_ignore_occlusion_culling(&mut self, ignore: bool) {
        self.ignore_occlusion_culling = ignore;
    }

    pub fn is_ignoring_occlusion_culling(&self) -> bool {
        self.ignore_occlusion_culling
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vector3::Vector3;

    #[test]
    fn visual_instance_defaults_and_layers() {
        let mut v = VisualInstance3DData::new();
        assert_eq!(v.get_layer_mask(), 1);
        assert_eq!(v.get_sorting_offset(), 0.0);
        assert!(v.is_sorting_use_aabb_center());

        v.set_layer_mask(0);
        assert!(v.set_layer_mask_value(1, true));
        assert!(v.set_layer_mask_value(20, true));
        assert_eq!(v.get_layer_mask(), 1 | (1 << 19));
        assert_eq!(v.get_layer_mask_value(20), Some(true));
        assert_eq!(v.get_layer_mask_value(2), Some(false));
        assert!(v.set_layer_mask_value(20, false));
        assert_eq!(v.get_layer_mask(), 1);
        assert!(!v.set_layer_mask_value(0, true));
        assert!(!v.set_layer_mask_value(21, true));
        assert_eq!(v.get_layer_mask_value(21), None);

        v.set_sorting_offset(-1.5);
        v.set_sorting_use_aabb_center(false);
        assert_eq!(v.get_sorting_offset(), -1.5);
        assert!(!v.is_sorting_use_aabb_center());
    }

    #[test]
    fn geometry_defaults() {
        let g = GeometryInstance3DData::new();
        assert_eq!(g.get_cast_shadows_setting(), ShadowCastingSetting::On);
        assert!(g.get_material_override().is_none());
        assert!(g.get_material_overlay().is_none());
        assert_eq!(g.get_visibility_range_begin(), 0.0);
        assert_eq!(g.get_visibility_range_end(), 0.0);
        assert_eq!(g.get_visibility_range_begin_margin(), 0.0);
        assert_eq!(g.get_visibility_range_end_margin(), 0.0);
        assert_eq!(
            g.get_visibility_range_fade_mode(),
            VisibilityRangeFadeMode::Disabled
        );
        assert_eq!(g.get_transparency(), 0.0);
        assert_eq!(g.get_lod_bias(), 1.0);
        assert!(g.get_instance_shader_parameter_list().is_empty());
        assert_eq!(g.get_extra_cull_margin(), 0.0);
        assert_eq!(g.get_lightmap_texel_scale(), 1.0);
        assert_eq!(g.get_gi_mode(), GIMode::Static);
        assert!(!g.is_ignoring_occlusion_culling());
    }

    #[test]
    fn geometry_values_and_clamps() {
        let mut g = GeometryInstance3DData::new();
        g.set_transparency(-1.0);
        assert_eq!(g.get_transparency(), 0.0);
        g.set_transparency(2.0);
        assert_eq!(g.get_transparency(), 1.0);
        g.set_transparency(0.25);
        assert_eq!(g.get_transparency(), 0.25);

        assert!(!g.set_lod_bias(-0.1));
        assert_eq!(g.get_lod_bias(), 1.0);
        assert!(g.set_lod_bias(0.0));
        assert_eq!(g.get_lod_bias(), 0.0);

        g.set_cast_shadows_setting(ShadowCastingSetting::ShadowsOnly);
        g.set_material_override(Some(Rid::from_u64(1)));
        g.set_material_overlay(Some(Rid::from_u64(2)));
        g.set_visibility_range_begin(5.0);
        g.set_visibility_range_end(100.0);
        g.set_visibility_range_begin_margin(1.0);
        g.set_visibility_range_end_margin(2.0);
        g.set_visibility_range_fade_mode(VisibilityRangeFadeMode::Dependencies);
        g.set_extra_cull_margin(3.0);
        g.set_custom_aabb(Aabb::new(Vector3::ZERO, Vector3::new(1.0, 2.0, 3.0)));
        g.set_lightmap_texel_scale(2.0);
        g.set_gi_mode(GIMode::Dynamic);
        g.set_ignore_occlusion_culling(true);

        assert_eq!(g.get_cast_shadows_setting(), ShadowCastingSetting::ShadowsOnly);
        assert_eq!(g.get_material_override(), Some(Rid::from_u64(1)));
        assert_eq!(g.get_material_overlay(), Some(Rid::from_u64(2)));
        assert_eq!(g.get_visibility_range_begin(), 5.0);
        assert_eq!(g.get_visibility_range_end(), 100.0);
        assert_eq!(g.get_visibility_range_begin_margin(), 1.0);
        assert_eq!(g.get_visibility_range_end_margin(), 2.0);
        assert_eq!(
            g.get_visibility_range_fade_mode(),
            VisibilityRangeFadeMode::Dependencies
        );
        assert_eq!(g.get_extra_cull_margin(), 3.0);
        assert_eq!(g.get_custom_aabb().size, Vector3::new(1.0, 2.0, 3.0));
        assert_eq!(g.get_lightmap_texel_scale(), 2.0);
        assert_eq!(g.get_gi_mode(), GIMode::Dynamic);
        assert!(g.is_ignoring_occlusion_culling());

        assert_eq!(GIMode::Disabled as i32, 0);
        assert_eq!(GIMode::Static as i32, 1);
        assert_eq!(GIMode::Dynamic as i32, 2);
        assert_eq!(LightmapScale::Scale1X as i32, 0);
        assert_eq!(LightmapScale::Scale8X as i32, 3);
        assert_eq!(LightmapScale::Max as i32, 4);
        assert_eq!(VisibilityRangeFadeMode::Disabled as i32, 0);
        assert_eq!(VisibilityRangeFadeMode::SelfOnly as i32, 1);
        assert_eq!(VisibilityRangeFadeMode::Dependencies as i32, 2);
    }

    #[test]
    fn instance_shader_parameters() {
        let mut g = GeometryInstance3DData::new();
        g.set_instance_shader_parameter("tint", Variant::Float(0.5));
        g.set_instance_shader_parameter("mask", Variant::Bool(true));
        assert_eq!(g.get_instance_shader_parameter("tint"), Some(Variant::Float(0.5)));
        let names: Vec<String> = g
            .get_instance_shader_parameter_list()
            .iter()
            .map(|n| n.as_string())
            .collect();
        assert_eq!(names, vec!["mask", "tint"]);

        g.set_instance_shader_parameter("tint", Variant::Nil);
        assert!(g.get_instance_shader_parameter("tint").is_none());
        assert_eq!(g.get_instance_shader_parameter_list().len(), 1);
    }
}
