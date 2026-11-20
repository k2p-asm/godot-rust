//! `scene/3d/importer_mesh_instance_3d.*`: imported mesh instance state.
//!
//! Stores an `ImporterMesh` handle, an optional skin, the skeleton path,
//! per-surface material overrides (the list grows to the requested index), the
//! layer mask (1), shadow casting (on) and the visibility range/margins plus
//! fade mode (disabled). No rendering or importing is ported.

use crate::math::vector2::Vector2;
use crate::servers::rid::Rid;
use crate::scene::visual_instance_3d::{ShadowCastingSetting, VisibilityRangeFadeMode};

#[derive(Clone, Debug, Default)]
pub struct ImporterMeshInstance3DData {
    mesh: Option<Rid>,
    skin: Option<Rid>,
    skeleton_path: String,
    surface_materials: Vec<Option<Rid>>,
    layer_mask: u32,
    shadow_casting_setting: ShadowCastingSetting,
    visibility_range_begin: f32,
    visibility_range_end: f32,
    visibility_range_begin_margin: f32,
    visibility_range_end_margin: f32,
    visibility_range_fade_mode: VisibilityRangeFadeMode,
}

impl ImporterMeshInstance3DData {
    pub fn new() -> Self {
        Self {
            mesh: None,
            skin: None,
            skeleton_path: String::new(),
            surface_materials: Vec::new(),
            layer_mask: 1,
            shadow_casting_setting: ShadowCastingSetting::On,
            visibility_range_begin: 0.0,
            visibility_range_end: 0.0,
            visibility_range_begin_margin: 0.0,
            visibility_range_end_margin: 0.0,
            visibility_range_fade_mode: VisibilityRangeFadeMode::Disabled,
        }
    }

    pub fn set_mesh(&mut self, mesh: Option<Rid>) {
        self.mesh = mesh;
    }

    pub fn get_mesh(&self) -> Option<Rid> {
        self.mesh
    }

    pub fn set_skin(&mut self, skin: Option<Rid>) {
        self.skin = skin;
    }

    pub fn get_skin(&self) -> Option<Rid> {
        self.skin
    }

    pub fn set_skeleton_path(&mut self, path: &str) {
        self.skeleton_path = path.to_string();
    }

    pub fn get_skeleton_path(&self) -> &str {
        &self.skeleton_path
    }

    /// Negative indices are rejected; larger indices grow the list with nulls.
    pub fn set_surface_material(&mut self, index: i32, material: Option<Rid>) -> bool {
        if index < 0 {
            return false;
        }
        let index = index as usize;
        if index >= self.surface_materials.len() {
            self.surface_materials.resize(index + 1, None);
        }
        self.surface_materials[index] = material;
        true
    }

    pub fn get_surface_material(&self, index: i32) -> Option<Rid> {
        if index < 0 {
            return None;
        }
        self.surface_materials
            .get(index as usize)
            .copied()
            .flatten()
    }

    pub fn get_surface_material_count(&self) -> usize {
        self.surface_materials.len()
    }

    pub fn set_layer_mask(&mut self, mask: u32) {
        self.layer_mask = mask;
    }

    pub fn get_layer_mask(&self) -> u32 {
        self.layer_mask
    }

    pub fn set_cast_shadows_setting(&mut self, setting: ShadowCastingSetting) {
        self.shadow_casting_setting = setting;
    }

    pub fn get_cast_shadows_setting(&self) -> ShadowCastingSetting {
        self.shadow_casting_setting
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

    /// Placeholder for the engine's `Vector2`-free API parity; returns zero.
    pub fn get_visibility_range_margins(&self) -> Vector2 {
        Vector2::new(
            self.visibility_range_begin_margin,
            self.visibility_range_end_margin,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let i = ImporterMeshInstance3DData::new();
        assert!(i.get_mesh().is_none());
        assert!(i.get_skin().is_none());
        assert_eq!(i.get_skeleton_path(), "");
        assert_eq!(i.get_surface_material_count(), 0);
        assert_eq!(i.get_layer_mask(), 1);
        assert_eq!(i.get_cast_shadows_setting(), ShadowCastingSetting::On);
        assert_eq!(i.get_visibility_range_begin(), 0.0);
        assert_eq!(i.get_visibility_range_end(), 0.0);
        assert_eq!(i.get_visibility_range_begin_margin(), 0.0);
        assert_eq!(i.get_visibility_range_end_margin(), 0.0);
        assert_eq!(
            i.get_visibility_range_fade_mode(),
            VisibilityRangeFadeMode::Disabled
        );
    }

    #[test]
    fn surface_materials_grow_and_validate() {
        let mut i = ImporterMeshInstance3DData::new();
        assert!(!i.set_surface_material(-1, None));
        assert!(i.set_surface_material(2, Some(Rid::from_u64(8))));
        assert_eq!(i.get_surface_material_count(), 3);
        assert_eq!(i.get_surface_material(2), Some(Rid::from_u64(8)));
        assert_eq!(i.get_surface_material(0), None);
        assert_eq!(i.get_surface_material(9), None);
        assert_eq!(i.get_surface_material(-1), None);

        assert!(i.set_surface_material(0, Some(Rid::from_u64(1))));
        assert_eq!(i.get_surface_material(0), Some(Rid::from_u64(1)));
    }

    #[test]
    fn other_values() {
        let mut i = ImporterMeshInstance3DData::new();
        i.set_mesh(Some(Rid::from_u64(2)));
        i.set_skin(Some(Rid::from_u64(3)));
        i.set_skeleton_path("../Skeleton3D");
        i.set_layer_mask(0xFFFF);
        i.set_cast_shadows_setting(ShadowCastingSetting::Off);
        i.set_visibility_range_begin(5.0);
        i.set_visibility_range_end(50.0);
        i.set_visibility_range_begin_margin(1.0);
        i.set_visibility_range_end_margin(2.0);
        i.set_visibility_range_fade_mode(VisibilityRangeFadeMode::SelfOnly);

        assert_eq!(i.get_mesh(), Some(Rid::from_u64(2)));
        assert_eq!(i.get_skin(), Some(Rid::from_u64(3)));
        assert_eq!(i.get_skeleton_path(), "../Skeleton3D");
        assert_eq!(i.get_layer_mask(), 0xFFFF);
        assert_eq!(i.get_cast_shadows_setting(), ShadowCastingSetting::Off);
        assert_eq!(i.get_visibility_range_begin(), 5.0);
        assert_eq!(i.get_visibility_range_end(), 50.0);
        assert_eq!(i.get_visibility_range_margins(), Vector2::new(1.0, 2.0));
        assert_eq!(i.get_visibility_range_fade_mode(), VisibilityRangeFadeMode::SelfOnly);
    }
}
