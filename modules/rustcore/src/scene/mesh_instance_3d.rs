//! `scene/3d/mesh_instance_3d.*`: mesh, skin and blend-shape state.
//!
//! `MeshInstance3D` extends `GeometryInstance3D` with the mesh/skin handles, the
//! skeleton path, blend-shape tracks (one float per shape with a name->index
//! map) and per-surface override materials. Indexed accessors validate and
//! return the engine defaults; `set_blend_shape_value` rejects out-of-range
//! indices.
//!
//! Skin references, `create_trimesh_collision` and the RenderingServer updates
//! are not ported.

use std::collections::HashMap;

use crate::scene::visual_instance_3d::GeometryInstance3DData;
use crate::servers::rid::Rid;
use crate::string::string_name::StringName;

#[derive(Clone, Debug, Default)]
pub struct MeshInstance3DData {
    pub geometry: GeometryInstance3DData,
    mesh: Option<Rid>,
    skin: Option<Rid>,
    skin_internal: Option<Rid>,
    skin_ref: Option<Rid>,
    skeleton_path: String,
    blend_shape_tracks: Vec<f32>,
    blend_shape_properties: HashMap<StringName, usize>,
    surface_override_materials: Vec<Option<Rid>>,
}

impl MeshInstance3DData {
    pub fn new() -> Self {
        Self::default()
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

    pub fn get_skin_internal(&self) -> Option<Rid> {
        self.skin_internal
    }

    pub fn get_skin_reference(&self) -> Option<Rid> {
        self.skin_ref
    }

    pub fn set_skeleton_path(&mut self, path: &str) {
        self.skeleton_path = path.to_string();
    }

    pub fn get_skeleton_path(&self) -> &str {
        &self.skeleton_path
    }

    /// Number of blend-shape tracks.
    pub fn get_blend_shape_count(&self) -> usize {
        self.blend_shape_tracks.len()
    }

    /// Registers a blend shape name (the mesh normally does this on assignment).
    pub fn add_blend_shape(&mut self, name: &str) {
        let index = self.blend_shape_tracks.len();
        self.blend_shape_tracks.push(0.0);
        self.blend_shape_properties
            .insert(StringName::new(name), index);
    }

    pub fn find_blend_shape_by_name(&self, name: &str) -> Option<usize> {
        self.blend_shape_properties
            .get(&StringName::new(name))
            .copied()
    }

    pub fn get_blend_shape_value(&self, index: usize) -> Option<f32> {
        self.blend_shape_tracks.get(index).copied()
    }

    pub fn set_blend_shape_value(&mut self, index: usize, value: f32) -> bool {
        match self.blend_shape_tracks.get_mut(index) {
            Some(v) => {
                *v = value;
                true
            }
            None => false,
        }
    }

    pub fn get_surface_override_material_count(&self) -> usize {
        self.surface_override_materials.len()
    }

    pub fn set_surface_override_material(&mut self, surface: usize, material: Option<Rid>) -> bool {
        match self.surface_override_materials.get_mut(surface) {
            Some(m) => {
                *m = material;
                true
            }
            None => false,
        }
    }

    pub fn get_surface_override_material(&self, surface: usize) -> Option<Rid> {
        self.surface_override_materials
            .get(surface)
            .copied()
            .flatten()
    }

    /// Grows the override list to `count` (normally derived from the mesh).
    pub fn set_surface_count(&mut self, count: usize) {
        self.surface_override_materials.resize(count, None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults() {
        let m = MeshInstance3DData::new();
        assert!(m.get_mesh().is_none());
        assert!(m.get_skin().is_none());
        assert!(m.get_skin_internal().is_none());
        assert!(m.get_skin_reference().is_none());
        assert_eq!(m.get_skeleton_path(), "");
        assert_eq!(m.get_blend_shape_count(), 0);
        assert_eq!(m.get_surface_override_material_count(), 0);
        assert_eq!(m.geometry.get_cast_shadows_setting(), crate::scene::mesh_library::ShadowCastingSetting::On);
    }

    #[test]
    fn mesh_skin_and_path() {
        let mut m = MeshInstance3DData::new();
        m.set_mesh(Some(Rid::from_u64(1)));
        m.set_skin(Some(Rid::from_u64(2)));
        m.set_skeleton_path("../Skeleton3D");
        assert_eq!(m.get_mesh(), Some(Rid::from_u64(1)));
        assert_eq!(m.get_skin(), Some(Rid::from_u64(2)));
        assert_eq!(m.get_skeleton_path(), "../Skeleton3D");

        m.set_mesh(None);
        assert!(m.get_mesh().is_none());
    }

    #[test]
    fn blend_shapes() {
        let mut m = MeshInstance3DData::new();
        m.add_blend_shape("smile");
        m.add_blend_shape("frown");
        assert_eq!(m.get_blend_shape_count(), 2);
        assert_eq!(m.find_blend_shape_by_name("smile"), Some(0));
        assert_eq!(m.find_blend_shape_by_name("frown"), Some(1));
        assert_eq!(m.find_blend_shape_by_name("missing"), None);

        assert_eq!(m.get_blend_shape_value(0), Some(0.0));
        assert!(m.set_blend_shape_value(1, 0.75));
        assert_eq!(m.get_blend_shape_value(1), Some(0.75));
        assert!(!m.set_blend_shape_value(2, 1.0));
        assert_eq!(m.get_blend_shape_value(2), None);
    }

    #[test]
    fn surface_override_materials() {
        let mut m = MeshInstance3DData::new();
        m.set_surface_count(3);
        assert_eq!(m.get_surface_override_material_count(), 3);
        assert!(m.get_surface_override_material(0).is_none());

        assert!(m.set_surface_override_material(1, Some(Rid::from_u64(7))));
        assert_eq!(m.get_surface_override_material(1), Some(Rid::from_u64(7)));
        assert!(!m.set_surface_override_material(3, Some(Rid::from_u64(7))));
        assert!(m.get_surface_override_material(3).is_none());

        // Clearing one surface leaves the others.
        assert!(m.set_surface_override_material(1, None));
        assert!(m.get_surface_override_material(1).is_none());
        assert_eq!(m.get_surface_override_material_count(), 3);
    }
}
