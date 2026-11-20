//! `scene/resources/3d/importer_mesh.*`: import-time mesh surface storage.
//!
//! Importers build an `ImporterMesh` instead of an `ArrayMesh` so the data is
//! never registered with the rendering server. This port keeps the surface
//! tables and the engine's validation:
//!
//! - blend shapes can only be declared while there are no surfaces, and their
//!   names have `:` replaced with `_`;
//! - `add_surface` requires the array list to be `ARRAY_MAX` (13) entries, one
//!   blend-shape array list per declared shape (each also 13 entries), a
//!   non-empty vertex array and blend-shape vertex counts matching the surface.
//!
//! Materials and the shadow mesh are stored as `Rid` handles / skipped; LOD
//! generation, index optimisation, lightmap unwrapping and the conversion to
//! `ArrayMesh` are not ported.

use crate::math::vector2i::Vector2i;
use crate::servers::rid::Rid;
use crate::variant::Variant;

/// `Mesh::ARRAY_MAX`.
pub const ARRAY_MAX: usize = 13;
/// `Mesh::ARRAY_VERTEX`.
pub const ARRAY_VERTEX: usize = 0;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BlendShapeMode {
    Normalized,
    Relative,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum PrimitiveType {
    Points,
    Lines,
    LineStrip,
    Triangles,
    TriangleStrip,
}

#[derive(Clone, Debug, Default)]
pub struct ImporterLod {
    pub indices: Vec<i32>,
    pub distance: f32,
}

#[derive(Clone, Debug)]
pub struct ImporterSurface {
    pub primitive: PrimitiveType,
    pub arrays: Vec<Variant>,
    pub blend_shape_data: Vec<Vec<Variant>>,
    pub lods: Vec<ImporterLod>,
    pub material: Option<Rid>,
    pub name: String,
    pub flags: u64,
}

impl Default for ImporterSurface {
    fn default() -> Self {
        Self {
            primitive: PrimitiveType::Triangles,
            arrays: vec![Variant::Nil; ARRAY_MAX],
            blend_shape_data: Vec::new(),
            lods: Vec::new(),
            material: None,
            name: String::new(),
            flags: 0,
        }
    }
}

#[derive(Clone, Debug)]
pub struct ImporterMeshData {
    surfaces: Vec<ImporterSurface>,
    blend_shapes: Vec<String>,
    blend_shape_mode: BlendShapeMode,
    lightmap_size_hint: Vector2i,
}

impl Default for ImporterMeshData {
    fn default() -> Self {
        Self::new()
    }
}

/// Vertex count of an array list, using the packed vertex array's length.
fn vertex_count(arrays: &[Variant]) -> usize {
    match arrays.get(ARRAY_VERTEX) {
        Some(Variant::PackedVector3Array(points)) => points.len(),
        _ => 0,
    }
}

impl ImporterMeshData {
    pub fn new() -> Self {
        Self {
            surfaces: Vec::new(),
            blend_shapes: Vec::new(),
            blend_shape_mode: BlendShapeMode::Normalized,
            lightmap_size_hint: Vector2i::ZERO,
        }
    }

    /// `:` is the separator used by the mesh format, so it becomes `_`.
    pub fn validate_blend_shape_name(name: &str) -> String {
        name.replace(':', "_")
    }

    /// Fails once any surface exists (`ERR_FAIL_COND(surfaces.size() > 0)`).
    pub fn add_blend_shape(&mut self, name: &str) -> bool {
        if !self.surfaces.is_empty() {
            return false;
        }
        self.blend_shapes
            .push(Self::validate_blend_shape_name(name));
        true
    }

    pub fn get_blend_shape_count(&self) -> usize {
        self.blend_shapes.len()
    }

    pub fn get_blend_shape_name(&self, shape: usize) -> String {
        self.blend_shapes.get(shape).cloned().unwrap_or_default()
    }

    pub fn set_blend_shape_mode(&mut self, mode: BlendShapeMode) {
        self.blend_shape_mode = mode;
    }

    pub fn get_blend_shape_mode(&self) -> BlendShapeMode {
        self.blend_shape_mode
    }

    /// Adds a surface, rejecting every shape mismatch the engine checks.
    pub fn add_surface(
        &mut self,
        primitive: PrimitiveType,
        arrays: Vec<Variant>,
        blend_shape_arrays: Vec<Vec<Variant>>,
        lods: Vec<ImporterLod>,
        material: Option<Rid>,
        name: &str,
        flags: u64,
    ) -> bool {
        if blend_shape_arrays.len() != self.blend_shapes.len() {
            return false;
        }
        if arrays.len() != ARRAY_MAX {
            return false;
        }
        let vertices = vertex_count(&arrays);
        if vertices == 0 {
            return false;
        }
        for shape in &blend_shape_arrays {
            if shape.len() != ARRAY_MAX || vertex_count(shape) != vertices {
                return false;
            }
        }

        self.surfaces.push(ImporterSurface {
            primitive,
            arrays,
            blend_shape_data: blend_shape_arrays,
            lods,
            material,
            name: name.to_string(),
            flags,
        });
        true
    }

    pub fn get_surface_count(&self) -> usize {
        self.surfaces.len()
    }

    pub fn get_surface(&self, surface: usize) -> Option<&ImporterSurface> {
        self.surfaces.get(surface)
    }

    pub fn get_surface_primitive_type(&self, surface: usize) -> Option<PrimitiveType> {
        self.surfaces.get(surface).map(|s| s.primitive)
    }

    pub fn get_surface_name(&self, surface: usize) -> String {
        self.surfaces
            .get(surface)
            .map(|s| s.name.clone())
            .unwrap_or_default()
    }

    pub fn set_surface_name(&mut self, surface: usize, name: &str) -> bool {
        match self.surfaces.get_mut(surface) {
            Some(s) => {
                s.name = name.to_string();
                true
            }
            None => false,
        }
    }

    pub fn get_surface_arrays(&self, surface: usize) -> Option<&[Variant]> {
        self.surfaces.get(surface).map(|s| s.arrays.as_slice())
    }

    pub fn get_surface_blend_shape_arrays(
        &self,
        surface: usize,
        shape: usize,
    ) -> Option<&[Variant]> {
        self.surfaces
            .get(surface)
            .and_then(|s| s.blend_shape_data.get(shape))
            .map(|a| a.as_slice())
    }

    pub fn get_surface_lod_count(&self, surface: usize) -> usize {
        self.surfaces
            .get(surface)
            .map(|s| s.lods.len())
            .unwrap_or(0)
    }

    pub fn get_surface_lod_indices(&self, surface: usize, lod: usize) -> Option<&[i32]> {
        self.surfaces
            .get(surface)
            .and_then(|s| s.lods.get(lod))
            .map(|l| l.indices.as_slice())
    }

    pub fn get_surface_lod_size(&self, surface: usize, lod: usize) -> f32 {
        self.surfaces
            .get(surface)
            .and_then(|s| s.lods.get(lod))
            .map(|l| l.distance)
            .unwrap_or(0.0)
    }

    pub fn get_surface_format(&self, surface: usize) -> Option<u64> {
        self.surfaces.get(surface).map(|s| s.flags)
    }

    pub fn get_surface_material(&self, surface: usize) -> Option<Rid> {
        self.surfaces.get(surface).and_then(|s| s.material)
    }

    pub fn set_surface_material(&mut self, surface: usize, material: Option<Rid>) -> bool {
        match self.surfaces.get_mut(surface) {
            Some(s) => {
                s.material = material;
                true
            }
            None => false,
        }
    }

    pub fn set_lightmap_size_hint(&mut self, size: Vector2i) {
        self.lightmap_size_hint = size;
    }

    pub fn get_lightmap_size_hint(&self) -> Vector2i {
        self.lightmap_size_hint
    }

    pub fn clear(&mut self) {
        self.surfaces.clear();
        self.blend_shapes.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vector3::Vector3;

    fn arrays_with(vertices: &[Vector3]) -> Vec<Variant> {
        let mut arrays = vec![Variant::Nil; ARRAY_MAX];
        arrays[ARRAY_VERTEX] = Variant::PackedVector3Array(vertices.to_vec());
        arrays
    }

    #[test]
    fn blend_shape_names_are_sanitised_and_locked_after_surfaces() {
        let mut m = ImporterMeshData::new();
        assert_eq!(ImporterMeshData::validate_blend_shape_name("a:b"), "a_b");
        assert_eq!(ImporterMeshData::validate_blend_shape_name("ok"), "ok");

        assert!(m.add_blend_shape("smile:wide"));
        assert_eq!(m.get_blend_shape_count(), 1);
        assert_eq!(m.get_blend_shape_name(0), "smile_wide");
        assert_eq!(m.get_blend_shape_name(5), "");
        assert_eq!(m.get_blend_shape_mode(), BlendShapeMode::Normalized);

        let arrays = arrays_with(&[Vector3::ZERO]);
        assert!(m.add_surface(PrimitiveType::Triangles, arrays, vec![Vec::new()], Vec::new(), None, "s", 0)
            == false, "blend shape vertex count mismatch");
    }

    #[test]
    fn surfaces_validate_arrays_and_blend_shapes() {
        let mut m = ImporterMeshData::new();
        assert!(m.add_blend_shape("smile"));
        let arrays = arrays_with(&[Vector3::ZERO, Vector3::ONE, Vector3::UP]);
        let shape = arrays_with(&[Vector3::ZERO, Vector3::ONE, Vector3::UP]);

        // Wrong array count.
        assert!(!m.add_surface(PrimitiveType::Triangles, vec![Variant::Nil], vec![shape.clone()], Vec::new(), None, "s", 0));
        // Wrong blend-shape count (none supplied for one declared shape).
        assert!(!m.add_surface(PrimitiveType::Triangles, arrays.clone(), Vec::new(), Vec::new(), None, "s", 0));
        // Empty vertex array.
        let empty = arrays_with(&[]);
        assert!(!m.add_surface(PrimitiveType::Triangles, empty, vec![shape.clone()], Vec::new(), None, "s", 0));
        // Blend shape with a different vertex count.
        let short = arrays_with(&[Vector3::ZERO]);
        assert!(!m.add_surface(PrimitiveType::Triangles, arrays.clone(), vec![short], Vec::new(), None, "s", 0));

        // Everything matching is accepted.
        assert!(m.add_surface(
            PrimitiveType::Triangles,
            arrays.clone(),
            vec![shape.clone()],
            vec![ImporterLod { indices: vec![0, 1, 2], distance: 0.5 }],
            Some(Rid::from_u64(4)),
            "body",
            7,
        ));
        assert_eq!(m.get_surface_count(), 1);
        assert_eq!(m.get_surface_primitive_type(0), Some(PrimitiveType::Triangles));
        assert_eq!(m.get_surface_name(0), "body");
        assert_eq!(m.get_surface_arrays(0).map(|a| a.len()), Some(ARRAY_MAX));
        assert_eq!(
            m.get_surface_blend_shape_arrays(0, 0).map(|a| a.len()),
            Some(ARRAY_MAX)
        );
        assert_eq!(m.get_surface_lod_count(0), 1);
        assert_eq!(m.get_surface_lod_indices(0, 0), Some(&[0, 1, 2][..]));
        assert_eq!(m.get_surface_lod_size(0, 0), 0.5);
        assert_eq!(m.get_surface_format(0), Some(7));
        assert_eq!(m.get_surface_material(0), Some(Rid::from_u64(4)));

        // Blend shapes can no longer be added once a surface exists.
        assert!(!m.add_blend_shape("late"));
        assert_eq!(m.get_blend_shape_count(), 1);
    }

    #[test]
    fn surface_accessors_validate_indices() {
        let mut m = ImporterMeshData::new();
        assert_eq!(m.get_surface_count(), 0);
        assert_eq!(m.get_surface_name(0), "");
        assert_eq!(m.get_surface_primitive_type(0), None);
        assert_eq!(m.get_surface_arrays(0), None);
        assert_eq!(m.get_surface_blend_shape_arrays(0, 0), None);
        assert_eq!(m.get_surface_lod_count(0), 0);
        assert_eq!(m.get_surface_lod_indices(0, 0), None);
        assert_eq!(m.get_surface_lod_size(0, 0), 0.0);
        assert_eq!(m.get_surface_format(0), None);
        assert_eq!(m.get_surface_material(0), None);
        assert!(!m.set_surface_name(0, "x"));
        assert!(!m.set_surface_material(0, None));

        m.add_blend_shape("a");
        let shape = arrays_with(&[Vector3::ZERO]);
        m.add_surface(PrimitiveType::Points, shape.clone(), vec![shape], Vec::new(), None, "p", 0);
        assert!(m.set_surface_name(0, "renamed"));
        assert_eq!(m.get_surface_name(0), "renamed");
        assert!(m.set_surface_material(0, Some(Rid::from_u64(2))));
        assert_eq!(m.get_surface_material(0), Some(Rid::from_u64(2)));

        m.set_lightmap_size_hint(Vector2i::new(64, 32));
        assert_eq!(m.get_lightmap_size_hint(), Vector2i::new(64, 32));

        m.clear();
        assert_eq!(m.get_surface_count(), 0);
        assert_eq!(m.get_blend_shape_count(), 0);
    }

    #[test]
    fn primitive_enum_values() {
        assert_eq!(PrimitiveType::Points as i32, 0);
        assert_eq!(PrimitiveType::Lines as i32, 1);
        assert_eq!(PrimitiveType::LineStrip as i32, 2);
        assert_eq!(PrimitiveType::Triangles as i32, 3);
        assert_eq!(PrimitiveType::TriangleStrip as i32, 4);
        assert_eq!(BlendShapeMode::Normalized as i32, 0);
        assert_eq!(BlendShapeMode::Relative as i32, 1);
    }
}
