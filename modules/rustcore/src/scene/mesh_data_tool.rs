//! `scene/resources/mesh_data_tool.*`: triangle-mesh data editing.
//!
//! Ported from `mesh_data_tool.cpp`: the vertex/edge/face tables, the
//! `create_from_surface` core (triangle index validation, per-face edge
//! construction with sorted vertex pairs and shared-edge deduplication, the
//! vertex edge/face lists) and the full accessor set (position, normal, tangent,
//! UV/UV2, colour, bones, weights, meta, edge vertices/faces, face
//! vertices/edges, material and format).
//!
//! The `ArrayMesh` input/output is not ported; [`MeshDataTool::create_from_arrays`]
//! takes vertex positions and triangle indices directly (other arrays can be
//! filled through the setters). `commit_to_surface` is not ported.

use crate::math::color::Color;
use crate::math::plane::Plane;
use crate::math::vector2::Vector2;
use crate::math::vector3::Vector3;
use crate::servers::rid::Rid;
use crate::variant::Variant;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeshDataError {
    /// `ERR_INVALID_PARAMETER`: empty input, a non-triangle count or an index out
    /// of range.
    InvalidParameter,
}

#[derive(Clone, Debug)]
pub struct MeshVertex {
    pub vertex: Vector3,
    pub color: Color,
    pub normal: Vector3,
    pub tangent: Plane,
    pub uv: Vector2,
    pub uv2: Vector2,
    pub bones: Vec<i32>,
    pub weights: Vec<f32>,
    pub edges: Vec<usize>,
    pub faces: Vec<usize>,
    pub meta: Variant,
}

impl Default for MeshVertex {
    fn default() -> Self {
        Self {
            vertex: Vector3::ZERO,
            color: Color::new(1.0, 1.0, 1.0, 1.0),
            normal: Vector3::ZERO,
            tangent: Plane::default(),
            uv: Vector2::ZERO,
            uv2: Vector2::ZERO,
            bones: Vec::new(),
            weights: Vec::new(),
            edges: Vec::new(),
            faces: Vec::new(),
            meta: Variant::Nil,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct MeshEdge {
    pub vertex: [usize; 2],
    pub faces: Vec<usize>,
    pub meta: Variant,
}

#[derive(Clone, Debug, Default)]
pub struct MeshFace {
    pub v: [usize; 3],
    pub edges: [usize; 3],
    pub meta: Variant,
}

#[derive(Clone, Debug, Default)]
pub struct MeshDataTool {
    format: u64,
    vertices: Vec<MeshVertex>,
    edges: Vec<MeshEdge>,
    faces: Vec<MeshFace>,
    material: Option<Rid>,
}

impl MeshDataTool {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clear(&mut self) {
        self.vertices.clear();
        self.edges.clear();
        self.faces.clear();
        self.format = 0;
        self.material = None;
    }

    /// The `create_from_surface` core over positions and triangle indices.
    pub fn create_from_arrays(
        &mut self,
        positions: &[Vector3],
        indices: &[i32],
    ) -> Result<(), MeshDataError> {
        if positions.is_empty() || indices.is_empty() || indices.len() % 3 != 0 {
            return Err(MeshDataError::InvalidParameter);
        }
        for index in indices {
            if *index < 0 || *index as usize >= positions.len() {
                return Err(MeshDataError::InvalidParameter);
            }
        }

        self.clear();
        self.vertices = positions
            .iter()
            .map(|position| MeshVertex {
                vertex: *position,
                ..MeshVertex::default()
            })
            .collect();

        let mut edge_indices: std::collections::HashMap<(usize, usize), usize> =
            std::collections::HashMap::new();

        for triangle in indices.chunks_exact(3) {
            let face_index = self.faces.len();
            let mut face = MeshFace::default();
            for j in 0..3 {
                let current = triangle[j] as usize;
                let next = triangle[(j + 1) % 3] as usize;
                face.v[j] = current;

                let edge = if current <= next {
                    (current, next)
                } else {
                    (next, current)
                };
                match edge_indices.get(&edge) {
                    Some(index) => face.edges[j] = *index,
                    None => {
                        let index = self.edges.len();
                        edge_indices.insert(edge, index);
                        self.edges.push(MeshEdge {
                            vertex: [edge.0, edge.1],
                            faces: Vec::new(),
                            meta: Variant::Nil,
                        });
                        face.edges[j] = index;
                        self.vertices[current].edges.push(index);
                        self.vertices[next].edges.push(index);
                    }
                }
                self.edges[face.edges[j]].faces.push(face_index);
                self.vertices[current].faces.push(face_index);
            }
            self.faces.push(face);
        }

        Ok(())
    }

    pub fn get_format(&self) -> u64 {
        self.format
    }

    pub fn set_format(&mut self, format: u64) {
        self.format = format;
    }

    pub fn get_material(&self) -> Option<Rid> {
        self.material
    }

    pub fn set_material(&mut self, material: Option<Rid>) {
        self.material = material;
    }

    pub fn get_vertex_count(&self) -> usize {
        self.vertices.len()
    }

    pub fn get_edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn get_face_count(&self) -> usize {
        self.faces.len()
    }

    pub fn get_vertex(&self, index: usize) -> Vector3 {
        self.vertices
            .get(index)
            .map(|vertex| vertex.vertex)
            .unwrap_or(Vector3::ZERO)
    }

    pub fn set_vertex(&mut self, index: usize, value: Vector3) {
        if let Some(vertex) = self.vertices.get_mut(index) {
            vertex.vertex = value;
        }
    }

    pub fn get_vertex_normal(&self, index: usize) -> Vector3 {
        self.vertices
            .get(index)
            .map(|vertex| vertex.normal)
            .unwrap_or(Vector3::ZERO)
    }

    pub fn set_vertex_normal(&mut self, index: usize, value: Vector3) {
        if let Some(vertex) = self.vertices.get_mut(index) {
            vertex.normal = value;
        }
    }

    pub fn get_vertex_tangent(&self, index: usize) -> Plane {
        self.vertices
            .get(index)
            .map(|vertex| vertex.tangent)
            .unwrap_or_default()
    }

    pub fn set_vertex_tangent(&mut self, index: usize, value: Plane) {
        if let Some(vertex) = self.vertices.get_mut(index) {
            vertex.tangent = value;
        }
    }

    pub fn get_vertex_uv(&self, index: usize) -> Vector2 {
        self.vertices
            .get(index)
            .map(|vertex| vertex.uv)
            .unwrap_or(Vector2::ZERO)
    }

    pub fn set_vertex_uv(&mut self, index: usize, value: Vector2) {
        if let Some(vertex) = self.vertices.get_mut(index) {
            vertex.uv = value;
        }
    }

    pub fn get_vertex_uv2(&self, index: usize) -> Vector2 {
        self.vertices
            .get(index)
            .map(|vertex| vertex.uv2)
            .unwrap_or(Vector2::ZERO)
    }

    pub fn set_vertex_uv2(&mut self, index: usize, value: Vector2) {
        if let Some(vertex) = self.vertices.get_mut(index) {
            vertex.uv2 = value;
        }
    }

    pub fn get_vertex_color(&self, index: usize) -> Color {
        self.vertices
            .get(index)
            .map(|vertex| vertex.color)
            .unwrap_or(Color::new(1.0, 1.0, 1.0, 1.0))
    }

    pub fn set_vertex_color(&mut self, index: usize, value: Color) {
        if let Some(vertex) = self.vertices.get_mut(index) {
            vertex.color = value;
        }
    }

    pub fn get_vertex_bones(&self, index: usize) -> &[i32] {
        self.vertices
            .get(index)
            .map(|vertex| vertex.bones.as_slice())
            .unwrap_or(&[])
    }

    pub fn set_vertex_bones(&mut self, index: usize, bones: Vec<i32>) {
        if let Some(vertex) = self.vertices.get_mut(index) {
            vertex.bones = bones;
        }
    }

    pub fn get_vertex_weights(&self, index: usize) -> &[f32] {
        self.vertices
            .get(index)
            .map(|vertex| vertex.weights.as_slice())
            .unwrap_or(&[])
    }

    pub fn set_vertex_weights(&mut self, index: usize, weights: Vec<f32>) {
        if let Some(vertex) = self.vertices.get_mut(index) {
            vertex.weights = weights;
        }
    }

    pub fn get_vertex_meta(&self, index: usize) -> Variant {
        self.vertices
            .get(index)
            .map(|vertex| vertex.meta.clone())
            .unwrap_or(Variant::Nil)
    }

    pub fn set_vertex_meta(&mut self, index: usize, meta: Variant) {
        if let Some(vertex) = self.vertices.get_mut(index) {
            vertex.meta = meta;
        }
    }

    pub fn get_vertex_edges(&self, index: usize) -> &[usize] {
        self.vertices
            .get(index)
            .map(|vertex| vertex.edges.as_slice())
            .unwrap_or(&[])
    }

    pub fn get_vertex_faces(&self, index: usize) -> &[usize] {
        self.vertices
            .get(index)
            .map(|vertex| vertex.faces.as_slice())
            .unwrap_or(&[])
    }

    /// `get_edge_vertex`: `vertex` is 0 or 1; out-of-range reads return -1.
    pub fn get_edge_vertex(&self, edge: usize, vertex: usize) -> i32 {
        self.edges
            .get(edge)
            .and_then(|edge| edge.vertex.get(vertex).copied())
            .map(|index| index as i32)
            .unwrap_or(-1)
    }

    pub fn get_edge_faces(&self, edge: usize) -> &[usize] {
        self.edges
            .get(edge)
            .map(|edge| edge.faces.as_slice())
            .unwrap_or(&[])
    }

    pub fn get_edge_meta(&self, edge: usize) -> Variant {
        self.edges
            .get(edge)
            .map(|edge| edge.meta.clone())
            .unwrap_or(Variant::Nil)
    }

    pub fn set_edge_meta(&mut self, edge: usize, meta: Variant) {
        if let Some(edge) = self.edges.get_mut(edge) {
            edge.meta = meta;
        }
    }

    pub fn get_face_vertex(&self, face: usize, vertex: usize) -> i32 {
        self.faces
            .get(face)
            .and_then(|face| face.v.get(vertex).copied())
            .map(|index| index as i32)
            .unwrap_or(-1)
    }

    pub fn set_face_vertex(&mut self, face: usize, vertex: usize, index: i32) {
        if let Some(face) = self.faces.get_mut(face) {
            if let Some(slot) = face.v.get_mut(vertex) {
                *slot = index.max(0) as usize;
            }
        }
    }

    pub fn get_face_edge(&self, face: usize, edge: usize) -> i32 {
        self.faces
            .get(face)
            .and_then(|face| face.edges.get(edge).copied())
            .map(|index| index as i32)
            .unwrap_or(-1)
    }

    pub fn get_face_meta(&self, face: usize) -> Variant {
        self.faces
            .get(face)
            .map(|face| face.meta.clone())
            .unwrap_or(Variant::Nil)
    }

    pub fn set_face_meta(&mut self, face: usize, meta: Variant) {
        if let Some(face) = self.faces.get_mut(face) {
            face.meta = meta;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::string::gstring::GoString;

    #[test]
    fn single_triangle() {
        let mut tool = MeshDataTool::new();
        let positions = [
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        ];
        tool.create_from_arrays(&positions, &[0, 1, 2])
            .expect("created");
        assert_eq!(tool.get_vertex_count(), 3);
        assert_eq!(tool.get_edge_count(), 3);
        assert_eq!(tool.get_face_count(), 1);
        assert_eq!(tool.get_vertex(1), positions[1]);

        // Edges are sorted vertex pairs, each touched by the one face.
        for edge in 0..3 {
            let a = tool.get_edge_vertex(edge, 0);
            let b = tool.get_edge_vertex(edge, 1);
            assert!(a < b, "sorted edge {edge}");
            assert_eq!(tool.get_edge_faces(edge), &[0]);
        }
        assert_eq!(tool.get_edge_vertex(9, 0), -1);

        // Each vertex has two incident edges and one face.
        for vertex in 0..3 {
            assert_eq!(tool.get_vertex_edges(vertex).len(), 2);
            assert_eq!(tool.get_vertex_faces(vertex), &[0]);
        }
        assert_eq!(tool.get_face_vertex(0, 0), 0);
        assert_eq!(tool.get_face_vertex(0, 1), 1);
        assert_eq!(tool.get_face_vertex(0, 2), 2);
        assert!(tool.get_face_edge(0, 0) >= 0);
        assert_eq!(tool.get_face_vertex(9, 0), -1);
    }

    #[test]
    fn quad_shares_an_edge() {
        let mut tool = MeshDataTool::new();
        let positions = [
            Vector3::new(0.0, 0.0, 0.0),
            Vector3::new(1.0, 0.0, 0.0),
            Vector3::new(1.0, 1.0, 0.0),
            Vector3::new(0.0, 1.0, 0.0),
        ];
        tool.create_from_arrays(&positions, &[0, 1, 2, 0, 2, 3])
            .expect("created");
        assert_eq!(tool.get_vertex_count(), 4);
        assert_eq!(tool.get_face_count(), 2);
        // Four border edges plus the shared diagonal.
        assert_eq!(tool.get_edge_count(), 5);

        let shared = (0..5)
            .find(|edge| tool.get_edge_faces(*edge).len() == 2)
            .expect("shared edge");
        assert_eq!(tool.get_edge_vertex(shared, 0), 0);
        assert_eq!(tool.get_edge_vertex(shared, 1), 2);
        assert_eq!(tool.get_edge_faces(shared), &[0, 1]);
        // Vertex 0 belongs to both faces and three edges.
        assert_eq!(tool.get_vertex_faces(0), &[0, 1]);
        assert_eq!(tool.get_vertex_edges(0).len(), 3);
    }

    #[test]
    fn invalid_arrays() {
        let mut tool = MeshDataTool::new();
        assert_eq!(
            tool.create_from_arrays(&[], &[]).unwrap_err(),
            MeshDataError::InvalidParameter
        );
        assert_eq!(
            tool.create_from_arrays(&[Vector3::ZERO], &[0, 1, 2]).unwrap_err(),
            MeshDataError::InvalidParameter
        );
        assert_eq!(
            tool.create_from_arrays(&[Vector3::ZERO, Vector3::ONE, Vector3::ZERO], &[0, 1])
                .unwrap_err(),
            MeshDataError::InvalidParameter
        );
        assert_eq!(
            tool.create_from_arrays(
                &[Vector3::ZERO, Vector3::ONE, Vector3::ZERO],
                &[0, 1, 3]
            )
            .unwrap_err(),
            MeshDataError::InvalidParameter
        );
        assert_eq!(tool.get_vertex_count(), 0, "failed creation leaves it empty");
    }

    #[test]
    fn vertex_and_face_attributes() {
        let mut tool = MeshDataTool::new();
        tool.create_from_arrays(
            &[
                Vector3::ZERO,
                Vector3::new(1.0, 0.0, 0.0),
                Vector3::new(0.0, 1.0, 0.0),
            ],
            &[0, 1, 2],
        )
        .expect("created");

        tool.set_vertex_normal(0, Vector3::new(0.0, 0.0, 1.0));
        tool.set_vertex_tangent(0, Plane::new(1.0, 0.0, 0.0, 0.5));
        tool.set_vertex_uv(0, Vector2::new(0.25, 0.75));
        tool.set_vertex_uv2(0, Vector2::new(0.1, 0.2));
        tool.set_vertex_color(0, Color::new(1.0, 0.0, 0.0, 1.0));
        tool.set_vertex_bones(0, vec![1, 2, 3, 4]);
        tool.set_vertex_weights(0, vec![0.1, 0.2, 0.3, 0.4]);
        tool.set_vertex_meta(0, Variant::String(GoString::from_str("corner")));

        assert_eq!(tool.get_vertex_normal(0), Vector3::new(0.0, 0.0, 1.0));
        assert_eq!(tool.get_vertex_tangent(0), Plane::new(1.0, 0.0, 0.0, 0.5));
        assert_eq!(tool.get_vertex_uv(0), Vector2::new(0.25, 0.75));
        assert_eq!(tool.get_vertex_uv2(0), Vector2::new(0.1, 0.2));
        assert_eq!(tool.get_vertex_color(0), Color::new(1.0, 0.0, 0.0, 1.0));
        assert_eq!(tool.get_vertex_bones(0), &[1, 2, 3, 4]);
        assert_eq!(tool.get_vertex_weights(0), &[0.1, 0.2, 0.3, 0.4]);
        assert_eq!(
            tool.get_vertex_meta(0),
            Variant::String(GoString::from_str("corner"))
        );
        // Out-of-range reads return defaults.
        assert_eq!(tool.get_vertex(9), Vector3::ZERO);
        assert!(tool.get_vertex_bones(9).is_empty());
        assert_eq!(tool.get_vertex_meta(9), Variant::Nil);

        tool.set_face_meta(0, Variant::Int(7));
        assert_eq!(tool.get_face_meta(0), Variant::Int(7));
        tool.set_edge_meta(0, Variant::Bool(true));
        assert_eq!(tool.get_edge_meta(0), Variant::Bool(true));
        tool.set_face_vertex(0, 0, 2);
        assert_eq!(tool.get_face_vertex(0, 0), 2);
        tool.set_face_vertex(0, 9, 1); // ignored

        tool.set_material(Some(Rid::from_u64(3)));
        assert_eq!(tool.get_material(), Some(Rid::from_u64(3)));
        tool.set_format(1 << 1);
        assert_eq!(tool.get_format(), 2);

        tool.clear();
        assert_eq!(tool.get_vertex_count(), 0);
        assert_eq!(tool.get_edge_count(), 0);
        assert_eq!(tool.get_face_count(), 0);
        assert!(tool.get_material().is_none());
        assert_eq!(tool.get_format(), 0);
    }
}
