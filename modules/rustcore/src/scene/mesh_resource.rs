//! `scene/resources/mesh.*`: the `Mesh` base data (surfaces, formats, AABB).
//!
//! The engine's `Mesh` is abstract: subclasses implement `_surface_get_*`. This
//! port stores the surface table directly (`MeshSurface`) and implements the
//! base accessors on top of it, so the enums, format bits and derived helpers
//! work without an `ArrayMesh`.
//!
//! Ported: `PrimitiveType`, `ArrayType`, `ArrayCustomFormat`, the `ArrayFormat`
//! bit constants, `BlendShapeMode`, per-surface arrays/format/primitive/material/
//! name/LOD storage, blend-shape names and mode, the lightmap size hint and the
//! cached AABB. `get_faces` builds the triangle soup from the vertex and index
//! arrays like the engine.
//!
//! Subclass-specific pieces (surface creation, shadow meshes, convex/trimesh
//! shape creation, navigation meshes) are not ported.

use crate::math::aabb::Aabb;
use crate::math::vector3::Vector3;
use crate::servers::rid::Rid;
use crate::variant::Variant;

/// `Mesh::PrimitiveType` (matches `RenderingServer::PrimitiveType`).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum PrimitiveType {
    Points,
    Lines,
    LineStrip,
    Triangles,
    TriangleStrip,
    Max,
}

/// `Mesh::BlendShapeMode`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum BlendShapeMode {
    Normalized,
    Relative,
}

/// `Mesh::ArrayType`: the slots every surface array must fill.
pub const ARRAY_VERTEX: usize = 0;
pub const ARRAY_NORMAL: usize = 1;
pub const ARRAY_TANGENT: usize = 2;
pub const ARRAY_COLOR: usize = 3;
pub const ARRAY_TEX_UV: usize = 4;
pub const ARRAY_TEX_UV2: usize = 5;
pub const ARRAY_CUSTOM0: usize = 6;
pub const ARRAY_CUSTOM1: usize = 7;
pub const ARRAY_CUSTOM2: usize = 8;
pub const ARRAY_CUSTOM3: usize = 9;
pub const ARRAY_BONES: usize = 10;
pub const ARRAY_WEIGHTS: usize = 11;
pub const ARRAY_INDEX: usize = 12;
pub const ARRAY_MAX: usize = 13;

/// `Mesh::ArrayCustomFormat`.
pub const ARRAY_CUSTOM_RGBA8_UNORM: u64 = 0;
pub const ARRAY_CUSTOM_RGBA8_SNORM: u64 = 1;
pub const ARRAY_CUSTOM_RG_HALF: u64 = 2;
pub const ARRAY_CUSTOM_RGBA_HALF: u64 = 3;
pub const ARRAY_CUSTOM_R_FLOAT: u64 = 4;
pub const ARRAY_CUSTOM_RG_FLOAT: u64 = 5;
pub const ARRAY_CUSTOM_RGB_FLOAT: u64 = 6;
pub const ARRAY_CUSTOM_RGBA_FLOAT: u64 = 7;
pub const ARRAY_CUSTOM_MAX: u64 = 8;

/// `Mesh::ArrayFormat` bits (`RenderingServer::ArrayFormat`).
pub const ARRAY_FORMAT_VERTEX: u64 = 1 << ARRAY_VERTEX;
pub const ARRAY_FORMAT_NORMAL: u64 = 1 << ARRAY_NORMAL;
pub const ARRAY_FORMAT_TANGENT: u64 = 1 << ARRAY_TANGENT;
pub const ARRAY_FORMAT_COLOR: u64 = 1 << ARRAY_COLOR;
pub const ARRAY_FORMAT_TEX_UV: u64 = 1 << ARRAY_TEX_UV;
pub const ARRAY_FORMAT_TEX_UV2: u64 = 1 << ARRAY_TEX_UV2;
pub const ARRAY_FORMAT_CUSTOM0: u64 = 1 << ARRAY_CUSTOM0;
pub const ARRAY_FORMAT_CUSTOM1: u64 = 1 << ARRAY_CUSTOM1;
pub const ARRAY_FORMAT_CUSTOM2: u64 = 1 << ARRAY_CUSTOM2;
pub const ARRAY_FORMAT_CUSTOM3: u64 = 1 << ARRAY_CUSTOM3;
pub const ARRAY_FORMAT_BONES: u64 = 1 << ARRAY_BONES;
pub const ARRAY_FORMAT_WEIGHTS: u64 = 1 << ARRAY_WEIGHTS;
pub const ARRAY_FORMAT_INDEX: u64 = 1 << ARRAY_INDEX;
pub const ARRAY_FORMAT_BLEND_SHAPE_MASK: u64 = 0b1111_1111_1000;
pub const ARRAY_FORMAT_CUSTOM_BASE: u64 = ARRAY_CUSTOM0 as u64;
pub const ARRAY_FORMAT_CUSTOM_BITS: u64 = 3;
pub const ARRAY_FORMAT_CUSTOM0_SHIFT: u64 = ARRAY_FORMAT_CUSTOM_BASE;
pub const ARRAY_FORMAT_CUSTOM2_SHIFT: u64 =
    ARRAY_FORMAT_CUSTOM0_SHIFT + 2 * ARRAY_FORMAT_CUSTOM_BITS;
pub const ARRAY_FORMAT_CUSTOM_MASK: u64 = (1 << ARRAY_FORMAT_CUSTOM_BITS) - 1;

/// `Mesh::NO_INDEX_ARRAY`.
pub const NO_INDEX_ARRAY: i32 = -1;

#[derive(Clone, Debug)]
pub struct MeshSurface {
    pub arrays: Vec<Variant>,
    pub format: u64,
    pub primitive: PrimitiveType,
    pub material: Option<Rid>,
    pub name: String,
    pub blend_shapes: Vec<Vec<Variant>>,
    /// `(indices, distance)` per LOD.
    pub lods: Vec<(Vec<i32>, f32)>,
}

impl Default for MeshSurface {
    fn default() -> Self {
        Self {
            arrays: vec![Variant::Nil; ARRAY_MAX],
            format: 0,
            primitive: PrimitiveType::Triangles,
            material: None,
            name: String::new(),
            blend_shapes: Vec::new(),
            lods: Vec::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct MeshData {
    surfaces: Vec<MeshSurface>,
    blend_shapes: Vec<String>,
    blend_shape_mode: BlendShapeMode,
    lightmap_size_hint: crate::math::vector2i::Vector2i,
    aabb: Aabb,
}

impl Default for MeshData {
    fn default() -> Self {
        Self::new()
    }
}

impl MeshData {
    pub fn new() -> Self {
        Self {
            surfaces: Vec::new(),
            blend_shapes: Vec::new(),
            blend_shape_mode: BlendShapeMode::Normalized,
            lightmap_size_hint: crate::math::vector2i::Vector2i::ZERO,
            aabb: Aabb::default(),
        }
    }

    /// Adds a surface as given (the engine's subclasses do this).
    pub fn add_surface(&mut self, surface: MeshSurface) {
        self.surfaces.push(surface);
    }

    pub fn get_surface_count(&self) -> usize {
        self.surfaces.len()
    }

    pub fn surface_get_arrays(&self, surface: usize) -> Option<&[Variant]> {
        self.surfaces.get(surface).map(|s| s.arrays.as_slice())
    }

    pub fn surface_get_array_len(&self, surface: usize) -> i32 {
        match self
            .surfaces
            .get(surface)
            .and_then(|s| s.arrays.get(ARRAY_VERTEX))
        {
            Some(Variant::PackedVector3Array(points)) => points.len() as i32,
            _ => 0,
        }
    }

    pub fn surface_get_array_index_len(&self, surface: usize) -> i32 {
        match self
            .surfaces
            .get(surface)
            .and_then(|s| s.arrays.get(ARRAY_INDEX))
        {
            Some(Variant::PackedInt32Array(indices)) => indices.len() as i32,
            _ => 0,
        }
    }

    pub fn surface_get_format(&self, surface: usize) -> u64 {
        self.surfaces.get(surface).map(|s| s.format).unwrap_or(0)
    }

    pub fn surface_get_primitive_type(&self, surface: usize) -> PrimitiveType {
        self.surfaces
            .get(surface)
            .map(|s| s.primitive)
            .unwrap_or(PrimitiveType::Max)
    }

    pub fn surface_set_material(&mut self, surface: usize, material: Option<Rid>) -> bool {
        match self.surfaces.get_mut(surface) {
            Some(s) => {
                s.material = material;
                true
            }
            None => false,
        }
    }

    pub fn surface_get_material(&self, surface: usize) -> Option<Rid> {
        self.surfaces.get(surface).and_then(|s| s.material)
    }

    pub fn surface_set_name(&mut self, surface: usize, name: &str) -> bool {
        match self.surfaces.get_mut(surface) {
            Some(s) => {
                s.name = name.to_string();
                true
            }
            None => false,
        }
    }

    pub fn surface_get_name(&self, surface: usize) -> String {
        self.surfaces
            .get(surface)
            .map(|s| s.name.clone())
            .unwrap_or_default()
    }

    pub fn surface_get_blend_shape_arrays(&self, surface: usize, shape: usize) -> Option<&[Variant]> {
        self.surfaces
            .get(surface)
            .and_then(|s| s.blend_shapes.get(shape))
            .map(|a| a.as_slice())
    }

    pub fn surface_get_lods(&self, surface: usize) -> Option<&[(Vec<i32>, f32)]> {
        self.surfaces.get(surface).map(|s| s.lods.as_slice())
    }

    pub fn set_blend_shape_name(&mut self, index: usize, name: &str) -> bool {
        match self.blend_shapes.get_mut(index) {
            Some(slot) => {
                *slot = name.to_string();
                true
            }
            None => false,
        }
    }

    pub fn add_blend_shape(&mut self, name: &str) {
        self.blend_shapes.push(name.to_string());
    }

    pub fn get_blend_shape_count(&self) -> usize {
        self.blend_shapes.len()
    }

    pub fn get_blend_shape_name(&self, index: usize) -> String {
        self.blend_shapes.get(index).cloned().unwrap_or_default()
    }

    pub fn set_blend_shape_mode(&mut self, mode: BlendShapeMode) {
        self.blend_shape_mode = mode;
    }

    pub fn get_blend_shape_mode(&self) -> BlendShapeMode {
        self.blend_shape_mode
    }

    pub fn set_lightmap_size_hint(&mut self, size: crate::math::vector2i::Vector2i) {
        self.lightmap_size_hint = size;
    }

    pub fn get_lightmap_size_hint(&self) -> crate::math::vector2i::Vector2i {
        self.lightmap_size_hint
    }

    /// The cached AABB (`_get_aabb` in subclasses).
    pub fn get_aabb(&self) -> Aabb {
        self.aabb
    }

    pub fn set_aabb(&mut self, aabb: Aabb) {
        self.aabb = aabb;
    }

    /// `get_faces`: expands triangles (or triangle strips) into a vertex soup.
    pub fn get_faces(&self) -> Vec<Vector3> {
        let mut faces = Vec::new();
        for surface in &self.surfaces {
            if surface.primitive != PrimitiveType::Triangles
                && surface.primitive != PrimitiveType::TriangleStrip
            {
                continue;
            }
            let Some(Variant::PackedVector3Array(vertices)) = surface.arrays.get(ARRAY_VERTEX)
            else {
                continue;
            };
            let indices: Option<&Vec<i32>> = match surface.arrays.get(ARRAY_INDEX) {
                Some(Variant::PackedInt32Array(indices)) => Some(indices),
                _ => None,
            };
            match surface.primitive {
                PrimitiveType::Triangles => {
                    let count = indices.map(|i| i.len()).unwrap_or(vertices.len());
                    let mut i = 0;
                    while i + 2 < count {
                        for offset in 0..3 {
                            let index = match indices {
                                Some(list) => list[i + offset] as usize,
                                None => i + offset,
                            };
                            if let Some(vertex) = vertices.get(index) {
                                faces.push(*vertex);
                            }
                        }
                        i += 3;
                    }
                }
                PrimitiveType::TriangleStrip => {
                    let count = indices.map(|i| i.len()).unwrap_or(vertices.len());
                    if count < 3 {
                        continue;
                    }
                    for i in 0..count - 2 {
                        // Alternate winding to keep the strip consistent.
                        let tri = if i % 2 == 0 {
                            [i, i + 1, i + 2]
                        } else {
                            [i + 1, i, i + 2]
                        };
                        for &position in &tri {
                            let index = match indices {
                                Some(list) => list[position] as usize,
                                None => position,
                            };
                            if let Some(vertex) = vertices.get(index) {
                                faces.push(*vertex);
                            }
                        }
                    }
                }
                _ => {}
            }
        }
        faces
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::vector2i::Vector2i;

    fn surface(vertices: Vec<Vector3>, indices: Vec<i32>) -> MeshSurface {
        let mut arrays = vec![Variant::Nil; ARRAY_MAX];
        arrays[ARRAY_VERTEX] = Variant::PackedVector3Array(vertices);
        arrays[ARRAY_INDEX] = Variant::PackedInt32Array(indices);
        MeshSurface {
            arrays,
            format: ARRAY_FORMAT_VERTEX | ARRAY_FORMAT_INDEX,
            primitive: PrimitiveType::Triangles,
            ..MeshSurface::default()
        }
    }

    #[test]
    fn surface_accessors() {
        let mut m = MeshData::new();
        assert_eq!(m.get_surface_count(), 0);
        assert_eq!(m.surface_get_array_len(0), 0);
        assert_eq!(m.surface_get_format(0), 0);
        assert_eq!(m.surface_get_primitive_type(0), PrimitiveType::Max);
        assert!(m.surface_get_arrays(0).is_none());

        m.add_surface(surface(
            vec![
                Vector3::ZERO,
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(0.0, 0.0, 1.0),
            ],
            vec![0, 1, 2],
        ));
        assert_eq!(m.get_surface_count(), 1);
        assert_eq!(m.surface_get_array_len(0), 3);
        assert_eq!(m.surface_get_array_index_len(0), 3);
        assert_eq!(
            m.surface_get_format(0),
            ARRAY_FORMAT_VERTEX | ARRAY_FORMAT_INDEX
        );
        assert_eq!(m.surface_get_primitive_type(0), PrimitiveType::Triangles);
        assert_eq!(m.surface_get_arrays(0).map(|a| a.len()), Some(ARRAY_MAX));
        assert_eq!(m.surface_get_name(0), "");

        assert!(m.surface_set_material(0, Some(Rid::from_u64(4))));
        assert_eq!(m.surface_get_material(0), Some(Rid::from_u64(4)));
        assert!(m.surface_set_name(0, "body"));
        assert_eq!(m.surface_get_name(0), "body");

        assert!(!m.surface_set_material(9, None));
        assert!(!m.surface_set_name(9, "x"));
        assert_eq!(m.surface_get_material(9), None);
        assert_eq!(m.surface_get_name(9), "");
    }

    #[test]
    fn blend_shapes_and_hints() {
        let mut m = MeshData::new();
        assert_eq!(m.get_blend_shape_count(), 0);
        assert_eq!(m.get_blend_shape_mode(), BlendShapeMode::Normalized);
        assert_eq!(m.get_blend_shape_name(0), "");

        m.add_blend_shape("smile");
        assert_eq!(m.get_blend_shape_count(), 1);
        assert_eq!(m.get_blend_shape_name(0), "smile");
        assert!(m.set_blend_shape_name(0, "frown"));
        assert!(!m.set_blend_shape_name(4, "x"));
        assert_eq!(m.get_blend_shape_name(0), "frown");

        m.set_blend_shape_mode(BlendShapeMode::Relative);
        assert_eq!(m.get_blend_shape_mode(), BlendShapeMode::Relative);

        m.set_lightmap_size_hint(Vector2i::new(64, 32));
        assert_eq!(m.get_lightmap_size_hint(), Vector2i::new(64, 32));

        m.set_aabb(Aabb::new(Vector3::ZERO, Vector3::ONE));
        assert_eq!(m.get_aabb().size, Vector3::ONE);
    }

    #[test]
    fn faces_from_triangles_and_strips() {
        let mut m = MeshData::new();
        let quad = vec![
            Vector3::ZERO,
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        ];

        m.add_surface(surface(quad.clone(), vec![0, 1, 2, 0, 2, 3]));
        let faces = m.get_faces();
        assert_eq!(faces.len(), 6, "two triangles");
        assert_eq!(faces[0], Vector3::ZERO);
        assert_eq!(faces[5], Vector3::new(0.0, 1.0, 0.0));

        // Without indices the vertices are read in order.
        let mut m = MeshData::new();
        let mut s = surface(quad.clone(), vec![]);
        s.arrays[ARRAY_INDEX] = Variant::Nil;
        s.format = ARRAY_FORMAT_VERTEX;
        m.add_surface(s);
        assert_eq!(m.get_faces().len(), 3, "one triangle from four vertices");

        // Strips produce count - 2 triangles with alternating winding.
        let mut m = MeshData::new();
        let mut strip = surface(quad.clone(), vec![]);
        strip.arrays[ARRAY_INDEX] = Variant::Nil;
        strip.primitive = PrimitiveType::TriangleStrip;
        m.add_surface(strip);
        let faces = m.get_faces();
        assert_eq!(faces.len(), 6);
        // The second triangle uses the flipped order.
        assert_eq!(faces[3], quad[2]);
        assert_eq!(faces[4], quad[1]);

        // Non-triangle primitives contribute nothing.
        let mut m = MeshData::new();
        let mut lines = surface(quad, vec![]);
        lines.primitive = PrimitiveType::Lines;
        m.add_surface(lines);
        assert!(m.get_faces().is_empty());
    }

    #[test]
    fn enum_values_and_format_bits() {
        assert_eq!(PrimitiveType::Points as i32, 0);
        assert_eq!(PrimitiveType::Triangles as i32, 3);
        assert_eq!(PrimitiveType::Max as i32, 5);
        assert_eq!(BlendShapeMode::Relative as i32, 1);
        assert_eq!(ARRAY_VERTEX, 0);
        assert_eq!(ARRAY_INDEX, 12);
        assert_eq!(ARRAY_MAX, 13);
        assert_eq!(ARRAY_FORMAT_VERTEX, 1);
        assert_eq!(ARRAY_FORMAT_INDEX, 1 << 12);
        assert_eq!(NO_INDEX_ARRAY, -1);
        assert_eq!(ARRAY_CUSTOM_MAX, 8);
        assert_eq!(ARRAY_FORMAT_CUSTOM0_SHIFT, 6);
    }
}
