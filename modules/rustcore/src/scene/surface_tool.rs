//! `scene/resources/surface_tool.*`: vertex/index accumulation (subset).
//!
//! Ported: `begin`, `add_vertex` with the sticky attribute state, the
//! `set_color/normal/tangent/uv/uv2/smooth_group/bones/weights` setters (with
//! the engine's "first vertex or matching format bit" rule), `add_index`,
//! `index()` (dedupe by full vertex equality), `deindex()`, `generate_normals`
//! (per-face normals, smooth-group averaging, the `UINT32_MAX` "flat" group and
//! the optional flip), and `commit_to_arrays()` producing the points, normals,
//! tangents (four floats with the binormal sign), colors, UV/UV2 and index
//! arrays under the engine's `ARRAY_FORMAT_*` bits.
//!
//! Not ported: custom channels, mikktspace `generate_tangents`,
//! simplification/optimisation, `commit` into an `ArrayMesh`, and material
//! assignment beyond storing the handle.

use crate::math::color::Color;
use crate::math::plane::Plane;
use crate::math::vector2::Vector2;
use crate::math::vector3::Vector3;
use crate::scene::mesh_resource::{
    PrimitiveType, ARRAY_FORMAT_COLOR, ARRAY_FORMAT_INDEX, ARRAY_FORMAT_NORMAL,
    ARRAY_FORMAT_TANGENT, ARRAY_FORMAT_TEX_UV, ARRAY_FORMAT_TEX_UV2, ARRAY_FORMAT_VERTEX,
};
use crate::servers::rid::Rid;

/// `SurfaceTool::Vertex`.
#[derive(Clone, Debug, PartialEq)]
pub struct SurfaceVertex {
    pub smooth_group: u32,
    pub color: Color,
    pub normal: Vector3,
    pub binormal: Vector3,
    pub tangent: Vector3,
    pub uv: Vector2,
    pub uv2: Vector2,
    pub vertex: Vector3,
    pub bones: Vec<i32>,
    pub weights: Vec<f32>,
}

impl Default for SurfaceVertex {
    fn default() -> Self {
        Self {
            smooth_group: 0,
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            normal: Vector3::ZERO,
            binormal: Vector3::ZERO,
            tangent: Vector3::ZERO,
            uv: Vector2::ZERO,
            uv2: Vector2::ZERO,
            vertex: Vector3::ZERO,
            bones: Vec::new(),
            weights: Vec::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(i32)]
pub enum SkinWeightCount {
    Four,
    Eight,
}

/// The arrays `commit_to_arrays` would place in a mesh surface.
#[derive(Clone, Debug, Default)]
pub struct SurfaceArrays {
    pub format: u64,
    pub points: Vec<Vector3>,
    pub normals: Vec<Vector3>,
    /// Four floats per vertex (`xyz`, sign), only when tangents are in `format`.
    pub tangents: Vec<f32>,
    pub colors: Vec<Color>,
    pub uvs: Vec<Vector2>,
    pub uv2s: Vec<Vector2>,
    pub indices: Vec<i32>,
}

#[derive(Clone, Debug)]
pub struct SurfaceToolData {
    begun: bool,
    first: bool,
    primitive: PrimitiveType,
    format: u64,
    material: Option<Rid>,
    vertices: Vec<SurfaceVertex>,
    indices: Vec<i32>,
    last_color: Color,
    last_normal: Vector3,
    last_uv: Vector2,
    last_uv2: Vector2,
    last_bones: Vec<i32>,
    last_weights: Vec<f32>,
    last_tangent: Plane,
    last_smooth_group: u32,
    skin_weights: SkinWeightCount,
}

impl Default for SurfaceToolData {
    fn default() -> Self {
        Self::new()
    }
}

impl SurfaceToolData {
    pub fn new() -> Self {
        Self {
            begun: false,
            first: false,
            primitive: PrimitiveType::Lines,
            format: 0,
            material: None,
            vertices: Vec::new(),
            indices: Vec::new(),
            last_color: Color::new(1.0, 1.0, 1.0, 1.0),
            last_normal: Vector3::ZERO,
            last_uv: Vector2::ZERO,
            last_uv2: Vector2::ZERO,
            last_bones: Vec::new(),
            last_weights: Vec::new(),
            last_tangent: Plane::new(0.0, 0.0, 0.0, 0.0),
            last_smooth_group: 0,
            skin_weights: SkinWeightCount::Four,
        }
    }

    /// `begin`; the engine refuses to begin twice without `clear`.
    pub fn begin(&mut self, primitive: PrimitiveType) -> bool {
        if self.begun {
            return false;
        }
        self.begun = true;
        self.first = true;
        self.primitive = primitive;
        true
    }

    pub fn get_primitive_type(&self) -> PrimitiveType {
        self.primitive
    }

    pub fn is_begun(&self) -> bool {
        self.begun
    }

    pub fn set_material(&mut self, material: Option<Rid>) {
        self.material = material;
    }

    pub fn get_material(&self) -> Option<Rid> {
        self.material
    }

    /// `clear`.
    pub fn clear(&mut self) {
        *self = Self::new();
    }

    /// `add_vertex`: the sticky attributes are copied into the new vertex.
    pub fn add_vertex(&mut self, position: Vector3) -> bool {
        if !self.begun {
            return false;
        }
        if self.first {
            self.format |= ARRAY_FORMAT_VERTEX;
            self.first = false;
        }

        let tangent = self.last_tangent;
        let mut vertex = SurfaceVertex {
            smooth_group: self.last_smooth_group,
            color: self.last_color,
            normal: self.last_normal,
            binormal: self.last_normal.cross(tangent.normal).normalized() * tangent.d,
            tangent: tangent.normal,
            uv: self.last_uv,
            uv2: self.last_uv2,
            vertex: position,
            bones: self.last_bones.clone(),
            weights: self.last_weights.clone(),
        };

        if self.format & (crate::scene::mesh_resource::ARRAY_FORMAT_BONES
            | crate::scene::mesh_resource::ARRAY_FORMAT_WEIGHTS)
            != 0
        {
            let expected = match self.skin_weights {
                SkinWeightCount::Four => 4,
                SkinWeightCount::Eight => 8,
            };
            if vertex.weights.len() < expected {
                while vertex.weights.len() < expected {
                    vertex.weights.push(0.0);
                    vertex.bones.push(0);
                }
            } else if vertex.weights.len() > expected {
                vertex.weights.truncate(expected);
                vertex.bones.truncate(expected);
            }
        }

        self.vertices.push(vertex);
        true
    }

    fn allows(&self, flag: u64) -> bool {
        self.begun && (self.first || self.format & flag != 0)
    }

    pub fn set_color(&mut self, color: Color) -> bool {
        if !self.allows(ARRAY_FORMAT_COLOR) {
            return false;
        }
        self.format |= ARRAY_FORMAT_COLOR;
        self.last_color = color;
        true
    }

    pub fn set_normal(&mut self, normal: Vector3) -> bool {
        if !self.allows(ARRAY_FORMAT_NORMAL) {
            return false;
        }
        self.format |= ARRAY_FORMAT_NORMAL;
        self.last_normal = normal;
        true
    }

    /// `set_tangent`: the plane's `d` is the handedness sign.
    pub fn set_tangent(&mut self, tangent: Plane) -> bool {
        if !self.allows(ARRAY_FORMAT_TANGENT) {
            return false;
        }
        self.format |= ARRAY_FORMAT_TANGENT;
        self.last_tangent = tangent;
        true
    }

    pub fn set_uv(&mut self, uv: Vector2) -> bool {
        if !self.allows(ARRAY_FORMAT_TEX_UV) {
            return false;
        }
        self.format |= ARRAY_FORMAT_TEX_UV;
        self.last_uv = uv;
        true
    }

    pub fn set_uv2(&mut self, uv2: Vector2) -> bool {
        if !self.allows(ARRAY_FORMAT_TEX_UV2) {
            return false;
        }
        self.format |= ARRAY_FORMAT_TEX_UV2;
        self.last_uv2 = uv2;
        true
    }

    pub fn set_smooth_group(&mut self, group: u32) -> bool {
        if !self.begun {
            return false;
        }
        self.last_smooth_group = group;
        true
    }

    pub fn set_bones(&mut self, bones: Vec<i32>) -> bool {
        if !self.begun {
            return false;
        }
        self.format |= crate::scene::mesh_resource::ARRAY_FORMAT_BONES;
        self.last_bones = bones;
        true
    }

    pub fn set_weights(&mut self, weights: Vec<f32>) -> bool {
        if !self.begun {
            return false;
        }
        self.format |= crate::scene::mesh_resource::ARRAY_FORMAT_WEIGHTS;
        self.last_weights = weights;
        true
    }

    pub fn set_skin_weight_count(&mut self, count: SkinWeightCount) -> bool {
        if self.begun {
            return false;
        }
        self.skin_weights = count;
        true
    }

    pub fn get_skin_weight_count(&self) -> SkinWeightCount {
        self.skin_weights
    }

    pub fn add_index(&mut self, index: i32) -> bool {
        if !self.begun || index < 0 {
            return false;
        }
        self.format |= ARRAY_FORMAT_INDEX;
        self.indices.push(index);
        true
    }

    pub fn get_vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn get_index_count(&self) -> usize {
        self.indices.len()
    }

    /// `index()`: collapses equal vertices and builds the index list.
    pub fn index(&mut self) {
        if !self.indices.is_empty() {
            return;
        }
        let mut unique: Vec<SurfaceVertex> = Vec::new();
        let mut indices: Vec<i32> = Vec::with_capacity(self.vertices.len());
        for vertex in &self.vertices {
            match unique.iter().position(|existing| existing == vertex) {
                Some(index) => indices.push(index as i32),
                None => {
                    indices.push(unique.len() as i32);
                    unique.push(vertex.clone());
                }
            }
        }
        self.vertices = unique;
        self.indices = indices;
        self.format |= ARRAY_FORMAT_INDEX;
    }

    /// `deindex()`: expands the index list into plain vertices.
    pub fn deindex(&mut self) {
        if self.indices.is_empty() {
            return;
        }
        let old = self.vertices.clone();
        let mut vertices = Vec::with_capacity(self.indices.len());
        for &index in &self.indices {
            if let Some(vertex) = old.get(index as usize) {
                vertices.push(vertex.clone());
            }
        }
        self.vertices = vertices;
        self.indices.clear();
        self.format &= !ARRAY_FORMAT_INDEX;
    }

    /// `generate_normals`: per-face normals with smooth-group averaging.
    pub fn generate_normals(&mut self, flip: bool) -> bool {
        if self.primitive != PrimitiveType::Triangles {
            return false;
        }
        let was_indexed = !self.indices.is_empty();
        self.deindex();
        if self.vertices.len() % 3 != 0 {
            return false;
        }

        // Key: (position, smooth group).
        let mut smooth: Vec<((Vector3, u32), Vector3)> = Vec::new();
        let mut index = 0;
        while index < self.vertices.len() {
            let (v0, v1, v2) = (index, index + 1, index + 2);
            let normal = if !flip {
                Plane::from_points(
                    self.vertices[v0].vertex,
                    self.vertices[v1].vertex,
                    self.vertices[v2].vertex,
                )
                .normal
            } else {
                Plane::from_points(
                    self.vertices[v2].vertex,
                    self.vertices[v1].vertex,
                    self.vertices[v0].vertex,
                )
                .normal
            };

            for vertex_index in [v0, v1, v2] {
                if self.vertices[vertex_index].smooth_group != u32::MAX {
                    let key = (
                        self.vertices[vertex_index].vertex,
                        self.vertices[vertex_index].smooth_group,
                    );
                    match smooth.iter_mut().find(|(existing, _)| *existing == key) {
                        Some((_, accumulated)) => *accumulated += normal,
                        None => smooth.push((key, normal)),
                    }
                } else {
                    self.vertices[vertex_index].normal = normal;
                }
            }
            index += 3;
        }

        for vertex in self.vertices.iter_mut() {
            if vertex.smooth_group != u32::MAX {
                let key = (vertex.vertex, vertex.smooth_group);
                vertex.normal = match smooth.iter().find(|(existing, _)| *existing == key) {
                    Some((_, accumulated)) => accumulated.normalized(),
                    None => Vector3::ZERO,
                };
            }
        }

        self.format |= ARRAY_FORMAT_NORMAL;
        if was_indexed {
            self.index();
        }
        true
    }

    /// `commit_to_arrays`.
    pub fn commit_to_arrays(&self) -> SurfaceArrays {
        let mut arrays = SurfaceArrays {
            format: self.format,
            indices: self.indices.clone(),
            ..SurfaceArrays::default()
        };
        let count = self.vertices.len();
        if self.format & ARRAY_FORMAT_VERTEX != 0 {
            arrays.points = self.vertices.iter().map(|v| v.vertex).collect();
        }
        if self.format & ARRAY_FORMAT_NORMAL != 0 {
            arrays.normals = self.vertices.iter().map(|v| v.normal).collect();
        }
        if self.format & ARRAY_FORMAT_TANGENT != 0 {
            arrays.tangents.reserve(count * 4);
            for vertex in &self.vertices {
                arrays.tangents.push(vertex.tangent.x);
                arrays.tangents.push(vertex.tangent.y);
                arrays.tangents.push(vertex.tangent.z);
                let d = vertex.binormal.dot(vertex.normal.cross(vertex.tangent));
                arrays.tangents.push(if d < 0.0 { -1.0 } else { 1.0 });
            }
        }
        if self.format & ARRAY_FORMAT_COLOR != 0 {
            arrays.colors = self.vertices.iter().map(|v| v.color).collect();
        }
        if self.format & ARRAY_FORMAT_TEX_UV != 0 {
            arrays.uvs = self.vertices.iter().map(|v| v.uv).collect();
        }
        if self.format & ARRAY_FORMAT_TEX_UV2 != 0 {
            arrays.uv2s = self.vertices.iter().map(|v| v.uv2).collect();
        }
        arrays
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn triangle() -> SurfaceToolData {
        let mut tool = SurfaceToolData::new();
        assert!(tool.begin(PrimitiveType::Triangles));
        assert!(tool.set_normal(Vector3::new(0.0, 0.0, 1.0)));
        assert!(tool.set_uv(Vector2::new(0.0, 0.0)));
        assert!(tool.add_vertex(Vector3::new(0.0, 0.0, 0.0)));
        assert!(tool.add_vertex(Vector3::new(1.0, 0.0, 0.0)));
        assert!(tool.add_vertex(Vector3::new(0.0, 1.0, 0.0)));
        tool
    }

    #[test]
    fn begin_and_attribute_rules() {
        let mut tool = SurfaceToolData::new();
        assert!(!tool.begin(PrimitiveType::Triangles) == false);
        assert!(!tool.begin(PrimitiveType::Triangles), "already begun");
        // Before the first vertex any attribute is allowed.
        assert!(tool.set_color(Color::new(1.0, 0.0, 0.0, 1.0)));
        assert!(tool.add_vertex(Vector3::ZERO));
        // After it, only attributes already in the format are accepted.
        assert!(!tool.set_normal(Vector3::UP), "normal was not set first");
        assert!(tool.set_color(Color::new(0.0, 1.0, 0.0, 1.0)));
        assert!(tool.add_vertex(Vector3::ONE));
        assert_eq!(tool.get_vertex_count(), 2);
        let arrays = tool.commit_to_arrays();
        assert_eq!(arrays.colors.len(), 2);
        assert_eq!(arrays.colors[0], Color::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(arrays.colors[1], Color::new(0.0, 1.0, 0.0, 1.0));
        assert!(arrays.normals.is_empty());
    }

    #[test]
    fn commit_arrays_layout() {
        let tool = triangle();
        let arrays = tool.commit_to_arrays();
        assert_eq!(arrays.format & ARRAY_FORMAT_VERTEX, ARRAY_FORMAT_VERTEX);
        assert_eq!(arrays.format & ARRAY_FORMAT_NORMAL, ARRAY_FORMAT_NORMAL);
        assert_eq!(arrays.format & ARRAY_FORMAT_TEX_UV, ARRAY_FORMAT_TEX_UV);
        assert_eq!(arrays.format & ARRAY_FORMAT_INDEX, 0, "not indexed");
        assert_eq!(arrays.points.len(), 3);
        assert_eq!(arrays.normals.len(), 3);
        assert_eq!(arrays.uvs.len(), 3);
        assert_eq!(arrays.normals[0], Vector3::new(0.0, 0.0, 1.0));
        assert!(arrays.indices.is_empty());
    }

    #[test]
    fn tangents_carry_handedness() {
        let mut tool = SurfaceToolData::new();
        tool.begin(PrimitiveType::Triangles);
        tool.set_normal(Vector3::new(0.0, 0.0, 1.0));
        tool.set_tangent(Plane::new(1.0, 0.0, 0.0, 1.0));
        tool.add_vertex(Vector3::ZERO);
        let arrays = tool.commit_to_arrays();
        assert_eq!(arrays.tangents.len(), 4);
        assert_eq!(&arrays.tangents[0..3], &[1.0, 0.0, 0.0]);
        // binormal = normal x tangent, so the sign is positive here.
        assert_eq!(arrays.tangents[3], 1.0);

        let mut flipped = SurfaceToolData::new();
        flipped.begin(PrimitiveType::Triangles);
        flipped.set_normal(Vector3::new(0.0, 0.0, 1.0));
        flipped.set_tangent(Plane::new(1.0, 0.0, 0.0, -1.0));
        flipped.add_vertex(Vector3::ZERO);
        assert_eq!(flipped.commit_to_arrays().tangents[3], -1.0);
    }

    #[test]
    fn index_dedupes_and_deindex_expands() {
        // Two triangles sharing an edge: six vertices, four unique.
        let mut tool = SurfaceToolData::new();
        tool.begin(PrimitiveType::Triangles);
        for point in [
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        ] {
            tool.add_vertex(point);
        }
        tool.index();
        assert_eq!(tool.get_vertex_count(), 4);
        assert_eq!(tool.get_index_count(), 6);
        let arrays = tool.commit_to_arrays();
        assert_eq!(arrays.format & ARRAY_FORMAT_INDEX, ARRAY_FORMAT_INDEX);
        assert_eq!(arrays.indices, vec![0, 1, 2, 1, 3, 2]);

        tool.deindex();
        assert_eq!(tool.get_vertex_count(), 6);
        assert_eq!(tool.get_index_count(), 0);
        assert_eq!(tool.commit_to_arrays().format & ARRAY_FORMAT_INDEX, 0);
    }

    #[test]
    fn add_index_requires_begin_and_non_negative() {
        let mut tool = SurfaceToolData::new();
        assert!(!tool.add_index(0));
        tool.begin(PrimitiveType::Triangles);
        assert!(!tool.add_index(-1));
        assert!(tool.add_index(0));
        assert_eq!(tool.get_index_count(), 1);
    }

    #[test]
    fn generate_normals_flat_and_smooth() {
        // A counter-clockwise triangle in the XY plane has a +Z normal.
        let mut tool = SurfaceToolData::new();
        tool.begin(PrimitiveType::Triangles);
        tool.set_smooth_group(u32::MAX);
        tool.add_vertex(Vector3::new(0.0, 0.0, 0.0));
        tool.add_vertex(Vector3::new(0.0, 1.0, 0.0));
        tool.add_vertex(Vector3::new(1.0, 0.0, 0.0));
        assert!(tool.generate_normals(false));
        let arrays = tool.commit_to_arrays();
        assert_eq!(arrays.normals.len(), 3);
        for normal in &arrays.normals {
            assert!((normal.length() - 1.0).abs() < 1e-5);
        }
        // Flipping mirrors the normal.
        let mut flipped = SurfaceToolData::new();
        flipped.begin(PrimitiveType::Triangles);
        flipped.set_smooth_group(u32::MAX);
        flipped.add_vertex(Vector3::new(0.0, 0.0, 0.0));
        flipped.add_vertex(Vector3::new(0.0, 1.0, 0.0));
        flipped.add_vertex(Vector3::new(1.0, 0.0, 0.0));
        assert!(flipped.generate_normals(true));
        let flipped_arrays = flipped.commit_to_arrays();
        assert!(
            (arrays.normals[0] + flipped_arrays.normals[0]).length() < 1e-5,
            "flip inverts the face normal"
        );
    }

    #[test]
    fn generate_normals_averages_smooth_groups() {
        // Two triangles at right angles sharing the edge y = 0, same group.
        let mut tool = SurfaceToolData::new();
        tool.begin(PrimitiveType::Triangles);
        tool.set_smooth_group(1);
        // Triangle in the XY plane (+Z normal).
        tool.add_vertex(Vector3::new(0.0, 0.0, 0.0));
        tool.add_vertex(Vector3::new(0.0, 1.0, 0.0));
        tool.add_vertex(Vector3::new(1.0, 0.0, 0.0));
        // Triangle in the XZ plane (+Y normal).
        tool.add_vertex(Vector3::new(0.0, 0.0, 0.0));
        tool.add_vertex(Vector3::new(0.0, 0.0, 1.0));
        tool.add_vertex(Vector3::new(1.0, 0.0, 0.0));
        assert!(tool.generate_normals(false));
        let arrays = tool.commit_to_arrays();
        // The shared vertices have both face normals added and normalized.
        let shared = arrays.normals[0];
        assert!(shared.length() > 0.5);
        assert!(shared.z > 0.0 && shared.y < 0.0, "averaged +Z and -Y face normals");
        assert!(arrays.normals.iter().all(|n| n.is_finite()));
    }

    #[test]
    fn generate_normals_requires_triangles() {
        let mut tool = SurfaceToolData::new();
        tool.begin(PrimitiveType::Lines);
        tool.add_vertex(Vector3::ZERO);
        assert!(!tool.generate_normals(false));
    }

    #[test]
    fn clear_resets_everything() {
        let mut tool = triangle();
        tool.clear();
        assert!(!tool.is_begun());
        assert_eq!(tool.get_primitive_type(), PrimitiveType::Lines);
        assert_eq!(tool.get_vertex_count(), 0);
        assert_eq!(tool.get_skin_weight_count(), SkinWeightCount::Four);
    }
}
